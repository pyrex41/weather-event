use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use core::models::{Booking, BookingStatus, Location};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingRequest {
    pub student_id: String,
    pub scheduled_date: DateTime<Utc>,
    pub departure_location: Location,
}

#[derive(Debug, Serialize)]
pub struct BookingResponse {
    pub id: String,
    pub student_id: String,
    pub scheduled_date: DateTime<Utc>,
    pub departure_location: Location,
    pub status: String,
}

impl From<Booking> for BookingResponse {
    fn from(booking: Booking) -> Self {
        Self {
            id: booking.id,
            student_id: booking.student_id,
            scheduled_date: booking.scheduled_date,
            departure_location: booking.departure_location,
            status: booking.status.as_str().to_string(),
        }
    }
}

pub async fn list_bookings(
    Query(params): Query<PaginationParams>,
    State(state): State<AppState>,
) -> Result<Json<Vec<BookingResponse>>, StatusCode> {
    // Validate and sanitize pagination parameters
    let page = params.page.max(1);
    let limit = params.limit.clamp(1, 100); // Max 100 items per page
    let offset = (page - 1) * limit;

    let bookings = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, scheduled_date, departure_location, status
         FROM bookings
         ORDER BY scheduled_date DESC
         LIMIT ? OFFSET ?"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch bookings: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(bookings.into_iter().map(BookingResponse::from).collect()))
}

pub async fn get_booking(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<BookingResponse>, StatusCode> {
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, scheduled_date, departure_location, status FROM bookings WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch booking: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(booking.into()))
}

pub async fn create_booking(
    State(state): State<AppState>,
    Json(req): Json<CreateBookingRequest>,
) -> Result<(StatusCode, Json<BookingResponse>), StatusCode> {
    // Generate UUID
    let id = uuid::Uuid::new_v4().to_string();

    // Serialize location to JSON
    let location_json = serde_json::to_string(&req.departure_location)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Insert booking
    sqlx::query(
        "INSERT INTO bookings (id, student_id, scheduled_date, departure_location, status) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.student_id)
    .bind(&req.scheduled_date)
    .bind(&location_json)
    .bind(BookingStatus::Scheduled.as_str())
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create booking: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Fetch created booking
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, scheduled_date, departure_location, status FROM bookings WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch created booking: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(booking.into())))
}

// Add uuid dependency to server/Cargo.toml
