use icarus_macros::tool;

/// Tool function with optional parameters
#[tool]
fn process_data(input: String, format: Option<String>) -> String {
    match format {
        Some(f) => format!("Formatted {} as {}", input, f),
        None => format!("Raw: {}", input),
    }
}

fn main() {}