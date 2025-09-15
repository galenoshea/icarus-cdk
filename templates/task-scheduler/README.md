# Task Scheduler Template

A production-ready template for time-based operations, cron jobs, and background task processing on ICP.

## Features

### ⏰ Scheduling Options
- **One-time Tasks**: Execute once at a specific time
- **Interval Tasks**: Repeat every N seconds/minutes/hours
- **Daily Tasks**: Run daily at a specific hour
- **Weekly Tasks**: Run weekly on specific day and hour
- **Monthly Tasks**: Run monthly on specific day and hour
- **Cron Support**: Traditional cron expressions (basic support)

### 🔄 Task Management
- **Retry Logic**: Configurable retry attempts with exponential backoff
- **Timeout Handling**: Per-task timeout configuration
- **Status Tracking**: Real-time task status monitoring
- **Execution History**: Detailed logs of all task executions
- **Error Handling**: Comprehensive error tracking and reporting

### 📊 Monitoring & Analytics
- **Real-time Statistics**: Task performance metrics
- **Execution Tracking**: Success/failure rates and timing
- **Resource Monitoring**: System resource usage tracking
- **Alert System**: Configurable alerts for task failures

## Quick Start

1. **Create from Template**:
   ```bash
   icarus new my-scheduler --template task-scheduler
   cd my-scheduler
   ```

2. **Deploy Locally**:
   ```bash
   dfx start --background
   icarus build
   icarus deploy --network local
   ```

3. **Connect to Claude Desktop**:
   ```json
   {
     "mcpServers": {
       "my-scheduler": {
         "command": "icarus",
         "args": ["bridge", "start", "--canister-id", "YOUR_CANISTER_ID"]
       }
     }
   }
   ```

## Available MCP Tools

### Task Management
- `create_task` - Create a new scheduled task
- `get_task` - Retrieve task details by ID
- `update_task` - Update task configuration
- `delete_task` - Delete a task and cancel its schedule
- `get_my_tasks` - Get all tasks for current user

### Task Execution
- `run_task_now` - Trigger immediate task execution
- `get_task_executions` - Get execution history for a task

### Pre-built Templates
- `create_backup_task` - Create daily backup task
- `create_cleanup_task` - Create data cleanup task

### Monitoring
- `get_stats` - Get scheduler statistics and metrics

## Usage Examples

### Creating a Daily Backup Task
```javascript
Human: Create a daily backup task that runs at 2 AM with description "Daily data backup"

Claude: I'll create a daily backup task for you.
[Uses create_backup_task with hour=2]
✅ Daily backup task created with ID: backup-123-abc
```

### Creating a Custom Cleanup Task
```javascript
Human: Create a cleanup task that runs every 6 hours to remove old logs

Claude: I'll set up a cleanup task for you.
[Uses create_cleanup_task with interval_hours=6]
✅ Cleanup task created with ID: cleanup-456-def
```

### Monitoring Task Performance
```javascript
Human: Show me the scheduler statistics

Claude: Here are your scheduler statistics:
[Uses get_stats tool]
📊 Total Tasks: 12
🏃 Active Tasks: 8
✅ Completed Today: 24
❌ Failed Today: 1
⏱️ Average Duration: 1.2 seconds
```

## Task Types

### Built-in Task Types
- **DataCleanup**: Remove old data and logs
- **Backup**: Data backup operations
- **Notification**: Send alerts and notifications
- **Analytics**: Calculate metrics and reports
- **Maintenance**: System maintenance tasks
- **Custom**: User-defined custom tasks

### Schedule Types
```rust
pub enum Schedule {
    Once(u64),                    // Run once at timestamp
    Interval(u64),               // Every N seconds
    Daily(u8),                   // Daily at hour (0-23)
    Weekly(u8, u8),              // Weekly: day (0-6), hour (0-23)
    Monthly(u8, u8),             // Monthly: day (1-31), hour (0-23)
    Cron(String),                // Cron expression
}
```

## Configuration

Customize the scheduler in `src/lib.rs`:

```rust
impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,        // Max parallel executions
            max_tasks_per_user: 100,         // Per-user task limit
            max_execution_history: 1000,     // History retention
            default_timeout_seconds: 300,    // 5-minute default timeout
            cleanup_interval_hours: 24,      // Daily cleanup
        }
    }
}
```

## Task Execution Lifecycle

1. **Scheduling**: Task is created and scheduled based on its schedule
2. **Queuing**: Task is added to execution queue at scheduled time
3. **Execution**: Task logic runs with timeout and error handling
4. **Retry Logic**: Failed tasks retry up to max_retries limit
5. **Completion**: Results are logged and next execution is scheduled
6. **Cleanup**: Old execution records are cleaned up periodically

## Advanced Features

### Custom Task Implementation

```rust
#[icarus_tool("Create custom monitoring task")]
pub async fn create_monitoring_task(
    endpoint: String,
    check_interval_minutes: u64
) -> Result<String, String> {
    let args = CreateTaskArgs {
        name: format!("Monitor {}", endpoint),
        description: format!("Health check for {}", endpoint),
        task_type: TaskType::Custom("health_check".to_string()),
        schedule: Schedule::Interval(check_interval_minutes * 60),
        max_retries: Some(3),
        timeout_seconds: Some(30),
    };

    create_task(args).await
}
```

### Error Handling and Retries

```rust
// Tasks automatically retry on failure with exponential backoff
// Retry timing: 5 minutes -> 10 minutes -> 20 minutes
// After max_retries, task is marked as permanently failed
```

### Resource Management

```rust
// Built-in resource limits prevent system overload
// - Maximum concurrent tasks
// - Per-user task limits
// - Execution timeout limits
// - Memory usage monitoring
```

## Production Deployment

### Performance Optimization
```rust
// Recommended settings for production
max_concurrent_tasks: 20,
max_tasks_per_user: 500,
max_execution_history: 5000,
default_timeout_seconds: 600,
cleanup_interval_hours: 12,
```

### Monitoring Setup
```bash
# Monitor canister cycles and performance
dfx canister status YOUR_CANISTER_ID --network ic

# Set up alerts for task failures
# (implement webhook notifications in custom tasks)
```

### Backup Strategy
```rust
// Built-in backup task template
create_backup_task(2, "Daily system backup".to_string()).await
```

## Best Practices

### Task Design
- Keep tasks idempotent (safe to run multiple times)
- Use appropriate timeouts based on expected execution time
- Handle errors gracefully with meaningful error messages
- Log important events for debugging

### Schedule Planning
- Avoid scheduling too many tasks at the same time
- Consider timezone implications for daily/weekly tasks
- Use intervals appropriate for task complexity
- Plan for maintenance windows

### Resource Management
- Monitor canister cycles usage
- Set reasonable task limits per user
- Clean up old data regularly
- Use appropriate retry counts

### Error Handling
- Implement proper error messages
- Log failures for debugging
- Set up alerts for critical task failures
- Consider fallback mechanisms

## License

This template is provided under the BSL-1.1 license. See the main SDK license for details.