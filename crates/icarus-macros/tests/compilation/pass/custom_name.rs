use icarus_macros::tool;
use std::borrow::Cow;

#[tool(name = "custom-tool-name")]
fn my_tool_function(x: i32) -> String {
    format!("Result: {}", x)
}

#[tool(name = "kebab-case-tool", description = "A tool with kebab-case name")]
fn another_function(a: String, b: i32) -> String {
    format!("{}: {}", a, b)
}

#[tool("Description first", name = "mixed-args-tool")]
fn mixed_args_tool(value: i32) -> String {
    value.to_string()
}

fn main() {
    // Verify the tools are registered with custom names
    let tool1 = my_tool_function_tool_info();
    assert_eq!(tool1.name, "custom-tool-name");

    let tool2 = another_function_tool_info();
    assert_eq!(tool2.name, "kebab-case-tool");
    assert_eq!(
        tool2.description,
        Some(Cow::Borrowed("A tool with kebab-case name"))
    );

    let tool3 = mixed_args_tool_tool_info();
    assert_eq!(tool3.name, "mixed-args-tool");
    assert_eq!(tool3.description, Some(Cow::Borrowed("Description first")));
}