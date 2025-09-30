//! Tool parameter definitions and utilities.
//!
//! This module provides parameter-related types including the `SmallParameters`
//! optimization wrapper and `ToolParameter` struct.

use std::ops::{Deref, DerefMut};

use candid::{CandidType, Deserialize};
use serde::Serialize;
use smallvec::SmallVec;

use crate::{IcarusError, ToolId};

use super::schema::ToolSchema;

/// A newtype wrapper around `SmallVec` that implements `CandidType` for IC compatibility.
///
/// This wrapper provides the performance benefits of `SmallVec` for collections with ≤4 elements
/// while maintaining compatibility with Candid serialization for Internet Computer canisters.
/// The wrapper transparently converts to/from `Vec` during serialization.
///
/// # Performance
///
/// - Stack-allocated for ≤4 elements (zero heap allocations)
/// - Heap-allocated `Vec` fallback for larger collections
/// - Transparent access to all `SmallVec` methods via `Deref`/`DerefMut`
///
/// # Examples
///
/// ```rust
/// use icarus_core::tool::SmallParameters;
///
/// let mut params = SmallParameters::<i32>::new();
/// params.push(42);
/// params.push(100);
///
/// let params = SmallParameters::from_vec(vec![1, 2, 3]);
/// assert_eq!(params.len(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct SmallParameters<T>(SmallVec<[T; 4]>);

impl<T> SmallParameters<T> {
    /// Creates a new empty `SmallParameters` collection.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    /// Creates a new `SmallParameters` collection with the specified capacity.
    #[must_use]
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(SmallVec::with_capacity(capacity))
    }

    /// Creates a `SmallParameters` collection from a `Vec`.
    #[must_use]
    #[inline]
    pub fn from_vec(vec: Vec<T>) -> Self {
        Self(SmallVec::from_vec(vec))
    }

    /// Converts the `SmallParameters` to a `Vec`.
    #[must_use]
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.0.into_vec()
    }

    /// Returns a reference to the inner `SmallVec`.
    #[must_use]
    #[inline]
    pub fn as_smallvec(&self) -> &SmallVec<[T; 4]> {
        &self.0
    }

    /// Returns a mutable reference to the inner `SmallVec`.
    #[must_use]
    #[inline]
    pub fn as_smallvec_mut(&mut self) -> &mut SmallVec<[T; 4]> {
        &mut self.0
    }
}

impl<T> Default for SmallParameters<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for SmallParameters<T> {
    type Target = SmallVec<[T; 4]>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for SmallParameters<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Vec<T>> for SmallParameters<T> {
    #[inline]
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}

impl<T> From<SmallVec<[T; 4]>> for SmallParameters<T> {
    #[inline]
    fn from(small_vec: SmallVec<[T; 4]>) -> Self {
        Self(small_vec)
    }
}

impl<T> From<SmallParameters<T>> for Vec<T> {
    #[inline]
    fn from(small_params: SmallParameters<T>) -> Self {
        small_params.into_vec()
    }
}

impl<T> FromIterator<T> for SmallParameters<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(SmallVec::from_iter(iter))
    }
}

impl<T> Extend<T> for SmallParameters<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

// Candid serialization: convert to/from Vec for IC compatibility
impl<T: CandidType + Clone> CandidType for SmallParameters<T> {
    fn _ty() -> candid::types::Type {
        Vec::<T>::_ty()
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        // Convert to Vec for serialization
        let vec: Vec<T> = self.0.iter().cloned().collect();
        vec.idl_serialize(serializer)
    }
}

// Serde serialization: convert to/from Vec
impl<T: Serialize> Serialize for SmallParameters<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for SmallParameters<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec = Vec::<T>::deserialize(deserializer)?;
        Ok(Self::from_vec(vec))
    }
}

/// Tool parameter definition with schema information.
///
/// Represents a single parameter that a tool accepts, including its type,
/// validation rules, and documentation.
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct ToolParameter {
    /// Parameter name (must be a valid identifier).
    pub name: String,
    /// Human-readable description of the parameter.
    pub description: String,
    /// JSON schema for the parameter type and validation.
    pub schema: ToolSchema,
    /// Whether this parameter is required.
    pub required: bool,
    /// Default value for optional parameters as JSON string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

impl ToolParameter {
    /// Creates a new required parameter.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        schema: ToolSchema,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            schema,
            required: true,
            default: None,
        }
    }

    /// Creates a new optional parameter.
    #[must_use]
    pub fn optional(
        name: impl Into<String>,
        description: impl Into<String>,
        schema: ToolSchema,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            schema,
            required: false,
            default: None,
        }
    }

    /// Creates a new optional parameter with a default value.
    #[must_use]
    pub fn with_default(
        name: impl Into<String>,
        description: impl Into<String>,
        schema: ToolSchema,
        default: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            schema,
            required: false,
            default: Some(default.into()),
        }
    }

    /// Validates the parameter definition.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::InvalidSchema` if the parameter is invalid.
    pub fn validate(&self) -> Result<(), IcarusError> {
        if self.name.is_empty() {
            return Err(IcarusError::InvalidSchema {
                tool_id: ToolId::new("unknown").unwrap_or_else(|_| unreachable!()),
                message: "Parameter name cannot be empty".to_string(),
            });
        }

        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(IcarusError::InvalidSchema {
                tool_id: ToolId::new("unknown").unwrap_or_else(|_| unreachable!()),
                message: format!("Invalid parameter name: {}", self.name),
            });
        }

        if self.description.is_empty() {
            return Err(IcarusError::InvalidSchema {
                tool_id: ToolId::new("unknown").unwrap_or_else(|_| unreachable!()),
                message: format!("Parameter '{}' description cannot be empty", self.name),
            });
        }

        self.schema.validate()?;

        Ok(())
    }
}
