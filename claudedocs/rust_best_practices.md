# Comprehensive Rust Best Practices Document for 2025

## Table of Contents

1. [Naming Conventions](#1-naming-conventions)
2. [Project Structure for Large Projects](#2-project-structure-for-large-projects)
3. [Idiomatic Rust Patterns](#3-idiomatic-rust-patterns-and-which-to-prefer-first)
4. [Powerful Rust Features and Optimal Usage](#4-powerful-rust-features-and-their-optimal-usage)
5. [Testing Best Practices](#5-testing-best-practices)
6. [Error Handling](#6-error-handling)
7. [Performance and Optimization](#7-performance-and-optimization)
8. [Code Organization](#8-code-organization)
9. [Concurrency Patterns](#9-concurrency-patterns)
10. [Latest 2024/2025 Features](#10-latest-20242025-features)

---

## 1. Naming Conventions

### Core Naming Rules (RFC 430)

| Item                 | Convention                             | Examples                              |
| -------------------- | -------------------------------------- | ------------------------------------- |
| **Crates**           | `snake_case` (prefer single word)      | `serde`, `tokio`, `regex`             |
| **Modules**          | `snake_case`                           | `std::collections`, `file_system`     |
| **Types/Structs**    | `UpperCamelCase`                       | `String`, `HashMap`, `UserAccount`    |
| **Traits**           | `UpperCamelCase`                       | `Iterator`, `Clone`, `Debug`          |
| **Enum variants**    | `UpperCamelCase`                       | `Some`, `None`, `Ok`, `Err`           |
| **Functions**        | `snake_case`                           | `push`, `is_empty`, `calculate_total` |
| **Methods**          | `snake_case`                           | `as_str`, `to_string`, `into_iter`    |
| **Local variables**  | `snake_case`                           | `user_id`, `file_name`, `total_count` |
| **Constants**        | `SCREAMING_SNAKE_CASE`                 | `MAX_SIZE`, `DEFAULT_CAPACITY`        |
| **Static variables** | `SCREAMING_SNAKE_CASE`                 | `GLOBAL_COUNTER`                      |
| **Type parameters**  | Single uppercase letter or descriptive | `T`, `U`, `K`, `V`, `Item`, `Output`  |
| **Lifetimes**        | Short lowercase                        | `'a`, `'b`, `'static`                 |

### Method Naming Patterns

**Conversion Methods:**

```rust
// Free conversion - borrowed to borrowed
fn as_bytes(&self) -> &[u8]

// Expensive conversion - any to owned/borrowed
fn to_lowercase(&self) -> String

// Consuming conversion - owned to owned
fn into_bytes(self) -> Vec<u8>
```

**Constructor Patterns:**

```rust
// Basic constructor
fn new() -> Self

// Constructor with configuration
fn with_capacity(capacity: usize) -> Self

// Conversion constructor
fn from_str(s: &str) -> Result<Self, ParseError>
```

**Getter Methods:**

```rust
// No get_ prefix for simple field access
fn name(&self) -> &str  // NOT get_name()

// Use get only for complex operations
fn get(&self, index: usize) -> Option<&T>

// Mutable getters
fn name_mut(&mut self) -> &mut String
```

**Iterator Methods (RFC 199):**

```rust
fn iter(&self) -> Iter<'_, T>        // Borrows items
fn iter_mut(&mut self) -> IterMut<'_, T>  // Mutably borrows
fn into_iter(self) -> IntoIter<T>    // Consumes and yields owned items
```

**Boolean Predicates:**

```rust
fn is_empty(&self) -> bool
fn has_children(&self) -> bool
fn can_read(&self) -> bool
```

### Special Cases

**Acronyms:**

- In `UpperCamelCase`: `Uuid` not `UUID`, `Stdin` not `StdIn`
- In `snake_case`: `is_xid_start` not `is_XID_start`

**Error Types:**
Use verb-object-error pattern:

```rust
ParseAddrError
JoinPathsError
RecvTimeoutError
```

---

## 2. Project Structure for Large Projects

### Workspace Organization

```
project-root/
├── Cargo.toml              # Workspace manifest
├── Cargo.lock             # Shared dependency lock
├── rustfmt.toml           # Formatting configuration
├── .github/workflows/     # CI/CD
├── crates/                # All crates
│   ├── core/              # Core business logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config/
│   │       ├── models/
│   │       └── internal/
│   ├── cli/               # Command-line interface
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   ├── server/            # Web server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── handlers/
│   └── shared/            # Shared utilities
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── tests/                 # Integration tests
├── benches/              # Benchmarks
├── examples/             # Usage examples
└── docs/                 # Documentation
```

### Workspace Configuration (2025)

```toml
# Root Cargo.toml
[workspace]
resolver = "3"  # Use latest resolver
members = ["crates/*"]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"

# Individual crate inherits
[dependencies]
serde = { workspace = true }
tokio = { workspace = true }
```

### Module Organization Best Practices

```rust
// src/lib.rs - Library root
#![warn(missing_docs)]
#![warn(clippy::all)]

//! Core library documentation

pub mod config;
pub mod models;
pub mod error;

// Re-export key types
pub use config::Config;
pub use error::{Error, Result};
pub use models::{User, Account};

// Type aliases for convenience
pub type Result<T> = std::result::Result<T, Error>;

// Private implementation modules
mod internal;
```

### Feature Flag Organization

```toml
[features]
default = ["std"]
std = []
async = ["tokio", "futures"]
serde = ["dep:serde", "serde/derive"]
full = ["async", "serde", "std"]

[dependencies]
tokio = { version = "1.0", optional = true }
serde = { version = "1.0", optional = true }
```

### Build Script Patterns

```rust
// build.rs
fn main() {
    // Minimal rerun conditions
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=src/config.toml");

    // Set compile-time environment
    println!("cargo::rustc-env=BUILD_TIME={}", chrono::Utc::now());

    // Platform-specific configuration
    #[cfg(target_os = "linux")]
    println!("cargo::rustc-cfg=platform=\"linux\"");
}
```

---

## 3. Idiomatic Rust Patterns and Which to Prefer First

### Pattern Priority Guide

**What Experienced Rust Developers Reach For First:**

1. **For Type Safety:** Newtype pattern → Option/Result → Phantom types
2. **For Data Processing:** Iterator combinators → Functional patterns → Manual loops
3. **For Error Handling:** ? operator → Combinators → Explicit matching
4. **For Resource Management:** RAII with Drop → Standard library types → Custom guards
5. **For Polymorphism:** Static dispatch (generics) → Dynamic dispatch → Enums
6. **For State Management:** Type-state pattern → Enums → Boolean flags
7. **For Configuration:** Builder pattern → Default trait → Multiple constructors

### Builder Pattern

**When to use FIRST:**

- Complex objects with many optional parameters
- Configuration structs
- Step-by-step construction needed

```rust
pub struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<Duration>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        ConfigBuilder {
            host: None,
            port: Some(8080),
            timeout: Some(Duration::from_secs(30)),
        }
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn build(self) -> Result<Config, String> {
        Ok(Config {
            host: self.host.ok_or("host is required")?,
            port: self.port.unwrap(),
            timeout: self.timeout.unwrap(),
        })
    }
}
```

### Newtype Pattern

**Use FIRST for:**

- Type safety for primitives (IDs, quantities, units)
- Domain concepts
- Security (passwords, tokens)

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct UserId(pub u64);

pub struct Password(String);

impl Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "********")  // Never expose
    }
}
```

### Type State Pattern

**Use for:**

- Compile-time state machine guarantees
- API protocols
- Resource lifecycle management

```rust
struct Connection<State> {
    socket: TcpStream,
    _state: PhantomData<State>,
}

struct Disconnected;
struct Connected;
struct Authenticated;

impl Connection<Disconnected> {
    pub fn connect(self) -> Connection<Connected> {
        Connection {
            socket: self.socket,
            _state: PhantomData,
        }
    }
}

impl Connection<Authenticated> {
    pub fn send_data(&self, data: &[u8]) {
        // Only authenticated connections can send
    }
}
```

### Strategy Pattern

**Prefer static dispatch FIRST:**

```rust
trait CompressionStrategy {
    fn compress(&self, data: &[u8]) -> Vec<u8>;
}

// Static dispatch - zero cost
fn compress_data<S: CompressionStrategy>(strategy: &S, data: &[u8]) -> Vec<u8> {
    strategy.compress(data)
}

// Dynamic dispatch - only when runtime flexibility needed
fn compress_with_dynamic(strategy: &dyn CompressionStrategy, data: &[u8]) -> Vec<u8> {
    strategy.compress(data)
}
```

### RAII Patterns

**ALWAYS use for resource management:**

```rust
struct FileResource {
    file: File,
    temp_path: PathBuf,
}

impl Drop for FileResource {
    fn drop(&mut self) {
        // Automatic cleanup
        let _ = fs::remove_file(&self.temp_path);
    }
}
```

### Iterator Patterns

**FIRST choice for data processing:**

```rust
// Prefer iterator combinators
let results: Vec<_> = data
    .iter()
    .filter(|&x| x > &0)
    .map(|x| x * 2)
    .take(10)
    .collect();

// Over manual loops
let mut results = Vec::new();
for item in data {
    if item > 0 {
        results.push(item * 2);
        if results.len() >= 10 {
            break;
        }
    }
}
```

### Option/Result Patterns

**ALWAYS prefer combinators:**

```rust
// Good - combinator chain
fn process_data(input: Option<String>) -> Result<i32, ParseError> {
    input
        .ok_or(ParseError::Missing)?
        .trim()
        .parse::<i32>()
        .map_err(ParseError::Invalid)
}

// Avoid - explicit matching when combinators work
fn process_data_bad(input: Option<String>) -> Result<i32, ParseError> {
    match input {
        Some(s) => match s.trim().parse::<i32>() {
            Ok(n) => Ok(n),
            Err(e) => Err(ParseError::Invalid(e)),
        },
        None => Err(ParseError::Missing),
    }
}
```

---

## 4. Powerful Rust Features and Their Optimal Usage

### Traits: Advanced Usage

#### Trait Bounds Best Practices

```rust
// Simple bounds - inline
fn process<T: Display + Clone>(value: T) {}

// Complex bounds - where clause
fn complex<T, U>()
where
    T: Display + Clone + Send + Sync,
    U: Iterator<Item = T>,
    U::Item: Debug,
{}
```

#### Associated Types vs Generic Parameters

```rust
// Associated type - one implementation per type
trait Iterator {
    type Item;  // Exactly one Item type per Iterator
    fn next(&mut self) -> Option<Self::Item>;
}

// Generic parameter - multiple implementations
trait From<T> {  // Can implement From for many T
    fn from(value: T) -> Self;
}
```

#### Trait Objects (Dynamic Dispatch)

```rust
// Dyn-compatible trait
trait Drawable {
    fn draw(&self);

    // Non-dispatchable method
    fn create() -> Self where Self: Sized;
}

// Usage
let shapes: Vec<Box<dyn Drawable>> = vec![
    Box::new(Circle { radius: 5.0 }),
    Box::new(Rectangle { width: 10.0, height: 20.0 }),
];
```

#### Async Traits (2025)

```rust
// Fully stabilized
trait AsyncService {
    async fn process(&self, data: String) -> Result<String, Error>;
}

impl AsyncService for MyService {
    async fn process(&self, data: String) -> Result<String, Error> {
        let result = self.client.request(data).await?;
        Ok(result.transform())
    }
}

// Async closures (Rust 2024 Edition)
let async_closure = async |x| {
    process_async(x).await
};
```

### Enums: Pattern Matching Excellence

```rust
enum State {
    Loading,
    Success(Data),
    Error(String),
}

// Exhaustive matching - preferred
match state {
    State::Loading => render_spinner(),
    State::Success(data) => render_data(data),
    State::Error(msg) => render_error(msg),
}

// Guard clauses
match temperature {
    temp if temp > 100.0 => "Too hot!",
    temp if temp < 0.0 => "Freezing!",
    temp if (20.0..=25.0).contains(&temp) => "Perfect!",
    _ => "Acceptable",
}

// State machines
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { session_id: String },
    Failed { error: String, retry_after: Duration },
}
```

### Macros: Decision Matrix

**Use Declarative (`macro_rules!`) when:**

- Simple pattern matching
- Creating DSLs
- Repetitive code generation

```rust
#[macro_export]
macro_rules! vec_of_strings {
    ($($x:expr),*) => {
        vec![$($x.to_string()),*]
    };
}
```

**Use Procedural when:**

- Complex syntax transformation
- Implementing derive macros
- Need full AST manipulation

```rust
#[proc_macro_derive(MyTrait)]
pub fn derive_my_trait(input: TokenStream) -> TokenStream {
    // Parse and generate code
}
```

### Generics and Lifetimes

#### Lifetime Elision Rules

1. Each input lifetime gets its own lifetime
2. If one input lifetime, assign to all outputs
3. If `&self` or `&mut self`, assign its lifetime to outputs

#### Higher-Ranked Trait Bounds (HRTB)

```rust
fn higher_ranked<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    // F works with any lifetime
}
```

#### Const Generics (2025)

```rust
// Type-level assertions
struct Assert<const COND: bool>;
trait IsTrue {}
impl IsTrue for Assert<true> {}

// Compile-time validation
struct SafeBuffer<T, const N: usize>
where
    Assert<{N <= 1024}>: IsTrue,
{
    data: [T; N],
}

// Matrix with compile-time dimension checking
struct Matrix<T, const ROWS: usize, const COLS: usize> {
    data: [[T; COLS]; ROWS],
}
```

### Smart Pointers Usage Matrix

| Pointer      | Thread Safety | Use Case                            |
| ------------ | ------------- | ----------------------------------- |
| `Box<T>`     | ✓             | Single ownership, heap allocation   |
| `Rc<T>`      | ✗             | Shared ownership, single-threaded   |
| `Arc<T>`     | ✓             | Shared ownership, multi-threaded    |
| `RefCell<T>` | ✗             | Interior mutability, runtime checks |
| `Mutex<T>`   | ✓             | Interior mutability, thread-safe    |
| `RwLock<T>`  | ✓             | Multiple readers, single writer     |

### Async/Await Best Practices

```rust
// Async trait methods (stabilized)
trait AsyncProcessor {
    async fn process(&self, input: &str) -> String;
}

// Pinning for self-referential futures
use std::pin::Pin;
use std::future::Future;

struct SelfReferencingFuture {
    data: String,
    reference: *const String,
}

// Executor selection:
// - Tokio: Network services, rich ecosystem
// - async-std: Learning, drop-in std replacement
// - smol: Lightweight, embedded
// - embassy: Embedded-specific
```

### Unsafe Code Guidelines

```rust
// Only use unsafe for:
// 1. FFI boundaries
// 2. Performance-critical low-level code
// 3. Implementing safe abstractions

unsafe fn dangerous() {
    // Document safety invariants
    // Minimize unsafe blocks
    // Prefer safe abstractions
}

// Wrap unsafe in safe APIs
pub fn safe_wrapper(data: &[u8]) -> Result<String, Error> {
    // Safety: data is valid UTF-8
    unsafe {
        Ok(String::from_utf8_unchecked(data.to_vec()))
    }
}
```

---

## 5. Testing Best Practices

### Unit Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_behavior() {
        // One behavior per test
        assert_eq!(add(2, 2), 4);
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_panic_condition() {
        divide(1, 0);
    }
}
```

### Integration Test Structure

```
tests/
├── integration_test.rs    # Each file is separate crate
└── common/
    └── mod.rs            # Shared test utilities
```

```rust
// tests/integration_test.rs
use my_crate::Client;

#[test]
fn test_full_workflow() {
    let client = Client::new();
    // Test multiple modules together
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_invariant(n in 1..100000) {
        let result = my_function(n);
        assert!(result >= 0);
        assert!(result <= n * 2);
    }
}
```

### Benchmark Organization

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("algorithms");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("sort", size),
            size,
            |b, &size| {
                let data = generate_data(size);
                b.iter(|| sort_algorithm(data.clone()));
            }
        );
    }
}

criterion_group!(benches, bench_algorithms);
criterion_main!(benches);
```

### Test Coverage

```bash
# Using llvm-cov
cargo install cargo-llvm-cov
cargo llvm-cov --html

# Using tarpaulin
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Mocking with mockall

```rust
use mockall::automock;

#[automock]
trait Database {
    fn get_user(&self, id: u64) -> Option<User>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockDatabase::new();
    mock.expect_get_user()
        .with(eq(42))
        .times(1)
        .returning(|_| Some(User::new("test")));

    assert!(service_using_db(&mock, 42).is_ok());
}
```

---

## 6. Error Handling

### Custom Error Types

```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct CustomError {
    kind: ErrorKind,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl Error for CustomError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}
```

### thiserror vs anyhow

**Use thiserror for libraries:**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibraryError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Network timeout after {0} seconds")]
    Timeout(u64),
}
```

**Use anyhow for applications:**

```rust
use anyhow::{Context, Result};

fn application_logic() -> Result<()> {
    let data = read_file("config.toml")
        .context("Failed to read configuration")?;

    let parsed = parse_config(&data)
        .context("Invalid configuration format")?;

    Ok(())
}
```

### Error Propagation Best Practices

```rust
// Automatic conversion with From
impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> Self {
        MyError::Io(err)
    }
}

// Use ? operator everywhere possible
fn process() -> Result<String, MyError> {
    let data = read_file()?;  // Automatic conversion
    let processed = transform(data)?;
    Ok(processed)
}

// Rich error context
use anyhow::Context;

fn detailed_errors() -> anyhow::Result<Data> {
    parse_input()
        .with_context(|| format!("Failed to parse input at line {}", line_num))?
}
```

### Panic vs Result Guidelines

**Use panic for:**

- Programmer errors (index out of bounds)
- Unrecoverable states
- Test assertions

**Use Result for:**

- I/O operations
- Network requests
- User input validation
- Any recoverable error

---

## 7. Performance and Optimization

### Memory Layout Optimization

```rust
// Minimize padding with field ordering
#[repr(C)]
struct Optimized {
    large: u64,  // 8 bytes
    medium: u32, // 4 bytes
    small: u8,   // 1 byte
    // 3 bytes padding
}

// Pack tightly
#[repr(packed)]
struct Packed {
    flag: u8,
    value: u32,
}

// Cache-line alignment
#[repr(align(64))]
struct CacheAligned {
    data: [u8; 64],
}
```

### Zero-Copy Patterns

```rust
use std::borrow::Cow;

// Conditional ownership
fn process_string(input: &str) -> Cow<str> {
    if input.contains("replace") {
        Cow::Owned(input.replace("replace", "with"))
    } else {
        Cow::Borrowed(input)  // No allocation
    }
}

// Use references instead of cloning
fn process_data(data: &[String]) -> Vec<&str> {
    data.iter().map(|s| s.as_str()).collect()
}
```

### Iterator Performance

```rust
// Iterators optimize better than loops
let sum: i32 = data
    .iter()
    .filter(|&&x| x > 0)
    .map(|&x| x * x)
    .fold(0, |acc, x| acc + x);  // More efficient than sum() for complex ops

// Use chunks for batch processing
data.chunks(1024)
    .for_each(|chunk| process_batch(chunk));
```

### Allocation Strategies

```rust
// Pre-allocate collections
let mut vec = Vec::with_capacity(1000);

// Small vector optimization
use smallvec::{SmallVec, smallvec};
let mut vec: SmallVec<[u32; 4]> = smallvec![1, 2, 3];

// Arena allocator for temporary allocations
use bumpalo::Bump;
let arena = Bump::new();
let data = arena.alloc_slice_fill_default(1000);
```

### SIMD Optimization

```rust
// Using wide crate (stable)
use wide::f32x4;

fn simd_add(a: &[f32], b: &[f32]) -> Vec<f32> {
    a.chunks_exact(4)
        .zip(b.chunks_exact(4))
        .flat_map(|(a_chunk, b_chunk)| {
            let a_simd = f32x4::from(a_chunk);
            let b_simd = f32x4::from(b_chunk);
            (a_simd + b_simd).to_array()
        })
        .collect()
}
```

### Profile-Guided Optimization

```toml
# Cargo.toml
[profile.release]
lto = "fat"
codegen-units = 1
```

```bash
# Build with PGO
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" cargo build --release
# Run workload
./target/release/app
# Rebuild with profile
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data" cargo build --release
```

### Const Functions for Compile-Time Computation

```rust
const fn factorial(n: u32) -> u32 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1)
    }
}

const PRECOMPUTED: u32 = factorial(10);  // Computed at compile time
```

---

## 8. Code Organization

### Module Visibility Rules

```rust
mod my_mod {
    fn private_function() {}           // Private to module
    pub fn public_function() {}        // Fully public
    pub(crate) fn crate_visible() {}   // Visible in crate
    pub(super) fn parent_visible() {}  // Visible to parent
    pub(in crate::my_mod) fn specific_scope() {}  // Specific path
}
```

### API Design Principles

```rust
// Minimal public surface
pub struct Client {
    // Private fields
    inner: InnerClient,
}

impl Client {
    // Builder pattern for construction
    pub fn builder() -> ClientBuilder { }

    // Clear, focused methods
    pub fn send_request(&self, request: Request) -> Result<Response> { }
}

// Re-export key types
pub use crate::request::Request;
pub use crate::response::Response;

// Hide implementation details
mod internal;  // Not pub
```

### Documentation Standards

````rust
/// Processes user data according to business rules.
///
/// # Arguments
///
/// * `user` - The user to process
/// * `rules` - Business rules to apply
///
/// # Returns
///
/// Returns processed user data or error if validation fails
///
/// # Examples
///
/// ```
/// use my_crate::process_user;
///
/// let user = User::new("Alice");
/// let result = process_user(user, &default_rules())?;
/// assert!(result.is_valid());
/// ```
///
/// # Errors
///
/// Returns `ValidationError` if user data doesn't meet requirements
pub fn process_user(user: User, rules: &Rules) -> Result<ProcessedUser> {
    // Implementation
}
````

### Public API Stability

```rust
// Use #[non_exhaustive] for future extensibility
#[non_exhaustive]
pub enum Event {
    Click { x: i32, y: i32 },
    KeyPress { key: char },
}

// Seal traits to prevent external implementation
mod sealed {
    pub trait Sealed {}
}

pub trait MyTrait: sealed::Sealed {
    fn method(&self);
}
```

---

## 9. Concurrency Patterns

### Thread Safety Fundamentals

```rust
// Send: Can be transferred between threads
// Sync: Can be shared between threads (&T)

// Common patterns:
Arc<Mutex<T>>      // Shared mutable state
Arc<RwLock<T>>     // Multiple readers, one writer
Arc<T>             // Shared immutable state (T: Sync)
```

### Channel Patterns

```rust
// std::sync::mpsc - basic channel
use std::sync::mpsc;
let (tx, rx) = mpsc::channel();

// crossbeam - more powerful
use crossbeam_channel::{bounded, select};
let (tx1, rx1) = bounded(100);
let (tx2, rx2) = bounded(100);

select! {
    recv(rx1) -> msg => handle_msg1(msg),
    recv(rx2) -> msg => handle_msg2(msg),
}
```

### Shared State Management

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        let mut num = counter.lock().unwrap();
        *num += 1;
    }));
}

for handle in handles {
    handle.join().unwrap();
}
```

### Data Parallelism with Rayon

```rust
use rayon::prelude::*;

// Parallel iteration - just change iter() to par_iter()
let sum: i32 = data.par_iter()
    .map(|x| expensive_computation(x))
    .sum();

// Parallel sorting
let mut data = vec![5, 2, 8, 1, 9];
data.par_sort();
```

### Async vs Threads Decision Matrix

| Use Case                    | Approach               | Reasoning                                     |
| --------------------------- | ---------------------- | --------------------------------------------- |
| I/O-bound, high concurrency | Async                  | Memory efficient for thousands of connections |
| CPU-bound computation       | Rayon/Threads          | Better CPU utilization                        |
| Mixed I/O + CPU             | Async + spawn_blocking | Best of both worlds                           |
| Simple concurrency          | Threads                | Easier mental model                           |
| Network services            | Async (Tokio)          | Industry standard                             |

### Actor Pattern

```rust
use std::sync::mpsc;

enum ActorMessage {
    GetState { responder: mpsc::Sender<State> },
    UpdateState(StateUpdate),
    Stop,
}

struct Actor {
    state: State,
    receiver: mpsc::Receiver<ActorMessage>,
}

impl Actor {
    fn run(mut self) {
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                ActorMessage::GetState { responder } => {
                    let _ = responder.send(self.state.clone());
                }
                ActorMessage::UpdateState(update) => {
                    self.state.apply(update);
                }
                ActorMessage::Stop => break,
            }
        }
    }
}
```

---

## 10. Latest 2024/2025 Features

### Rust 2024 Edition (Stabilized February 2025)

**Major Changes:**

- RPIT lifetime capture improvements
- Async closures stabilized
- Better temporary value handling
- Unsafe extern blocks

### Async Improvements

```rust
// Async closures (NEW in 2024 Edition)
let closure = async |x: i32| {
    some_async_operation(x).await
};

// AsyncFn traits now in prelude
fn take_async_fn<F>(f: F)
where
    F: AsyncFn(i32) -> String
{
    // Use async functions as first-class values
}

// Async trait methods (stabilized)
trait Service {
    async fn handle(&self, request: Request) -> Response;
}
```

### Pattern Matching Enhancements

```rust
// let-else pattern (stabilized)
let Ok(value) = try_parse(input) else {
    return Err(ParseError);
};

// if-let chains (coming soon)
if let Some(x) = opt && x > 10 {
    println!("Large value: {}", x);
}
```

### Const Evaluation Improvements

```rust
// Many std functions now const
const SIZE: usize = mem::size_of::<MyStruct>();
const SWAPPED: (i32, i32) = {
    let mut a = (1, 2);
    mem::swap(&mut a.0, &mut a.1);
    a
};

// Const mutable references
const fn const_mut_ref() {
    let mut x = 42;
    let r = &mut x;
    *r += 1;
}
```

### Performance Features

```rust
// Portable SIMD (nearing stabilization)
#![feature(portable_simd)]
use std::simd::f32x4;

// Profile-guided optimization improvements
// Now with better cross-crate inlining

// Link-time optimization improvements
[profile.release]
lto = "fat"
codegen-units = 1
```

### Migration to 2024 Edition

```bash
# Automatic migration
cargo fix --edition

# Update Cargo.toml
[package]
edition = "2024"
```

---

## Summary: Key Principles for Excellence

### Pattern Selection Priority

1. **Always prefer zero-cost abstractions**: Iterators, newtype, RAII
2. **Use the type system**: Make invalid states unrepresentable
3. **Prefer composition over inheritance**: Traits and generics
4. **Fail at compile time**: Type-state pattern, const generics
5. **Make it hard to misuse**: Builder pattern, sealed traits

### Performance First Principles

1. **Profile before optimizing**: Use criterion and flamegraph
2. **Algorithm before micro-optimization**: O(n) beats optimized O(n²)
3. **Zero-copy when possible**: Cow, references, slices
4. **Pre-allocate when size known**: with_capacity, SmallVec
5. **Leverage LLVM**: const functions, PGO, LTO

### Error Handling Philosophy

1. **Libraries expose, applications handle**: thiserror vs anyhow
2. **Rich context over error codes**: Use error chains
3. **Fail fast, recover gracefully**: ? operator everywhere
4. **Panic for bugs, Result for errors**: Clear distinction

### Testing Excellence

1. **Unit test behaviors, not implementation**: One assertion per test
2. **Property test invariants**: Find edge cases automatically
3. **Benchmark realistically**: Use production-like data
4. **Mock at boundaries**: Database, network, filesystem
5. **Coverage guides, doesn't dictate**: Focus on critical paths

### Concurrency Best Practices

1. **Message passing > shared state**: Channels first
2. **Async for I/O, threads for CPU**: Right tool for the job
3. **Rayon for data parallelism**: Simple and safe
4. **Arc<Mutex<T>> for shared mutation**: Standard pattern
5. **Actor pattern for complex state**: Encapsulation and safety

This comprehensive guide provides a complete foundation for writing idiomatic, performant, and maintainable Rust code in 2025, suitable for an LLM to follow precisely when generating Rust code.
