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

    // Job 1: Run every hour (at minute 0) - Conflict detection
    let hourly_db = db.clone();
    let hourly_tx = notification_tx.clone();
    let hourly_job = Job::new_async("0 0 * * * *", move |_uuid, _lock| {
        let db = hourly_db.clone();
        let tx = hourly_tx.clone();

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

    // Job 2: Run every 5 minutes - Weather alert generation
    let alert_db = db.clone();
    let alert_tx = notification_tx.clone();
    let alert_job = Job::new_async("0 */5 * * * *", move |_uuid, _lock| {
        let db = alert_db.clone();
        let tx = alert_tx.clone();

        Box::pin(async move {
            tracing::info!("Running 5-minute weather alert check...");

            match generate_weather_alerts(&db, &tx).await {
                Ok(alert_count) => {
                    tracing::info!("Generated {} weather alerts", alert_count);
                }
                Err(e) => {
                    tracing::error!("Weather alert generation failed: {}", e);
                }
            }
        })
    })?;

    scheduler.add(hourly_job).await?;
    scheduler.add(alert_job).await?;
    scheduler.start().await?;

    tracing::info!("Weather monitoring scheduler started (hourly conflicts + 5-minute alerts)");

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

/// Generate weather alerts for upcoming bookings
/// Runs every 5 minutes and sends alerts based on weather severity
async fn generate_weather_alerts(
    db: &SqlitePool,
    notification_tx: &NotificationChannel,
) -> anyhow::Result<usize> {
    use core::models::Student;
    use core::weather::{WeatherClient, calculate_weather_score};

    let now = Utc::now();
    let check_until = now + Duration::hours(24);

    // Query upcoming bookings in next 24 hours
    let bookings = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, scheduled_date, departure_location, status
         FROM bookings
         WHERE status IN ('SCHEDULED', 'RESCHEDULED')
         AND scheduled_date BETWEEN ? AND ?
         ORDER BY scheduled_date"
    )
    .bind(now)
    .bind(check_until)
    .fetch_all(db)
    .await?;

    if bookings.is_empty() {
        tracing::debug!("No upcoming bookings to check for alerts");
        return Ok(0);
    }

    tracing::info!("Checking weather alerts for {} upcoming bookings", bookings.len());

    // Get weather client
    let weather_client = match WeatherClient::from_env() {
        Ok(client) => client,
        Err(e) => {
            tracing::warn!("Weather client not available: {}. Skipping alert generation.", e);
            return Ok(0);
        }
    };

    let mut alert_count = 0;

    // Group bookings by location to minimize API calls
    let mut location_cache: std::collections::HashMap<String, core::weather::WeatherData> =
        std::collections::HashMap::new();

    for booking in bookings {
        // Fetch student
        let student = match sqlx::query_as::<_, Student>(
            "SELECT id, name, email, phone, training_level FROM students WHERE id = ?"
        )
        .bind(&booking.student_id)
        .fetch_one(db)
        .await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to fetch student {}: {}", booking.student_id, e);
                continue;
            }
        };

        // Get weather (cached by location)
        let location_key = format!("{},{}", booking.departure_location.lat, booking.departure_location.lon);
        let weather = if let Some(cached) = location_cache.get(&location_key) {
            cached.clone()
        } else {
            match weather_client.fetch_current_weather(
                booking.departure_location.lat,
                booking.departure_location.lon,
            ).await {
                Ok(w) => {
                    location_cache.insert(location_key.clone(), w.clone());
                    w
                }
                Err(e) => {
                    tracing::error!("Failed to fetch weather for booking {}: {}", booking.id, e);
                    continue;
                }
            }
        };

        // Calculate weather score and severity
        let score = calculate_weather_score(&student.training_level, &weather);
        let severity = determine_severity(score as f64, &weather);

        // Generate alert if weather is concerning (score < 9.0)
        if score < 9.0 {
            let message = create_alert_message(&severity, &weather, &student, score as f64);
            let alert_id = uuid::Uuid::new_v4().to_string();
            let now = Utc::now();

            let location_str = format!("({:.4}, {:.4})",
                booking.departure_location.lat,
                booking.departure_location.lon
            );

            // Persist alert to database
            if let Err(e) = sqlx::query(
                "INSERT INTO weather_alerts (id, booking_id, severity, message, location, student_name, original_date, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&alert_id)
            .bind(&booking.id)
            .bind(severity_to_string(&severity))
            .bind(&message)
            .bind(&location_str)
            .bind(&student.name)
            .bind(&booking.scheduled_date)
            .bind(&now)
            .execute(db)
            .await {
                tracing::error!("Failed to persist alert to database: {}", e);
                continue;
            }

            let alert = json!({
                "type": "weather_alert",
                "id": alert_id,
                "booking_id": booking.id,
                "message": message,
                "severity": severity_to_string(&severity),
                "location": location_str,
                "timestamp": now.to_rfc3339(),
                "student_name": student.name,
                "original_date": booking.scheduled_date.to_rfc3339(),
            });

            match notification_tx.send(serde_json::to_string(&alert)?) {
                Ok(_) => {
                    alert_count += 1;
                    tracing::info!(
                        "Sent {} alert for booking {} (score: {:.1})",
                        severity_to_string(&severity),
                        booking.id,
                        score
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to send alert for booking {}: {}", booking.id, e);
                }
            }
        }
    }

    Ok(alert_count)
}

#[derive(Debug, Clone)]
enum AlertSeverity {
    Severe,
    High,
    Moderate,
    Low,
    Clear,
}

fn determine_severity(score: f64, weather: &core::weather::WeatherData) -> AlertSeverity {
    // Check for critical conditions first
    if weather.has_thunderstorms {
        return AlertSeverity::Severe;
    }

    if weather.visibility_miles < 1.0 {  // < 1 mile
        return AlertSeverity::Severe;
    }

    // Score-based severity
    if score < 4.0 {
        AlertSeverity::Severe
    } else if score < 6.0 {
        AlertSeverity::High
    } else if score < 7.5 {
        AlertSeverity::Moderate
    } else if score < 9.0 {
        AlertSeverity::Low
    } else {
        AlertSeverity::Clear
    }
}

fn severity_to_string(severity: &AlertSeverity) -> &'static str {
    match severity {
        AlertSeverity::Severe => "severe",
        AlertSeverity::High => "high",
        AlertSeverity::Moderate => "moderate",
        AlertSeverity::Low => "low",
        AlertSeverity::Clear => "clear",
    }
}

fn create_alert_message(
    severity: &AlertSeverity,
    weather: &core::weather::WeatherData,
    student: &core::models::Student,
    score: f64,
) -> String {
    use core::models::TrainingLevel;

    let training_level_str = match student.training_level {
        TrainingLevel::StudentPilot => "student pilot",
        TrainingLevel::PrivatePilot => "private pilot",
        TrainingLevel::InstrumentRated => "instrument-rated pilot",
    };

    match severity {
        AlertSeverity::Severe => {
            if weather.has_thunderstorms {
                format!(
                    "SEVERE WEATHER ALERT: Thunderstorms reported. Flight not safe for {}. Consider rescheduling.",
                    training_level_str
                )
            } else if weather.visibility_miles < 1.0 {
                format!(
                    "SEVERE WEATHER ALERT: Visibility {:.1} miles, below safe minimums. Flight cancelled for safety.",
                    weather.visibility_miles
                )
            } else {
                format!(
                    "SEVERE WEATHER ALERT: Dangerous conditions detected (score: {:.1}/10). Flight should be cancelled.",
                    score
                )
            }
        }
        AlertSeverity::High => {
            format!(
                "HIGH ALERT: Poor weather conditions (score: {:.1}/10). Visibility {:.1} miles, winds {:.0} kt. Not recommended for {}.",
                score,
                weather.visibility_miles,
                weather.wind_speed_knots,
                training_level_str
            )
        }
        AlertSeverity::Moderate => {
            format!(
                "MODERATE ALERT: Marginal weather conditions (score: {:.1}/10). Winds {:.0} kt, visibility {:.1} miles. Use caution.",
                score,
                weather.wind_speed_knots,
                weather.visibility_miles
            )
        }
        AlertSeverity::Low => {
            format!(
                "Weather advisory: Conditions may be challenging (score: {:.1}/10). Winds {:.0} kt. Monitor before departure.",
                score,
                weather.wind_speed_knots
            )
        }
        AlertSeverity::Clear => {
            String::from("Weather conditions are favorable for flight.")
        }
    }
}
