use icarus_derive::*;

#[icarus_tool("Tool without Result return type")]
fn bad_tool() -> String {
    "Should fail".to_string()
}

fn main() {}