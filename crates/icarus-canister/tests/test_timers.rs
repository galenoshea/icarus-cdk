//! Tests for timer functionality

use icarus_canister::timers::{TimerError, TimerInfo, TimerType};
use serde_json;
use std::time::Duration;

/// Test TimerInfo creation and serialization
#[test]
fn test_timer_info_creation() {
    let info = TimerInfo {
        id: 123,
        name: "test_timer".to_string(),
        timer_type: TimerType::Once,
        created_at: 1640995200000000000, // Jan 1, 2022 in nanoseconds
        interval_secs: Some(60),
    };

    assert_eq!(info.id, 123);
    assert_eq!(info.name, "test_timer");
    assert!(matches!(info.timer_type, TimerType::Once));
    assert_eq!(info.created_at, 1640995200000000000);
    assert_eq!(info.interval_secs, Some(60));
}

/// Test TimerInfo clone functionality
#[test]
fn test_timer_info_clone() {
    let info = TimerInfo {
        id: 456,
        name: "cloneable_timer".to_string(),
        timer_type: TimerType::Periodic,
        created_at: 1640995200000000000,
        interval_secs: Some(300),
    };

    let cloned = info.clone();
    assert_eq!(info.id, cloned.id);
    assert_eq!(info.name, cloned.name);
    assert!(matches!(cloned.timer_type, TimerType::Periodic));
    assert_eq!(info.created_at, cloned.created_at);
    assert_eq!(info.interval_secs, cloned.interval_secs);
}

/// Test TimerInfo debug formatting
#[test]
fn test_timer_info_debug() {
    let info = TimerInfo {
        id: 789,
        name: "debug_timer".to_string(),
        timer_type: TimerType::Once,
        created_at: 1640995200000000000,
        interval_secs: Some(120),
    };

    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("TimerInfo"));
    assert!(debug_str.contains("789"));
    assert!(debug_str.contains("debug_timer"));
    assert!(debug_str.contains("Once"));
    assert!(debug_str.contains("120"));
}

/// Test TimerType variants
#[test]
fn test_timer_type_variants() {
    let once = TimerType::Once;
    let periodic = TimerType::Periodic;

    // Test that they can be cloned
    let once_clone = once.clone();
    let periodic_clone = periodic.clone();

    assert!(matches!(once_clone, TimerType::Once));
    assert!(matches!(periodic_clone, TimerType::Periodic));

    // Test debug formatting
    assert_eq!(format!("{:?}", once), "Once");
    assert_eq!(format!("{:?}", periodic), "Periodic");
}

/// Test TimerInfo JSON serialization
#[test]
fn test_timer_info_json_serialization() {
    let info = TimerInfo {
        id: 100,
        name: "json_timer".to_string(),
        timer_type: TimerType::Periodic,
        created_at: 1640995200000000000,
        interval_secs: Some(180),
    };

    // Test serialization
    let json = serde_json::to_string(&info).expect("Should serialize to JSON");
    assert!(json.contains("json_timer"));
    assert!(json.contains("Periodic"));
    assert!(json.contains("100"));
    assert!(json.contains("180"));

    // Test deserialization
    let deserialized: TimerInfo = serde_json::from_str(&json).expect("Should deserialize from JSON");
    assert_eq!(deserialized.id, info.id);
    assert_eq!(deserialized.name, info.name);
    assert!(matches!(deserialized.timer_type, TimerType::Periodic));
    assert_eq!(deserialized.created_at, info.created_at);
    assert_eq!(deserialized.interval_secs, info.interval_secs);
}

/// Test TimerType JSON serialization
#[test]
fn test_timer_type_json_serialization() {
    let once = TimerType::Once;
    let periodic = TimerType::Periodic;

    // Test serialization
    let once_json = serde_json::to_string(&once).expect("Should serialize Once");
    let periodic_json = serde_json::to_string(&periodic).expect("Should serialize Periodic");

    assert_eq!(once_json, "\"Once\"");
    assert_eq!(periodic_json, "\"Periodic\"");

    // Test deserialization
    let once_deserialized: TimerType = serde_json::from_str(&once_json).expect("Should deserialize Once");
    let periodic_deserialized: TimerType = serde_json::from_str(&periodic_json).expect("Should deserialize Periodic");

    assert!(matches!(once_deserialized, TimerType::Once));
    assert!(matches!(periodic_deserialized, TimerType::Periodic));
}

/// Test TimerError variants and formatting
#[test]
fn test_timer_error_types() {
    // Test TooManyTimers error
    let too_many = TimerError::TooManyTimers { max: 100, current: 100 };
    assert_eq!(format!("{}", too_many), "Too many timers: 100/100");

    // Test TimerNotFound error
    let not_found = TimerError::TimerNotFound("timer_123".to_string());
    assert_eq!(format!("{}", not_found), "Timer not found: timer_123");

    // Test InvalidInterval error
    let invalid_interval = TimerError::InvalidInterval(0);
    assert_eq!(format!("{}", invalid_interval), "Invalid interval: 0 seconds");
}

/// Test TimerError debug formatting
#[test]
fn test_timer_error_debug() {
    let error = TimerError::TooManyTimers { max: 50, current: 45 };
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("TooManyTimers"));
    assert!(debug_str.contains("50"));
    assert!(debug_str.contains("45"));
}

/// Test TimerError as std::error::Error
#[test]
fn test_timer_error_std_error() {
    let error = TimerError::InvalidInterval(5);
    let error_trait: &dyn std::error::Error = &error;
    assert_eq!(error_trait.to_string(), "Invalid interval: 5 seconds");

    // Test source (should be None for our simple errors)
    assert!(error_trait.source().is_none());
}

/// Test TimerInfo with None interval
#[test]
fn test_timer_info_none_interval() {
    let info = TimerInfo {
        id: 999,
        name: "no_interval_timer".to_string(),
        timer_type: TimerType::Once,
        created_at: 1640995200000000000,
        interval_secs: None,
    };

    assert_eq!(info.interval_secs, None);

    // Test serialization with None
    let json = serde_json::to_string(&info).expect("Should serialize with None interval");
    assert!(json.contains("null"));

    let deserialized: TimerInfo = serde_json::from_str(&json).expect("Should deserialize with None interval");
    assert_eq!(deserialized.interval_secs, None);
}

/// Test TimerInfo with edge case values
#[test]
fn test_timer_info_edge_cases() {
    // Test with minimum values
    let min_info = TimerInfo {
        id: 0,
        name: "".to_string(),
        timer_type: TimerType::Once,
        created_at: 0,
        interval_secs: Some(1),
    };

    assert_eq!(min_info.id, 0);
    assert_eq!(min_info.name, "");
    assert_eq!(min_info.created_at, 0);
    assert_eq!(min_info.interval_secs, Some(1));

    // Test with maximum values
    let max_info = TimerInfo {
        id: u64::MAX,
        name: "a".repeat(1000), // Very long name
        timer_type: TimerType::Periodic,
        created_at: u64::MAX,
        interval_secs: Some(u64::MAX),
    };

    assert_eq!(max_info.id, u64::MAX);
    assert_eq!(max_info.name.len(), 1000);
    assert_eq!(max_info.created_at, u64::MAX);
    assert_eq!(max_info.interval_secs, Some(u64::MAX));
}

/// Test TimerError comprehensive error scenarios
#[test]
fn test_timer_error_comprehensive() {
    let errors = vec![
        TimerError::TooManyTimers { max: 1, current: 1 },
        TimerError::TooManyTimers { max: 100, current: 99 },
        TimerError::TimerNotFound("".to_string()),
        TimerError::TimerNotFound("very_long_timer_name_that_exceeds_normal_length".to_string()),
        TimerError::InvalidInterval(0),
        TimerError::InvalidInterval(u64::MAX),
    ];

    for error in errors {
        // Each error should format without panicking
        let display_str = format!("{}", error);
        let debug_str = format!("{:?}", error);

        assert!(!display_str.is_empty());
        assert!(!debug_str.is_empty());

        // Test that they implement std::error::Error
        let _: &dyn std::error::Error = &error;
    }
}

/// Test TimerInfo with special characters in name
#[test]
fn test_timer_info_special_characters() {
    let special_names = vec![
        "timer with spaces",
        "timer-with-dashes",
        "timer_with_underscores",
        "timer.with.dots",
        "timer/with/slashes",
        "timer\\with\\backslashes",
        "timer\"with\"quotes",
        "timer'with'apostrophes",
        "timer\nwith\nnewlines",
        "timer\twith\ttabs",
        "timer🚀with🎯emojis",
        "タイマー", // Japanese
        "계시기", // Korean
        "定时器", // Chinese
    ];

    for name in special_names {
        let info = TimerInfo {
            id: 1,
            name: name.to_string(),
            timer_type: TimerType::Once,
            created_at: 1640995200000000000,
            interval_secs: Some(60),
        };

        // Should be able to serialize and deserialize
        let json = serde_json::to_string(&info).expect("Should serialize special characters");
        let deserialized: TimerInfo = serde_json::from_str(&json).expect("Should deserialize special characters");
        assert_eq!(deserialized.name, name);
    }
}

/// Test TimerType exhaustive matching
#[test]
fn test_timer_type_exhaustive() {
    fn match_timer_type(timer_type: TimerType) -> &'static str {
        match timer_type {
            TimerType::Once => "once",
            TimerType::Periodic => "periodic",
        }
    }

    assert_eq!(match_timer_type(TimerType::Once), "once");
    assert_eq!(match_timer_type(TimerType::Periodic), "periodic");
}

/// Test large scale TimerInfo operations
#[test]
fn test_timer_info_large_scale() {
    let mut timers = Vec::new();

    // Create 1000 timer infos
    for i in 0..1000 {
        let info = TimerInfo {
            id: i,
            name: format!("timer_{}", i),
            timer_type: if i % 2 == 0 { TimerType::Once } else { TimerType::Periodic },
            created_at: 1640995200000000000 + i,
            interval_secs: Some(60 + i),
        };
        timers.push(info);
    }

    assert_eq!(timers.len(), 1000);

    // Test that we can serialize all of them
    let json = serde_json::to_string(&timers).expect("Should serialize large vector");
    assert!(json.len() > 10000); // Should be substantial

    // Test that we can deserialize all of them
    let deserialized: Vec<TimerInfo> = serde_json::from_str(&json).expect("Should deserialize large vector");
    assert_eq!(deserialized.len(), 1000);

    // Spot check some values
    assert_eq!(deserialized[0].id, 0);
    assert_eq!(deserialized[999].id, 999);
    assert!(matches!(deserialized[0].timer_type, TimerType::Once));
    assert!(matches!(deserialized[1].timer_type, TimerType::Periodic));
}

/// Test Duration conversion (for future async timer tests)
#[test]
fn test_duration_conversion() {
    // Test that we can convert seconds to Duration
    let one_sec = Duration::from_secs(1);
    let sixty_secs = Duration::from_secs(60);
    let one_hour = Duration::from_secs(3600);

    assert_eq!(one_sec.as_secs(), 1);
    assert_eq!(sixty_secs.as_secs(), 60);
    assert_eq!(one_hour.as_secs(), 3600);

    // Test Duration arithmetic
    let total = one_sec + sixty_secs + one_hour;
    assert_eq!(total.as_secs(), 3661); // 1 + 60 + 3600
}

/// Test timer interval validation logic
#[test]
fn test_interval_validation_logic() {
    // Test the same validation logic used in the timer functions
    fn validate_interval(interval_secs: u64) -> Result<(), TimerError> {
        if interval_secs == 0 {
            Err(TimerError::InvalidInterval(interval_secs))
        } else {
            Ok(())
        }
    }

    // Valid intervals
    assert!(validate_interval(1).is_ok());
    assert!(validate_interval(60).is_ok());
    assert!(validate_interval(3600).is_ok());
    assert!(validate_interval(u64::MAX).is_ok());

    // Invalid intervals
    assert!(validate_interval(0).is_err());
    if let Err(TimerError::InvalidInterval(val)) = validate_interval(0) {
        assert_eq!(val, 0);
    }
}

/// Test timer registry capacity logic
#[test]
fn test_timer_registry_capacity_logic() {
    const MAX_TIMERS: usize = 100;

    // Test the same capacity logic used in TimerRegistry
    fn check_capacity(current_count: usize) -> Result<(), TimerError> {
        if current_count >= MAX_TIMERS {
            Err(TimerError::TooManyTimers {
                max: MAX_TIMERS,
                current: current_count,
            })
        } else {
            Ok(())
        }
    }

    // Valid capacities
    assert!(check_capacity(0).is_ok());
    assert!(check_capacity(50).is_ok());
    assert!(check_capacity(99).is_ok());

    // Invalid capacities
    assert!(check_capacity(100).is_err());
    assert!(check_capacity(101).is_err());
    assert!(check_capacity(1000).is_err());

    if let Err(TimerError::TooManyTimers { max, current }) = check_capacity(100) {
        assert_eq!(max, MAX_TIMERS);
        assert_eq!(current, 100);
    }
}

/// Test backoff calculation logic
#[test]
fn test_backoff_calculation_logic() {
    // Test the backoff logic used in schedule_with_backoff
    fn calculate_next_delay(current_delay: u64, multiplier: f64) -> u64 {
        (current_delay as f64 * multiplier) as u64
    }

    // Test exponential backoff
    let mut delay = 1;
    delay = calculate_next_delay(delay, 2.0);
    assert_eq!(delay, 2);

    delay = calculate_next_delay(delay, 2.0);
    assert_eq!(delay, 4);

    delay = calculate_next_delay(delay, 2.0);
    assert_eq!(delay, 8);

    // Test with different multipliers
    let delay = calculate_next_delay(10, 1.5);
    assert_eq!(delay, 15);

    let delay = calculate_next_delay(100, 3.0);
    assert_eq!(delay, 300);
}

/// Test timer name generation patterns
#[test]
fn test_timer_name_patterns() {
    // Test common timer naming patterns
    let patterns = vec![
        "cleanup",
        "heartbeat",
        "backup",
        "health_check",
        "retry_failed_requests",
        "garbage_collection",
        "session_timeout",
        "cache_refresh",
        "log_rotation",
        "metrics_collection",
    ];

    for pattern in patterns {
        let info = TimerInfo {
            id: 1,
            name: pattern.to_string(),
            timer_type: TimerType::Periodic,
            created_at: 1640995200000000000,
            interval_secs: Some(300),
        };

        assert_eq!(info.name, pattern);
        assert!(!info.name.is_empty());
    }
}

/// Test high precision timestamp handling
#[test]
fn test_high_precision_timestamps() {
    // Test with nanosecond precision timestamps (as used by ic_cdk::api::time())
    let timestamps = vec![
        0,
        1_000_000_000,          // 1 second in nanoseconds
        1_640_995_200_000_000_000, // Jan 1, 2022 in nanoseconds
        u64::MAX,               // Maximum timestamp
    ];

    for timestamp in timestamps {
        let info = TimerInfo {
            id: 1,
            name: "timestamp_test".to_string(),
            timer_type: TimerType::Once,
            created_at: timestamp,
            interval_secs: Some(60),
        };

        assert_eq!(info.created_at, timestamp);

        // Should serialize/deserialize correctly
        let json = serde_json::to_string(&info).expect("Should serialize timestamp");
        let deserialized: TimerInfo = serde_json::from_str(&json).expect("Should deserialize timestamp");
        assert_eq!(deserialized.created_at, timestamp);
    }
}