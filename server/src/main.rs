use axum::{
    routing::{get, patch, post},
    Router,
};
use core::ai::{AiCache, AiRescheduleClient};
use dotenv::dotenv;
use sqlx::sqlite::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;
mod scheduler;

use routes::websocket;

pub type NotificationChannel = broadcast::Sender<String>;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub notification_tx: NotificationChannel,
    pub ai_client: Arc<AiRescheduleClient>,
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

    tracing::info!("Connecting to database: {}", database_url);

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
        AiRescheduleClient::from_env(ai_cache)
            .map_err(|e| {
                tracing::warn!("Failed to initialize AI client: {}. Reschedule features will not use AI.", e);
                e
            })
            .unwrap_or_else(|_| {
                // Fallback: create client with dummy key (will always use fallback logic)
                AiRescheduleClient::new("dummy_key".to_string(), Arc::new(AiCache::new()))
            })
    );

    // Create app state
    let state = AppState {
        db: db.clone(),
        notification_tx: notification_tx.clone(),
        ai_client,
    };

    // Configure CORS
    let cors = if let Ok(origins_str) = std::env::var("ALLOWED_ORIGINS") {
        if origins_str.trim() == "*" {
            // Allow any origin
            tracing::info!("CORS configured to allow any origin");
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
        // Development: permissive CORS
        tracing::warn!("CORS configured in permissive mode (ALLOWED_ORIGINS not set)");
        CorsLayer::permissive()
    };

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        // API routes
        .route("/api/bookings", get(routes::bookings::list_bookings))
        .route("/api/bookings", post(routes::bookings::create_booking))
        .route("/api/bookings/:id", get(routes::bookings::get_booking))
        .route("/api/bookings/:id/reschedule-suggestions", get(routes::bookings::get_reschedule_suggestions))
        .route("/api/bookings/:id/reschedule", patch(routes::bookings::reschedule_booking))
        .route("/api/students", get(routes::students::list_students))
        .route("/api/students", post(routes::students::create_student))
        // WebSocket
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
