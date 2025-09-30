use icarus_macros::tool;

/// This should fail - lifetime parameters are not supported
#[tool]
fn lifetime_tool<'a>(x: &'a str) -> &'a str {
    x
}

fn main() {}