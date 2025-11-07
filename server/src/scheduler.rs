use crate::NotificationChannel;
use chrono::{Duration, Utc};
use core::models::{Booking, BookingStatus};
use serde_json::json;
use sqlx::SqlitePool;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn start_weather_monitor(
    db: SqlitePool,
    notification_tx: NotificationChannel,
) -> anyhow::Result<()> {
    tracing::info!("Starting weather monitoring scheduler...");

    let scheduler = JobScheduler::new().await?;

    // Run every hour (at minute 0)
    let job = Job::new_async("0 0 * * * *", move |_uuid, _lock| {
        let db = db.clone();
        let tx = notification_tx.clone();

        Box::pin(async move {
            tracing::info!("Running hourly weather check...");

            match check_all_flights(&db, &tx).await {
                Ok(summary) => {
                    tracing::info!(
                        "Weather check completed: {} flights checked, {} conflicts found",
                        summary.total_checked,
                        summary.conflicts_found
                    );
                }
                Err(e) => {
                    tracing::error!("Weather check failed: {}", e);
                }
            }
        })
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    tracing::info!("Weather monitoring scheduler started");

    // Keep scheduler running
    tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)).await;

    Ok(())
}

#[derive(Debug)]
pub struct ConflictSummary {
    pub total_checked: usize,
    pub conflicts_found: usize,
}

async fn check_all_flights(
    db: &SqlitePool,
    notification_tx: &NotificationChannel,
) -> anyhow::Result<ConflictSummary> {
    let now = Utc::now();
    let check_until = now + Duration::hours(48);

    // Query bookings in next 48 hours
    let bookings = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, scheduled_date, departure_location, status
         FROM bookings
         WHERE status = 'SCHEDULED'
         AND scheduled_date BETWEEN ? AND ?
         ORDER BY scheduled_date"
    )
    .bind(now)
    .bind(check_until)
    .fetch_all(db)
    .await?;

    let total = bookings.len();
    let mut conflicts = 0;

    tracing::info!("Checking {} scheduled flights", total);

    for booking in bookings {
        match check_flight_safety(db, &booking, notification_tx).await {
            Ok(true) => {
                // Flight is safe, no action needed
            }
            Ok(false) => {
                conflicts += 1;
                tracing::warn!("Conflict detected for booking {}", booking.id);
            }
            Err(e) => {
                tracing::error!("Error checking booking {}: {}", booking.id, e);
            }
        }
    }

    Ok(ConflictSummary {
        total_checked: total,
        conflicts_found: conflicts,
    })
}

async fn check_flight_safety(
    db: &SqlitePool,
    booking: &Booking,
    notification_tx: &NotificationChannel,
) -> anyhow::Result<bool> {
    use core::models::Student;
    use core::weather::{is_flight_safe, default_weather_minimums, WeatherClient};

    // Fetch student
    let student = sqlx::query_as::<_, Student>(
        "SELECT id, name, email, phone, training_level FROM students WHERE id = ?"
    )
    .bind(&booking.student_id)
    .fetch_one(db)
    .await?;

    // Get weather client
    let weather_client = WeatherClient::from_env()?;

    // Fetch current weather for departure location
    let weather = weather_client
        .fetch_current_weather(
            booking.departure_location.lat,
            booking.departure_location.lon,
        )
        .await?;

    // Check safety
    let minimums = default_weather_minimums();
    let student_minimums = minimums
        .get(&student.training_level)
        .ok_or_else(|| anyhow::anyhow!("No minimums for training level"))?;

    let (is_safe, reason) = is_flight_safe(&student.training_level, &weather, student_minimums);

    if !is_safe {
        tracing::warn!(
            "Unsafe weather for booking {}: {}",
            booking.id,
            reason.as_deref().unwrap_or("Unknown")
        );

        // Cancel booking
        sqlx::query(
            "UPDATE bookings SET status = ? WHERE id = ?"
        )
        .bind(BookingStatus::Cancelled.as_str())
        .bind(&booking.id)
        .execute(db)
        .await?;

        // Create reschedule event
        let reschedule_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO reschedule_events (id, booking_id, original_date, new_date, suggested_by)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&reschedule_id)
        .bind(&booking.id)
        .bind(&booking.scheduled_date)
        .bind(&booking.scheduled_date) // Placeholder, will be updated when student reschedules
        .bind("SYSTEM")
        .execute(db)
        .await?;

        // Send WebSocket notification
        let notification = json!({
            "type": "WEATHER_CONFLICT",
            "booking_id": booking.id,
            "message": format!("Flight cancelled: {}", reason.unwrap_or_default()),
            "student_name": student.name,
            "original_date": booking.scheduled_date.to_rfc3339(),
        });

        let _ = notification_tx.send(serde_json::to_string(&notification)?);

        // Log notification sent
        tracing::info!("Sent conflict notification for booking {}", booking.id);

        // Here we would also send email/SMS notifications
        // but that requires additional setup, so logging for now

        return Ok(false);
    }

    Ok(true)
}
