use icarus_derive::*;
use ic_cdk::update;

#[icarus_tool("A tool with parameters")]
#[update]
fn test_tool_with_params(name: String, age: u32) -> Result<String, String> {
    Ok(format!("Hello {}, age {}", name, age))
}

fn main() {}