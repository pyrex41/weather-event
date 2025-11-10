use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, AppState};

#[derive(Deserialize)]
pub struct WeatherQuery {
    lat: f64,
    lon: f64,
}

#[derive(Serialize)]
pub struct WeatherResponse {
    pub location: String,
    pub temperature_f: f64,
    pub conditions: String,
    pub visibility_miles: f64,
    pub wind_speed_knots: f64,
    pub ceiling_ft: Option<f64>,
    pub has_thunderstorms: bool,
    pub has_icing: bool,
}

pub async fn get_weather(
    Query(params): Query<WeatherQuery>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<WeatherResponse>, ApiError> {
    tracing::info!("Weather route called with lat={}, lon={}", params.lat, params.lon);

    // Debug: Check weather client configuration
    tracing::debug!("Weather client base_url: {}", state.weather_client.base_url());
    tracing::debug!("Weather client api_key length: {}", state.weather_client.api_key().len());

    let weather_data = state
        .weather_client
        .fetch_current_weather(params.lat, params.lon)
        .await
        .map_err(|e| {
            tracing::error!("Weather API error for lat={}, lon={}: {}", params.lat, params.lon, e);
            ApiError::external_api_error("OpenWeatherMap", format!("Unable to fetch weather data: {}", e))
        })?;

    let response = WeatherResponse {
        location: format!("{:.4},{:.4}", params.lat, params.lon),
        temperature_f: weather_data.temperature_f,
        conditions: weather_data.conditions.clone(),
        visibility_miles: weather_data.visibility_miles,
        wind_speed_knots: weather_data.wind_speed_knots,
        ceiling_ft: weather_data.ceiling_ft,
        has_thunderstorms: weather_data.has_thunderstorms,
        has_icing: weather_data.has_icing,
    };

    Ok(Json(response))
}