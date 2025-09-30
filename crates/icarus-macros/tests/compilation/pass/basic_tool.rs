use icarus_macros::tool;

/// Basic tool function that should compile successfully
#[tool]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {}