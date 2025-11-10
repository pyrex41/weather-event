use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};
use core::ai::{AiCache, AiRescheduleClient};
use core::weather::api::WeatherClient;
use dotenv::dotenv;
use sqlx::sqlite::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
};

mod auth;
mod error;
mod routes;
mod scheduler;

use routes::websocket;

pub type NotificationChannel = broadcast::Sender<String>;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub notification_tx: NotificationChannel,
    pub ai_client: Arc<AiRescheduleClient>,
    pub weather_client: Arc<WeatherClient>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Weather Event Server...");

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            tracing::warn!("DATABASE_URL not set, using default: sqlite:weather_app.db");
            "sqlite:weather_app.db".to_string()
        });

    tracing::info!("Connecting to database...");

    let db = SqlitePool::connect(&database_url)
        .await
        .map_err(|e| {
            tracing::error!("Failed to connect to database '{}': {}", database_url, e);
            e
        })?;

    // Run migrations
    tracing::info!("Running database migrations...");
    sqlx::migrate!("../migrations")
        .run(&db)
        .await
        .map_err(|e| {
            tracing::error!("Database migration failed: {}", e);
            e
        })?;

    tracing::info!("Database migrations completed");

    // Create notification channel
    let (notification_tx, _) = broadcast::channel::<String>(100);

    // Initialize AI client
    let ai_cache = Arc::new(AiCache::new());
    let ai_client = Arc::new(
        AiRescheduleClient::from_env(ai_cache.clone())
            .map_err(|e| {
                tracing::warn!("Failed to initialize AI client: {}. Reschedule features will not use AI.", e);
                e
            })
            .unwrap_or_else(|_| {
                // Fallback: create client with dummy key (will always use fallback logic)
                AiRescheduleClient::new("dummy_key".to_string(), Arc::new(AiCache::new()))
            })
    );

    // Initialize weather client
    let weather_client = Arc::new(
        WeatherClient::from_env()
            .map_err(|e| {
                tracing::warn!("Failed to initialize weather client: {}. Weather features may not work.", e);
                e
            })
            .unwrap_or_else(|_| {
                // Fallback: create client with empty key
                WeatherClient::new(String::new(), None)
            })
    );

    // Spawn cache cleanup task
    let cache_clone = ai_cache.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour
        loop {
            interval.tick().await;
            cache_clone.clear_expired().await;
            tracing::info!("Cleared expired AI cache entries");
        }
    });

    // Create app state
    let state = AppState {
        db: db.clone(),
        notification_tx: notification_tx.clone(),
        ai_client,
        weather_client,
    };

    // Configure CORS
    let cors = if let Ok(origins_str) = std::env::var("ALLOWED_ORIGINS") {
        if origins_str.trim() == "*" {
            // Allow any origin (only for development)
            tracing::warn!("CORS configured to allow any origin - NOT RECOMMENDED FOR PRODUCTION");
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PATCH])
                .allow_headers([axum::http::header::CONTENT_TYPE])
        } else {
            // Production: use specified origins
            let origins: Vec<_> = origins_str
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();

            tracing::info!("CORS configured with allowed origins: {:?}", origins);

            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PATCH])
                .allow_headers([axum::http::header::CONTENT_TYPE])
        }
    } else {
        // Development: restrictive default
        tracing::warn!("ALLOWED_ORIGINS not set, using default (http://localhost:8000)");
        let origins = vec!["http://localhost:8000".parse().unwrap()];
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PATCH])
            .allow_headers([axum::http::header::CONTENT_TYPE])
    };

    // Build protected API routes with authentication
    // Note: Rate limiting temporarily disabled for testing
    // TODO: Add back with proper IP extraction configuration
    let api_routes = Router::new()
        .route("/alerts", get(routes::alerts::list_alerts))
        .route("/bookings", get(routes::bookings::list_bookings))
        .route("/bookings", post(routes::bookings::create_booking))
        .route("/bookings/:id", get(routes::bookings::get_booking))
        .route("/bookings/:id/reschedule-suggestions", get(routes::bookings::get_reschedule_suggestions))
        .route("/bookings/:id/reschedule", patch(routes::bookings::reschedule_booking))
        .route("/students", get(routes::students::list_students))
        .route("/students", post(routes::students::create_student))
        .route_layer(middleware::from_fn(auth::auth_middleware));

    // Build main router
    let app = Router::new()
        // Health check (public)
        .route("/health", get(health_check))
        // Protected API routes
        .nest("/api", api_routes)
        // WebSocket (public for now - add auth if needed)
        .route("/ws", get(websocket::ws_handler))
        // Static files (for Elm frontend)
        .fallback_service(ServeDir::new("dist").not_found_service(get(routes::serve_spa)))
        // CORS
        .layer(cors)
        // State
        .with_state(state);

    // Start background scheduler
    let scheduler_db = db.clone();
    let scheduler_tx = notification_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = scheduler::start_weather_monitor(scheduler_db, scheduler_tx).await {
            tracing::error!("Scheduler error: {}", e);
        }
    });

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ok" }))
}
