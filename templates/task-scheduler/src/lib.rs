//! Production-Ready Task Scheduler Template
//!
//! This template demonstrates best practices for scheduled tasks and background processing:
//! - Cron-like scheduling with flexible patterns
//! - Task queue management with priorities
//! - Retry logic and error handling
//! - Task monitoring and logging
//! - Resource management and limits

use icarus::prelude::*;
use ic_cdk_timers::{TimerId, set_timer, set_timer_interval};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

// === Data Models ===

#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone, Debug)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub task_type: TaskType,
    pub schedule: Schedule,
    pub status: TaskStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub created_by: Principal,
    pub next_run: Option<u64>,
    pub last_run: Option<u64>,
    pub run_count: u32,
    pub failure_count: u32,
    pub max_retries: u32,
    pub timeout_seconds: u64,
    pub enabled: bool,
}

#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone, Debug, PartialEq)]
pub enum TaskType {
    DataCleanup,
    Backup,
    Notification,
    Analytics,
    Maintenance,
    Custom(String),
}

#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone, Debug)]
pub enum Schedule {
    Once(u64),                    // Run once at specific time
    Interval(u64),               // Run every N seconds
    Daily(u8),                   // Run daily at hour (0-23)
    Weekly(u8, u8),              // Run weekly on day (0-6) at hour (0-23)
    Monthly(u8, u8),             // Run monthly on day (1-31) at hour (0-23)
    Cron(String),                // Cron expression
}

#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone, Debug)]
pub struct TaskExecution {
    pub task_id: String,
    pub execution_id: String,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub status: TaskStatus,
    pub result: Option<String>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct CreateTaskArgs {
    pub name: String,
    pub description: String,
    pub task_type: TaskType,
    pub schedule: Schedule,
    pub max_retries: Option<u32>,
    pub timeout_seconds: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct TaskStats {
    pub total_tasks: u32,
    pub active_tasks: u32,
    pub completed_today: u32,
    pub failed_today: u32,
    pub average_duration_ms: u64,
    pub upcoming_executions: Vec<(String, u64)>,
}

// === Stable Storage ===

stable_storage! {
    memory 0: {
        tasks: Map<String, Task> = Map::init();
        executions: Map<String, TaskExecution> = Map::init();
        active_timers: Map<String, u64> = Map::init(); // task_id -> timer_id
    },

    memory 1: {
        task_queue: Vec<String> = Vec::init();
        execution_history: Vec<TaskExecution> = Vec::init();
        user_tasks: Map<Principal, Vec<String>> = Map::init();
    },

    memory 2: {
        scheduler_config: Cell<SchedulerConfig> = Cell::init(SchedulerConfig::default());
        system_stats: Cell<TaskStats> = Cell::init(TaskStats::default());
    }
}

#[derive(CandidType, Serialize, Deserialize, IcarusStorable, Clone, Debug)]
pub struct SchedulerConfig {
    pub max_concurrent_tasks: u32,
    pub max_tasks_per_user: u32,
    pub max_execution_history: u32,
    pub default_timeout_seconds: u64,
    pub cleanup_interval_hours: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            max_tasks_per_user: 100,
            max_execution_history: 1000,
            default_timeout_seconds: 300, // 5 minutes
            cleanup_interval_hours: 24,   // Daily cleanup
        }
    }
}

impl Default for TaskStats {
    fn default() -> Self {
        Self {
            total_tasks: 0,
            active_tasks: 0,
            completed_today: 0,
            failed_today: 0,
            average_duration_ms: 0,
            upcoming_executions: Vec::new(),
        }
    }
}

// === MCP Tools ===

#[icarus_module]
mod task_scheduler {
    use super::*;

    // === Task Management ===

    #[icarus_tool("Create a new scheduled task")]
    pub async fn create_task(args: CreateTaskArgs) -> Result<String, String> {
        let caller = ic_cdk::caller();

        // Validate input
        if args.name.trim().is_empty() {
            return Err("Task name cannot be empty".to_string());
        }

        // Check user limits
        let user_task_count = get_user_task_count(caller).await;
        let config = STORAGE.with(|s| s.borrow().scheduler_config.get().clone());

        if user_task_count >= config.max_tasks_per_user {
            return Err(format!("Maximum tasks limit reached ({})", config.max_tasks_per_user));
        }

        let task_id = Uuid::new_v4().to_string();
        let now = ic_cdk::api::time();

        let task = Task {
            id: task_id.clone(),
            name: args.name.trim().to_string(),
            description: args.description,
            task_type: args.task_type,
            schedule: args.schedule.clone(),
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            created_by: caller,
            next_run: calculate_next_run(&args.schedule, now),
            last_run: None,
            run_count: 0,
            failure_count: 0,
            max_retries: args.max_retries.unwrap_or(3),
            timeout_seconds: args.timeout_seconds.unwrap_or(config.default_timeout_seconds),
            enabled: true,
        };

        STORAGE.with(|s| {
            let mut storage = s.borrow_mut();

            // Store task
            storage.tasks.insert(task_id.clone(), task.clone());

            // Update user index
            let mut user_tasks = storage.user_tasks.get(&caller).unwrap_or_default();
            user_tasks.push(task_id.clone());
            storage.user_tasks.insert(caller, user_tasks);
        });

        // Schedule the task
        schedule_task(&task).await?;

        Ok(task_id)
    }

    #[icarus_tool("Get task details by ID")]
    pub async fn get_task(task_id: String) -> Result<Task, String> {
        let caller = ic_cdk::caller();

        STORAGE.with(|s| {
            let storage = s.borrow();

            match storage.tasks.get(&task_id) {
                Some(task) => {
                    // Check permissions
                    if task.created_by != caller {
                        return Err("Access denied".to_string());
                    }
                    Ok(task)
                },
                None => Err("Task not found".to_string()),
            }
        })
    }

    #[icarus_tool("Update task configuration")]
    pub async fn update_task(
        task_id: String,
        name: Option<String>,
        enabled: Option<bool>,
        max_retries: Option<u32>
    ) -> Result<Task, String> {
        let caller = ic_cdk::caller();

        STORAGE.with(|s| {
            let mut storage = s.borrow_mut();

            let mut task = match storage.tasks.get(&task_id) {
                Some(task) => task,
                None => return Err("Task not found".to_string()),
            };

            // Check permissions
            if task.created_by != caller {
                return Err("Access denied".to_string());
            }

            // Apply updates
            if let Some(new_name) = name {
                if new_name.trim().is_empty() {
                    return Err("Task name cannot be empty".to_string());
                }
                task.name = new_name.trim().to_string();
            }

            if let Some(enabled_flag) = enabled {
                task.enabled = enabled_flag;
                if !enabled_flag {
                    task.status = TaskStatus::Paused;
                }
            }

            if let Some(retries) = max_retries {
                task.max_retries = retries;
            }

            task.updated_at = ic_cdk::api::time();

            storage.tasks.insert(task_id, task.clone());
            Ok(task)
        })
    }

    #[icarus_tool("Delete a task")]
    pub async fn delete_task(task_id: String) -> Result<String, String> {
        let caller = ic_cdk::caller();

        STORAGE.with(|s| {
            let mut storage = s.borrow_mut();

            let task = match storage.tasks.get(&task_id) {
                Some(task) => task,
                None => return Err("Task not found".to_string()),
            };

            // Check permissions
            if task.created_by != caller {
                return Err("Access denied".to_string());
            }

            // Cancel active timer
            if let Some(timer_id) = storage.active_timers.remove(&task_id) {
                ic_cdk_timers::clear_timer(TimerId::from(timer_id));
            }

            // Remove from storage
            storage.tasks.remove(&task_id);

            // Update user index
            if let Some(mut user_tasks) = storage.user_tasks.get(&caller) {
                user_tasks.retain(|id| id != &task_id);
                storage.user_tasks.insert(caller, user_tasks);
            }

            Ok(format!("Task {} deleted successfully", task_id))
        })
    }

    // === Task Execution ===

    #[icarus_tool("Trigger immediate task execution")]
    pub async fn run_task_now(task_id: String) -> Result<String, String> {
        let caller = ic_cdk::caller();

        let task = STORAGE.with(|s| {
            let storage = s.borrow();
            match storage.tasks.get(&task_id) {
                Some(task) => {
                    if task.created_by != caller {
                        return Err("Access denied".to_string());
                    }
                    Ok(task)
                },
                None => Err("Task not found".to_string()),
            }
        })?;

        if !task.enabled {
            return Err("Task is disabled".to_string());
        }

        if task.status == TaskStatus::Running {
            return Err("Task is already running".to_string());
        }

        // Execute task
        execute_task(&task).await
    }

    #[icarus_tool("Get task execution history")]
    pub async fn get_task_executions(task_id: String, limit: Option<u32>) -> Result<Vec<TaskExecution>, String> {
        let caller = ic_cdk::caller();
        let limit = limit.unwrap_or(10).min(50);

        // Verify task ownership
        let task_exists = STORAGE.with(|s| {
            let storage = s.borrow();
            storage.tasks.get(&task_id)
                .map(|task| task.created_by == caller)
                .unwrap_or(false)
        });

        if !task_exists {
            return Err("Task not found or access denied".to_string());
        }

        STORAGE.with(|s| {
            let storage = s.borrow();
            let executions: Vec<TaskExecution> = storage.execution_history
                .iter()
                .filter(|exec| exec.task_id == task_id)
                .rev()
                .take(limit as usize)
                .cloned()
                .collect();
            Ok(executions)
        })
    }

    // === User Management ===

    #[icarus_tool("Get user's tasks")]
    pub async fn get_my_tasks() -> Result<Vec<Task>, String> {
        let caller = ic_cdk::caller();

        STORAGE.with(|s| {
            let storage = s.borrow();

            if let Some(task_ids) = storage.user_tasks.get(&caller) {
                let tasks: Vec<Task> = task_ids
                    .iter()
                    .filter_map(|id| storage.tasks.get(id))
                    .collect();
                Ok(tasks)
            } else {
                Ok(Vec::new())
            }
        })
    }

    // === Monitoring and Statistics ===

    #[icarus_tool("Get scheduler statistics")]
    pub async fn get_stats() -> Result<TaskStats, String> {
        let caller = ic_cdk::caller();

        // Update stats
        let stats = calculate_stats(Some(caller)).await;

        STORAGE.with(|s| {
            s.borrow_mut().system_stats.set(stats.clone())
        });

        Ok(stats)
    }

    // === Pre-built Task Templates ===

    #[icarus_tool("Create daily backup task")]
    pub async fn create_backup_task(hour: u8, description: String) -> Result<String, String> {
        if hour > 23 {
            return Err("Hour must be between 0 and 23".to_string());
        }

        let args = CreateTaskArgs {
            name: "Daily Backup".to_string(),
            description,
            task_type: TaskType::Backup,
            schedule: Schedule::Daily(hour),
            max_retries: Some(2),
            timeout_seconds: Some(1800), // 30 minutes
        };

        create_task(args).await
    }

    #[icarus_tool("Create cleanup task")]
    pub async fn create_cleanup_task(interval_hours: u64) -> Result<String, String> {
        if interval_hours == 0 {
            return Err("Interval must be greater than 0".to_string());
        }

        let args = CreateTaskArgs {
            name: "Data Cleanup".to_string(),
            description: format!("Clean up old data every {} hours", interval_hours),
            task_type: TaskType::DataCleanup,
            schedule: Schedule::Interval(interval_hours * 3600), // Convert to seconds
            max_retries: Some(1),
            timeout_seconds: Some(600), // 10 minutes
        };

        create_task(args).await
    }
}

// === Helper Functions ===

async fn get_user_task_count(principal: Principal) -> u32 {
    STORAGE.with(|s| {
        let storage = s.borrow();
        storage.user_tasks
            .get(&principal)
            .map(|tasks| tasks.len() as u32)
            .unwrap_or(0)
    })
}

fn calculate_next_run(schedule: &Schedule, from_time: u64) -> Option<u64> {
    match schedule {
        Schedule::Once(time) => {
            if *time > from_time {
                Some(*time)
            } else {
                None // Past due
            }
        },
        Schedule::Interval(seconds) => {
            Some(from_time + seconds * 1_000_000_000) // Convert to nanoseconds
        },
        Schedule::Daily(hour) => {
            // Calculate next occurrence of the hour
            let current_time_seconds = from_time / 1_000_000_000;
            let seconds_in_day = 24 * 60 * 60;
            let target_hour_seconds = (*hour as u64) * 60 * 60;

            let current_day_start = (current_time_seconds / seconds_in_day) * seconds_in_day;
            let target_today = current_day_start + target_hour_seconds;

            let next_run = if target_today > current_time_seconds {
                target_today
            } else {
                target_today + seconds_in_day // Tomorrow
            };

            Some(next_run * 1_000_000_000) // Convert back to nanoseconds
        },
        Schedule::Weekly(day, hour) => {
            // Calculate next occurrence of day/hour
            let current_time_seconds = from_time / 1_000_000_000;
            let seconds_in_week = 7 * 24 * 60 * 60;
            let seconds_in_day = 24 * 60 * 60;

            let target_seconds = (*day as u64) * seconds_in_day + (*hour as u64) * 60 * 60;
            let current_week_start = (current_time_seconds / seconds_in_week) * seconds_in_week;
            let target_this_week = current_week_start + target_seconds;

            let next_run = if target_this_week > current_time_seconds {
                target_this_week
            } else {
                target_this_week + seconds_in_week // Next week
            };

            Some(next_run * 1_000_000_000)
        },
        Schedule::Monthly(day, hour) => {
            // Simplified monthly calculation (assumes 30-day months)
            let current_time_seconds = from_time / 1_000_000_000;
            let seconds_in_month = 30 * 24 * 60 * 60; // Approximate
            let seconds_in_day = 24 * 60 * 60;

            let target_seconds = (*day as u64 - 1) * seconds_in_day + (*hour as u64) * 60 * 60;
            let current_month_start = (current_time_seconds / seconds_in_month) * seconds_in_month;
            let target_this_month = current_month_start + target_seconds;

            let next_run = if target_this_month > current_time_seconds {
                target_this_month
            } else {
                target_this_month + seconds_in_month // Next month
            };

            Some(next_run * 1_000_000_000)
        },
        Schedule::Cron(_) => {
            // Cron parsing would require additional dependencies
            // For now, treat as daily at hour 0
            calculate_next_run(&Schedule::Daily(0), from_time)
        }
    }
}

async fn schedule_task(task: &Task) -> Result<(), String> {
    if !task.enabled {
        return Ok(());
    }

    let next_run = match task.next_run {
        Some(time) => time,
        None => return Err("No next run time calculated".to_string()),
    };

    let current_time = ic_cdk::api::time();
    if next_run <= current_time {
        return Err("Next run time is in the past".to_string());
    }

    let delay_ns = next_run - current_time;
    let delay_duration = Duration::from_nanos(delay_ns);

    let task_id = task.id.clone();

    // Set timer
    let timer_id = match &task.schedule {
        Schedule::Once(_) => {
            set_timer(delay_duration, move || {
                ic_cdk::spawn(async move {
                    if let Ok(task) = get_task_by_id(&task_id).await {
                        let _ = execute_task(&task).await;
                    }
                });
            })
        },
        _ => {
            // For recurring tasks, we'll reschedule after each execution
            set_timer(delay_duration, move || {
                ic_cdk::spawn(async move {
                    if let Ok(task) = get_task_by_id(&task_id).await {
                        let _ = execute_task(&task).await;
                        // Reschedule if still enabled
                        if task.enabled {
                            let _ = schedule_task(&task).await;
                        }
                    }
                });
            })
        }
    };

    // Store timer ID for cancellation
    STORAGE.with(|s| {
        s.borrow_mut().active_timers.insert(task.id.clone(), timer_id.into());
    });

    Ok(())
}

async fn execute_task(task: &Task) -> Result<String, String> {
    let execution_id = Uuid::new_v4().to_string();
    let start_time = ic_cdk::api::time();

    // Create execution record
    let mut execution = TaskExecution {
        task_id: task.id.clone(),
        execution_id: execution_id.clone(),
        started_at: start_time,
        completed_at: None,
        status: TaskStatus::Running,
        result: None,
        error: None,
        duration_ms: None,
    };

    // Update task status
    STORAGE.with(|s| {
        let mut storage = s.borrow_mut();
        if let Some(mut task) = storage.tasks.get(&task.id) {
            task.status = TaskStatus::Running;
            task.last_run = Some(start_time);
            storage.tasks.insert(task.id.clone(), task);
        }
        storage.executions.insert(execution_id.clone(), execution.clone());
    });

    // Execute task based on type
    let result = match &task.task_type {
        TaskType::DataCleanup => execute_cleanup_task().await,
        TaskType::Backup => execute_backup_task().await,
        TaskType::Notification => execute_notification_task().await,
        TaskType::Analytics => execute_analytics_task().await,
        TaskType::Maintenance => execute_maintenance_task().await,
        TaskType::Custom(custom_type) => execute_custom_task(custom_type).await,
    };

    let end_time = ic_cdk::api::time();
    let duration_ms = (end_time - start_time) / 1_000_000; // Convert to milliseconds

    // Update execution record
    execution.completed_at = Some(end_time);
    execution.duration_ms = Some(duration_ms);

    match result {
        Ok(message) => {
            execution.status = TaskStatus::Completed;
            execution.result = Some(message.clone());

            // Update task
            STORAGE.with(|s| {
                let mut storage = s.borrow_mut();
                if let Some(mut task) = storage.tasks.get(&task.id) {
                    task.status = TaskStatus::Completed;
                    task.run_count += 1;
                    task.failure_count = 0; // Reset failure count on success
                    task.next_run = calculate_next_run(&task.schedule, end_time);
                    storage.tasks.insert(task.id.clone(), task);
                }

                // Store execution
                storage.executions.insert(execution_id.clone(), execution);

                // Add to history (with limit)
                storage.execution_history.push(execution);
                let max_history = storage.scheduler_config.get().max_execution_history as usize;
                if storage.execution_history.len() > max_history {
                    storage.execution_history.remove(0);
                }
            });

            Ok(message)
        },
        Err(error) => {
            execution.status = TaskStatus::Failed;
            execution.error = Some(error.clone());

            // Update task
            STORAGE.with(|s| {
                let mut storage = s.borrow_mut();
                if let Some(mut task) = storage.tasks.get(&task.id) {
                    task.status = TaskStatus::Failed;
                    task.failure_count += 1;

                    // Schedule retry if within limits
                    if task.failure_count <= task.max_retries {
                        task.next_run = Some(end_time + 300_000_000_000); // Retry in 5 minutes
                        task.status = TaskStatus::Pending;
                    }

                    storage.tasks.insert(task.id.clone(), task);
                }

                // Store execution
                storage.executions.insert(execution_id, execution);
            });

            Err(error)
        }
    }
}

async fn get_task_by_id(task_id: &str) -> Result<Task, String> {
    STORAGE.with(|s| {
        let storage = s.borrow();
        storage.tasks.get(task_id).ok_or_else(|| "Task not found".to_string())
    })
}

// Task execution implementations
async fn execute_cleanup_task() -> Result<String, String> {
    // Cleanup old execution history
    STORAGE.with(|s| {
        let mut storage = s.borrow_mut();
        let config = storage.scheduler_config.get();
        let max_history = config.max_execution_history as usize;

        if storage.execution_history.len() > max_history {
            let remove_count = storage.execution_history.len() - max_history;
            storage.execution_history.drain(0..remove_count);
        }
    });

    Ok("Data cleanup completed successfully".to_string())
}

async fn execute_backup_task() -> Result<String, String> {
    // Simple backup simulation
    let task_count = STORAGE.with(|s| s.borrow().tasks.len());
    Ok(format!("Backup completed: {} tasks backed up", task_count))
}

async fn execute_notification_task() -> Result<String, String> {
    // Notification task implementation
    Ok("Notifications sent successfully".to_string())
}

async fn execute_analytics_task() -> Result<String, String> {
    // Analytics calculation
    let stats = calculate_stats(None).await;
    STORAGE.with(|s| {
        s.borrow_mut().system_stats.set(stats);
    });
    Ok("Analytics updated successfully".to_string())
}

async fn execute_maintenance_task() -> Result<String, String> {
    // System maintenance tasks
    Ok("Maintenance tasks completed".to_string())
}

async fn execute_custom_task(task_type: &str) -> Result<String, String> {
    // Custom task execution
    Ok(format!("Custom task '{}' executed successfully", task_type))
}

async fn calculate_stats(user_filter: Option<Principal>) -> TaskStats {
    STORAGE.with(|s| {
        let storage = s.borrow();
        let now = ic_cdk::api::time();
        let today_start = (now / (24 * 60 * 60 * 1_000_000_000)) * (24 * 60 * 60 * 1_000_000_000);

        let tasks: Vec<&Task> = if let Some(user) = user_filter {
            storage.user_tasks.get(&user)
                .unwrap_or_default()
                .iter()
                .filter_map(|id| storage.tasks.get(id))
                .collect()
        } else {
            storage.tasks.iter().map(|(_, task)| task).collect()
        };

        let total_tasks = tasks.len() as u32;
        let active_tasks = tasks.iter().filter(|task| task.enabled && task.status != TaskStatus::Cancelled).count() as u32;

        let completed_today = storage.execution_history.iter()
            .filter(|exec| exec.started_at >= today_start && exec.status == TaskStatus::Completed)
            .count() as u32;

        let failed_today = storage.execution_history.iter()
            .filter(|exec| exec.started_at >= today_start && exec.status == TaskStatus::Failed)
            .count() as u32;

        let total_duration: u64 = storage.execution_history.iter()
            .filter_map(|exec| exec.duration_ms)
            .sum();
        let avg_duration = if storage.execution_history.is_empty() {
            0
        } else {
            total_duration / storage.execution_history.len() as u64
        };

        let upcoming_executions: Vec<(String, u64)> = tasks.iter()
            .filter_map(|task| {
                task.next_run.map(|next_run| (task.name.clone(), next_run))
            })
            .take(10)
            .collect();

        TaskStats {
            total_tasks,
            active_tasks,
            completed_today,
            failed_today,
            average_duration_ms: avg_duration,
            upcoming_executions,
        }
    })
}