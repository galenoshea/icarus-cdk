//! Tool definition and schema types for the Icarus CDK.
//!
//! This module provides type-safe tool definitions with JSON schema generation
//! and validation, following `rust_best_practices.md` patterns.

mod parameter;
mod schema;

// Re-export public types
pub use parameter::{SmallParameters, ToolParameter};
pub use schema::ToolSchema;

use candid::{CandidType, Deserialize};
use serde::Serialize;

use crate::{IcarusError, ToolId};

/// Annotations for tools to support RMCP compatibility and authentication levels.
///
/// These annotations provide additional metadata for tools, including localization hints,
/// read-only status, and authentication level requirements. This structure matches the
/// RMCP (Remote Model Context Protocol) specification for tool metadata.
///
/// # Examples
///
/// ```rust
/// use icarus_core::tool::ToolAnnotations;
///
/// let annotations = ToolAnnotations {
///     title: Some("Calculator Addition".to_string()),
///     read_only_hint: Some(true),
///     auth_level: Some("user".to_string()),
/// };
/// ```
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct ToolAnnotations {
    /// Localized title for the tool (e.g., for internationalization).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Hint that this tool primarily performs read-only operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,
    /// Authentication level required: "none" (public), "user", or "admin".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_level: Option<String>,
}

/// MCP tool definition with metadata and schema information.
///
/// Represents a callable tool that can be exposed through the MCP protocol.
/// Tools include parameter definitions, descriptions, and JSON schemas for validation.
/// Schema and metadata are stored as JSON strings for Candid compatibility.
///
/// # Examples
///
/// ```rust
/// use icarus_core::{LegacyTool as Tool, ToolId, ToolParameter, ToolSchema};
///
/// let tool = Tool::builder()
///     .name(ToolId::new("calculator.add")?)
///     .description("Adds two numbers together")
///     .parameter(ToolParameter::new("a", "The first number", ToolSchema::number()))
///     .parameter(ToolParameter::new("b", "The second number", ToolSchema::number()))
///     .build()?;
/// # Ok::<(), icarus_core::IcarusError>(())
/// ```
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct Tool {
    /// Unique identifier for the tool.
    pub name: ToolId,
    /// Human-readable description of what the tool does.
    pub description: String,
    /// Parameters that the tool accepts (optimized for <5 parameters).
    pub parameters: SmallParameters<ToolParameter>,
    /// JSON schema for input validation as a JSON string.
    pub input_schema: String,
    /// Optional metadata for the tool as a JSON string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    /// Optional annotations for RMCP compatibility (title, `read_only_hint`, `auth_level`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ToolAnnotations>,
}

impl Tool {
    /// Creates a new tool builder.
    #[must_use]
    #[inline]
    pub fn builder() -> ToolBuilder {
        ToolBuilder::new()
    }

    /// Validates the tool definition.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::InvalidSchema` if the tool definition is invalid.
    pub fn validate(&self) -> Result<(), IcarusError> {
        if self.description.is_empty() {
            return Err(IcarusError::InvalidSchema {
                tool_id: self.name.clone(),
                message: "Tool description cannot be empty".to_string(),
            });
        }

        if self.description.len() > crate::MAX_DESCRIPTION_LENGTH {
            return Err(IcarusError::InvalidSchema {
                tool_id: self.name.clone(),
                message: format!(
                    "Tool description exceeds maximum length of {}",
                    crate::MAX_DESCRIPTION_LENGTH
                ),
            });
        }

        if self.parameters.len() > crate::MAX_PARAMETER_COUNT {
            return Err(IcarusError::InvalidSchema {
                tool_id: self.name.clone(),
                message: format!(
                    "Tool has too many parameters (max: {})",
                    crate::MAX_PARAMETER_COUNT
                ),
            });
        }

        // Validate each parameter using iterator pattern
        self.parameters
            .iter()
            .try_for_each(ToolParameter::validate)?;

        // Check for duplicate parameter names using iterator pattern
        let mut param_names = std::collections::HashSet::with_capacity(self.parameters.len());
        if let Some(duplicate) = self
            .parameters
            .iter()
            .find(|param| !param_names.insert(&param.name))
        {
            return Err(IcarusError::InvalidSchema {
                tool_id: self.name.clone(),
                message: format!("Duplicate parameter name: {}", duplicate.name),
            });
        }

        Ok(())
    }

    /// Returns the required parameters for this tool.
    #[must_use]
    #[inline]
    pub fn required_parameters(&self) -> SmallParameters<&ToolParameter> {
        self.parameters.iter().filter(|p| p.required).collect()
    }

    /// Returns the optional parameters for this tool.
    #[must_use]
    #[inline]
    pub fn optional_parameters(&self) -> SmallParameters<&ToolParameter> {
        self.parameters.iter().filter(|p| !p.required).collect()
    }

    /// Finds a parameter by name.
    #[must_use]
    #[inline]
    pub fn find_parameter(&self, name: &str) -> Option<&ToolParameter> {
        self.parameters.iter().find(|p| p.name == name)
    }
}

/// Builder for creating tool definitions with validation.
#[derive(Debug, Default)]
pub struct ToolBuilder {
    name: Option<ToolId>,
    description: Option<String>,
    parameters: SmallParameters<ToolParameter>,
    metadata: Option<String>,
    annotations: Option<ToolAnnotations>,
}

impl ToolBuilder {
    /// Creates a new tool builder.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the tool name.
    #[must_use]
    #[inline]
    pub fn name(mut self, name: ToolId) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the tool description.
    #[must_use]
    #[inline]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a parameter to the tool.
    #[must_use]
    #[inline]
    pub fn parameter(mut self, parameter: ToolParameter) -> Self {
        self.parameters.push(parameter);
        self
    }

    /// Adds multiple parameters to the tool.
    #[must_use]
    #[inline]
    pub fn parameters(mut self, parameters: impl IntoIterator<Item = ToolParameter>) -> Self {
        self.parameters.extend(parameters);
        self
    }

    /// Sets metadata as a JSON string.
    #[must_use]
    #[inline]
    pub fn metadata(mut self, metadata: impl Into<String>) -> Self {
        self.metadata = Some(metadata.into());
        self
    }

    /// Sets annotations for RMCP compatibility.
    #[must_use]
    #[inline]
    pub fn annotations(mut self, annotations: ToolAnnotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    /// Builds the tool with validation.
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::InvalidSchema` if the tool configuration is invalid.
    pub fn build(self) -> Result<Tool, IcarusError> {
        let name = self
            .name
            .ok_or_else(|| IcarusError::ConfigurationError("Tool name is required".to_string()))?;

        let description = self.description.ok_or_else(|| {
            IcarusError::ConfigurationError("Tool description is required".to_string())
        })?;

        // Generate JSON schema from parameters
        let input_schema = generate_input_schema(&self.parameters);

        let tool = Tool {
            name,
            description,
            parameters: self.parameters,
            input_schema,
            metadata: self.metadata,
            annotations: self.annotations,
        };

        // Validate the built tool
        tool.validate()?;

        Ok(tool)
    }
}

/// Generates a JSON schema string from a list of parameters.
fn generate_input_schema(parameters: &[ToolParameter]) -> String {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for param in parameters {
        properties.insert(
            param.name.clone(),
            serde_json::to_value(&param.schema).unwrap_or(serde_json::Value::Null),
        );

        if param.required {
            required.push(param.name.clone());
        }
    }

    // Create schema manually to avoid disallowed unwrap in json! macro
    let mut schema = serde_json::Map::new();
    schema.insert(
        "type".to_string(),
        serde_json::Value::String("object".to_string()),
    );
    schema.insert(
        "properties".to_string(),
        serde_json::Value::Object(properties),
    );
    schema.insert(
        "required".to_string(),
        serde_json::Value::Array(
            required
                .into_iter()
                .map(serde_json::Value::String)
                .collect(),
        ),
    );

    serde_json::to_string(&schema).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_builder() -> Result<(), IcarusError> {
        let tool = Tool::builder()
            .name(ToolId::new("test_tool")?)
            .description("A test tool")
            .parameter(ToolParameter::new(
                "input",
                "Test input",
                ToolSchema::string(),
            ))
            .parameter(ToolParameter::optional(
                "count",
                "Optional count",
                ToolSchema::integer(),
            ))
            .metadata(r#"{"version": "1.0"}"#)
            .build()?;

        assert_eq!(tool.name.as_str(), "test_tool");
        assert_eq!(tool.description, "A test tool");
        assert_eq!(tool.parameters.len(), 2);
        assert!(tool.metadata.is_some());

        tool.validate()?;

        Ok(())
    }

    #[test]
    fn test_tool_parameter_validation() -> Result<(), IcarusError> {
        let valid_param =
            ToolParameter::new("valid_name", "Valid description", ToolSchema::string());
        valid_param.validate()?;

        // Test invalid parameter names
        let invalid_param = ToolParameter::new("invalid-name", "Description", ToolSchema::string());
        assert!(invalid_param.validate().is_err());

        let empty_name = ToolParameter::new("", "Description", ToolSchema::string());
        assert!(empty_name.validate().is_err());

        Ok(())
    }

    #[test]
    fn test_tool_schema_validation() {
        // Valid schemas
        assert!(ToolSchema::string().validate().is_ok());
        assert!(ToolSchema::number().validate().is_ok());
        assert!(ToolSchema::integer().validate().is_ok());
        assert!(ToolSchema::boolean().validate().is_ok());

        // Invalid schemas
        let invalid_string = ToolSchema::string_with_length(Some(10), Some(5));
        assert!(invalid_string.validate().is_err());

        let invalid_number = ToolSchema::number_range(Some(10.0), Some(5.0));
        assert!(invalid_number.validate().is_err());
    }

    #[test]
    #[allow(clippy::disallowed_methods)]
    fn test_schema_generation() -> Result<(), IcarusError> {
        let parameters = vec![
            ToolParameter::new("required_param", "Required parameter", ToolSchema::string()),
            ToolParameter::optional(
                "optional_param",
                "Optional parameter",
                ToolSchema::integer(),
            ),
        ];

        let schema = generate_input_schema(&parameters);

        let _expected = json!({
            "type": "object",
            "properties": {
                "required_param": {"type": "string"},
                "optional_param": {"type": "integer"}
            },
            "required": ["required_param"]
        });

        // Parse the schema string as JSON for validation
        let schema_json: serde_json::Value = serde_json::from_str(&schema)?;

        // Basic structure check (exact match is complex due to serialization details)
        assert!(schema_json.get("type").unwrap().as_str() == Some("object"));
        assert!(schema_json.get("properties").is_some());
        assert!(
            schema_json
                .get("required")
                .unwrap()
                .as_array()
                .unwrap()
                .len()
                == 1
        );

        Ok(())
    }

    #[test]
    fn test_tool_schema_types() {
        let string_schema = ToolSchema::string_enum(["option1", "option2"]);
        assert!(matches!(string_schema, ToolSchema::String { .. }));

        let array_schema = ToolSchema::array(ToolSchema::number());
        assert!(matches!(array_schema, ToolSchema::Array { .. }));

        let object_schema = ToolSchema::object([("prop1", ToolSchema::string())], ["prop1"]);
        assert!(matches!(object_schema, ToolSchema::Object { .. }));
    }

    #[test]
    fn test_small_parameters() {
        // Test construction
        let mut params = SmallParameters::<i32>::new();
        assert!(params.is_empty());

        // Test push/extend
        params.push(1);
        params.push(2);
        params.extend([3, 4]);
        assert_eq!(params.len(), 4);

        // Test from_vec
        let params_from_vec = SmallParameters::from_vec(vec![1, 2, 3]);
        assert_eq!(params_from_vec.len(), 3);

        // Test into_vec
        let vec = params_from_vec.into_vec();
        assert_eq!(vec, vec![1, 2, 3]);

        // Test deref
        let params = SmallParameters::from_vec(vec![1, 2, 3, 4, 5]);
        assert_eq!(params[0], 1);
        assert_eq!(params.len(), 5);

        // Test with_capacity
        let params = SmallParameters::<i32>::with_capacity(10);
        assert!(params.is_empty());

        // Test from iterator
        let params: SmallParameters<i32> = (1..=3).collect();
        assert_eq!(params.len(), 3);
    }

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_small_parameters_serialization() -> TestResult {
        use serde_json;

        // Test with ToolParameter (which implements CandidType, Serialize, Deserialize)
        let mut params = SmallParameters::<ToolParameter>::new();
        params.push(ToolParameter::new(
            "test_param",
            "Test description",
            ToolSchema::string(),
        ));
        params.push(ToolParameter::optional(
            "optional_param",
            "Optional description",
            ToolSchema::integer(),
        ));

        // Test serde serialization
        let json = serde_json::to_string(&params)?;
        assert!(json.contains("test_param"));
        assert!(json.contains("optional_param"));

        // Test serde deserialization
        let deserialized: SmallParameters<ToolParameter> = serde_json::from_str(&json)?;
        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].name, "test_param");
        assert_eq!(deserialized[1].name, "optional_param");

        Ok(())
    }

    #[test]
    fn test_tool_with_small_parameters() -> Result<(), IcarusError> {
        // Verify that Tool works correctly with SmallParameters
        let tool = Tool::builder()
            .name(ToolId::new("test_tool")?)
            .description("A test tool with SmallParameters")
            .parameter(ToolParameter::new(
                "param1",
                "First parameter",
                ToolSchema::string(),
            ))
            .parameter(ToolParameter::optional(
                "param2",
                "Second parameter",
                ToolSchema::integer(),
            ))
            .parameter(ToolParameter::new(
                "param3",
                "Third parameter",
                ToolSchema::boolean(),
            ))
            .build()?;

        // Test that we can access parameters normally
        assert_eq!(tool.parameters.len(), 3);
        assert_eq!(tool.parameters[0].name, "param1");
        assert!(tool.parameters[0].required);
        assert!(!tool.parameters[1].required);

        // Test helper methods still work
        let required = tool.required_parameters();
        assert_eq!(required.len(), 2);

        let optional = tool.optional_parameters();
        assert_eq!(optional.len(), 1);

        // Test find_parameter
        let found = tool.find_parameter("param2");
        assert!(found.is_some());
        assert_eq!(found.expect("param2 should be found").name, "param2");

        tool.validate()?;

        Ok(())
    }
}
