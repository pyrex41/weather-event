use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use anyhow::{Context, Result};

const METERS_TO_MILES: f64 = 0.000621371;
const MS_TO_KNOTS: f64 = 1.94384;

/// Weather data normalized to aviation units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub visibility_miles: f64,
    pub wind_speed_knots: f64,
    pub ceiling_ft: Option<f64>,
    pub temperature_f: f64,
    pub conditions: String,
    pub has_thunderstorms: bool,
    pub has_icing: bool,
    pub date_time: DateTime<Utc>,
}

/// OpenWeatherMap API client
pub struct WeatherClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct OpenWeatherMapResponse {
    weather: Vec<WeatherCondition>,
    main: MainWeatherData,
    visibility: Option<f64>,
    wind: WindData,
    clouds: Option<CloudData>,
    dt: i64,
}

#[derive(Debug, Deserialize)]
struct WeatherCondition {
    main: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct MainWeatherData {
    temp: f64,
}

#[derive(Debug, Deserialize)]
struct WindData {
    speed: f64,
}

#[derive(Debug, Deserialize)]
struct CloudData {
    all: f64,
}

#[derive(Debug, Deserialize)]
struct ForecastResponse {
    list: Vec<OpenWeatherMapResponse>,
}

impl WeatherClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openweathermap.org/data/2.5".to_string()),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("WEATHER_API_KEY")
            .context("WEATHER_API_KEY environment variable not set")?;
        let base_url = std::env::var("WEATHER_API_BASE_URL").ok();

        Ok(Self::new(api_key, base_url))
    }

    pub async fn fetch_current_weather(&self, lat: f64, lon: f64) -> Result<WeatherData> {
        self.retry_with_backoff(|| self.fetch_current_weather_inner(lat, lon), 3).await
    }

    pub async fn fetch_forecast(&self, lat: f64, lon: f64) -> Result<Vec<WeatherData>> {
        self.retry_with_backoff(|| self.fetch_forecast_inner(lat, lon), 3).await
    }

    async fn fetch_current_weather_inner(&self, lat: f64, lon: f64) -> Result<WeatherData> {
        let url = format!(
            "{}/weather?lat={}&lon={}&appid={}",
            self.base_url, lat, lon, self.api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch current weather")?;

        if !response.status().is_success() {
            anyhow::bail!("Weather API returned status: {}", response.status());
        }

        let data: OpenWeatherMapResponse = response
            .json()
            .await
            .context("Failed to parse weather response")?;

        Ok(Self::convert_to_weather_data(data))
    }

    async fn fetch_forecast_inner(&self, lat: f64, lon: f64) -> Result<Vec<WeatherData>> {
        let url = format!(
            "{}/forecast?lat={}&lon={}&appid={}&cnt=56",
            self.base_url, lat, lon, self.api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch forecast")?;

        if !response.status().is_success() {
            anyhow::bail!("Weather API returned status: {}", response.status());
        }

        let data: ForecastResponse = response
            .json()
            .await
            .context("Failed to parse forecast response")?;

        Ok(data.list.into_iter().map(Self::convert_to_weather_data).collect())
    }

    fn convert_to_weather_data(data: OpenWeatherMapResponse) -> WeatherData {
        let visibility_miles = data.visibility.unwrap_or(10000.0) * METERS_TO_MILES;
        let wind_speed_knots = data.wind.speed * MS_TO_KNOTS;
        let temperature_f = kelvin_to_fahrenheit(data.main.temp);

        let conditions = data.weather.first()
            .map(|w| w.description.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let has_thunderstorms = data.weather.iter()
            .any(|w| w.main.to_lowercase().contains("thunderstorm"));

        // Icing risk: temperature below freezing and cloudy conditions
        let has_icing = temperature_f < 32.0 &&
            data.clouds.as_ref().map(|c| c.all > 50.0).unwrap_or(false);

        // Estimate ceiling from cloud data (simplified)
        let ceiling_ft = data.clouds.as_ref().and_then(|c| {
            if c.all > 80.0 {
                Some(2000.0) // Low clouds
            } else if c.all > 50.0 {
                Some(5000.0) // Mid clouds
            } else {
                None // Clear or scattered
            }
        });

        WeatherData {
            visibility_miles,
            wind_speed_knots,
            ceiling_ft,
            temperature_f,
            conditions,
            has_thunderstorms,
            has_icing,
            date_time: DateTime::from_timestamp(data.dt, 0).unwrap_or_else(Utc::now),
        }
    }

    async fn retry_with_backoff<F, Fut, T>(&self, mut f: F, max_attempts: u32) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;

        for attempt in 0..max_attempts {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_attempts - 1 {
                        let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }
}

fn kelvin_to_fahrenheit(kelvin: f64) -> f64 {
    (kelvin - 273.15) * 9.0 / 5.0 + 32.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversions() {
        // 0 meters = 0 miles
        assert_eq!(0.0 * METERS_TO_MILES, 0.0);

        // 10 m/s ≈ 19.4 knots
        let knots = 10.0 * MS_TO_KNOTS;
        assert!((knots - 19.4).abs() < 0.1);

        // 273.15 K = 32°F (freezing point)
        let temp_f = kelvin_to_fahrenheit(273.15);
        assert!((temp_f - 32.0).abs() < 0.1);

        // 0 K = -459.67°F
        let absolute_zero = kelvin_to_fahrenheit(0.0);
        assert!((absolute_zero - (-459.67)).abs() < 0.1);
    }

    #[test]
    fn test_location_serialization() {
        use crate::models::Location;

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
}
