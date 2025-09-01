//! Unit tests for the prompts module

use icarus_core::prompts::{PromptBuilder, PromptRegistry};
use std::collections::HashMap;

#[test]
fn test_prompt_builder() {
    let prompt = PromptBuilder::new("test_prompt")
        .description("A test prompt")
        .template("Hello {{name}}, you are {{age}} years old")
        .arg("name", "The person's name", true, None)
        .arg("age", "The person's age", true, Some("unknown".to_string()))
        .build();

    assert_eq!(prompt.name, "test_prompt");
    assert_eq!(prompt.description, "A test prompt");
    assert_eq!(prompt.arguments.len(), 2);
}

#[test]
fn test_prompt_registry() {
    let mut registry = PromptRegistry::new();
    
    let prompt = PromptBuilder::new("greeting")
        .description("Greeting prompt")
        .template("Hello {{name}}!")
        .arg("name", "Name to greet", true, None)
        .build();
    
    registry.register(prompt);
    
    let retrieved = registry.get("greeting");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "greeting");
}

#[test]
fn test_prompt_rendering() {
    let mut registry = PromptRegistry::new();
    
    let prompt = PromptBuilder::new("welcome")
        .description("Welcome message")
        .template("Welcome {{name}} to {{place}}!")
        .arg("name", "User name", true, None)
        .arg("place", "Location", false, Some("Icarus".to_string()))
        .build();
    
    registry.register(prompt);
    
    let mut args = HashMap::new();
    args.insert("name".to_string(), "Alice".to_string());
    
    let rendered = registry.render("welcome", &args).unwrap();
    assert_eq!(rendered, "Welcome Alice to Icarus!");
}

#[test]
fn test_prompt_rendering_with_all_args() {
    let mut registry = PromptRegistry::new();
    
    let prompt = PromptBuilder::new("full")
        .description("Full message")
        .template("{{greeting}} {{name}}, welcome to {{place}}!")
        .arg("greeting", "Greeting word", false, Some("Hello".to_string()))
        .arg("name", "User name", true, None)
        .arg("place", "Location", true, None)
        .build();
    
    registry.register(prompt);
    
    let mut args = HashMap::new();
    args.insert("name".to_string(), "Bob".to_string());
    args.insert("place".to_string(), "ICP".to_string());
    args.insert("greeting".to_string(), "Hi".to_string());
    
    let rendered = registry.render("full", &args).unwrap();
    assert_eq!(rendered, "Hi Bob, welcome to ICP!");
}

#[test]
fn test_missing_required_argument() {
    let mut registry = PromptRegistry::new();
    
    let prompt = PromptBuilder::new("strict")
        .description("Strict prompt")
        .template("Hello {{name}}!")
        .arg("name", "Required name", true, None)
        .build();
    
    registry.register(prompt);
    
    let args = HashMap::new();
    let result = registry.render("strict", &args);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Missing required argument"));
}

#[test]
fn test_prompt_not_found() {
    let registry = PromptRegistry::new();
    let args = HashMap::new();
    
    let result = registry.render("nonexistent", &args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_default_registry() {
    let registry = PromptRegistry::default();
    assert!(registry.get("nonexistent").is_none());
}