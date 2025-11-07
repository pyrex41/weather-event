use crate::models::{Booking, Student};
use crate::weather::{is_flight_safe, WeatherData};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RescheduleOption {
    pub date_time: DateTime<Utc>,
    pub reason: String,
    pub weather_score: f32,
    pub instructor_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RescheduleResponse {
    pub options: Vec<RescheduleOption>,
}

/// AI cache with TTL (6 hours)
pub struct AiCache {
    cache: Arc<RwLock<HashMap<String, (RescheduleResponse, DateTime<Utc>)>>>,
    ttl_hours: i64,
}

impl AiCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl_hours: 6,
        }
    }

    pub async fn get(&self, key: &str) -> Option<RescheduleResponse> {
        let cache = self.cache.read().await;
        if let Some((response, timestamp)) = cache.get(key) {
            let now = Utc::now();
            let age = now.signed_duration_since(*timestamp).num_hours();
            if age < self.ttl_hours {
                return Some(response.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: String, response: RescheduleResponse) {
        let mut cache = self.cache.write().await;
        cache.insert(key, (response, Utc::now()));
    }

    pub async fn clear_expired(&self) {
        let mut cache = self.cache.write().await;
        let now = Utc::now();
        cache.retain(|_, (_, timestamp)| {
            now.signed_duration_since(*timestamp).num_hours() < self.ttl_hours
        });
    }
}

impl Default for AiCache {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AiRescheduleClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    cache: Arc<AiCache>,
}

impl AiRescheduleClient {
    pub fn new(api_key: String, cache: Arc<AiCache>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            cache,
        }
    }

    pub fn from_env(cache: Arc<AiCache>) -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY environment variable not set")?;
        Ok(Self::new(api_key, cache))
    }

    pub async fn generate_reschedule_options(
        &self,
        booking: &Booking,
        student: &Student,
        weather_forecast: &[WeatherData],
        instructor_schedule: &[Booking],
    ) -> Result<Vec<RescheduleOption>> {
        // Check cache first
        let cache_key = format!("{}_{}", booking.id, booking.scheduled_date.timestamp());
        if let Some(cached) = self.cache.get(&cache_key).await {
            if cached.options.len() >= 3 {
                return Ok(cached.options);
            }
        }

        // Try AI first
        match self
            .generate_with_ai(booking, student, weather_forecast, instructor_schedule)
            .await
        {
            Ok(options) if options.len() >= 3 => {
                // Cache successful response
                self.cache
                    .set(cache_key, RescheduleResponse { options: options.clone() })
                    .await;
                Ok(options)
            }
            _ => {
                // Fallback to rule-based
                tracing::warn!("AI reschedule failed or insufficient options, using fallback");
                self.generate_fallback_options(booking, student, weather_forecast, instructor_schedule)
                    .await
            }
        }
    }

    async fn generate_with_ai(
        &self,
        booking: &Booking,
        student: &Student,
        weather_forecast: &[WeatherData],
        instructor_schedule: &[Booking],
    ) -> Result<Vec<RescheduleOption>> {
        let prompt = self.build_prompt(booking, student, weather_forecast, instructor_schedule);

        #[derive(Serialize)]
        struct ChatMessage {
            role: String,
            content: String,
        }

        #[derive(Serialize)]
        struct ChatRequest {
            model: String,
            messages: Vec<ChatMessage>,
            temperature: f32,
            response_format: serde_json::Value,
        }

        let request = ChatRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "You are a flight scheduling assistant. Always return valid JSON with exactly 3 reschedule options. Each option must have: date_time (ISO 8601 format), reason (string explaining why this time is good), weather_score (float 0-10), and instructor_available (boolean).".to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.7,
            response_format: serde_json::json!({ "type": "json_object" }),
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to call OpenAI API")?;

        if !response.status().is_success() {
            anyhow::bail!("OpenAI API returned status: {}", response.status());
        }

        #[derive(Deserialize)]
        struct ChatResponse {
            choices: Vec<Choice>,
        }

        #[derive(Deserialize)]
        struct Choice {
            message: Message,
        }

        #[derive(Deserialize)]
        struct Message {
            content: String,
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        let content = chat_response
            .choices
            .first()
            .map(|c| &c.message.content)
            .context("No choices in OpenAI response")?;

        let reschedule_response: RescheduleResponse = serde_json::from_str(content)
            .context("Failed to parse AI response as RescheduleResponse")?;

        Ok(reschedule_response.options)
    }

    fn build_prompt(
        &self,
        booking: &Booking,
        student: &Student,
        weather_forecast: &[WeatherData],
        _instructor_schedule: &[Booking],
    ) -> String {
        let weather_summary: String = weather_forecast
            .iter()
            .take(7)
            .map(|w| {
                format!(
                    "{}: vis {:.1}mi, wind {:.1}kt, temp {:.0}°F, {}",
                    w.date_time.format("%Y-%m-%d %H:%M"),
                    w.visibility_miles,
                    w.wind_speed_knots,
                    w.temperature_f,
                    w.conditions
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Flight booking needs rescheduling due to weather conflict.

Student: {} (Training Level: {:?})
Original booking: {}
Departure location: {}

7-day weather forecast:
{}

Please suggest 3 alternative times for rescheduling this flight lesson. Consider:
1. Weather conditions suitable for {:?} training level
2. Time of day (prefer daylight hours)
3. Spread options across different days

Return JSON with this exact structure:
{{
  "options": [
    {{
      "date_time": "2024-01-15T14:00:00Z",
      "reason": "Clear skies with light winds, excellent training conditions",
      "weather_score": 9.5,
      "instructor_available": true
    }}
  ]
}}
"#,
            student.name,
            student.training_level,
            booking.scheduled_date.format("%Y-%m-%d %H:%M UTC"),
            booking.departure_location.name,
            weather_summary,
            student.training_level
        )
    }

    async fn generate_fallback_options(
        &self,
        booking: &Booking,
        student: &Student,
        weather_forecast: &[WeatherData],
        _instructor_schedule: &[Booking],
    ) -> Result<Vec<RescheduleOption>> {
        use crate::weather::{calculate_weather_score, default_weather_minimums};

        let minimums = default_weather_minimums();
        let student_minimums = minimums
            .get(&student.training_level)
            .context("No minimums for training level")?;

        let mut options = Vec::new();

        for weather in weather_forecast.iter().take(14) {
            if options.len() >= 3 {
                break;
            }

            let (is_safe, _) = is_flight_safe(&student.training_level, weather, student_minimums);

            if is_safe {
                let score = calculate_weather_score(&student.training_level, weather);
                options.push(RescheduleOption {
                    date_time: weather.date_time,
                    reason: format!("Good weather conditions: {} with {:.0}kt winds", weather.conditions, weather.wind_speed_knots),
                    weather_score: score,
                    instructor_available: true, // Simplified assumption
                });
            }
        }

        // If still not enough options, add marginal weather days
        if options.len() < 3 {
            for weather in weather_forecast.iter().skip(options.len()).take(3 - options.len()) {
                let score = calculate_weather_score(&student.training_level, weather);
                options.push(RescheduleOption {
                    date_time: weather.date_time,
                    reason: format!("Marginal conditions: {}", weather.conditions),
                    weather_score: score,
                    instructor_available: true,
                });
            }
        }

        // If STILL not enough options (forecast too short), add placeholder options
        while options.len() < 3 {
            let days_ahead = options.len() + 1;
            let placeholder_date = booking.scheduled_date + chrono::Duration::days(days_ahead as i64);
            options.push(RescheduleOption {
                date_time: placeholder_date,
                reason: "Please contact your instructor to schedule - limited weather data available".to_string(),
                weather_score: 5.0,
                instructor_available: false,
            });
        }

        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BookingStatus, Location, TrainingLevel};

    fn create_test_booking() -> Booking {
        Booking {
            id: "test123".to_string(),
            student_id: "student1".to_string(),
            scheduled_date: Utc::now(),
            departure_location: Location {
                lat: 33.8113,
                lon: -118.1515,
                name: "KTOA".to_string(),
            },
            status: BookingStatus::Scheduled,
        }
    }

    fn create_test_student() -> Student {
        Student {
            id: "student1".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            phone: "+1234567890".to_string(),
            training_level: TrainingLevel::StudentPilot,
        }
    }

    fn create_test_weather() -> Vec<WeatherData> {
        vec![
            WeatherData {
                visibility_miles: 10.0,
                wind_speed_knots: 5.0,
                ceiling_ft: Some(5000.0),
                temperature_f: 65.0,
                conditions: "Clear".to_string(),
                has_thunderstorms: false,
                has_icing: false,
                date_time: Utc::now(),
            },
            WeatherData {
                visibility_miles: 8.0,
                wind_speed_knots: 8.0,
                ceiling_ft: Some(4000.0),
                temperature_f: 68.0,
                conditions: "Partly Cloudy".to_string(),
                has_thunderstorms: false,
                has_icing: false,
                date_time: Utc::now() + chrono::Duration::hours(24),
            },
            WeatherData {
                visibility_miles: 6.0,
                wind_speed_knots: 10.0,
                ceiling_ft: Some(3500.0),
                temperature_f: 70.0,
                conditions: "Scattered Clouds".to_string(),
                has_thunderstorms: false,
                has_icing: false,
                date_time: Utc::now() + chrono::Duration::hours(48),
            },
        ]
    }

    #[tokio::test]
    async fn test_cache() {
        let cache = AiCache::new();
        let key = "test_key".to_string();

        // Cache miss
        assert!(cache.get(&key).await.is_none());

        // Set cache
        let response = RescheduleResponse {
            options: vec![],
        };
        cache.set(key.clone(), response.clone()).await;

        // Cache hit
        assert!(cache.get(&key).await.is_some());
    }

    #[tokio::test]
    async fn test_fallback_generation() {
        let cache = Arc::new(AiCache::new());
        // Use dummy key since we won't make real API calls
        let client = AiRescheduleClient::new("dummy_key".to_string(), cache);

        let booking = create_test_booking();
        let student = create_test_student();
        let weather = create_test_weather();

        let options = client
            .generate_fallback_options(&booking, &student, &weather, &[])
            .await
            .unwrap();

        assert_eq!(options.len(), 3);
        assert!(options[0].weather_score > 0.0);
    }
}
