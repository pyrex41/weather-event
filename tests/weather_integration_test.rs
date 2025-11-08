use weather_core::weather::{calculate_weather_score, default_weather_minimums, is_flight_safe, WeatherData};
use weather_core::models::TrainingLevel;
use chrono::Utc;

#[test]
fn test_student_pilot_weather_safety_integration() {
    let minimums = default_weather_minimums();
    let student_minimums = minimums.get(&TrainingLevel::StudentPilot).unwrap();

    // Test 1: Perfect weather - should be safe
    let perfect_weather = WeatherData {
        visibility_miles: 10.0,
        wind_speed_knots: 8.0,
        ceiling_ft: Some(5000.0),
        temperature_f: 70.0,
        conditions: "Clear skies".to_string(),
        has_thunderstorms: false,
        has_icing: false,
        date_time: Utc::now(),
    };

    let (is_safe, reason) = is_flight_safe(
        &TrainingLevel::StudentPilot,
        &perfect_weather,
        student_minimums,
    );
    assert!(is_safe, "Perfect weather should be safe for student pilot: {:?}", reason);

    let score = calculate_weather_score(&TrainingLevel::StudentPilot, &perfect_weather);
    assert!(score >= 8.0, "Perfect weather should score high: {}", score);

    // Test 2: Marginal weather - borderline
    let marginal_weather = WeatherData {
        visibility_miles: 5.0, // At minimum
        wind_speed_knots: 12.0, // At maximum
        ceiling_ft: Some(3000.0), // At minimum
        temperature_f: 65.0,
        conditions: "Scattered clouds".to_string(),
        has_thunderstorms: false,
        has_icing: false,
        date_time: Utc::now(),
    };

    let (is_safe, _) = is_flight_safe(
        &TrainingLevel::StudentPilot,
        &marginal_weather,
        student_minimums,
    );
    assert!(is_safe, "Marginal weather at minimums should be safe");

    // Test 3: Unsafe weather - high winds
    let unsafe_weather = WeatherData {
        visibility_miles: 10.0,
        wind_speed_knots: 15.0, // Above maximum
        ceiling_ft: Some(5000.0),
        temperature_f: 70.0,
        conditions: "Clear".to_string(),
        has_thunderstorms: false,
        has_icing: false,
        date_time: Utc::now(),
    };

    let (is_safe, reason) = is_flight_safe(
        &TrainingLevel::StudentPilot,
        &unsafe_weather,
        student_minimums,
    );
    assert!(!is_safe, "High winds should be unsafe for student pilot");
    assert!(reason.unwrap().contains("Wind"), "Reason should mention wind");

    // Test 4: Thunderstorms - always unsafe
    let thunderstorm_weather = WeatherData {
        visibility_miles: 10.0,
        wind_speed_knots: 8.0,
        ceiling_ft: Some(5000.0),
        temperature_f: 70.0,
        conditions: "Thunderstorms".to_string(),
        has_thunderstorms: true,
        has_icing: false,
        date_time: Utc::now(),
    };

    let (is_safe, reason) = is_flight_safe(
        &TrainingLevel::StudentPilot,
        &thunderstorm_weather,
        student_minimums,
    );
    assert!(!is_safe, "Thunderstorms should always be unsafe");
    assert!(reason.unwrap().contains("Thunderstorms"), "Reason should mention thunderstorms");
}

#[test]
fn test_training_level_progression() {
    let minimums = default_weather_minimums();

    // Marginal weather that's unsafe for student but safe for private pilot
    let marginal_weather = WeatherData {
        visibility_miles: 4.0,
        wind_speed_knots: 15.0,
        ceiling_ft: Some(2000.0),
        temperature_f: 65.0,
        conditions: "Overcast".to_string(),
        has_thunderstorms: false,
        has_icing: false,
        date_time: Utc::now(),
    };

    // Student pilot - should be unsafe
    let (student_safe, _) = is_flight_safe(
        &TrainingLevel::StudentPilot,
        &marginal_weather,
        minimums.get(&TrainingLevel::StudentPilot).unwrap(),
    );
    assert!(!student_safe, "Marginal weather should be unsafe for student pilot");

    // Private pilot - should be safe
    let (private_safe, _) = is_flight_safe(
        &TrainingLevel::PrivatePilot,
        &marginal_weather,
        minimums.get(&TrainingLevel::PrivatePilot).unwrap(),
    );
    assert!(private_safe, "Marginal weather should be safe for private pilot");

    // Instrument rated - should definitely be safe
    let (instrument_safe, _) = is_flight_safe(
        &TrainingLevel::InstrumentRated,
        &marginal_weather,
        minimums.get(&TrainingLevel::InstrumentRated).unwrap(),
    );
    assert!(instrument_safe, "Marginal weather should be safe for instrument rated");
}

#[test]
fn test_weather_scoring_consistency() {
    let weather_scenarios = vec![
        (
            WeatherData {
                visibility_miles: 10.0,
                wind_speed_knots: 5.0,
                ceiling_ft: Some(8000.0),
                temperature_f: 70.0,
                conditions: "Clear".to_string(),
                has_thunderstorms: false,
                has_icing: false,
                date_time: Utc::now(),
            },
            "perfect",
        ),
        (
            WeatherData {
                visibility_miles: 5.0,
                wind_speed_knots: 12.0,
                ceiling_ft: Some(3000.0),
                temperature_f: 60.0,
                conditions: "Scattered clouds".to_string(),
                has_thunderstorms: false,
                has_icing: false,
                date_time: Utc::now(),
            },
            "good",
        ),
        (
            WeatherData {
                visibility_miles: 3.0,
                wind_speed_knots: 18.0,
                ceiling_ft: Some(1500.0),
                temperature_f: 55.0,
                conditions: "Overcast".to_string(),
                has_thunderstorms: false,
                has_icing: false,
                date_time: Utc::now(),
            },
            "marginal",
        ),
        (
            WeatherData {
                visibility_miles: 1.0,
                wind_speed_knots: 25.0,
                ceiling_ft: Some(500.0),
                temperature_f: 28.0,
                conditions: "Rain".to_string(),
                has_thunderstorms: false,
                has_icing: true,
                date_time: Utc::now(),
            },
            "poor",
        ),
    ];

    let mut last_score = 10.0;
    for (weather, category) in weather_scenarios {
        let score = calculate_weather_score(&TrainingLevel::PrivatePilot, &weather);
        assert!(
            score <= last_score,
            "Weather score should decrease as conditions worsen: {} ({}) should be <= previous score {}",
            score,
            category,
            last_score
        );
        assert!(
            score >= 0.0 && score <= 10.0,
            "Weather score should be between 0 and 10: {}",
            score
        );
        last_score = score;
    }
}

#[test]
fn test_edge_cases() {
    let minimums = default_weather_minimums();
    let student_minimums = minimums.get(&TrainingLevel::StudentPilot).unwrap();

    // Edge case 1: Exactly at minimums
    let at_minimums = WeatherData {
        visibility_miles: 5.0, // Exactly at minimum
        wind_speed_knots: 12.0, // Exactly at maximum
        ceiling_ft: Some(3000.0), // Exactly at minimum
        temperature_f: 65.0,
        conditions: "Clear".to_string(),
        has_thunderstorms: false,
        has_icing: false,
        date_time: Utc::now(),
    };

    let (is_safe, _) = is_flight_safe(
        &TrainingLevel::StudentPilot,
        &at_minimums,
        student_minimums,
    );
    assert!(is_safe, "Weather exactly at minimums should be safe");

    // Edge case 2: Just below minimums
    let below_minimums = WeatherData {
        visibility_miles: 4.9, // Just below minimum
        wind_speed_knots: 12.1, // Just above maximum
        ceiling_ft: Some(2999.0), // Just below minimum
        temperature_f: 65.0,
        conditions: "Clear".to_string(),
        has_thunderstorms: false,
        has_icing: false,
        date_time: Utc::now(),
    };

    let (is_safe, _) = is_flight_safe(
        &TrainingLevel::StudentPilot,
        &below_minimums,
        student_minimums,
    );
    assert!(!is_safe, "Weather just below minimums should be unsafe");

    // Edge case 3: No ceiling data
    let no_ceiling = WeatherData {
        visibility_miles: 10.0,
        wind_speed_knots: 8.0,
        ceiling_ft: None, // Unlimited ceiling
        temperature_f: 70.0,
        conditions: "Clear".to_string(),
        has_thunderstorms: false,
        has_icing: false,
        date_time: Utc::now(),
    };

    let (is_safe, _) = is_flight_safe(
        &TrainingLevel::StudentPilot,
        &no_ceiling,
        student_minimums,
    );
    // This should be safe as unlimited ceiling is ideal
    assert!(is_safe, "Unlimited ceiling should be safe");
}

#[test]
fn test_multiple_violations() {
    let minimums = default_weather_minimums();
    let student_minimums = minimums.get(&TrainingLevel::StudentPilot).unwrap();

    // Weather with multiple violations
    let bad_weather = WeatherData {
        visibility_miles: 2.0, // Below minimum
        wind_speed_knots: 20.0, // Above maximum
        ceiling_ft: Some(1500.0), // Below minimum
        temperature_f: 25.0,
        conditions: "Low clouds".to_string(),
        has_thunderstorms: false,
        has_icing: true, // Icing conditions
        date_time: Utc::now(),
    };

    let (is_safe, reason) = is_flight_safe(
        &TrainingLevel::StudentPilot,
        &bad_weather,
        student_minimums,
    );

    assert!(!is_safe, "Multiple violations should result in unsafe");
    let reason_str = reason.unwrap();

    // Should mention multiple issues
    assert!(reason_str.contains("Visibility") || reason_str.contains("visibility"));
    // Should have multiple semicolon-separated reasons
    assert!(reason_str.contains(";"), "Should have multiple reasons separated by semicolons");
}
