//! A non-blocking task scheduler for Rust
//! 
//! This library provides a flexible, non-blocking task scheduler that can be used
//! to schedule and execute tasks at specified intervals or times. It supports:
//! 
//! - Scheduling tasks at fixed intervals (seconds, minutes, hours, days)
//! - Daily tasks at specific times
//! - Task persistence across application restarts
//! - Error handling and retry mechanisms
//! - Async/await support with Tokio
//! 
//! # Features
//! 
//! - **Non-blocking execution**: Tasks run asynchronously without blocking the main thread
//! - **Flexible scheduling**: Support for various scheduling patterns
//! - **Persistence**: Optional task persistence using SQLite
//! - **Error handling**: Comprehensive error types and recovery strategies
//! - **Builder pattern**: Fluent interface for creating tasks
//! 
//! # Example
//! ```rust
//! use scheduler::{Scheduler, TaskBuilder, SchedulerError};
//! use std::time::Duration;
//! use tokio;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let scheduler = Scheduler::new();
//!     
//!     // Schedule a task to run every minute
//!     let task1 = TaskBuilder::new("print_task", || {
//!         println!("Task executed!");
//!         Ok(())
//!     })
//!     .every_seconds(1)
//!     .build();
//!     
//!     // Schedule a task to run at specific time
//!     let task2 = TaskBuilder::new("daily_task", || {
//!         println!("Daily task at 10:30");
//!         Ok(())
//!     })
//!     .daily()
//!     .at("10:30")
//!     .unwrap()
//!     .build();
//!     
//!     // Add tasks to the scheduler
//!     scheduler.add_task(task1).await?;
//!     scheduler.add_task(task2).await?;
//!     
//!     // Start the scheduler
//!     let mut rx = scheduler.start().await;
//!     
//!     // Wait for a short duration
//!     tokio::time::sleep(Duration::from_secs(2)).await;
//!     
//!     // Stop the scheduler
//!     scheduler.stop().await?;
//!     
//!     Ok(())
//! }
//! ```
//! 
//! # Modules
//! 
//! - [`error`]: Error types and handling
//! - [`task`]: Task definition and execution
//! - [`scheduler`]: Core scheduling functionality
//! - [`persistence`]: Task persistence and storage

pub mod error;
pub mod task;
pub mod scheduler;
pub mod persistence;

pub use error::SchedulerError;
pub use scheduler::{Scheduler, Job};
pub use task::{Task, TaskStatus};

/// A builder for creating tasks with a fluent interface
/// 
/// The TaskBuilder provides a convenient way to configure and create tasks
/// with various scheduling options. It supports:
/// 
/// - Setting task name and function
/// - Configuring execution interval
/// - Setting specific execution times
/// - Configuring retry behavior
/// 
/// # Example
/// 
/// ```rust
/// use scheduler::TaskBuilder;
/// 
/// let task = TaskBuilder::new("my_task", || {
///     println!("Task executed!");
///     Ok(())
/// })
/// .every_seconds(30)
/// .build();
/// ```
pub struct TaskBuilder {
    name: String,
    task: Option<Box<dyn Fn() -> Result<(), SchedulerError> + Send + Sync + 'static>>,
    interval: Option<scheduler::Interval>,
    at_time: Option<chrono::NaiveTime>,
    retries: u32,
}

impl Clone for TaskBuilder {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            task: None, // We can't clone the task function
            interval: self.interval.clone(),
            at_time: self.at_time,
            retries: self.retries,
        }
    }
}

impl TaskBuilder {
    /// Creates a new TaskBuilder with a name and task function
    /// 
    /// # Arguments
    /// 
    /// * `name` - A string identifier for the task
    /// * `task` - A function that implements the task logic
    /// 
    /// # Returns
    /// 
    /// A new TaskBuilder instance configured with the provided name and task
    pub fn new<F>(name: &str, task: F) -> Self 
    where 
        F: Fn() -> Result<(), SchedulerError> + Send + Sync + 'static,
    {
        TaskBuilder {
            name: name.to_string(),
            task: Some(Box::new(task)),
            interval: None,
            at_time: None,
            retries: 3,
        }
    }

    /// Sets the task to run every specified number of seconds
    /// 
    /// # Arguments
    /// 
    /// * `count` - The number of seconds between executions
    /// 
    /// # Returns
    /// 
    /// The builder instance for method chaining
    pub fn every_seconds(mut self, count: u32) -> Self {
        self.interval = Some(scheduler::Interval::Second(count));
        self
    }

    /// Sets the task to run every minute
    pub fn every_minutes(mut self, count: u32) -> Self {
        self.interval = Some(scheduler::Interval::Minute(count));
        self
    }

    /// Sets the task to run daily
    /// 
    /// This configures the task to run once per day. Use with `at()` to 
    /// specify the exact time.
    /// 
    /// # Returns
    /// 
    /// The builder instance for method chaining
    pub fn daily(mut self) -> Self {
        self.interval = Some(scheduler::Interval::Day(1));
        self
    }

    /// Sets the specific time for task execution
    /// 
    /// # Arguments
    /// 
    /// * `time` - A string in "HH:MM" or "HH:MM:SS" format
    /// 
    /// # Returns
    /// 
    /// * `Ok(Self)` - The builder instance for method chaining
    /// * `Err(SchedulerError)` - If the time format is invalid
    pub fn at(mut self, time: &str) -> Result<Self, SchedulerError> {
        let parts: Vec<&str> = time.split(':').collect();
        
        let (hour, minute, second) = match parts.len() {
            2 => {
                let h: u32 = parts[0].parse().map_err(|_| SchedulerError::InvalidTimeFormat(format!("Invalid hour: {}", parts[0])))?;
                let m: u32 = parts[1].parse().map_err(|_| SchedulerError::InvalidTimeFormat(format!("Invalid minute: {}", parts[1])))?;
                (h, m, 0)
            }
            3 => {
                let h: u32 = parts[0].parse().map_err(|_| SchedulerError::InvalidTimeFormat(format!("Invalid hour: {}", parts[0])))?;
                let m: u32 = parts[1].parse().map_err(|_| SchedulerError::InvalidTimeFormat(format!("Invalid minute: {}", parts[1])))?;
                let s: u32 = parts[2].parse().map_err(|_| SchedulerError::InvalidTimeFormat(format!("Invalid second: {}", parts[2])))?;
                (h, m, s)
            }
            _ => return Err(SchedulerError::InvalidTimeFormat(format!("Invalid time format: {}", time))),
        };

        self.at_time = Some(chrono::NaiveTime::from_hms_opt(hour, minute, second).unwrap());
        Ok(self)
    }

    /// Builds and returns a new Task instance
    /// 
    /// This method consumes the builder and creates a new Task with the
    /// configured settings.
    /// 
    /// # Returns
    /// 
    /// A new Task instance ready for scheduling
    pub fn build(self) -> Task {
        let next_run = if let Some(at_time) = self.at_time {
            let now = chrono::Utc::now();
            let today = now.date_naive();
            let next_run = today.and_time(at_time).and_utc();
            Some(if next_run <= now { next_run + chrono::Duration::days(1) } else { next_run })
        } else {
            Some(chrono::Utc::now())
        };

        let interval = self.interval.map(|i| match i {
            scheduler::Interval::Second(n) => chrono::Duration::seconds(n as i64),
            scheduler::Interval::Minute(n) => chrono::Duration::minutes(n as i64),
            scheduler::Interval::Day(n) => chrono::Duration::days(n as i64),
            _ => chrono::Duration::minutes(1), // Default fallback
        });

        Task::new(
            self.name.clone(),
            format!("{}_task", self.name),
            self.task.unwrap(),
            next_run,
            interval,
            self.retries,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_basic_scheduling() {
        let scheduler = Scheduler::new();
        let _rx = scheduler.start().await;
        
        // Create a task that runs every minute
        let task = TaskBuilder::new("test_task", || {
            println!("Task executed!");
            Ok(())
        })
        .every_seconds(1)
        .build();

        let task_id = scheduler.add_task(task).await.unwrap();

        // Wait a bit and check task status
        sleep(Duration::from_secs(2)).await;
        let status = scheduler.get_task_status(&task_id).await.unwrap();
        assert!(matches!(status, TaskStatus::Completed));

        scheduler.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_daily_schedule() {
        let scheduler = Scheduler::new();
        let _rx = scheduler.start().await;
        
        // Create a daily task
        let task = TaskBuilder::new("daily_task", || {
            println!("Daily task executed!");
            Ok(())
        })
        .daily()
        .at("00:00")
        .unwrap()
        .build();

        let task_id = scheduler.add_task(task).await.unwrap();

        // Wait a bit and check task status
        sleep(Duration::from_secs(2)).await;
        let status = scheduler.get_task_status(&task_id).await.unwrap();
        assert!(matches!(status, TaskStatus::Pending));

        scheduler.stop().await.unwrap();
    }
}
