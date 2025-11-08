pub mod bookings;
pub mod students;
pub mod websocket;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

pub async fn serve_spa() -> Response {
    match tokio::fs::read_to_string("dist/index.html").await {
        Ok(content) => Html(content).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Html("<h1>Frontend not built yet</h1><p>Run: cd elm && npm run build</p>"),
        )
            .into_response(),
    }
}
