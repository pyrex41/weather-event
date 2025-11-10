use axum::{
    extract::State,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{error::ApiResult, AppState};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WeatherAlert {
    pub id: String,
    pub booking_id: Option<String>,
    pub severity: String,
    pub message: String,
    pub location: String,
    pub student_name: Option<String>,
    pub original_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub dismissed_at: Option<DateTime<Utc>>,
}

/// GET /api/alerts - Retrieve all weather alerts
/// Query params:
/// - dismissed: bool (optional) - include dismissed alerts
pub async fn list_alerts(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<WeatherAlert>>> {
    let alerts = sqlx::query_as::<_, WeatherAlert>(
        "SELECT id, booking_id, severity, message, location, student_name, original_date, created_at, dismissed_at
         FROM weather_alerts
         WHERE dismissed_at IS NULL
         ORDER BY created_at DESC
         LIMIT 100"
    )
    .fetch_all(&state.db)
    .await?;

    tracing::debug!("Retrieved {} weather alerts", alerts.len());
    Ok(Json(alerts))
}
