//! A non-blocking task scheduler for Rust
//! 
//! This library provides a flexible, non-blocking task scheduler that can be used
//! to schedule and execute tasks at specified intervals or times.
//! 
//! # Example
//! ```rust
//! use scheduler::{Scheduler, SchedulerError};
//! use std::time::Duration;
//! use tokio;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let scheduler = Scheduler::new();
//!     
//!     // Schedule a task to run every minute
//!     scheduler.every(1)
//!         .minute()
//!         .do_job("print_task", || {
//!             println!("Task executed!");
//!             Ok(())
//!         })
//!         .await?;
//!     
//!     // Schedule a task to run at specific time
//!     scheduler.every(1)
//!         .day()
//!         .at("10:30")
//!         .do_job("daily_task", || {
//!             println!("Daily task at 10:30");
//!             Ok(())
//!         })
//!         .await?;
//!     
//!     // Schedule a task to run on Monday
//!     scheduler.every(1)
//!         .monday()
//!         .do_job("monday_task", || {
//!             println!("Monday task");
//!             Ok(())
//!         })
//!         .await?;
//!     
//!     // Start the scheduler
//!     let mut rx = scheduler.start().await;
//!     
//!     // Wait for a signal to stop
//!     rx.recv().await.ok();
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod scheduler;
pub mod task;

pub use error::SchedulerError;
pub use scheduler::{Scheduler, Job};
pub use task::{Task, TaskStatus};

/// A builder for creating tasks with a fluent interface
pub struct TaskBuilder {
    name: String,
    task: Option<Box<dyn Fn() -> Result<(), SchedulerError> + Send + Sync + 'static>>,
    interval: Option<scheduler::Interval>,
    at_time: Option<chrono::NaiveTime>,
    retries: u32,
}

impl TaskBuilder {
    /// Create a new TaskBuilder with a name and task
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

    /// Set the task to run every minute
    pub fn every_minutes(mut self, count: u32) -> Self {
        self.interval = Some(scheduler::Interval::Minute(count));
        self
    }

    /// Set the task to run daily
    pub fn daily(mut self) -> Self {
        self.interval = Some(scheduler::Interval::Day(1));
        self
    }

    /// Set the time for the task
    pub fn at(mut self, time: &str) -> Result<Self, SchedulerError> {
        let parts: Vec<&str> = time.split(':').collect();
        
        let (hour, minute, second) = match parts.len() {
            2 => {
                let h: u32 = parts[0].parse().map_err(|_| SchedulerError::InvalidTimeFormat)?;
                let m: u32 = parts[1].parse().map_err(|_| SchedulerError::InvalidTimeFormat)?;
                (h, m, 0)
            }
            3 => {
                let h: u32 = parts[0].parse().map_err(|_| SchedulerError::InvalidTimeFormat)?;
                let m: u32 = parts[1].parse().map_err(|_| SchedulerError::InvalidTimeFormat)?;
                let s: u32 = parts[2].parse().map_err(|_| SchedulerError::InvalidTimeFormat)?;
                (h, m, s)
            }
            _ => return Err(SchedulerError::InvalidTimeFormat),
        };

        self.at_time = Some(chrono::NaiveTime::from_hms_opt(hour, minute, second).unwrap());
        Ok(self)
    }

    /// Build the task
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
        
        // Create a task that runs every minute
        let task = TaskBuilder::new("test_task", || {
            println!("Task executed!");
            Ok(())
        })
        .every_minutes(1)
        .build();

        let task_id = scheduler.add_task(task).await.unwrap();
        scheduler.start().await;

        // Wait a bit and check task status
        sleep(Duration::from_secs(2)).await;
        let status = scheduler.get_task_status(&task_id).await.unwrap();
        assert!(matches!(status, TaskStatus::Completed));

        scheduler.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_daily_schedule() {
        let scheduler = Scheduler::new();
        
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
        scheduler.start().await;

        // Wait a bit and check task status
        sleep(Duration::from_secs(2)).await;
        let status = scheduler.get_task_status(&task_id).await.unwrap();
        assert!(matches!(status, TaskStatus::Completed));

        scheduler.stop().await.unwrap();
    }
}
