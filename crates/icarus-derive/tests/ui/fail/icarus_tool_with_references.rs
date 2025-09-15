use icarus_derive::*;

#[icarus_tool("Tool with reference parameters")]
fn bad_tool(data: &str) -> Result<String, String> {
    Ok(data.to_string())
}

fn main() {}