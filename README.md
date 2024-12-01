# Tokio Task Scheduler

[![Crates.io](https://img.shields.io/crates/v/tokio-task-scheduler.svg)](https://crates.io/crates/tokio-task-scheduler)
[![Documentation](https://docs.rs/tokio-task-scheduler/badge.svg)](https://docs.rs/tokio-task-scheduler)
[![License:MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![dependency status](https://deps.rs/repo/github/cploutarchou/scheduler/status.svg)](https://deps.rs/repo/github/cploutarchou/scheduler)

A powerful, non-blocking task scheduler for Rust with async/await support, built on top of Tokio.

## Features

✨ **Async First**: Built on Tokio for true asynchronous task execution

🔄 **Flexible Scheduling**: Interval-based (seconds, minutes, hours, days), daily at specific times, and custom scheduling patterns

📦 **Persistence**: Optional SQLite-based task storage

🛡️ **Robust Error Handling**: Comprehensive error types and recovery strategies

🔧 **Builder Pattern**: Intuitive task configuration

🧪 **Well Tested**: Extensive test coverage

🚀 **Production Ready**: Version 1.0.0 with stable API

## New Features (Coming Soon)

🔄 **Advanced Scheduling**:
  - Cron-style expressions for complex scheduling patterns
  - Task dependencies and chaining
  - Task groups for batch operations
  - Priority levels for task execution

📊 **Monitoring & Metrics**:
  - Real-time task execution metrics
  - Performance statistics and insights
  - Task history and audit logs
  - Webhook notifications for task events

⚙️ **Enhanced Control**:
  - Task timeouts with configurable actions
  - Concurrency limits per task and globally
  - Task tags for better organization
  - Filtering and search capabilities

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tokio-task-scheduler = "1.0.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use tokio_task_scheduler::{Scheduler, TaskBuilder};
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new scheduler
    let scheduler = Scheduler::new();
    
    // Schedule a recurring task
    let task = TaskBuilder::new("heartbeat", || {
        println!("System heartbeat");
        Ok(())
    })
    .every_seconds(30)
    .build();
    
    // Add and start the task
    scheduler.add_task(task).await?;
    let rx = scheduler.start().await;
    
    // Run for some time
    tokio::time::sleep(Duration::from_secs(120)).await;
    
    // Gracefully shutdown
    scheduler.stop().await?;
    Ok(())
}
```

## Advanced Usage

### Task Persistence

```rust
use tokio_task_scheduler::{TaskPersistenceManager, Task};

async fn persist_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let persistence = TaskPersistenceManager::new("tasks.db").await?;
    
    // Save a task
    let task = Task::new("important_job", || Ok(()));
    persistence.save_task(&task).await?;
    
    // Retrieve tasks
    let tasks = persistence.list_tasks().await?;
    Ok(())
}
```

### Daily Scheduled Tasks

```rust
let daily_report = TaskBuilder::new("daily_report", || {
    println!("Generating daily report");
    Ok(())
})
.daily()
.at("08:00")? // Runs every day at 8 AM
.build();
```

### Error Handling

```rust
use tokio_task_scheduler::SchedulerError;

match scheduler.get_task_status("task_id").await {
    Ok(status) => println!("Task status: {:?}", status),
    Err(SchedulerError::TaskNotFound(_)) => println!("Task not found"),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Task Dependencies

```rust
let task_b = TaskBuilder::new("dependent_task", || {
    println!("This runs after task_a");
    Ok(())
})
.depends_on("task_a")
.build();
```

### Task Groups

```rust
use tokio_task_scheduler::TaskGroup;

let group = TaskGroup::new("batch_jobs")
    .add_task(task_a)
    .add_task(task_b)
    .with_concurrency(2)
    .build();

scheduler.add_group(group).await?;
```

### Cron-style Scheduling

```rust
let cron_task = TaskBuilder::new("complex_schedule", || {
    println!("Running on custom schedule");
    Ok(())
})
.cron("0 0 * * *")? // Runs at midnight every day
.build();
```

### Task Monitoring

```rust
// Subscribe to task events
let mut events = scheduler.subscribe_events().await;

tokio::spawn(async move {
    while let Some(event) = events.recv().await {
        match event {
            TaskEvent::Started { id, .. } => println!("Task {} started", id),
            TaskEvent::Completed { id, duration, .. } => {
                println!("Task {} completed in {:?}", id, duration)
            }
            TaskEvent::Failed { id, error, .. } => {
                println!("Task {} failed: {}", id, error)
            }
        }
    }
});
```

### Task Timeouts

```rust
let timeout_task = TaskBuilder::new("long_running_task", || {
    // Long running operation
    Ok(())
})
.timeout(Duration::from_secs(30))
.on_timeout(|task| {
    println!("Task {} timed out", task.name());
    Ok(())
})
.build();
```

### Task Tags and Filtering

```rust
let tagged_task = TaskBuilder::new("tagged_task", || Ok(()))
    .tags(vec!["production", "critical"])
    .build();

// Filter tasks by tags
let critical_tasks = scheduler.get_tasks_by_tag("critical").await?;
```

## Task Status Lifecycle

Tasks go through the following states:
- `Pending`: Waiting to be executed
- `Running`: Currently executing
- `Completed`: Successfully finished
- `Failed`: Execution failed with error
- `Paused`: Temporarily suspended
- `Cancelled`: Permanently stopped

## Performance

Benchmarks run on MacBook Pro M1:
- Task Creation: ~1.2µs
- Task Scheduling: ~2.3µs
- Persistence Operations: ~5.1ms

## Configuration Options

### Scheduler Options
- Custom retry policies
- Persistence configuration
- Error recovery strategies

### Task Options
- Execution intervals
- Start times
- Retry attempts
- Custom error handlers

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Tokio](https://tokio.rs/)
- Persistence powered by [Rusqlite](https://github.com/rusqlite/rusqlite)
- Error handling with [thiserror](https://github.com/dtolnay/thiserror)
