use crate::AppState;
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
) -> Result<Json<Vec<BookingResponse>>, StatusCode> {
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
        "SELECT id, student_id, aircraft_type, scheduled_date, departure_location, status FROM bookings WHERE id = ?"
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
        "INSERT INTO bookings (id, student_id, aircraft_type, scheduled_date, departure_location, status) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.student_id)
    .bind(&req.aircraft_type)
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
        "SELECT id, student_id, aircraft_type, scheduled_date, departure_location, status FROM bookings WHERE id = ?"
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
) -> Result<Json<RescheduleOptionsResponse>, StatusCode> {
    // Fetch the booking
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, aircraft_type, scheduled_date, departure_location, status FROM bookings WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch booking: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Fetch the student
    let student = sqlx::query_as::<_, Student>(
        "SELECT id, name, email, phone, training_level FROM students WHERE id = ?"
    )
    .bind(&booking.student_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch student: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        tracing::error!("Student not found for booking {}", id);
        StatusCode::NOT_FOUND
    })?;

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
        .await
        .map_err(|e| {
            tracing::error!("Failed to generate reschedule options: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(RescheduleOptionsResponse { options }))
}

/// PATCH /api/bookings/:id/reschedule
/// Actually reschedules the booking with the selected option
pub async fn reschedule_booking(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<RescheduleRequest>,
) -> Result<Json<BookingResponse>, StatusCode> {
    // Fetch the booking
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, aircraft_type, scheduled_date, departure_location, status FROM bookings WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch booking: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Fetch the student for notification
    let student = sqlx::query_as::<_, Student>(
        "SELECT id, name, email, phone, training_level FROM students WHERE id = ?"
    )
    .bind(&booking.student_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch student: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        tracing::error!("Student not found for booking {}", id);
        StatusCode::NOT_FOUND
    })?;

    // Update booking with new date
    sqlx::query(
        "UPDATE bookings SET scheduled_date = ?, status = ? WHERE id = ?"
    )
    .bind(&req.new_scheduled_date)
    .bind(BookingStatus::Rescheduled.as_str())
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update booking: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch updated booking: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(updated_booking.into()))
}

// Add uuid dependency to server/Cargo.toml
