use icarus_macros::tool;

/// Tool function with string parameters
#[tool]
fn greet(name: String, greeting: String) -> String {
    format!("{}, {}!", greeting, name)
}

fn main() {}