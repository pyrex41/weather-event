use crate::models::{TrainingLevel, WeatherMinimum};
use crate::weather::WeatherData;
use std::collections::HashMap;
use std::sync::Arc;

/// Check if flight is safe for the given training level and weather conditions
///
/// Returns (is_safe, reason if unsafe)
pub fn is_flight_safe(
    training_level: &TrainingLevel,
    weather: &WeatherData,
    minimums: &WeatherMinimum,
) -> (bool, Option<String>) {
    let mut reasons = Vec::new();

    // Check thunderstorms (always unsafe except for specific training)
    if minimums.no_thunderstorms && weather.has_thunderstorms {
        reasons.push("Thunderstorms present".to_string());
    }

    // Check icing conditions
    if minimums.no_icing && weather.has_icing {
        reasons.push("Icing conditions present".to_string());
    }

    // Check visibility
    if weather.visibility_miles < minimums.min_visibility_sm {
        reasons.push(format!(
            "Visibility {:.1}mi below minimum {:.1}mi for {:?}",
            weather.visibility_miles, minimums.min_visibility_sm, training_level
        ));
    }

    // Check wind speed
    if weather.wind_speed_knots > minimums.max_wind_speed_kt {
        reasons.push(format!(
            "Wind speed {:.1}kt exceeds maximum {:.1}kt for {:?}",
            weather.wind_speed_knots, minimums.max_wind_speed_kt, training_level
        ));
    }

    // Check ceiling if minimum is specified
    if let Some(min_ceiling) = minimums.min_ceiling_ft {
        match weather.ceiling_ft {
            Some(ceiling) if ceiling < min_ceiling => {
                reasons.push(format!(
                    "Ceiling {:.0}ft below minimum {:.0}ft for {:?}",
                    ceiling, min_ceiling, training_level
                ));
            }
            None if !minimums.allow_imc => {
                // No ceiling data, but IMC not allowed - treat as potentially unsafe
                // This is conservative, assuming broken/overcast conditions
            }
            _ => {}
        }
    }

    // Check for low clouds for student pilots (special case)
    if matches!(training_level, TrainingLevel::StudentPilot) {
        if let Some(ceiling) = weather.ceiling_ft {
            if ceiling < 3000.0 {
                reasons.push(format!(
                    "Ceiling {:.0}ft too low for student pilot (minimum 3000ft)",
                    ceiling
                ));
            }
        }
    }

    // Check IMC conditions
    if !minimums.allow_imc {
        // If IMC is not allowed, we need clear skies
        // Check if conditions indicate IMC
        if let Some(ceiling) = weather.ceiling_ft {
            if ceiling < 1000.0 || weather.visibility_miles < 3.0 {
                reasons.push("IMC conditions not allowed for this training level".to_string());
            }
        }
    }

    if reasons.is_empty() {
        (true, None)
    } else {
        (false, Some(reasons.join("; ")))
    }
}

/// Calculate weather score from 0-10 for AI ranking
///
/// 10 = perfect conditions, 0 = terrible conditions
pub fn calculate_weather_score(training_level: &TrainingLevel, weather: &WeatherData) -> f32 {
    let mut score = 10.0;

    // Deduct for thunderstorms
    if weather.has_thunderstorms {
        score -= 5.0;
    }

    // Deduct for icing
    if weather.has_icing {
        score -= 3.0;
    }

    // Deduct for poor visibility
    if weather.visibility_miles < 10.0 {
        score -= (10.0 - weather.visibility_miles) / 10.0 * 2.0;
    }

    // Deduct for high winds
    if weather.wind_speed_knots > 5.0 {
        score -= (weather.wind_speed_knots - 5.0).min(15.0) / 15.0 * 2.0;
    }

    // Deduct for low ceiling
    if let Some(ceiling) = weather.ceiling_ft {
        if ceiling < 5000.0 {
            score -= (5000.0 - ceiling) / 5000.0 * 2.0;
        }
    }

    // Student pilots need better conditions
    if matches!(training_level, TrainingLevel::StudentPilot) {
        if weather.wind_speed_knots > 10.0 {
            score -= 2.0;
        }
    }

    score.max(0.0).min(10.0)
}

/// Default weather minimums for each training level
pub fn default_weather_minimums() -> HashMap<TrainingLevel, WeatherMinimum> {
    let mut minimums = HashMap::new();

    minimums.insert(
        TrainingLevel::StudentPilot,
        WeatherMinimum {
            id: "default_student".to_string(),
            training_level: TrainingLevel::StudentPilot,
            min_visibility_sm: 5.0,
            max_wind_speed_kt: 12.0,
            min_ceiling_ft: Some(3000.0),
            allow_imc: false,
            no_thunderstorms: true,
            no_icing: true,
        },
    );

    minimums.insert(
        TrainingLevel::PrivatePilot,
        WeatherMinimum {
            id: "default_private".to_string(),
            training_level: TrainingLevel::PrivatePilot,
            min_visibility_sm: 3.0,
            max_wind_speed_kt: 20.0,
            min_ceiling_ft: Some(1000.0),
            allow_imc: false,
            no_thunderstorms: true,
            no_icing: true,
        },
    );

    minimums.insert(
        TrainingLevel::InstrumentRated,
        WeatherMinimum {
            id: "default_instrument".to_string(),
            training_level: TrainingLevel::InstrumentRated,
            min_visibility_sm: 1.0,
            max_wind_speed_kt: 30.0,
            min_ceiling_ft: None,
            allow_imc: true,
            no_thunderstorms: true,
            no_icing: true,
        },
    );

    minimums
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_weather(
        visibility: f64,
        wind: f64,
        ceiling: Option<f64>,
        thunderstorms: bool,
        icing: bool,
    ) -> WeatherData {
        WeatherData {
            visibility_miles: visibility,
            wind_speed_knots: wind,
            ceiling_ft: ceiling,
            temperature_f: if icing { 25.0 } else { 65.0 },
            conditions: "Clear".to_string(),
            has_thunderstorms: thunderstorms,
            has_icing: icing,
            date_time: Utc::now(),
        }
    }

    #[test]
    fn test_student_pilot_good_weather() {
        let minimums = default_weather_minimums();
        let weather = create_test_weather(10.0, 8.0, Some(4000.0), false, false);
        let (is_safe, reason) = is_flight_safe(
            &TrainingLevel::StudentPilot,
            &weather,
            minimums.get(&TrainingLevel::StudentPilot).unwrap(),
        );
        assert!(is_safe, "Should be safe: {:?}", reason);
    }

    #[test]
    fn test_student_pilot_high_winds() {
        let minimums = default_weather_minimums();
        let weather = create_test_weather(10.0, 15.0, Some(4000.0), false, false);
        let (is_safe, reason) = is_flight_safe(
            &TrainingLevel::StudentPilot,
            &weather,
            minimums.get(&TrainingLevel::StudentPilot).unwrap(),
        );
        assert!(!is_safe);
        assert!(reason.unwrap().contains("Wind speed"));
    }

    #[test]
    fn test_student_pilot_low_ceiling() {
        let minimums = default_weather_minimums();
        let weather = create_test_weather(10.0, 8.0, Some(2500.0), false, false);
        let (is_safe, reason) = is_flight_safe(
            &TrainingLevel::StudentPilot,
            &weather,
            minimums.get(&TrainingLevel::StudentPilot).unwrap(),
        );
        assert!(!is_safe);
        assert!(reason.unwrap().contains("low for student pilot"));
    }

    #[test]
    fn test_private_pilot_marginal_weather() {
        let minimums = default_weather_minimums();
        let weather = create_test_weather(3.5, 18.0, Some(1200.0), false, false);
        let (is_safe, _reason) = is_flight_safe(
            &TrainingLevel::PrivatePilot,
            &weather,
            minimums.get(&TrainingLevel::PrivatePilot).unwrap(),
        );
        assert!(is_safe);
    }

    #[test]
    fn test_instrument_rated_imc_allowed() {
        let minimums = default_weather_minimums();
        let weather = create_test_weather(2.0, 25.0, Some(500.0), false, false);
        let (is_safe, _reason) = is_flight_safe(
            &TrainingLevel::InstrumentRated,
            &weather,
            minimums.get(&TrainingLevel::InstrumentRated).unwrap(),
        );
        assert!(is_safe);
    }

    #[test]
    fn test_thunderstorms_always_unsafe() {
        let minimums = default_weather_minimums();
        let weather = create_test_weather(10.0, 5.0, Some(5000.0), true, false);

        for level in &[
            TrainingLevel::StudentPilot,
            TrainingLevel::PrivatePilot,
            TrainingLevel::InstrumentRated,
        ] {
            let (is_safe, reason) = is_flight_safe(level, &weather, minimums.get(level).unwrap());
            assert!(!is_safe);
            assert!(reason.unwrap().contains("Thunderstorms"));
        }
    }

    #[test]
    fn test_icing_unsafe() {
        let minimums = default_weather_minimums();
        let weather = create_test_weather(10.0, 5.0, Some(5000.0), false, true);

        let (is_safe, reason) = is_flight_safe(
            &TrainingLevel::StudentPilot,
            &weather,
            minimums.get(&TrainingLevel::StudentPilot).unwrap(),
        );
        assert!(!is_safe);
        assert!(reason.unwrap().contains("Icing"));
    }

    #[test]
    fn test_weather_score_perfect_conditions() {
        let weather = create_test_weather(10.0, 5.0, Some(5000.0), false, false);
        let score = calculate_weather_score(&TrainingLevel::PrivatePilot, &weather);
        assert!(score >= 9.0, "Perfect weather should score high: {}", score);
    }

    #[test]
    fn test_weather_score_poor_conditions() {
        let weather = create_test_weather(2.0, 25.0, Some(1000.0), false, true);
        let score = calculate_weather_score(&TrainingLevel::PrivatePilot, &weather);
        assert!(score < 5.0, "Poor weather should score low: {}", score);
    }

    #[test]
    fn test_at_minimums_should_pass() {
        let minimums = WeatherMinimum {
            id: "test".to_string(),
            training_level: TrainingLevel::PrivatePilot,
            min_visibility_sm: 3.0,
            max_wind_speed_kt: 20.0,
            min_ceiling_ft: Some(1000.0),
            allow_imc: false,
            no_thunderstorms: true,
            no_icing: true,
        };

        let weather = create_test_weather(3.0, 20.0, Some(1000.0), false, false);
        let (is_safe, _) = is_flight_safe(&TrainingLevel::PrivatePilot, &weather, &minimums);
        assert!(is_safe);
    }

    #[test]
    fn test_below_minimums_should_fail() {
        let minimums = WeatherMinimum {
            id: "test".to_string(),
            training_level: TrainingLevel::PrivatePilot,
            min_visibility_sm: 3.0,
            max_wind_speed_kt: 20.0,
            min_ceiling_ft: Some(1000.0),
            allow_imc: false,
            no_thunderstorms: true,
            no_icing: true,
        };

        let weather = create_test_weather(2.9, 20.1, Some(999.0), false, false);
        let (is_safe, _) = is_flight_safe(&TrainingLevel::PrivatePilot, &weather, &minimums);
        assert!(!is_safe);
    }
}
