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

- [ ] More scheduling options
- [ ] Enhanced logging
- [ ] Persistent task storage
- [ ] Web dashboard for task monitoring

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.

## Contact

For questions, issues, or suggestions, please open an issue on GitHub.
