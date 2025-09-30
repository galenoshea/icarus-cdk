//! Tool registry for automatic tool discovery and management.

use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::future::Future;
use std::pin::Pin;
use std::sync::{OnceLock, RwLock};

use crate::{RuntimeError, RuntimeResult, TOOL_REGISTRY};
use icarus_core::{LegacyToolResult as ToolResult, Tool, ToolId};

/// Type alias for async tool execution function.
#[cfg(feature = "async")]
pub type AsyncToolExecutor =
    fn(&str) -> Pin<Box<dyn Future<Output = RuntimeResult<ToolResult<'static>>> + Send>>;

/// Type alias for sync tool execution function.
pub type SyncToolExecutor = fn(&str) -> RuntimeResult<ToolResult<'static>>;

/// Tool execution functions and dynamic tool storage.
#[derive(Default)]
struct ExecutorStorage {
    #[cfg(feature = "async")]
    async_executors: FxHashMap<ToolId, AsyncToolExecutor>,
    sync_executors: FxHashMap<ToolId, SyncToolExecutor>,
    /// Dynamic tools registered at runtime for hot-reload capability.
    dynamic_tools: FxHashMap<ToolId, Tool>,
}

/// Global registry for tool execution functions.
///
/// This stores the actual execution functions that are registered by
/// macro-generated code at runtime. It provides O(1) lookup for executors
/// with thread-safe read/write access.
static EXECUTOR_REGISTRY: OnceLock<RwLock<ExecutorStorage>> = OnceLock::new();

/// Cached index of all static tools for O(1) lookup performance.
///
/// This index is lazily built on first access and contains all compile-time
/// linkme tools. Dynamic tools are not included as they can change at runtime.
/// For complete lookups including dynamic tools, use `find_by_id` which checks
/// both the cached index and dynamic tools.
static TOOL_INDEX: OnceLock<FxHashMap<ToolId, Tool>> = OnceLock::new();

/// Registry operations for tool discovery and management.
///
/// The registry provides efficient access to tools registered at compile time
/// through the `linkme` distributed slice mechanism. All operations are
/// designed for <10ms execution times.
pub struct ToolRegistry;

impl ToolRegistry {
    /// Lists all available tools in the registry.
    ///
    /// This operation iterates through both the compile-time tool registry
    /// and dynamically registered tools, providing a unified view.
    ///
    /// # Performance
    ///
    /// - Time complexity: O(n + m) where n is static tools and m is dynamic tools
    /// - Target execution time: <5ms for up to 1000 tools
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_runtime::list_tools;
    ///
    /// let tools = list_tools();
    /// for tool in tools {
    ///     println!("Tool: {} - {:?}", tool.name.as_ref(), tool.description);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn list_all() -> SmallVec<[Tool; 16]> {
        let mut tools: SmallVec<[Tool; 16]> =
            TOOL_REGISTRY.iter().map(|tool_fn| tool_fn()).collect();

        // Add dynamic tools if any are registered
        let registry = EXECUTOR_REGISTRY.get_or_init(|| RwLock::new(ExecutorStorage::default()));
        if let Ok(read_guard) = registry.read() {
            for tool in read_guard.dynamic_tools.values() {
                tools.push(tool.clone());
            }
        }

        tools
    }

    /// Finds a specific tool by ID.
    ///
    /// This operation searches through both the compile-time linkme registry
    /// and dynamically registered tools. Dynamic tools take precedence if
    /// there are ID conflicts.
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to find
    ///
    /// # Returns
    ///
    /// * `Some(Tool)` if the tool is found
    /// * `None` if the tool is not registered
    ///
    /// # Performance
    ///
    /// - Time complexity: O(1) average case for static tools, O(m) for dynamic tool check
    /// - Target execution time: <1ms for up to 10,000 tools
    /// - First call builds an index cache (O(n)), subsequent calls are O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_runtime::find_tool;
    /// use icarus_core::ToolId;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tool_id = ToolId::new("add")?;
    /// if let Some(tool) = find_tool(&tool_id) {
    ///     println!("Found tool: {}", tool.description.as_deref().unwrap_or("No description"));
    /// } else {
    ///     println!("Tool not found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn find_by_id(tool_id: &ToolId) -> Option<Tool> {
        // Check dynamic tools first (they take precedence)
        let registry = EXECUTOR_REGISTRY.get_or_init(|| RwLock::new(ExecutorStorage::default()));
        if let Ok(read_guard) = registry.read() {
            if let Some(tool) = read_guard.dynamic_tools.get(tool_id) {
                return Some(tool.clone());
            }
        }

        // Use cached index for O(1) static tool lookup
        let index = TOOL_INDEX.get_or_init(|| {
            // Build index on first access
            TOOL_REGISTRY
                .iter()
                .filter_map(|tool_fn| {
                    let tool = tool_fn();
                    ToolId::new(tool.name.as_ref())
                        .ok()
                        .map(|tool_id| (tool_id, tool))
                })
                .collect()
        });

        index.get(tool_id).cloned()
    }

    /// Builds an index for faster tool lookups.
    ///
    /// This creates a hash map index of all tools (both static and dynamic)
    /// for O(1) lookup performance. Dynamic tools take precedence in case of ID conflicts.
    ///
    /// **Note**: For typical use cases, prefer `find_by_id()` which uses an internal
    /// cached index. This method is useful for advanced scenarios requiring custom
    /// index manipulation or when you need a snapshot of all tools including dynamic ones.
    ///
    /// # Returns
    ///
    /// A hash map indexed by tool ID for fast lookups.
    ///
    /// # Performance
    ///
    /// - Build time: O(n + m) where n is static tools and m is dynamic tools
    /// - Lookup time: O(1) average case
    /// - Memory overhead: ~32 bytes per tool
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_runtime::ToolRegistry;
    /// use icarus_core::ToolId;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let index = ToolRegistry::build_index();
    /// let tool_id = ToolId::new("add")?;
    ///
    /// if let Some(tool) = index.get(&tool_id) {
    ///     println!("Found tool: {}", tool.description.as_deref().unwrap_or("No description"));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn build_index() -> FxHashMap<ToolId, Tool> {
        let mut index: FxHashMap<ToolId, Tool> = TOOL_REGISTRY
            .iter()
            .filter_map(|tool_fn| {
                let tool = tool_fn();
                // Convert Cow<'static, str> to ToolId
                ToolId::new(tool.name.as_ref())
                    .ok()
                    .map(|tool_id| (tool_id, tool))
            })
            .collect();

        // Add dynamic tools (they take precedence if there are conflicts)
        let registry = EXECUTOR_REGISTRY.get_or_init(|| RwLock::new(ExecutorStorage::default()));
        if let Ok(read_guard) = registry.read() {
            for (tool_id, tool) in &read_guard.dynamic_tools {
                index.insert(tool_id.clone(), tool.clone());
            }
        }

        index
    }

    /// Validates registry integrity.
    ///
    /// Checks for common registry issues such as duplicate tool IDs,
    /// invalid tool definitions, or corrupted registry state.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the registry is valid
    /// * `Err(RuntimeError)` if validation fails
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::RegistryError`] if:
    /// - Tool count exceeds 10,000 (registry too large)
    /// - Any tool has an empty description
    /// - Duplicate tool IDs are found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_runtime::ToolRegistry;
    ///
    /// match ToolRegistry::validate() {
    ///     Ok(()) => println!("Registry is valid"),
    ///     Err(e) => println!("Registry validation failed: {}", e),
    /// }
    /// ```
    #[allow(dead_code)]
    pub fn validate() -> RuntimeResult<()> {
        let tools: SmallVec<[Tool; 16]> = TOOL_REGISTRY.iter().map(|tool_fn| tool_fn()).collect();

        let tool_count = tools.len();

        // Check for reasonable tool count
        if tool_count > 10000 {
            return Err(RuntimeError::registry_error(format!(
                "Excessive tool count: {tool_count} (limit: 10000)"
            )));
        }

        // Validate tool definitions using iterator patterns
        if let Some(invalid_tool) = tools.iter().find(|tool| {
            tool.description
                .as_ref()
                .map_or(true, |d| d.trim().is_empty())
        }) {
            return Err(RuntimeError::registry_error(format!(
                "Tool '{}' has empty description",
                invalid_tool.name.as_ref()
            )));
        }

        // Check for duplicate IDs using iterator patterns
        let mut seen_ids = std::collections::HashSet::with_capacity(tool_count);
        if let Some(duplicate_tool) = tools.iter().find(|tool| !seen_ids.insert(&tool.name)) {
            return Err(RuntimeError::registry_error(format!(
                "Duplicate tool ID found: {}",
                duplicate_tool.name.as_ref()
            )));
        }

        Ok(())
    }

    /// Returns registry statistics.
    ///
    /// Provides information about the current state of the tool registry
    /// including both static and dynamic tools, memory usage estimates,
    /// and validation status.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_runtime::ToolRegistry;
    ///
    /// let stats = ToolRegistry::stats();
    /// println!("Registry has {} tools ({} static, {} dynamic)",
    ///          stats.tool_count, stats.static_tool_count, stats.dynamic_tool_count);
    /// ```
    #[allow(dead_code)]
    #[must_use]
    pub fn stats() -> RegistryStats {
        let tools: SmallVec<[Tool; 16]> = Self::list_all();
        let tool_count = tools.len();

        // Count static vs dynamic tools
        let static_tool_count = TOOL_REGISTRY.len();
        let dynamic_tool_count = Self::list_dynamic_tools().len();

        // Estimate memory usage (rough calculation)
        let estimated_memory = tool_count * 256; // ~256 bytes per tool estimate

        // Check for duplicates
        let mut unique_ids = std::collections::HashSet::new();
        let has_duplicates = !tools
            .iter()
            .all(|tool| unique_ids.insert(tool.name.clone()));

        RegistryStats {
            tool_count,
            static_tool_count,
            dynamic_tool_count,
            estimated_memory_bytes: estimated_memory,
            has_duplicates,
            validation_status: Self::validate().is_ok(),
        }
    }

    /// Registers an async tool executor function.
    ///
    /// This method is typically called by macro-generated code to register
    /// the actual execution function for a tool. The executor is stored in
    /// a global registry for O(1) lookup during tool execution.
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to register an executor for
    /// * `executor` - The async executor function for this tool
    ///
    /// # Errors
    ///
    /// Returns `RuntimeError` if the registration fails due to lock contention.
    #[cfg(feature = "async")]
    pub fn register_async_executor(
        tool_id: ToolId,
        executor: AsyncToolExecutor,
    ) -> RuntimeResult<()> {
        let registry = EXECUTOR_REGISTRY.get_or_init(|| RwLock::new(ExecutorStorage::default()));

        let mut write_guard = registry.write().map_err(|_| {
            RuntimeError::registry_error("Failed to acquire write lock on executor registry")
        })?;

        write_guard.async_executors.insert(tool_id, executor);
        Ok(())
    }

    /// Registers a sync tool executor function.
    ///
    /// This method is typically called by macro-generated code to register
    /// the actual execution function for a tool. The executor is stored in
    /// a global registry for O(1) lookup during tool execution.
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to register an executor for
    /// * `executor` - The sync executor function for this tool
    ///
    /// # Errors
    ///
    /// Returns `RuntimeError` if the registration fails due to lock contention.
    pub fn register_sync_executor(
        tool_id: ToolId,
        executor: SyncToolExecutor,
    ) -> RuntimeResult<()> {
        let registry = EXECUTOR_REGISTRY.get_or_init(|| RwLock::new(ExecutorStorage::default()));

        let mut write_guard = registry.write().map_err(|_| {
            RuntimeError::registry_error("Failed to acquire write lock on executor registry")
        })?;

        write_guard.sync_executors.insert(tool_id, executor);
        Ok(())
    }

    /// Gets an executor for a specific tool and executes it asynchronously.
    ///
    /// This method provides a safe way to execute a tool by looking up its executor
    /// and calling it with the provided arguments. The execution is done within
    /// a read lock to ensure thread safety.
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to execute
    /// * `arguments` - JSON string containing the tool arguments
    ///
    /// # Returns
    ///
    /// * `Some(Ok(ToolResult))` if the tool exists and execution succeeded
    /// * `Some(Err(RuntimeError))` if the tool exists but execution failed
    /// * `None` if no executor is found for this tool
    #[cfg(feature = "async")]
    pub async fn execute_tool_async(
        tool_id: &ToolId,
        arguments: &str,
    ) -> Option<RuntimeResult<ToolResult<'static>>> {
        let registry = EXECUTOR_REGISTRY.get()?;
        let executor = {
            let read_guard = registry.read().ok()?;
            read_guard.async_executors.get(tool_id).copied()
        }?;

        Some(executor(arguments).await)
    }

    /// Gets an executor for a specific tool and executes it synchronously.
    ///
    /// This method provides a safe way to execute a tool by looking up its executor
    /// and calling it with the provided arguments. The execution is done within
    /// a read lock to ensure thread safety.
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to execute
    /// * `arguments` - JSON string containing the tool arguments
    ///
    /// # Returns
    ///
    /// * `Some(Ok(ToolResult))` if the tool exists and execution succeeded
    /// * `Some(Err(RuntimeError))` if the tool exists but execution failed
    /// * `None` if no executor is found for this tool
    pub fn execute_tool_sync(
        tool_id: &ToolId,
        arguments: &str,
    ) -> Option<RuntimeResult<ToolResult<'static>>> {
        let registry = EXECUTOR_REGISTRY.get()?;
        let read_guard = registry.read().ok()?;

        if let Some(&executor) = read_guard.sync_executors.get(tool_id) {
            // Copy the function pointer and drop the guard before calling
            drop(read_guard);
            Some(executor(arguments))
        } else {
            None
        }
    }

    /// Checks if a tool exists in the registry (static or dynamic).
    ///
    /// This is an optimized method that checks for tool existence without
    /// cloning the tool. Use this for validation instead of `find_by_id()`
    /// when you only need to check if a tool exists.
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to check
    ///
    /// # Returns
    ///
    /// * `true` if the tool is registered (static or dynamic)
    /// * `false` if the tool is not found
    ///
    /// # Performance
    ///
    /// - Time complexity: O(1) for both static and dynamic tools
    /// - No memory allocation or cloning
    /// - Target execution time: <100μs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_runtime::ToolRegistry;
    /// use icarus_core::ToolId;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tool_id = ToolId::new("calculate")?;
    /// if ToolRegistry::has_tool(&tool_id) {
    ///     println!("Tool exists!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn has_tool(tool_id: &ToolId) -> bool {
        // Check dynamic tools first
        let registry = EXECUTOR_REGISTRY.get_or_init(|| RwLock::new(ExecutorStorage::default()));
        if let Ok(read_guard) = registry.read() {
            if read_guard.dynamic_tools.contains_key(tool_id) {
                return true;
            }
        }

        // Check static tools in cached index
        let index = TOOL_INDEX.get_or_init(|| {
            TOOL_REGISTRY
                .iter()
                .filter_map(|tool_fn| {
                    let tool = tool_fn();
                    ToolId::new(tool.name.as_ref())
                        .ok()
                        .map(|tool_id| (tool_id, tool))
                })
                .collect()
        });

        index.contains_key(tool_id)
    }

    /// Checks if an executor is registered for a tool.
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to check
    ///
    /// # Returns
    ///
    /// * `true` if an executor is registered for this tool
    /// * `false` if no executor is found
    #[must_use]
    pub fn has_executor(tool_id: &ToolId) -> bool {
        EXECUTOR_REGISTRY
            .get()
            .and_then(|registry| registry.read().ok())
            .is_some_and(|read_guard| {
                #[cfg(feature = "async")]
                let has_async = read_guard.async_executors.contains_key(tool_id);
                #[cfg(not(feature = "async"))]
                let has_async = false;

                let has_sync_executor = read_guard.sync_executors.contains_key(tool_id);
                has_async || has_sync_executor
            })
    }

    /// Initializes the executor registry.
    ///
    /// This method ensures the executor registry is initialized. It's typically
    /// called by macro-generated code during static initialization.
    pub fn initialize_executors() {
        EXECUTOR_REGISTRY.get_or_init(|| RwLock::new(ExecutorStorage::default()));
    }

    /// Registers a tool dynamically for hot-reload capability.
    ///
    /// This allows tools to be added at runtime without recompilation.
    /// Dynamic tools are stored separately from compile-time linkme tools.
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool to register dynamically
    ///
    /// # Returns
    ///
    /// * `Ok(())` if registration succeeded
    /// * `Err(RuntimeError)` if registration failed due to lock contention
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::RegistryError`] if unable to acquire write lock
    /// on the executor registry due to lock contention.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_runtime::ToolRegistry;
    /// use icarus_core::Tool;
    /// use std::sync::Arc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tool = Tool::new(
    ///     "dynamic_tool",
    ///     "A dynamically registered tool",
    ///     Arc::new(serde_json::Map::new()),
    /// );
    ///
    /// ToolRegistry::register_dynamic_tool(tool)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn register_dynamic_tool(tool: Tool) -> RuntimeResult<()> {
        let registry = EXECUTOR_REGISTRY.get_or_init(|| RwLock::new(ExecutorStorage::default()));

        let mut write_guard = registry.write().map_err(|_| {
            RuntimeError::registry_error(
                "Failed to acquire write lock for dynamic tool registration",
            )
        })?;

        // Convert tool name to ToolId
        let tool_id = ToolId::new(tool.name.as_ref())
            .map_err(|e| RuntimeError::registry_error(format!("Invalid tool name: {e}")))?;

        write_guard.dynamic_tools.insert(tool_id, tool);
        Ok(())
    }

    /// Unregisters a dynamically registered tool.
    ///
    /// This removes a tool that was previously registered with `register_dynamic_tool`.
    /// This only affects dynamic tools, not compile-time linkme tools.
    ///
    /// # Arguments
    ///
    /// * `tool_id` - The ID of the tool to unregister
    ///
    /// # Returns
    ///
    /// * `Some(Tool)` if the tool was found and removed
    /// * `None` if no dynamic tool with that ID was found
    /// * `Err(RuntimeError)` if operation failed due to lock contention
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::RegistryError`] if unable to acquire write lock
    /// on the executor registry due to lock contention.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_runtime::ToolRegistry;
    /// use icarus_core::ToolId;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tool_id = ToolId::new("dynamic_tool")?;
    /// if let Ok(Some(removed_tool)) = ToolRegistry::unregister_dynamic_tool(&tool_id) {
    ///     println!("Removed tool: {}", removed_tool.description.as_deref().unwrap_or("No description"));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn unregister_dynamic_tool(tool_id: &ToolId) -> RuntimeResult<Option<Tool>> {
        let registry = EXECUTOR_REGISTRY.get_or_init(|| RwLock::new(ExecutorStorage::default()));

        let mut write_guard = registry.write().map_err(|_| {
            RuntimeError::registry_error(
                "Failed to acquire write lock for dynamic tool unregistration",
            )
        })?;

        Ok(write_guard.dynamic_tools.remove(tool_id))
    }

    /// Lists all dynamic tools currently registered for hot-reload.
    ///
    /// This returns only the tools that were registered dynamically,
    /// not the compile-time linkme tools.
    ///
    /// # Returns
    ///
    /// A vector of all dynamically registered tools.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_runtime::ToolRegistry;
    ///
    /// let dynamic_tools = ToolRegistry::list_dynamic_tools();
    /// for tool in dynamic_tools {
    ///     println!("Dynamic tool: {} - {:?}", tool.name.as_ref(), tool.description);
    /// }
    /// ```
    #[must_use]
    pub fn list_dynamic_tools() -> SmallVec<[Tool; 16]> {
        let registry = EXECUTOR_REGISTRY.get_or_init(|| RwLock::new(ExecutorStorage::default()));

        match registry.read() {
            Ok(read_guard) => read_guard.dynamic_tools.values().cloned().collect(),
            Err(_) => SmallVec::new(),
        }
    }

    /// Clears all dynamically registered tools.
    ///
    /// This removes all tools that were registered with `register_dynamic_tool`.
    /// This only affects dynamic tools, not compile-time linkme tools.
    ///
    /// # Returns
    ///
    /// The number of tools that were removed.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::RegistryError`] if unable to acquire write lock
    /// on the executor registry due to lock contention.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_runtime::ToolRegistry;
    ///
    /// let removed_count = ToolRegistry::clear_dynamic_tools().unwrap_or(0);
    /// println!("Cleared {} dynamic tools", removed_count);
    /// ```
    pub fn clear_dynamic_tools() -> RuntimeResult<usize> {
        let registry = EXECUTOR_REGISTRY.get_or_init(|| RwLock::new(ExecutorStorage::default()));

        let mut write_guard = registry.write().map_err(|_| {
            RuntimeError::registry_error("Failed to acquire write lock for dynamic tool clearing")
        })?;

        let count = write_guard.dynamic_tools.len();
        write_guard.dynamic_tools.clear();
        Ok(count)
    }
}

/// Statistics about the tool registry.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RegistryStats {
    /// Total number of tools in the registry (static + dynamic)
    pub tool_count: usize,
    /// Number of static tools (compile-time linkme)
    pub static_tool_count: usize,
    /// Number of dynamic tools (runtime registered)
    pub dynamic_tool_count: usize,
    /// Estimated memory usage in bytes
    pub estimated_memory_bytes: usize,
    /// Whether duplicate tool IDs were detected
    pub has_duplicates: bool,
    /// Whether registry validation passed
    pub validation_status: bool,
}

impl RegistryStats {
    /// Returns a human-readable summary of the registry stats.
    #[inline]
    #[allow(dead_code)]
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Registry: {} tools ({} static, {} dynamic), ~{} bytes, duplicates: {}, valid: {}",
            self.tool_count,
            self.static_tool_count,
            self.dynamic_tool_count,
            self.estimated_memory_bytes,
            self.has_duplicates,
            self.validation_status
        )
    }
}

/// Convenience function to list all available tools.
///
/// This is a shorthand for `ToolRegistry::list_all()`.
#[inline]
#[must_use]
pub fn list_tools() -> SmallVec<[Tool; 16]> {
    ToolRegistry::list_all()
}

/// Convenience function to find a tool by ID.
///
/// This is a shorthand for `ToolRegistry::find_by_id()`.
#[inline]
#[must_use]
pub fn find_tool(tool_id: &ToolId) -> Option<Tool> {
    ToolRegistry::find_by_id(tool_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools() {
        let _tools = list_tools();
        // Should not panic even if no tools are registered
        // tools.len() is always >= 0 since it's usize
    }

    #[test]
    fn test_find_nonexistent_tool() {
        let tool_id = ToolId::new("nonexistent_tool_12345").expect("Valid tool ID for test");
        let tool = find_tool(&tool_id);
        assert!(tool.is_none());
    }

    #[test]
    fn test_registry_validation() {
        // Validation should not fail for empty registry
        let result = ToolRegistry::validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_registry_stats() {
        let stats = ToolRegistry::stats();
        // stats.tool_count is always >= 0 since it's usize
        assert!(!stats.summary().is_empty());
    }

    #[test]
    fn test_build_index() {
        let index = ToolRegistry::build_index();
        // Should not panic and should be consistent with list_tools
        assert_eq!(index.len(), list_tools().len());
    }
}
