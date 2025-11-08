use weather_core::models::{Booking, BookingStatus, Location, Student, TrainingLevel};
use chrono::Utc;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

async fn setup_test_db() -> SqlitePool {
    // Create in-memory database for testing
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    // Run migrations
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[tokio::test]
async fn test_database_schema_creation() {
    let pool = setup_test_db().await;

    // Check that all tables exist
    let tables = sqlx::query_scalar::<_, String>(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query tables");

    assert!(tables.contains(&"students".to_string()));
    assert!(tables.contains(&"bookings".to_string()));
    assert!(tables.contains(&"weather_checks".to_string()));
    assert!(tables.contains(&"reschedule_events".to_string()));
    assert!(tables.contains(&"weather_minimums".to_string()));

    pool.close().await;
}

#[tokio::test]
async fn test_student_crud_operations() {
    let pool = setup_test_db().await;

    // Create a student
    let student_id = "test_student_1";
    sqlx::query(
        "INSERT INTO students (id, name, email, phone, training_level) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(student_id)
    .bind("John Doe")
    .bind("john@example.com")
    .bind("+1234567890")
    .bind(TrainingLevel::StudentPilot.as_str())
    .execute(&pool)
    .await
    .expect("Failed to insert student");

    // Read the student
    let student = sqlx::query_as::<_, Student>(
        "SELECT id, name, email, phone, training_level FROM students WHERE id = ?"
    )
    .bind(student_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch student");

    assert_eq!(student.id, student_id);
    assert_eq!(student.name, "John Doe");
    assert_eq!(student.email, "john@example.com");
    assert_eq!(student.training_level, TrainingLevel::StudentPilot);

    // Update the student
    sqlx::query("UPDATE students SET name = ? WHERE id = ?")
        .bind("Jane Doe")
        .bind(student_id)
        .execute(&pool)
        .await
        .expect("Failed to update student");

    let updated = sqlx::query_as::<_, Student>(
        "SELECT id, name, email, phone, training_level FROM students WHERE id = ?"
    )
    .bind(student_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch updated student");

    assert_eq!(updated.name, "Jane Doe");

    // Delete the student
    sqlx::query("DELETE FROM students WHERE id = ?")
        .bind(student_id)
        .execute(&pool)
        .await
        .expect("Failed to delete student");

    let result = sqlx::query_as::<_, Student>(
        "SELECT id, name, email, phone, training_level FROM students WHERE id = ?"
    )
    .bind(student_id)
    .fetch_optional(&pool)
    .await
    .expect("Failed to query student");

    assert!(result.is_none());

    pool.close().await;
}

#[tokio::test]
async fn test_booking_with_location_json() {
    let pool = setup_test_db().await;

    // First create a student
    let student_id = "test_student_2";
    sqlx::query(
        "INSERT INTO students (id, name, email, phone, training_level) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(student_id)
    .bind("Test Student")
    .bind("test@example.com")
    .bind("+1234567890")
    .bind(TrainingLevel::PrivatePilot.as_str())
    .execute(&pool)
    .await
    .expect("Failed to insert student");

    // Create a booking with JSON location
    let location = Location {
        lat: 33.8113,
        lon: -118.1515,
        name: "KTOA".to_string(),
    };
    let location_json = serde_json::to_string(&location).expect("Failed to serialize location");

    let booking_id = "test_booking_1";
    let scheduled_date = Utc::now();

    sqlx::query(
        "INSERT INTO bookings (id, student_id, scheduled_date, departure_location, status)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(booking_id)
    .bind(student_id)
    .bind(scheduled_date)
    .bind(&location_json)
    .bind(BookingStatus::Scheduled.as_str())
    .execute(&pool)
    .await
    .expect("Failed to insert booking");

    // Fetch and verify
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT id, student_id, scheduled_date, departure_location, status FROM bookings WHERE id = ?"
    )
    .bind(booking_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch booking");

    assert_eq!(booking.id, booking_id);
    assert_eq!(booking.student_id, student_id);
    assert_eq!(booking.departure_location.name, "KTOA");
    assert!((booking.departure_location.lat - 33.8113).abs() < 0.0001);
    assert!((booking.departure_location.lon - (-118.1515)).abs() < 0.0001);
    assert_eq!(booking.status, BookingStatus::Scheduled);

    pool.close().await;
}

#[tokio::test]
async fn test_foreign_key_constraints() {
    let pool = setup_test_db().await;

    // Try to create a booking without a student (should fail)
    let location = Location {
        lat: 33.8113,
        lon: -118.1515,
        name: "KTOA".to_string(),
    };
    let location_json = serde_json::to_string(&location).unwrap();

    let result = sqlx::query(
        "INSERT INTO bookings (id, student_id, scheduled_date, departure_location, status)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind("invalid_booking")
    .bind("nonexistent_student")
    .bind(Utc::now())
    .bind(&location_json)
    .bind(BookingStatus::Scheduled.as_str())
    .execute(&pool)
    .await;

    assert!(result.is_err(), "Should fail due to foreign key constraint");

    pool.close().await;
}

#[tokio::test]
async fn test_cascade_delete() {
    let pool = setup_test_db().await;

    // Create student and booking
    let student_id = "test_student_3";
    sqlx::query(
        "INSERT INTO students (id, name, email, phone, training_level) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(student_id)
    .bind("Test Student")
    .bind("test@example.com")
    .bind("+1234567890")
    .bind(TrainingLevel::StudentPilot.as_str())
    .execute(&pool)
    .await
    .expect("Failed to insert student");

    let location = Location {
        lat: 33.8113,
        lon: -118.1515,
        name: "KTOA".to_string(),
    };
    let location_json = serde_json::to_string(&location).unwrap();

    let booking_id = "test_booking_2";
    sqlx::query(
        "INSERT INTO bookings (id, student_id, scheduled_date, departure_location, status)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(booking_id)
    .bind(student_id)
    .bind(Utc::now())
    .bind(&location_json)
    .bind(BookingStatus::Scheduled.as_str())
    .execute(&pool)
    .await
    .expect("Failed to insert booking");

    // Verify booking exists
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookings WHERE id = ?")
        .bind(booking_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count bookings");
    assert_eq!(count, 1);

    // Delete student (should cascade to booking)
    sqlx::query("DELETE FROM students WHERE id = ?")
        .bind(student_id)
        .execute(&pool)
        .await
        .expect("Failed to delete student");

    // Verify booking was deleted
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookings WHERE id = ?")
        .bind(booking_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count bookings");
    assert_eq!(count, 0, "Booking should be deleted when student is deleted");

    pool.close().await;
}

#[tokio::test]
async fn test_weather_minimums_defaults() {
    let pool = setup_test_db().await;

    // Check that default minimums were inserted
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weather_minimums")
        .fetch_one(&pool)
        .await
        .expect("Failed to count weather minimums");

    assert_eq!(count, 3, "Should have 3 default weather minimum entries");

    // Verify student pilot minimums
    let (vis, wind): (f64, f64) = sqlx::query_as(
        "SELECT min_visibility_sm, max_wind_speed_kt FROM weather_minimums
         WHERE training_level = 'STUDENT_PILOT'"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch student pilot minimums");

    assert_eq!(vis, 5.0);
    assert_eq!(wind, 12.0);

    pool.close().await;
}

#[tokio::test]
async fn test_concurrent_writes() {
    let pool = setup_test_db().await;

    // Spawn multiple tasks to insert students concurrently
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let pool = pool.clone();
            tokio::spawn(async move {
                let student_id = format!("concurrent_student_{}", i);
                sqlx::query(
                    "INSERT INTO students (id, name, email, phone, training_level)
                     VALUES (?, ?, ?, ?, ?)"
                )
                .bind(&student_id)
                .bind(format!("Student {}", i))
                .bind(format!("student{}@example.com", i))
                .bind(format!("+1234567{:03}", i))
                .bind(TrainingLevel::PrivatePilot.as_str())
                .execute(&pool)
                .await
                .expect("Failed to insert student");
            })
        })
        .collect();

    // Wait for all tasks
    for handle in handles {
        handle.await.expect("Task failed");
    }

    // Verify all students were inserted
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM students WHERE id LIKE 'concurrent_student_%'")
        .fetch_one(&pool)
        .await
        .expect("Failed to count students");

    assert_eq!(count, 10, "All concurrent inserts should succeed");

    pool.close().await;
}

#[tokio::test]
async fn test_booking_status_transitions() {
    let pool = setup_test_db().await;

    // Create student and booking
    let student_id = "test_student_4";
    sqlx::query(
        "INSERT INTO students (id, name, email, phone, training_level) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(student_id)
    .bind("Test Student")
    .bind("test@example.com")
    .bind("+1234567890")
    .bind(TrainingLevel::StudentPilot.as_str())
    .execute(&pool)
    .await
    .expect("Failed to insert student");

    let location = Location {
        lat: 33.8113,
        lon: -118.1515,
        name: "KTOA".to_string(),
    };
    let location_json = serde_json::to_string(&location).unwrap();

    let booking_id = "test_booking_3";
    sqlx::query(
        "INSERT INTO bookings (id, student_id, scheduled_date, departure_location, status)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(booking_id)
    .bind(student_id)
    .bind(Utc::now())
    .bind(&location_json)
    .bind(BookingStatus::Scheduled.as_str())
    .execute(&pool)
    .await
    .expect("Failed to insert booking");

    // Test status transitions
    for status in &[BookingStatus::Cancelled, BookingStatus::Rescheduled, BookingStatus::Completed] {
        sqlx::query("UPDATE bookings SET status = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(booking_id)
            .execute(&pool)
            .await
            .expect("Failed to update status");

        let current_status: String = sqlx::query_scalar("SELECT status FROM bookings WHERE id = ?")
            .bind(booking_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch status");

        assert_eq!(current_status, status.as_str());
    }

    pool.close().await;
}
