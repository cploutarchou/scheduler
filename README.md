# Rust Scheduler

A non-blocking, flexible task scheduler for Rust with asynchronous execution.

## Features

- Non-blocking task scheduling
- Async runtime support (powered by Tokio)
- Multiple scheduling intervals (seconds, minutes, daily)
- Robust error handling and recovery strategies
- Flexible task management
- Unique task identification
- Thread-safe task execution
- Persistent task storage

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
scheduler = { git = "https://github.com/cploutarchou/scheduler" }
tokio = { version = "1.0", features = ["full"] }
```

## Basic Usage

### Scheduling Tasks

```rust
use scheduler::{Scheduler, TaskBuilder};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = Scheduler::new();
    
    // Schedule a task to run every second
    let task1 = TaskBuilder::new("print_task", || {
        println!("Task executed!");
        Ok(())
    })
    .every_seconds(1)
    .build();
    
    // Schedule a daily task at a specific time
    let task2 = TaskBuilder::new("daily_task", || {
        println!("Daily task at 10:30");
        Ok(())
    })
    .daily()
    .at("10:30")
    .unwrap()
    .build();
    
    // Add tasks to the scheduler
    scheduler.add_task(task1).await?;
    scheduler.add_task(task2).await?;
    
    // Start the scheduler
    let rx = scheduler.start().await;
    
    // Run for a short duration
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    
    // Stop the scheduler
    scheduler.stop().await?;
    
    Ok(())
}
```

## Performance Benchmarks

### Task Addition Performance
- Adding 100 tasks: **140.68 µs**
  - 9% of measurements had outliers (6 high mild, 3 high severe)

### Scheduler Operations
- Start/Stop Scheduler: **1.0958 µs**
  - 13% of measurements had outliers (1 low severe, 3 low mild, 4 high mild, 5 high severe)

## Error Handling

The Scheduler provides robust error handling with detailed error types and recovery strategies.

### Error Types

- `TaskExecutionFailed`: Occurs when a task fails to execute
- `TaskNotFound`: Indicates a task with a specific ID was not found
- `InvalidTimeFormat`: Signals an incorrect time format
- `InvalidSchedule`: Represents an invalid scheduling configuration
- `TaskAlreadyExists`: Prevents duplicate task registration
- `MaxRetriesExceeded`: Tracks repeated task failures
- `SchedulingConflict`: Detects scheduling conflicts

### Recovery Strategies

Each error type comes with a suggested recovery strategy:

- `Retry`: Automatically retry the task
- `Ignore`: Skip the error and continue
- `Correct`: Attempt to fix the error
- `Abort`: Stop further execution
- `Reschedule`: Attempt to schedule the task at a different time

## Persistent Task Storage

The Scheduler provides robust persistent task storage using SQLite, enabling you to save, retrieve, and manage tasks across application restarts.

### Key Features
- 💾 SQLite-based persistent storage
- 🔄 Save and retrieve tasks
- 📋 List all persisted tasks
- 🗑️ Delete individual tasks
- 🧹 Clear all tasks from storage
- 🔒 Thread-safe task persistence
- 🕒 Preserve task metadata (status, creation time, last execution)

### Usage Example

```rust
use scheduler::persistence::TaskPersistenceManager;
use scheduler::task::TaskBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a persistence manager with a database file
    let persistence_manager = TaskPersistenceManager::new("tasks.db").await?;
    
    // Create a daily backup task
    let backup_task = TaskBuilder::new("backup_task", || {
        // Backup logic
        Ok(())
    })
    .every_hours(24)
    .build();
    
    // Save the task to persistent storage
    persistence_manager.save_task(&backup_task).await?;
    
    // Retrieve all persisted tasks
    let tasks = persistence_manager.list_tasks().await?;
    println!("Persisted Tasks: {}", tasks.len());
    
    // Retrieve a specific task by ID
    let task_details = persistence_manager.get_task(&backup_task.id().to_string()).await?;
    
    // Delete a specific task
    persistence_manager.delete_task(&backup_task.id().to_string()).await?;
    
    // Clear all tasks from storage
    persistence_manager.clear_tasks().await?;
    
    Ok(())
}
```

### Advanced Persistence Scenarios

#### Handling Task Metadata
The `PersistableTask` struct captures comprehensive task information:
- Unique Task ID
- Task Name
- Current Status
- Creation Timestamp
- Last Execution Time
- Next Execution Time
- Interval Configuration
- Daily Scheduling Time

#### Concurrent Access
The persistence manager supports concurrent task storage and retrieval, making it suitable for multi-threaded applications.

### Performance Considerations
- Uses SQLite for lightweight, file-based storage
- Minimal overhead for task persistence
- Asynchronous operations to prevent blocking

### Error Handling
Robust error handling with detailed `PersistenceError` variants:
- Database connection errors
- Serialization/Deserialization issues
- Constraint violations

### Best Practices
1. Use a consistent database path
2. Handle potential persistence errors
3. Avoid storing large, complex task closures
4. Periodically clean up old or completed tasks

### Limitations
- Currently supports SQLite backend
- Task function/closure not persisted (only metadata)
- Recommended for metadata and scheduling information

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

### Development Setup

```bash
# Clone the repository
git clone https://github.com/cploutarchou/scheduler.git

# Change to project directory
cd scheduler

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Roadmap

- [x] Persistent task storage
- [ ] More scheduling options
- [ ] Enhanced logging
- [ ] Web dashboard for task monitoring

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.

## Contact

For questions, issues, or suggestions, please open an issue on GitHub.
