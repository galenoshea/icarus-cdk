use icarus_derive::*;
use ic_cdk::query;

#[icarus_tool("A simple test tool")]
#[query]
fn test_tool() -> Result<String, String> {
    Ok("Hello".to_string())
}

fn main() {}