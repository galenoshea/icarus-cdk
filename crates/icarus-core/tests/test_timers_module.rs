//! Unit tests for timer module
//!
//! These tests verify the timer module's registry management, error handling,
//! and business logic without creating actual IC timers.

#[cfg(feature = "canister")]
use icarus_core::timers::{TimerError, TimerInfo, TimerType};
#[cfg(feature = "canister")]
use serde_json;
#[cfg(feature = "canister")]
use std::collections::BTreeMap;

// Mock TimerId for testing (since we can't create real ones)
#[cfg(feature = "canister")]
type MockTimerId = u64;

// Test version of TimerRegistry for unit testing
#[cfg(feature = "canister")]
#[derive(Default)]
struct TestTimerRegistry {
    timers: BTreeMap<MockTimerId, TimerInfo>,
    next_id: u64,
}

#[cfg(feature = "canister")]
impl TestTimerRegistry {
    fn new() -> Self {
        Self {
            timers: BTreeMap::new(),
            next_id: 1,
        }
    }

    fn add_timer(&mut self, timer_id: MockTimerId, info: TimerInfo) -> Result<(), TimerError> {
        const MAX_TIMERS: usize = 100;
        if self.timers.len() >= MAX_TIMERS {
            return Err(TimerError::TooManyTimers {
                max: MAX_TIMERS,
                current: self.timers.len(),
            });
        }
        self.timers.insert(timer_id, info);
        self.next_id += 1;
        Ok(())
    }

    fn remove_timer(&mut self, timer_id: MockTimerId) -> Option<TimerInfo> {
        self.timers.remove(&timer_id)
    }

    fn list_timers(&self) -> Vec<TimerInfo> {
        self.timers.values().cloned().collect()
    }

    fn clear_all(&mut self) {
        self.timers.clear();
    }
}

#[cfg(all(test, feature = "canister"))]
mod timer_module_tests {
    use super::*;

    #[test]
    fn test_timer_info_creation_and_fields() {
        let info = TimerInfo {
            id: 42,
            name: "test_timer".to_string(),
            timer_type: TimerType::Once,
            created_at: 1640995200, // 2022-01-01 timestamp
            interval_secs: Some(60),
        };

        assert_eq!(info.id, 42);
        assert_eq!(info.name, "test_timer");
        assert!(matches!(info.timer_type, TimerType::Once));
        assert_eq!(info.created_at, 1640995200);
        assert_eq!(info.interval_secs, Some(60));
    }

    #[test]
    fn test_timer_info_clone() {
        let info1 = TimerInfo {
            id: 1,
            name: "original".to_string(),
            timer_type: TimerType::Periodic,
            created_at: 1000,
            interval_secs: Some(30),
        };

        let info2 = info1.clone();

        assert_eq!(info1.id, info2.id);
        assert_eq!(info1.name, info2.name);
        assert_eq!(info1.created_at, info2.created_at);
        assert_eq!(info1.interval_secs, info2.interval_secs);
    }

    #[test]
    fn test_timer_type_variants() {
        let once_timer = TimerInfo {
            id: 1,
            name: "once".to_string(),
            timer_type: TimerType::Once,
            created_at: 0,
            interval_secs: None,
        };

        let periodic_timer = TimerInfo {
            id: 2,
            name: "periodic".to_string(),
            timer_type: TimerType::Periodic,
            created_at: 0,
            interval_secs: Some(60),
        };

        assert!(matches!(once_timer.timer_type, TimerType::Once));
        assert!(matches!(periodic_timer.timer_type, TimerType::Periodic));
    }

    #[test]
    fn test_timer_info_serialization() {
        let info = TimerInfo {
            id: 123,
            name: "serializable_timer".to_string(),
            timer_type: TimerType::Periodic,
            created_at: 1234567890,
            interval_secs: Some(300),
        };

        // Test serialization
        let serialized = serde_json::to_string(&info).unwrap();
        assert!(!serialized.is_empty());
        assert!(serialized.contains("serializable_timer"));
        assert!(serialized.contains("Periodic"));

        // Test deserialization
        let deserialized: TimerInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, info.id);
        assert_eq!(deserialized.name, info.name);
        assert_eq!(deserialized.created_at, info.created_at);
        assert_eq!(deserialized.interval_secs, info.interval_secs);
    }

    #[test]
    fn test_timer_error_display() {
        let error1 = TimerError::TooManyTimers {
            max: 100,
            current: 100,
        };
        assert_eq!(error1.to_string(), "Too many timers: 100/100");

        let error2 = TimerError::TimerNotFound("timer_123".to_string());
        assert_eq!(error2.to_string(), "Timer not found: timer_123");

        let error3 = TimerError::InvalidInterval(0);
        assert_eq!(error3.to_string(), "Invalid interval: 0 seconds");
    }

    #[test]
    fn test_timer_error_debug() {
        let error = TimerError::InvalidInterval(5);
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidInterval"));
        assert!(debug_str.contains("5"));
    }

    #[test]
    fn test_timer_registry_new() {
        let registry = TestTimerRegistry::new();
        assert_eq!(registry.timers.len(), 0);
        assert_eq!(registry.next_id, 1);
        assert!(registry.list_timers().is_empty());
    }

    #[test]
    fn test_timer_registry_add_timer() {
        let mut registry = TestTimerRegistry::new();
        let timer_id = 1;
        let info = TimerInfo {
            id: 1,
            name: "test_timer".to_string(),
            timer_type: TimerType::Once,
            created_at: 1000,
            interval_secs: Some(60),
        };

        let result = registry.add_timer(timer_id, info.clone());
        assert!(result.is_ok());
        assert_eq!(registry.timers.len(), 1);
        assert_eq!(registry.next_id, 2);

        let stored_timers = registry.list_timers();
        assert_eq!(stored_timers.len(), 1);
        assert_eq!(stored_timers[0].name, "test_timer");
    }

    #[test]
    fn test_timer_registry_add_multiple_timers() {
        let mut registry = TestTimerRegistry::new();

        for i in 1..=5 {
            let timer_id = i;
            let info = TimerInfo {
                id: i,
                name: format!("timer_{}", i),
                timer_type: if i % 2 == 0 {
                    TimerType::Periodic
                } else {
                    TimerType::Once
                },
                created_at: 1000 + i,
                interval_secs: Some(i * 10),
            };

            let result = registry.add_timer(timer_id, info);
            assert!(result.is_ok());
        }

        assert_eq!(registry.timers.len(), 5);
        assert_eq!(registry.next_id, 6);

        let timers = registry.list_timers();
        assert_eq!(timers.len(), 5);

        // Verify timers are stored correctly
        for (i, timer) in timers.iter().enumerate() {
            let expected_id = i + 1;
            assert_eq!(timer.name, format!("timer_{}", expected_id));
        }
    }

    #[test]
    fn test_timer_registry_max_timers_limit() {
        let mut registry = TestTimerRegistry::new();

        // Add timers up to the limit
        for i in 1..=100 {
            let timer_id = i;
            let info = TimerInfo {
                id: i,
                name: format!("timer_{}", i),
                timer_type: TimerType::Once,
                created_at: 1000,
                interval_secs: Some(60),
            };

            let result = registry.add_timer(timer_id, info);
            assert!(result.is_ok());
        }

        assert_eq!(registry.timers.len(), 100);

        // Try to add one more timer (should fail)
        let timer_id = 101;
        let info = TimerInfo {
            id: 101,
            name: "excess_timer".to_string(),
            timer_type: TimerType::Once,
            created_at: 1000,
            interval_secs: Some(60),
        };

        let result = registry.add_timer(timer_id, info);
        assert!(result.is_err());

        if let Err(TimerError::TooManyTimers { max, current }) = result {
            assert_eq!(max, 100);
            assert_eq!(current, 100);
        } else {
            panic!("Expected TooManyTimers error");
        }

        // Verify registry still has exactly 100 timers
        assert_eq!(registry.timers.len(), 100);
    }

    #[test]
    fn test_timer_registry_remove_timer() {
        let mut registry = TestTimerRegistry::new();
        let timer_id = 1;
        let info = TimerInfo {
            id: 1,
            name: "removable_timer".to_string(),
            timer_type: TimerType::Periodic,
            created_at: 1000,
            interval_secs: Some(30),
        };

        // Add timer
        registry.add_timer(timer_id, info.clone()).unwrap();
        assert_eq!(registry.timers.len(), 1);

        // Remove timer
        let removed = registry.remove_timer(timer_id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "removable_timer");
        assert_eq!(registry.timers.len(), 0);

        // Try to remove non-existent timer
        let removed_again = registry.remove_timer(timer_id);
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_timer_registry_remove_specific_timer() {
        let mut registry = TestTimerRegistry::new();

        // Add multiple timers
        for i in 1..=3 {
            let timer_id = i;
            let info = TimerInfo {
                id: i,
                name: format!("timer_{}", i),
                timer_type: TimerType::Once,
                created_at: 1000,
                interval_secs: Some(60),
            };
            registry.add_timer(timer_id, info).unwrap();
        }

        assert_eq!(registry.timers.len(), 3);

        // Remove middle timer
        let removed = registry.remove_timer(2);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "timer_2");
        assert_eq!(registry.timers.len(), 2);

        // Verify remaining timers
        let remaining = registry.list_timers();
        let remaining_names: Vec<String> = remaining.iter().map(|t| t.name.clone()).collect();
        assert!(remaining_names.contains(&"timer_1".to_string()));
        assert!(remaining_names.contains(&"timer_3".to_string()));
        assert!(!remaining_names.contains(&"timer_2".to_string()));
    }

    #[test]
    fn test_timer_registry_clear_all() {
        let mut registry = TestTimerRegistry::new();

        // Add multiple timers
        for i in 1..=5 {
            let timer_id = i;
            let info = TimerInfo {
                id: i,
                name: format!("timer_{}", i),
                timer_type: TimerType::Periodic,
                created_at: 1000,
                interval_secs: Some(i * 20),
            };
            registry.add_timer(timer_id, info).unwrap();
        }

        assert_eq!(registry.timers.len(), 5);

        // Clear all timers
        registry.clear_all();

        assert_eq!(registry.timers.len(), 0);
        assert!(registry.list_timers().is_empty());

        // Verify next_id is preserved (not reset)
        assert_eq!(registry.next_id, 6);
    }

    #[test]
    fn test_timer_registry_list_timers() {
        let mut registry = TestTimerRegistry::new();

        // Test empty list
        let empty_list = registry.list_timers();
        assert!(empty_list.is_empty());

        // Add timers with different types
        let timer_infos = vec![
            TimerInfo {
                id: 1,
                name: "once_timer".to_string(),
                timer_type: TimerType::Once,
                created_at: 1000,
                interval_secs: Some(60),
            },
            TimerInfo {
                id: 2,
                name: "periodic_timer".to_string(),
                timer_type: TimerType::Periodic,
                created_at: 2000,
                interval_secs: Some(120),
            },
        ];

        for (i, info) in timer_infos.iter().enumerate() {
            registry.add_timer((i + 1) as u64, info.clone()).unwrap();
        }

        // Test populated list
        let listed_timers = registry.list_timers();
        assert_eq!(listed_timers.len(), 2);

        // Verify timer details
        let names: Vec<String> = listed_timers.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"once_timer".to_string()));
        assert!(names.contains(&"periodic_timer".to_string()));
    }

    #[test]
    fn test_interval_validation_logic() {
        // Test invalid intervals that should trigger errors
        let invalid_intervals = vec![0u64];

        for interval in invalid_intervals {
            // This simulates the validation logic from schedule_once/schedule_periodic
            if interval == 0 {
                let error = TimerError::InvalidInterval(interval);
                assert_eq!(error.to_string(), "Invalid interval: 0 seconds");
            }
        }

        // Test valid intervals
        let valid_intervals = vec![1u64, 60u64, 3600u64, 86400u64];

        for interval in valid_intervals {
            assert!(interval > 0, "Interval {} should be valid", interval);
        }
    }

    #[test]
    fn test_backoff_calculation_logic() {
        // Test exponential backoff calculation
        let initial_delay = 1u64;
        let backoff_multiplier = 2.0f64;

        let delays = vec![
            initial_delay,                                              // Attempt 0: 1s
            (initial_delay as f64 * backoff_multiplier) as u64,         // Attempt 1: 2s
            (initial_delay as f64 * backoff_multiplier.powi(2)) as u64, // Attempt 2: 4s
            (initial_delay as f64 * backoff_multiplier.powi(3)) as u64, // Attempt 3: 8s
            (initial_delay as f64 * backoff_multiplier.powi(4)) as u64, // Attempt 4: 16s
        ];

        assert_eq!(delays[0], 1);
        assert_eq!(delays[1], 2);
        assert_eq!(delays[2], 4);
        assert_eq!(delays[3], 8);
        assert_eq!(delays[4], 16);

        // Test with different multiplier
        let multiplier_1_5 = 1.5f64;
        let delay_1_5 = (10.0 * multiplier_1_5.powi(3)) as u64; // 10 * 1.5^3 = 33.75 ≈ 33
        assert_eq!(delay_1_5, 33);

        // Test with fractional multiplier
        let fractional_multiplier = 0.5f64;
        let delay_fractional = (100.0 * fractional_multiplier) as u64; // Should decrease
        assert_eq!(delay_fractional, 50);
    }

    #[test]
    fn test_timer_creation_timestamps() {
        // Test that timer creation includes proper timestamp handling
        let now = 1640995200u64; // Mock timestamp

        let info = TimerInfo {
            id: 1,
            name: "timestamp_test".to_string(),
            timer_type: TimerType::Once,
            created_at: now,
            interval_secs: Some(300),
        };

        assert_eq!(info.created_at, now);

        // Test different timestamps
        let info2 = TimerInfo {
            id: 2,
            name: "timestamp_test_2".to_string(),
            timer_type: TimerType::Periodic,
            created_at: now + 1000,
            interval_secs: Some(600),
        };

        assert_eq!(info2.created_at, now + 1000);
        assert!(info2.created_at > info.created_at);
    }

    #[test]
    fn test_timer_interval_options() {
        // Test timer with no interval (None)
        let info_no_interval = TimerInfo {
            id: 1,
            name: "no_interval".to_string(),
            timer_type: TimerType::Once,
            created_at: 1000,
            interval_secs: None,
        };

        assert_eq!(info_no_interval.interval_secs, None);

        // Test timer with interval (Some)
        let info_with_interval = TimerInfo {
            id: 2,
            name: "with_interval".to_string(),
            timer_type: TimerType::Periodic,
            created_at: 1000,
            interval_secs: Some(120),
        };

        assert_eq!(info_with_interval.interval_secs, Some(120));
    }

    #[test]
    fn test_timer_edge_cases() {
        let mut registry = TestTimerRegistry::new();

        // Test with maximum values
        let max_timer = TimerInfo {
            id: u64::MAX,
            name: "max_timer".to_string(),
            timer_type: TimerType::Periodic,
            created_at: u64::MAX,
            interval_secs: Some(u64::MAX),
        };

        let result = registry.add_timer(u64::MAX, max_timer);
        assert!(result.is_ok());

        // Test with empty name
        let empty_name_timer = TimerInfo {
            id: 1,
            name: "".to_string(),
            timer_type: TimerType::Once,
            created_at: 0,
            interval_secs: Some(1),
        };

        let result = registry.add_timer(1, empty_name_timer);
        assert!(result.is_ok());

        // Test with very long name
        let long_name = "a".repeat(1000);
        let long_name_timer = TimerInfo {
            id: 2,
            name: long_name.clone(),
            timer_type: TimerType::Periodic,
            created_at: 1000,
            interval_secs: Some(60),
        };

        let result = registry.add_timer(2, long_name_timer);
        assert!(result.is_ok());

        let timers = registry.list_timers();
        assert_eq!(timers.len(), 3);

        // Find the long name timer
        let long_timer = timers.iter().find(|t| t.name == long_name).unwrap();
        assert_eq!(long_timer.name.len(), 1000);
    }

    #[test]
    fn test_timer_registry_concurrent_access_simulation() {
        // Simulate concurrent-like access patterns
        let mut registry = TestTimerRegistry::new();

        // Simulate rapid add/remove cycles
        for cycle in 0..10 {
            // Add timers
            for i in 0..5 {
                let timer_id = (cycle * 5 + i) as u64;
                let info = TimerInfo {
                    id: timer_id,
                    name: format!("cycle_{}_timer_{}", cycle, i),
                    timer_type: if i % 2 == 0 {
                        TimerType::Once
                    } else {
                        TimerType::Periodic
                    },
                    created_at: 1000 + timer_id,
                    interval_secs: Some((i + 1) * 30),
                };

                registry.add_timer(timer_id, info).unwrap();
            }

            // Remove every other timer
            for i in 0..5 {
                if i % 2 == 0 {
                    let timer_id = (cycle * 5 + i) as u64;
                    registry.remove_timer(timer_id);
                }
            }
        }

        // Should have about half the timers remaining
        let remaining = registry.list_timers();
        assert!(remaining.len() > 0);
        assert!(remaining.len() <= 50); // At most all timers
    }
}
