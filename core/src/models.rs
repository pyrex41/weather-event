use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Training level of a student pilot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrainingLevel {
    StudentPilot,
    PrivatePilot,
    InstrumentRated,
}

/// Status of a booking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BookingStatus {
    Scheduled,
    Cancelled,
    Rescheduled,
    Completed,
}

/// Geographic location with coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
    pub name: String,
}

/// Student pilot information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Student {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    #[sqlx(try_from = "String")]
    pub training_level: TrainingLevel,
}

/// Flight booking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Booking {
    pub id: String,
    pub student_id: String,
    pub scheduled_date: DateTime<Utc>,
    /// Stored as JSON TEXT in SQLite
    #[sqlx(json)]
    pub departure_location: Location,
    #[sqlx(try_from = "String")]
    pub status: BookingStatus,
}

/// Weather check record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WeatherCheck {
    pub id: String,
    pub booking_id: String,
    pub checked_at: DateTime<Utc>,
    /// Raw weather data as JSON
    pub weather_data: String,
    pub is_safe: bool,
    pub reason: Option<String>,
}

/// Reschedule event tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RescheduleEvent {
    pub id: String,
    pub booking_id: String,
    pub original_date: DateTime<Utc>,
    pub new_date: DateTime<Utc>,
    pub suggested_by: String,
    /// AI suggestions as JSON
    pub ai_suggestions: Option<String>,
}

/// Weather minimums for each training level
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WeatherMinimum {
    pub id: String,
    #[sqlx(try_from = "String")]
    pub training_level: TrainingLevel,
    pub min_visibility_sm: f64,
    pub max_wind_speed_kt: f64,
    pub min_ceiling_ft: Option<f64>,
    #[sqlx(rename = "allow_imc")]
    pub allow_imc: bool,
    pub no_thunderstorms: bool,
    pub no_icing: bool,
}

impl TrainingLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrainingLevel::StudentPilot => "STUDENT_PILOT",
            TrainingLevel::PrivatePilot => "PRIVATE_PILOT",
            TrainingLevel::InstrumentRated => "INSTRUMENT_RATED",
        }
    }
}

impl BookingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BookingStatus::Scheduled => "SCHEDULED",
            BookingStatus::Cancelled => "CANCELLED",
            BookingStatus::Rescheduled => "RESCHEDULED",
            BookingStatus::Completed => "COMPLETED",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_json_serialization() {
        let location = Location {
            lat: 33.8113,
            lon: -118.1515,
            name: "KTOA".to_string(),
        };

        let json = serde_json::to_string(&location).unwrap();
        let deserialized: Location = serde_json::from_str(&json).unwrap();

        assert_eq!(location.lat, deserialized.lat);
        assert_eq!(location.lon, deserialized.lon);
        assert_eq!(location.name, deserialized.name);
    }

    #[test]
    fn test_training_level_serialization() {
        let level = TrainingLevel::StudentPilot;
        assert_eq!(level.as_str(), "STUDENT_PILOT");

        let json = serde_json::to_string(&level).unwrap();
        let deserialized: TrainingLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, deserialized);
    }

    #[test]
    fn test_booking_status_serialization() {
        let status = BookingStatus::Scheduled;
        assert_eq!(status.as_str(), "SCHEDULED");

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: BookingStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}
