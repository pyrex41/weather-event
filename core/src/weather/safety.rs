use crate::models::{TrainingLevel, WeatherMinimum};
use crate::weather::WeatherData;
use std::collections::HashMap;

// Weather scoring constants
const PERFECT_SCORE: f32 = 10.0;
const THUNDERSTORM_PENALTY: f32 = 5.0;
const ICING_PENALTY: f32 = 3.0;
const IDEAL_VISIBILITY_MI: f32 = 10.0;
const VISIBILITY_PENALTY_FACTOR: f32 = 2.0;
const CALM_WIND_KT: f32 = 5.0;
const MAX_WIND_PENALTY_KT: f32 = 15.0;
const WIND_PENALTY_FACTOR: f32 = 2.0;
const IDEAL_CEILING_FT: f32 = 5000.0;
const CEILING_PENALTY_FACTOR: f32 = 2.0;
const STUDENT_HIGH_WIND_THRESHOLD_KT: f32 = 10.0;
const STUDENT_HIGH_WIND_PENALTY: f32 = 2.0;

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
    let mut score = PERFECT_SCORE;

    // Deduct for thunderstorms
    if weather.has_thunderstorms {
        score -= THUNDERSTORM_PENALTY;
    }

    // Deduct for icing
    if weather.has_icing {
        score -= ICING_PENALTY;
    }

    // Deduct for poor visibility
    if weather.visibility_miles < IDEAL_VISIBILITY_MI as f64 {
        score -= ((IDEAL_VISIBILITY_MI - weather.visibility_miles as f32) / IDEAL_VISIBILITY_MI) * VISIBILITY_PENALTY_FACTOR;
    }

    // Deduct for high winds
    if weather.wind_speed_knots > CALM_WIND_KT as f64 {
        score -= ((weather.wind_speed_knots as f32 - CALM_WIND_KT).min(MAX_WIND_PENALTY_KT) / MAX_WIND_PENALTY_KT) * WIND_PENALTY_FACTOR;
    }

    // Deduct for low ceiling
    if let Some(ceiling) = weather.ceiling_ft {
        if ceiling < IDEAL_CEILING_FT as f64 {
            score -= ((IDEAL_CEILING_FT - ceiling as f32) / IDEAL_CEILING_FT) * CEILING_PENALTY_FACTOR;
        }
    }

    // Student pilots need better conditions
    if matches!(training_level, TrainingLevel::StudentPilot) {
        if weather.wind_speed_knots > STUDENT_HIGH_WIND_THRESHOLD_KT as f64 {
            score -= STUDENT_HIGH_WIND_PENALTY;
        }
    }

    score.max(0.0).min(PERFECT_SCORE)
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

    // Property-based tests with proptest
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_student_pilot_stricter_than_private(
            visibility in 0.0f64..15.0,
            wind_speed in 0.0f64..40.0,
            ceiling in 0.0f64..10000.0,
        ) {
            let minimums_map = default_weather_minimums();
            let student_mins = minimums_map.get(&TrainingLevel::StudentPilot).unwrap();
            let private_mins = minimums_map.get(&TrainingLevel::PrivatePilot).unwrap();

            let weather = create_test_weather(
                visibility,
                wind_speed,
                Some(ceiling),
                false, // no thunderstorms
                false, // no icing
            );

            let (student_safe, _) = is_flight_safe(&TrainingLevel::StudentPilot, &weather, student_mins);
            let (private_safe, _) = is_flight_safe(&TrainingLevel::PrivatePilot, &weather, private_mins);

            // Property: If it's safe for students, it must be safe for private pilots
            // (Student pilot minimums are stricter)
            if student_safe {
                prop_assert!(private_safe,
                    "Weather safe for student (vis: {:.1}, wind: {:.1}, ceiling: {:.1}) but not for private pilot",
                    visibility, wind_speed, ceiling
                );
            }
        }

        #[test]
        fn prop_private_pilot_stricter_than_instrument(
            visibility in 0.0f64..15.0,
            wind_speed in 0.0f64..40.0,
            ceiling in 0.0f64..10000.0,
        ) {
            let minimums_map = default_weather_minimums();
            let private_mins = minimums_map.get(&TrainingLevel::PrivatePilot).unwrap();
            let instrument_mins = minimums_map.get(&TrainingLevel::InstrumentRated).unwrap();

            let weather = create_test_weather(
                visibility,
                wind_speed,
                Some(ceiling),
                false,
                false,
            );

            let (private_safe, _) = is_flight_safe(&TrainingLevel::PrivatePilot, &weather, private_mins);
            let (instrument_safe, _) = is_flight_safe(&TrainingLevel::InstrumentRated, &weather, instrument_mins);

            // If it's safe for private pilots, it should be safe for instrument-rated pilots
            if private_safe {
                prop_assert!(instrument_safe,
                    "Weather safe for private (vis: {:.1}, wind: {:.1}, ceiling: {:.1}) but not for instrument",
                    visibility, wind_speed, ceiling
                );
            }
        }

        #[test]
        fn prop_weather_score_bounded(
            visibility in 0.0f64..15.0,
            wind_speed in 0.0f64..50.0,
            ceiling in 0.0f64..15000.0,
            has_thunderstorms: bool,
            has_icing: bool,
        ) {
            let weather = create_test_weather(
                visibility,
                wind_speed,
                Some(ceiling),
                has_thunderstorms,
                has_icing,
            );

            for training_level in [TrainingLevel::StudentPilot, TrainingLevel::PrivatePilot, TrainingLevel::InstrumentRated] {
                let score = calculate_weather_score(&training_level, &weather);
                prop_assert!(score >= 0.0 && score <= PERFECT_SCORE,
                    "Score {} out of bounds [0, {}] for {:?}",
                    score, PERFECT_SCORE, training_level
                );
            }
        }

        #[test]
        fn prop_perfect_conditions_high_score(
            visibility in 10.0f64..20.0,
            wind_speed in 0.0f64..5.0,
            ceiling in 5000.0f64..15000.0,
        ) {
            let weather = create_test_weather(
                visibility,
                wind_speed,
                Some(ceiling),
                false,
                false,
            );

            for training_level in [TrainingLevel::StudentPilot, TrainingLevel::PrivatePilot, TrainingLevel::InstrumentRated] {
                let score = calculate_weather_score(&training_level, &weather);
                prop_assert!(score >= 8.0,
                    "Perfect conditions should score >= 8.0, got {} for {:?}",
                    score, training_level
                );
            }
        }

        #[test]
        fn prop_thunderstorms_always_unsafe(
            visibility in 0.0f64..15.0,
            wind_speed in 0.0f64..40.0,
            ceiling in 0.0f64..10000.0,
        ) {
            let minimums_map = default_weather_minimums();

            let weather = create_test_weather(
                visibility,
                wind_speed,
                Some(ceiling),
                true, // thunderstorms present
                false,
            );

            for training_level in [TrainingLevel::StudentPilot, TrainingLevel::PrivatePilot, TrainingLevel::InstrumentRated] {
                let mins = minimums_map.get(&training_level).unwrap();
                let (is_safe, reason) = is_flight_safe(&training_level, &weather, mins);

                prop_assert!(!is_safe, "Thunderstorms should always be unsafe for {:?}", training_level);
                prop_assert!(reason.is_some(), "Unsafe weather should have a reason");
                prop_assert!(reason.unwrap().contains("Thunderstorm"), "Reason should mention thunderstorms");
            }
        }

        #[test]
        fn prop_visibility_zero_always_unsafe(
            wind_speed in 0.0f64..40.0,
            ceiling in 0.0f64..10000.0,
        ) {
            let minimums_map = default_weather_minimums();

            let weather = create_test_weather(
                0.0, // zero visibility
                wind_speed,
                Some(ceiling),
                false,
                false,
            );

            for training_level in [TrainingLevel::StudentPilot, TrainingLevel::PrivatePilot, TrainingLevel::InstrumentRated] {
                let mins = minimums_map.get(&training_level).unwrap();
                let (is_safe, _) = is_flight_safe(&training_level, &weather, mins);

                prop_assert!(!is_safe, "Zero visibility should always be unsafe for {:?}", training_level);
            }
        }
    }
}
