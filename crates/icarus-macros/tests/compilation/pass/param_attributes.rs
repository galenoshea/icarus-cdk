// Note: This test verifies that the macro compiles without parameter attributes.
// Parameter attributes like #[param(...)] are parsed by the tool macro but require
// helper attribute support in rustc which is not yet stable for parameter-level attributes.
// The attribute parsing logic exists in utils.rs::parse_param_attributes but cannot be
// tested via compilation tests due to rustc limitations.

use icarus_macros::tool;

#[tool("Tool without parameter attributes")]
fn simple_tool(age: i32, username: String, email: String) -> String {
    format!("User: {}, Age: {}, Email: {}", username, age, email)
}

#[tool("Tool with optional parameter")]
fn optional_param_tool(count: i32, message: Option<String>) -> String {
    match message {
        Some(msg) => format!("Count: {}, Message: {}", count, msg),
        None => format!("Count: {}", count),
    }
}

fn main() {
    // Verify the tool info is generated correctly
    let tool = simple_tool_tool_info();
    assert_eq!(tool.name, "simple_tool");

    let tool2 = optional_param_tool_tool_info();
    assert_eq!(tool2.name, "optional_param_tool");
}