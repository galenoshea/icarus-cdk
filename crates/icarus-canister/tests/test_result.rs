//! Tests for result types and error handling

use icarus_canister::result::{IcarusError, IcarusResult, TrapExt};
use candid::{decode_one, encode_one};

/// Test IcarusError creation and formatting
#[test]
fn test_icarus_error_creation() {
    // Test unauthorized error
    let auth_error = IcarusError::unauthorized();
    assert!(matches!(auth_error, IcarusError::Unauthorized(_)));
    if let IcarusError::Unauthorized(msg) = auth_error {
        assert_eq!(msg, "Unauthorized access");
    }

    // Test validation error
    let validation_error = IcarusError::validation("username", "must be at least 3 characters");
    assert!(matches!(validation_error, IcarusError::ValidationError { .. }));
    if let IcarusError::ValidationError { field, message } = validation_error {
        assert_eq!(field, "username");
        assert_eq!(message, "must be at least 3 characters");
    }

    // Test not found error
    let not_found_error = IcarusError::not_found("user");
    assert!(matches!(not_found_error, IcarusError::NotFound(_)));
    if let IcarusError::NotFound(msg) = not_found_error {
        assert_eq!(msg, "user not found");
    }

    // Test already exists error
    let exists_error = IcarusError::already_exists("user");
    assert!(matches!(exists_error, IcarusError::AlreadyExists(_)));
    if let IcarusError::AlreadyExists(msg) = exists_error {
        assert_eq!(msg, "user already exists");
    }

    // Test storage error
    let storage_error = IcarusError::storage("database connection failed");
    assert!(matches!(storage_error, IcarusError::StorageError(_)));
    if let IcarusError::StorageError(msg) = storage_error {
        assert_eq!(msg, "database connection failed");
    }
}

/// Test IcarusError Display formatting
#[test]
fn test_icarus_error_display() {
    // Test unauthorized display
    let auth_error = IcarusError::Unauthorized("Invalid token".to_string());
    assert_eq!(format!("{}", auth_error), "Unauthorized: Invalid token");

    // Test validation error display
    let validation_error = IcarusError::ValidationError {
        field: "email".to_string(),
        message: "invalid format".to_string(),
    };
    assert_eq!(format!("{}", validation_error), "Validation error on 'email': invalid format");

    // Test not found display
    let not_found_error = IcarusError::NotFound("Resource not found".to_string());
    assert_eq!(format!("{}", not_found_error), "Not found: Resource not found");

    // Test already exists display
    let exists_error = IcarusError::AlreadyExists("User already exists".to_string());
    assert_eq!(format!("{}", exists_error), "Already exists: User already exists");

    // Test storage error display
    let storage_error = IcarusError::StorageError("Connection timeout".to_string());
    assert_eq!(format!("{}", storage_error), "Storage error: Connection timeout");

    // Test other error display
    let other_error = IcarusError::Other("Custom error message".to_string());
    assert_eq!(format!("{}", other_error), "Custom error message");
}

/// Test From conversions for IcarusError
#[test]
fn test_icarus_error_from_conversions() {
    // Test From<String>
    let string_error = "Something went wrong".to_string();
    let error: IcarusError = string_error.clone().into();
    assert!(matches!(error, IcarusError::Other(_)));
    if let IcarusError::Other(msg) = error {
        assert_eq!(msg, string_error);
    }

    // Test From<&str>
    let str_error = "Another error occurred";
    let error: IcarusError = str_error.into();
    assert!(matches!(error, IcarusError::Other(_)));
    if let IcarusError::Other(msg) = error {
        assert_eq!(msg, str_error);
    }
}

/// Test IcarusError Clone and Debug traits
#[test]
fn test_icarus_error_traits() {
    let original = IcarusError::validation("field", "message");
    let cloned = original.clone();

    // Test that clone works
    assert!(matches!(cloned, IcarusError::ValidationError { .. }));
    if let IcarusError::ValidationError { field, message } = cloned {
        assert_eq!(field, "field");
        assert_eq!(message, "message");
    }

    // Test that debug formatting works
    let debug_str = format!("{:?}", original);
    assert!(debug_str.contains("ValidationError"));
    assert!(debug_str.contains("field"));
    assert!(debug_str.contains("message"));
}

/// Test IcarusError as std::error::Error
#[test]
fn test_icarus_error_std_error() {
    let error = IcarusError::storage("test error");

    // Test that it implements std::error::Error
    let error_trait: &dyn std::error::Error = &error;
    assert_eq!(error_trait.to_string(), "Storage error: test error");

    // Test source (should be None for our simple errors)
    assert!(error_trait.source().is_none());
}

/// Test IcarusError serialization with Candid
#[test]
fn test_icarus_error_candid_serialization() {
    let errors = vec![
        IcarusError::Unauthorized("test".to_string()),
        IcarusError::ValidationError {
            field: "test_field".to_string(),
            message: "test message".to_string(),
        },
        IcarusError::NotFound("test resource".to_string()),
        IcarusError::AlreadyExists("test item".to_string()),
        IcarusError::StorageError("test storage".to_string()),
        IcarusError::Other("test other".to_string()),
    ];

    for error in errors {
        // Test encoding
        let encoded = encode_one(&error).expect("Should encode successfully");
        assert!(encoded.len() > 0);

        // Test decoding
        let decoded: IcarusError = decode_one(&encoded).expect("Should decode successfully");

        // Verify the decoded error matches the original
        match (&error, &decoded) {
            (IcarusError::Unauthorized(a), IcarusError::Unauthorized(b)) => assert_eq!(a, b),
            (IcarusError::ValidationError { field: f1, message: m1 },
             IcarusError::ValidationError { field: f2, message: m2 }) => {
                assert_eq!(f1, f2);
                assert_eq!(m1, m2);
            },
            (IcarusError::NotFound(a), IcarusError::NotFound(b)) => assert_eq!(a, b),
            (IcarusError::AlreadyExists(a), IcarusError::AlreadyExists(b)) => assert_eq!(a, b),
            (IcarusError::StorageError(a), IcarusError::StorageError(b)) => assert_eq!(a, b),
            (IcarusError::Other(a), IcarusError::Other(b)) => assert_eq!(a, b),
            _ => panic!("Decoded error variant doesn't match original"),
        }
    }
}

/// Test IcarusError serialization with serde_json
#[test]
fn test_icarus_error_json_serialization() {
    let error = IcarusError::ValidationError {
        field: "username".to_string(),
        message: "too short".to_string(),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&error).expect("Should serialize to JSON");
    assert!(json.contains("ValidationError"));
    assert!(json.contains("username"));
    assert!(json.contains("too short"));

    // Test JSON deserialization
    let deserialized: IcarusError = serde_json::from_str(&json).expect("Should deserialize from JSON");
    if let IcarusError::ValidationError { field, message } = deserialized {
        assert_eq!(field, "username");
        assert_eq!(message, "too short");
    } else {
        panic!("Deserialized to wrong variant");
    }
}

/// Test IcarusResult type alias
#[test]
fn test_icarus_result_type() {
    // Test successful result
    let success: IcarusResult<String> = Ok("test".to_string());
    assert!(success.is_ok());
    assert_eq!(success.unwrap(), "test");

    // Test error result
    let error: IcarusResult<String> = Err(IcarusError::not_found("item"));
    assert!(error.is_err());
    if let Err(IcarusError::NotFound(msg)) = error {
        assert_eq!(msg, "item not found");
    }
}

/// Test TrapExt trait for successful results
#[test]
fn test_trap_ext_success() {
    let result: IcarusResult<i32> = Ok(42);
    let value = result.unwrap_or_trap();
    assert_eq!(value, 42);
}

/// Test TrapExt trait implementation
#[test]
fn test_trap_ext_trait_exists() {
    // Test that the trait exists and can be used
    let success: Result<String, IcarusError> = Ok("test".to_string());

    // This should compile and work
    let value = success.unwrap_or_trap();
    assert_eq!(value, "test");
}

/// Test error builder pattern with chaining
#[test]
fn test_error_builder_pattern() {
    // Test that we can use the builder functions in various ways
    let errors = vec![
        IcarusError::unauthorized(),
        IcarusError::validation("field1", "message1"),
        IcarusError::not_found("resource1"),
        IcarusError::already_exists("item1"),
        IcarusError::storage("storage issue"),
    ];

    assert_eq!(errors.len(), 5);

    // Verify each error type
    assert!(matches!(errors[0], IcarusError::Unauthorized(_)));
    assert!(matches!(errors[1], IcarusError::ValidationError { .. }));
    assert!(matches!(errors[2], IcarusError::NotFound(_)));
    assert!(matches!(errors[3], IcarusError::AlreadyExists(_)));
    assert!(matches!(errors[4], IcarusError::StorageError(_)));
}

/// Test error conversions in realistic scenarios
#[test]
fn test_error_conversion_scenarios() {
    // Scenario 1: Converting string literals
    let str_error: IcarusError = "Connection failed".into();
    assert_eq!(format!("{}", str_error), "Connection failed");

    // Scenario 2: Converting owned strings
    let owned_string = format!("Error code: {}", 404);
    let string_error: IcarusError = owned_string.into();
    assert_eq!(format!("{}", string_error), "Error code: 404");

    // Scenario 3: Building validation errors with dynamic content
    let field_name = "email";
    let validation_msg = format!("Invalid {} format", field_name);
    let validation_error = IcarusError::validation(field_name, validation_msg);
    assert_eq!(format!("{}", validation_error), "Validation error on 'email': Invalid email format");
}

/// Test error message formatting edge cases
#[test]
fn test_error_formatting_edge_cases() {
    // Empty string messages
    let empty_error = IcarusError::Other("".to_string());
    assert_eq!(format!("{}", empty_error), "");

    // Messages with special characters
    let special_chars = IcarusError::storage("Error with 'quotes' and \"double quotes\"");
    assert!(format!("{}", special_chars).contains("'quotes'"));
    assert!(format!("{}", special_chars).contains("\"double quotes\""));

    // Unicode characters
    let unicode_error = IcarusError::validation("名前", "ユーザー名が必要です");
    assert!(format!("{}", unicode_error).contains("名前"));
    assert!(format!("{}", unicode_error).contains("ユーザー名が必要です"));

    // Very long messages
    let long_message = "a".repeat(1000);
    let long_error = IcarusError::Other(long_message.clone());
    assert_eq!(format!("{}", long_error), long_message);
}

/// Test error equality and comparison
#[test]
fn test_error_equality() {
    let error1 = IcarusError::NotFound("user not found".to_string());
    let error2 = IcarusError::NotFound("user not found".to_string());
    let error3 = IcarusError::NotFound("item not found".to_string());

    // Clone creates equal errors
    let error1_clone = error1.clone();

    // Test debug format equality
    assert_eq!(format!("{:?}", error1), format!("{:?}", error1_clone));
    assert_eq!(format!("{:?}", error1), format!("{:?}", error2));
    assert_ne!(format!("{:?}", error1), format!("{:?}", error3));

    // Test display format equality
    assert_eq!(format!("{}", error1), format!("{}", error2));
    assert_ne!(format!("{}", error1), format!("{}", error3));
}

/// Test complex validation error scenarios
#[test]
fn test_complex_validation_scenarios() {
    // Multiple field validation
    let errors = vec![
        IcarusError::validation("username", "must be at least 3 characters"),
        IcarusError::validation("password", "must contain at least one uppercase letter"),
        IcarusError::validation("email", "invalid email format"),
    ];

    for (i, error) in errors.iter().enumerate() {
        if let IcarusError::ValidationError { field, message } = error {
            match i {
                0 => {
                    assert_eq!(field, "username");
                    assert!(message.contains("3 characters"));
                },
                1 => {
                    assert_eq!(field, "password");
                    assert!(message.contains("uppercase"));
                },
                2 => {
                    assert_eq!(field, "email");
                    assert!(message.contains("email format"));
                },
                _ => panic!("Unexpected error"),
            }
        } else {
            panic!("Expected ValidationError");
        }
    }
}