use icarus_macros::tool;

/// This should fail - generic functions are not supported
#[tool]
fn generic_tool<T>(x: T) -> T {
    x
}

fn main() {}