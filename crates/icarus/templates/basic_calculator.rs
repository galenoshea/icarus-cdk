//! # Basic Calculator Example
//!
//! This example demonstrates how to create a simple MCP server with basic arithmetic tools.
//!
//! ## Features
//! - Four basic arithmetic operations: add, subtract, multiply, divide
//! - Simple function-to-tool conversion with `#[tool]` macro
//! - Error handling for division by zero
//! - Minimal boilerplate with automatic MCP endpoint generation
//!
//! ## Usage
//!
//! ```bash
//! # Deploy to Internet Computer
//! dfx start --background
//! dfx deploy basic_calculator
//!
//! # Test the tools
//! dfx canister call basic_calculator list_tools
//! dfx canister call basic_calculator call_tool '(
//!   record {
//!     name = "add";
//!     arguments = "{\"a\": 5.0, \"b\": 3.0}"
//!   }
//! )'
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │         MCP Client (AI)             │
//! │    (Claude, ChatGPT, etc.)          │
//! └─────────────────┬───────────────────┘
//!                   │ JSON-RPC
//!                   │
//! ┌─────────────────▼───────────────────┐
//! │      Icarus Bridge (Local)          │
//! │   Translates MCP ↔ IC Candid        │
//! └─────────────────┬───────────────────┘
//!                   │ Candid
//!                   │
//! ┌─────────────────▼───────────────────┐
//! │   Calculator Canister (IC)          │
//! │  ┌──────────────────────────────┐   │
//! │  │ #[tool] add(a, b) -> f64     │   │
//! │  │ #[tool] subtract(a, b) -> f64│   │
//! │  │ #[tool] multiply(a, b) -> f64│   │
//! │  │ #[tool] divide(a, b) -> f64  │   │
//! │  └──────────────────────────────┘   │
//! │       Auto-generated MCP API        │
//! └─────────────────────────────────────┘
//! ```

use icarus_macros::tool;

/// Add two numbers together.
///
/// # Parameters
/// - `a`: First number
/// - `b`: Second number
///
/// # Returns
/// The sum of `a` and `b`
///
/// # Example
/// ```json
/// {
///   "a": 5.0,
///   "b": 3.0
/// }
/// ```
/// Returns: `8.0`
#[tool("Add two numbers together")]
fn add(a: f64, b: f64) -> f64 {
    a + b
}

/// Subtract one number from another.
///
/// # Parameters
/// - `a`: Number to subtract from
/// - `b`: Number to subtract
///
/// # Returns
/// The difference `a - b`
///
/// # Example
/// ```json
/// {
///   "a": 10.0,
///   "b": 3.0
/// }
/// ```
/// Returns: `7.0`
#[tool("Subtract b from a")]
fn subtract(a: f64, b: f64) -> f64 {
    a - b
}

/// Multiply two numbers together.
///
/// # Parameters
/// - `a`: First number
/// - `b`: Second number
///
/// # Returns
/// The product of `a` and `b`
///
/// # Example
/// ```json
/// {
///   "a": 4.0,
///   "b": 5.0
/// }
/// ```
/// Returns: `20.0`
#[tool("Multiply two numbers together")]
fn multiply(a: f64, b: f64) -> f64 {
    a * b
}

/// Divide one number by another.
///
/// # Parameters
/// - `a`: Dividend (number to be divided)
/// - `b`: Divisor (number to divide by)
///
/// # Returns
/// The quotient `a / b`
///
/// # Errors
/// Returns error string if `b` is zero
///
/// # Example
/// ```json
/// {
///   "a": 15.0,
///   "b": 3.0
/// }
/// ```
/// Returns: `5.0`
#[tool("Divide a by b")]
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        return Err("Cannot divide by zero".to_string());
    }
    Ok(a / b)
}

// Generate MCP server endpoints
// This macro creates:
// - list_tools() -> String (returns JSON array of tool definitions)
// - call_tool(request: String) -> String (executes tools and returns results)
// - mcp_server_info() -> String (returns server metadata)
icarus_macros::mcp! {}

// Note: For production use, consider adding:
// 1. Input validation (NaN, Infinity checks)
// 2. Precision limits for very large numbers
// 3. Authentication with icarus::auth! macro
// 4. Rate limiting for tool execution
// 5. Logging for debugging and monitoring

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        assert_eq!(add(2.0, 3.0), 5.0);
        assert_eq!(add(-1.0, 1.0), 0.0);
        assert_eq!(add(0.0, 0.0), 0.0);
    }

    #[test]
    fn test_subtraction() {
        assert_eq!(subtract(5.0, 3.0), 2.0);
        assert_eq!(subtract(1.0, 1.0), 0.0);
        assert_eq!(subtract(0.0, 5.0), -5.0);
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(multiply(3.0, 4.0), 12.0);
        assert_eq!(multiply(-2.0, 3.0), -6.0);
        assert_eq!(multiply(0.0, 100.0), 0.0);
    }

    #[test]
    fn test_division() {
        assert_eq!(divide(10.0, 2.0), Ok(5.0));
        assert_eq!(divide(7.0, 2.0), Ok(3.5));
        assert!(divide(5.0, 0.0).is_err());
    }

    #[test]
    fn test_division_by_zero() {
        let result = divide(10.0, 0.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot divide by zero");
    }
}