// Test: Tool with auth = "none" should compile without ic_cdk dependency
// Note: auth = "user" and auth = "admin" require ic_cdk::caller() and are tested
// in actual canister environments where ic_cdk is available
use icarus_macros::tool;

#[tool("Public tool", auth = "none")]
fn public_tool(x: i32) -> i32 {
    x + 1
}

fn main() {}