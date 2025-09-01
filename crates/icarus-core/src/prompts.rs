//! Prompt management for MCP servers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A prompt template for user interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Unique identifier for the prompt
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// The prompt template with placeholders
    pub template: String,
    /// Required arguments for the prompt
    pub arguments: Vec<PromptArgument>,
}

/// An argument for a prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    /// Name of the argument
    pub name: String,
    /// Description of what this argument is for
    pub description: String,
    /// Whether this argument is required
    pub required: bool,
    /// Default value if not provided
    pub default: Option<String>,
}

/// Registry for managing prompts
#[derive(Debug, Clone, Default)]
pub struct PromptRegistry {
    prompts: HashMap<String, Prompt>,
}

impl PromptRegistry {
    /// Create a new prompt registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new prompt
    pub fn register(&mut self, prompt: Prompt) {
        self.prompts.insert(prompt.name.clone(), prompt);
    }

    /// Get a prompt by name
    pub fn get(&self, name: &str) -> Option<&Prompt> {
        self.prompts.get(name)
    }

    /// List all registered prompts
    pub fn list(&self) -> Vec<&Prompt> {
        self.prompts.values().collect()
    }

    /// Render a prompt with the given arguments
    pub fn render(&self, name: &str, args: &HashMap<String, String>) -> Result<String, String> {
        let prompt = self
            .get(name)
            .ok_or_else(|| format!("Prompt '{}' not found", name))?;

        let mut rendered = prompt.template.clone();

        // Check required arguments
        for arg in &prompt.arguments {
            if arg.required && !args.contains_key(&arg.name) && arg.default.is_none() {
                return Err(format!("Missing required argument: {}", arg.name));
            }
        }

        // Replace placeholders
        for arg in &prompt.arguments {
            let value = args
                .get(&arg.name)
                .or(arg.default.as_ref())
                .cloned()
                .unwrap_or_default();

            let placeholder = format!("{{{{{}}}}}", arg.name);
            rendered = rendered.replace(&placeholder, &value);
        }

        Ok(rendered)
    }
}

/// Builder for creating prompts
pub struct PromptBuilder {
    name: String,
    description: String,
    template: String,
    arguments: Vec<PromptArgument>,
}

impl PromptBuilder {
    /// Create a new prompt builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            template: String::new(),
            arguments: Vec::new(),
        }
    }

    /// Set the description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set the template
    pub fn template(mut self, template: impl Into<String>) -> Self {
        self.template = template.into();
        self
    }

    /// Add an argument
    pub fn arg(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        self.arguments.push(PromptArgument {
            name: name.into(),
            description: description.into(),
            required,
            default: None,
        });
        self
    }

    /// Add an optional argument with default
    pub fn arg_with_default(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        default: impl Into<String>,
    ) -> Self {
        self.arguments.push(PromptArgument {
            name: name.into(),
            description: description.into(),
            required: false,
            default: Some(default.into()),
        });
        self
    }

    /// Build the prompt
    pub fn build(self) -> Prompt {
        Prompt {
            name: self.name,
            description: self.description,
            template: self.template,
            arguments: self.arguments,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder() {
        let prompt = PromptBuilder::new("greeting")
            .description("A friendly greeting")
            .template("Hello {{name}}, welcome to {{place}}!")
            .arg("name", "The person's name", true)
            .arg_with_default("place", "The location", "Icarus")
            .build();

        assert_eq!(prompt.name, "greeting");
        assert_eq!(prompt.arguments.len(), 2);
    }

    #[test]
    fn test_prompt_render() {
        let mut registry = PromptRegistry::new();

        let prompt = PromptBuilder::new("greeting")
            .template("Hello {{name}}, welcome to {{place}}!")
            .arg("name", "The person's name", true)
            .arg_with_default("place", "The location", "Icarus")
            .build();

        registry.register(prompt);

        let mut args = HashMap::new();
        args.insert("name".to_string(), "Alice".to_string());

        let rendered = registry.render("greeting", &args).unwrap();
        assert_eq!(rendered, "Hello Alice, welcome to Icarus!");
    }
}
