use crate::{error::ApiResult, AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use core::ai::RescheduleOption;
use core::models::{Booking, BookingStatus, Location, Student};
use core::weather::api::WeatherClient;
use serde::{Deserialize, Serialize};

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
    pub aircraft_type: String,
    pub scheduled_date: DateTime<Utc>,
    pub departure_location: Location,
}

#[derive(Debug, Serialize)]
pub struct BookingResponse {
    pub id: String,
    pub student_id: String,
    pub aircraft_type: String,
    pub scheduled_date: DateTime<Utc>,
    pub departure_location: Location,
    pub status: String,
}

impl From<Booking> for BookingResponse {
    fn from(booking: Booking) -> Self {
        Self {
            id: booking.id,
            student_id: booking.student_id,
            aircraft_type: booking.aircraft_type,
            scheduled_date: booking.scheduled_date,
            departure_location: booking.departure_location,
            status: booking.status.as_str().to_string(),
        }
    }
}

pub async fn list_bookings(
    Query(params): Query<PaginationParams>,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<BookingResponse>>> {
    // Validate and sanitize pagination parameters
    let page = params.page.max(1);
    let limit = params.limit.clamp(1, 100); // Max 100 items per page
    let offset = (page - 1) * limit;

    let bookings = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, aircraft_type, scheduled_date, departure_location, status
         FROM bookings
         ORDER BY scheduled_date DESC
         LIMIT ? OFFSET ?"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    tracing::debug!("Retrieved {} bookings (page={}, limit={})", bookings.len(), page, limit);
    Ok(Json(bookings.into_iter().map(BookingResponse::from).collect()))
}

pub async fn get_booking(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<BookingResponse>> {
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, aircraft_type, scheduled_date, departure_location, status FROM bookings WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::ApiError::not_found("Booking"))?;

    Ok(Json(booking.into()))
}

pub async fn create_booking(
    State(state): State<AppState>,
    Json(req): Json<CreateBookingRequest>,
) -> ApiResult<(StatusCode, Json<BookingResponse>)> {
    // Generate UUID
    let id = uuid::Uuid::new_v4().to_string();

    // Serialize location to JSON
    let location_json = serde_json::to_string(&req.departure_location)?;

    // Insert booking
    sqlx::query(
        "INSERT INTO bookings (id, student_id, aircraft_type, scheduled_date, departure_location, status) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.student_id)
    .bind(&req.aircraft_type)
    .bind(&req.scheduled_date)
    .bind(&location_json)
    .bind(BookingStatus::Scheduled.as_str())
    .execute(&state.db)
    .await?;

    // Fetch created booking
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, aircraft_type, scheduled_date, departure_location, status FROM bookings WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await?;

    tracing::info!("Created booking {} for student {}", booking.id, booking.student_id);
    Ok((StatusCode::CREATED, Json(booking.into())))
}

#[derive(Debug, Serialize)]
pub struct RescheduleOptionsResponse {
    pub options: Vec<RescheduleOption>,
}

#[derive(Debug, Deserialize)]
pub struct RescheduleRequest {
    pub new_scheduled_date: DateTime<Utc>,
}

/// GET /api/bookings/:id/reschedule-suggestions
/// Returns 3 AI-generated reschedule options
pub async fn get_reschedule_suggestions(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<RescheduleOptionsResponse>> {
    // Fetch the booking
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, aircraft_type, scheduled_date, departure_location, status FROM bookings WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::ApiError::not_found("Booking"))?;

    // Fetch the student
    let student = sqlx::query_as::<_, Student>(
        "SELECT id, name, email, phone, training_level FROM students WHERE id = ?"
    )
    .bind(&booking.student_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::ApiError::not_found("Student"))?;

    // Get weather API key from environment
    let weather_api_key = std::env::var("WEATHER_API_KEY")
        .unwrap_or_else(|_| {
            tracing::warn!("WEATHER_API_KEY not set, weather data may be unavailable");
            String::new()
        });

    // Fetch weather forecast
    let weather_client = WeatherClient::new(weather_api_key, None);
    let weather_forecast = weather_client
        .fetch_forecast(
            booking.departure_location.lat,
            booking.departure_location.lon
        )
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to fetch weather forecast: {}", e);
            vec![]
        });

    // Fetch instructor schedule (other bookings to determine availability)
    let instructor_schedule = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, aircraft_type, scheduled_date, departure_location, status
         FROM bookings
         WHERE status = 'SCHEDULED' AND scheduled_date > datetime('now')
         ORDER BY scheduled_date ASC
         LIMIT 50"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!("Failed to fetch instructor schedule: {}", e);
        vec![]
    });

    // Generate reschedule options using AI
    let options = state
        .ai_client
        .generate_reschedule_options(&booking, &student, &weather_forecast, &instructor_schedule)
        .await?;

    Ok(Json(RescheduleOptionsResponse { options }))
}

/// PATCH /api/bookings/:id/reschedule
/// Actually reschedules the booking with the selected option
pub async fn reschedule_booking(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<RescheduleRequest>,
) -> ApiResult<Json<BookingResponse>> {
    // Fetch the booking
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, aircraft_type, scheduled_date, departure_location, status FROM bookings WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::ApiError::not_found("Booking"))?;

    // Fetch the student for notification
    let student = sqlx::query_as::<_, Student>(
        "SELECT id, name, email, phone, training_level FROM students WHERE id = ?"
    )
    .bind(&booking.student_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::ApiError::not_found("Student"))?;

    // Update booking with new date
    sqlx::query(
        "UPDATE bookings SET scheduled_date = ?, status = ? WHERE id = ?"
    )
    .bind(&req.new_scheduled_date)
    .bind(BookingStatus::Rescheduled.as_str())
    .bind(&id)
    .execute(&state.db)
    .await?;

    // Log reschedule event
    let reschedule_event_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO reschedule_events (id, booking_id, old_date, new_date, reason, created_at)
         VALUES (?, ?, ?, ?, ?, datetime('now'))"
    )
    .bind(&reschedule_event_id)
    .bind(&id)
    .bind(&booking.scheduled_date)
    .bind(&req.new_scheduled_date)
    .bind("User requested reschedule")
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::warn!("Failed to log reschedule event: {}", e);
        // Don't fail the request if logging fails
        e
    })
    .ok();

    // Notify via WebSocket
    let notification = serde_json::json!({
        "type": "booking_rescheduled",
        "booking_id": id,
        "old_date": booking.scheduled_date,
        "new_date": req.new_scheduled_date,
        "student_name": student.name,
    });

    let _ = state.notification_tx.send(notification.to_string());

    // Fetch updated booking
    let updated_booking = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, aircraft_type, scheduled_date, departure_location, status FROM bookings WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await?;

    tracing::info!("Rescheduled booking {} from {} to {}", id, booking.scheduled_date, req.new_scheduled_date);
    Ok(Json(updated_booking.into()))
}

// Add uuid dependency to server/Cargo.toml
