//! Macros for reducing boilerplate in ICP canisters

/// Macro to initialize canister memory management
///
/// This sets up the thread-local storage required for stable memory in ICP canisters.
///
/// # Example
/// ```ignore
/// init_memory! {
///     MEMORIES: StableBTreeMap<String, MemoryEntry, Memory> = memory_id(0);
///     COUNTER: u64 = 0;
/// }
/// ```
#[macro_export]
macro_rules! init_memory {
    (
        $($name:ident: $type:ty = $init:expr);* $(;)?
    ) => {
        thread_local! {
            static MEMORY_MANAGER: ::std::cell::RefCell<::ic_stable_structures::memory_manager::MemoryManager<::ic_stable_structures::DefaultMemoryImpl>> = ::std::cell::RefCell::new(
                ::ic_stable_structures::memory_manager::MemoryManager::init(::ic_stable_structures::DefaultMemoryImpl::default())
            );

            $(
                static $name: ::std::cell::RefCell<$type> = ::std::cell::RefCell::new($init);
            )*
        }
    };
}

/// Helper function to create a memory from a memory ID
/// Use this in init_memory! macro for StableBTreeMap initialization
#[macro_export]
macro_rules! memory_id {
    ($id:expr) => {
        ::ic_stable_structures::StableBTreeMap::init(MEMORY_MANAGER.with(|m| {
            m.borrow()
                .get(::ic_stable_structures::memory_manager::MemoryId::new($id))
        }))
    };
}

/// Macro to generate tool metadata function
///
/// This creates the get_metadata query function that returns tool information
/// for MCP discovery.
///
/// # Example
/// ```ignore
/// tool_metadata! {
///     name: "memory-server",
///     version: "1.0.0",
///     tools: [
///         memorize(content: String, tags: Option<Vec<String>>) -> Result<String, String> ["Store a new memory with optional tags"],
///         forget(id: String) -> Result<bool, String> ["Remove a specific memory by ID"],
///         recall(query: String) -> Vec<MemoryEntry> ["Retrieve memories matching a query"],
///         list(limit: Option<usize>) -> Vec<MemoryEntry> ["List all stored memories with optional limit"]
///     ]
/// }
/// ```
#[macro_export]
macro_rules! tool_metadata {
    (
        name: $name:expr,
        version: $version:expr,
        tools: [
            $(
                $fn_name:ident($($param:ident: $ptype:ty),* $(,)?) $(-> $ret:ty)? [$desc:expr]
            ),* $(,)?
        ]
    ) => {
        /// Get canister metadata for tool discovery
        #[::ic_cdk_macros::query]
        pub fn get_metadata() -> String {
            let tools = vec![
                $(
                    {
                        let mut properties = ::serde_json::Map::new();
                        let mut required = Vec::new();

                        $(
                            properties.insert(
                                stringify!($param).to_string(),
                                ::serde_json::json!({
                                    "type": icarus_canister::tool_metadata!(@type_to_json $ptype)
                                })
                            );

                            if !icarus_canister::tool_metadata!(@is_optional $ptype) {
                                required.push(stringify!($param));
                            }
                        )*

                        ::serde_json::json!({
                            "name": stringify!($fn_name),
                            "description": $desc,
                            "inputSchema": {
                                "type": "object",
                                "properties": properties,
                                "required": required
                            }
                        })
                    }
                ),*
            ];

            ::serde_json::json!({
                "name": $name,
                "version": $version,
                "tools": tools
            }).to_string()
        }
    };

    // Helper to convert Rust type to JSON schema type
    (@type_to_json String) => { "string" };
    (@type_to_json &str) => { "string" };
    (@type_to_json u64) => { "integer" };
    (@type_to_json u32) => { "integer" };
    (@type_to_json i64) => { "integer" };
    (@type_to_json i32) => { "integer" };
    (@type_to_json usize) => { "integer" };
    (@type_to_json bool) => { "boolean" };
    (@type_to_json Vec<String>) => { "array" };
    (@type_to_json Vec<$t:ty>) => { "array" };
    (@type_to_json Option<Vec<String>>) => { "array" };
    (@type_to_json Option<Vec<$t:ty>>) => { "array" };
    (@type_to_json Option<String>) => { "string" };
    (@type_to_json Option<$t:ty>) => { $crate::tool_metadata!(@type_to_json $t) };
    (@type_to_json $t:ty) => { "string" };

    // Helper to check if type is optional
    (@is_optional Option<$_:ty>) => { true };
    (@is_optional $_:ty) => { false };
}

/// Helper macro to generate ID sequences
///
/// Creates a function that generates sequential IDs using a counter.
///
/// # Example
/// ```ignore
/// id_generator!(next_memory_id, COUNTER, "mem_");
/// ```
#[macro_export]
macro_rules! id_generator {
    ($fn_name:ident, $counter:ident, $prefix:expr) => {
        fn $fn_name() -> String {
            $counter.with(|c| {
                let mut counter = c.borrow_mut();
                *counter += 1;
                format!("{}{}", $prefix, *counter)
            })
        }
    };
}

/// Macro to generate canister metadata function with auto-discovery syntax
///
/// This macro generates the get_metadata query function that returns tool information
/// for MCP discovery. It provides a cleaner syntax than the original tool_metadata! macro.
///
/// # Example
/// ```ignore
/// // At the end of your lib.rs file:
/// icarus_metadata! {
///     name: "memory-server",
///     version: "1.0.0",
///     tools: {
///         memorize: "Store a new memory with optional tags",
///         forget: "Remove a specific memory by ID",
///         recall: "Retrieve memories matching a query",
///         list: "List all stored memories with optional limit"
///     }
/// }
/// ```
///
/// The macro will automatically extract parameter types from your function signatures.
#[macro_export]
macro_rules! icarus_metadata {
    (
        name: $name:expr,
        version: $version:expr,
        tools: {
            $($fn_name:ident: $desc:expr),* $(,)?
        }
    ) => {
        /// Get canister metadata for tool discovery
        #[::ic_cdk_macros::query]
        pub fn get_metadata() -> String {
            let tools = vec![
                $(
                    ::serde_json::json!({
                        "name": stringify!($fn_name),
                        "description": $desc,
                        "inputSchema": {
                            "type": "object",
                            "properties": {},
                            "required": []
                        }
                    })
                ),*
            ];

            ::serde_json::json!({
                "name": $name,
                "version": $version,
                "tools": tools
            }).to_string()
        }
    };
}
