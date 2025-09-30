//! JSON schema types for tool parameters.
//!
//! Provides type-safe representation of JSON Schema for parameter validation.

use std::collections::HashMap;

use candid::{CandidType, Deserialize};
use serde::Serialize;

use crate::{IcarusError, ToolId};

/// JSON schema types for tool parameters.
///
/// Provides type-safe representation of JSON Schema for parameter validation.
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolSchema {
    /// String type with optional constraints.
    String {
        /// Minimum string length.
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        /// Maximum string length.
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
        /// Pattern for validation (regex).
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        /// Enumerated values (if applicable).
        #[serde(skip_serializing_if = "Option::is_none")]
        r#enum: Option<Vec<String>>,
    },
    /// Number type (integer or float).
    Number {
        /// Minimum value.
        #[serde(skip_serializing_if = "Option::is_none")]
        minimum: Option<f64>,
        /// Maximum value.
        #[serde(skip_serializing_if = "Option::is_none")]
        maximum: Option<f64>,
    },
    /// Integer type.
    Integer {
        /// Minimum value.
        #[serde(skip_serializing_if = "Option::is_none")]
        minimum: Option<i64>,
        /// Maximum value.
        #[serde(skip_serializing_if = "Option::is_none")]
        maximum: Option<i64>,
    },
    /// Boolean type.
    Boolean,
    /// Array type with item schema.
    Array {
        /// Schema for array items.
        items: Box<ToolSchema>,
        /// Minimum number of items.
        #[serde(skip_serializing_if = "Option::is_none")]
        min_items: Option<usize>,
        /// Maximum number of items.
        #[serde(skip_serializing_if = "Option::is_none")]
        max_items: Option<usize>,
    },
    /// Object type with property schemas.
    Object {
        /// Schemas for object properties.
        properties: HashMap<String, ToolSchema>,
        /// Required property names.
        required: Vec<String>,
    },
}

impl ToolSchema {
    /// Creates a string schema.
    #[must_use]
    pub fn string() -> Self {
        Self::String {
            min_length: None,
            max_length: None,
            pattern: None,
            r#enum: None,
        }
    }

    /// Creates a string schema with length constraints.
    #[must_use]
    pub fn string_with_length(min: Option<usize>, max: Option<usize>) -> Self {
        Self::String {
            min_length: min,
            max_length: max,
            pattern: None,
            r#enum: None,
        }
    }

    /// Creates a string schema with enumerated values.
    #[must_use]
    pub fn string_enum(values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::String {
            min_length: None,
            max_length: None,
            pattern: None,
            r#enum: Some(values.into_iter().map(Into::into).collect()),
        }
    }

    /// Creates a number schema.
    #[must_use]
    pub fn number() -> Self {
        Self::Number {
            minimum: None,
            maximum: None,
        }
    }

    /// Creates a number schema with range constraints.
    #[must_use]
    pub fn number_range(min: Option<f64>, max: Option<f64>) -> Self {
        Self::Number {
            minimum: min,
            maximum: max,
        }
    }

    /// Creates an integer schema.
    #[must_use]
    pub fn integer() -> Self {
        Self::Integer {
            minimum: None,
            maximum: None,
        }
    }

    /// Creates an integer schema with range constraints.
    #[must_use]
    pub fn integer_range(min: Option<i64>, max: Option<i64>) -> Self {
        Self::Integer {
            minimum: min,
            maximum: max,
        }
    }

    /// Creates a boolean schema.
    #[must_use]
    pub fn boolean() -> Self {
        Self::Boolean
    }

    /// Creates an array schema.
    #[must_use]
    pub fn array(items: Self) -> Self {
        Self::Array {
            items: Box::new(items),
            min_items: None,
            max_items: None,
        }
    }

    /// Creates an object schema.
    #[must_use]
    pub fn object(
        properties: impl IntoIterator<Item = (impl Into<String>, Self)>,
        required: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self::Object {
            properties: properties.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            required: required.into_iter().map(Into::into).collect(),
        }
    }

    /// Validates the schema definition.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::InvalidSchema` if the schema is invalid.
    pub fn validate(&self) -> Result<(), IcarusError> {
        match self {
            Self::String {
                min_length,
                max_length,
                ..
            } => {
                if let (Some(min), Some(max)) = (min_length, max_length) {
                    if min > max {
                        return Err(IcarusError::InvalidSchema {
                            tool_id: ToolId::new("unknown").unwrap_or_else(|_| unreachable!()),
                            message: "String min_length cannot be greater than max_length"
                                .to_string(),
                        });
                    }
                }
            }
            Self::Number { minimum, maximum } => {
                if let (Some(min), Some(max)) = (minimum, maximum) {
                    if min > max {
                        return Err(IcarusError::InvalidSchema {
                            tool_id: ToolId::new("unknown").unwrap_or_else(|_| unreachable!()),
                            message: "Number minimum cannot be greater than maximum".to_string(),
                        });
                    }
                }
            }
            Self::Integer { minimum, maximum } => {
                if let (Some(min), Some(max)) = (minimum, maximum) {
                    if min > max {
                        return Err(IcarusError::InvalidSchema {
                            tool_id: ToolId::new("unknown").unwrap_or_else(|_| unreachable!()),
                            message: "Integer minimum cannot be greater than maximum".to_string(),
                        });
                    }
                }
            }
            Self::Array {
                items,
                min_items,
                max_items,
            } => {
                items.validate()?;
                if let (Some(min), Some(max)) = (min_items, max_items) {
                    if min > max {
                        return Err(IcarusError::InvalidSchema {
                            tool_id: ToolId::new("unknown").unwrap_or_else(|_| unreachable!()),
                            message: "Array min_items cannot be greater than max_items".to_string(),
                        });
                    }
                }
            }
            Self::Object { properties, .. } => {
                for schema in properties.values() {
                    schema.validate()?;
                }
            }
            Self::Boolean => {}
        }

        Ok(())
    }
}
