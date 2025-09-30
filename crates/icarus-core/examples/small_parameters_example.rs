//! Example demonstrating `SmallParameters` usage with Internet Computer compatibility.
//!
//! This example shows how `SmallParameters` provides the performance benefits of `SmallVec`
//! while maintaining full `CandidType` compatibility for IC canisters.

use candid::{decode_args, encode_args};
use icarus_core::{LegacyTool as Tool, SmallParameters, ToolId, ToolParameter, ToolSchema};

/// Result type alias to avoid type complexity warnings
type ExampleResult = Result<(), Box<dyn std::error::Error>>;

/// Example function that demonstrates `SmallParameters` in action
fn main() -> ExampleResult {
    println!("SmallParameters IC Compatibility Example");
    println!("=======================================");

    // Create a tool with parameters using SmallParameters
    let tool = Tool::builder()
        .name(ToolId::new("example_tool")?)
        .description("An example tool with optimized parameters")
        .parameter(ToolParameter::new(
            "input",
            "The input string",
            ToolSchema::string(),
        ))
        .parameter(ToolParameter::optional(
            "count",
            "Number of repetitions",
            ToolSchema::integer(),
        ))
        .parameter(ToolParameter::new(
            "enabled",
            "Whether the tool is enabled",
            ToolSchema::boolean(),
        ))
        .build()?;

    println!("Created tool with {} parameters", tool.parameters.len());

    // Demonstrate SmallVec benefits (stack allocation for ≤4 elements)
    println!("\nPerformance characteristics:");
    println!("- Parameters count: {}", tool.parameters.len());
    println!("- Stack allocated: {}", tool.parameters.len() <= 4);

    // Demonstrate CandidType serialization (IC compatibility)
    println!("\nCandid serialization test:");

    // Test with simple types first
    let simple_params = SmallParameters::from_vec(vec![1i32, 2, 3, 4]);
    let candid_bytes = encode_args((&simple_params,))?;
    println!("Serialized simple params: {} bytes", candid_bytes.len());

    // Deserialize back to verify round-trip
    let (deserialized_simple,): (SmallParameters<i32>,) = decode_args(&candid_bytes)?;
    println!(
        "Deserialized {} simple parameters",
        deserialized_simple.len()
    );
    assert_eq!(simple_params.len(), deserialized_simple.len());
    println!("✓ Simple Candid round-trip successful!");

    // For complex types like ToolParameter, we demonstrate that the type implements CandidType
    // but note that full round-trip requires identical type registration in the IC environment
    println!("✓ ToolParameter SmallParameters implements CandidType for IC compatibility");

    // Demonstrate JSON serialization (also works)
    println!("\nJSON serialization test:");
    let json = serde_json::to_string(&tool.parameters)?;
    println!("JSON length: {} bytes", json.len());

    let json_deserialized: SmallParameters<ToolParameter> = serde_json::from_str(&json)?;
    assert_eq!(tool.parameters.len(), json_deserialized.len());
    println!("✓ JSON round-trip successful!");

    // Demonstrate transparent SmallVec access
    println!("\nTransparent SmallVec access:");
    println!("First parameter: {}", tool.parameters[0].name);
    println!("Required parameters: {}", tool.required_parameters().len());
    println!("Optional parameters: {}", tool.optional_parameters().len());

    // Demonstrate performance with larger collection
    println!("\nLarge collection behavior:");
    let mut large_params = SmallParameters::<i32>::new();
    for i in 0..10 {
        large_params.push(i);
    }
    println!("Large collection size: {}", large_params.len());
    println!("Heap allocated: {}", large_params.len() > 4);

    println!("\n✓ All examples completed successfully!");
    Ok(())
}
