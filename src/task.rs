use crate::error::SchedulerError;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Represents the status of a task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

/// Represents a scheduled task
#[derive(Debug)]
pub struct Task {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) task: Arc<dyn Fn() -> Result<(), SchedulerError> + Send + Sync>,
    pub(crate) next_run: Option<DateTime<Utc>>,
    pub(crate) interval: Option<Duration>,
    pub(crate) retries: u32,
    pub(crate) retry_count: u32,
    pub(crate) status: Arc<Mutex<TaskStatus>>,
    pub(crate) last_run: Option<DateTime<Utc>>,
}

impl Task {
    pub(crate) fn new<F>(
        name: String,
        task: F,
        next_run: Option<DateTime<Utc>>,
        interval: Option<Duration>,
        retries: u32,
    ) -> Self
    where
        F: Fn() -> Result<(), SchedulerError> + Send + Sync + 'static,
    {
        Task {
            id: Uuid::new_v4().to_string(),
            name,
            task: Arc::new(task),
            next_run,
            interval,
            retries,
            retry_count: 0,
            status: Arc::new(Mutex::new(TaskStatus::Pending)),
            last_run: None,
        }
    }

    pub(crate) async fn execute(&mut self) -> Result<(), SchedulerError> {
        let mut status = self.status.lock().await;
        *status = TaskStatus::Running;
        drop(status);

        let result = (self.task)();
        let mut status = self.status.lock().await;

        match result {
            Ok(_) => {
                self.retry_count = 0;
                *status = TaskStatus::Completed;
                self.last_run = Some(Utc::now());
                
                if let Some(interval) = self.interval {
                    self.next_run = Some(Utc::now() + interval);
                }
                Ok(())
            }
            Err(e) => {
                self.retry_count += 1;
                if self.retry_count >= self.retries {
                    *status = TaskStatus::Failed(e.to_string());
                    Err(SchedulerError::MaxRetriesReached)
                } else {
                    *status = TaskStatus::Failed(format!("Retry {}/{}", self.retry_count, self.retries));
                    Err(e)
                }
            }
        }
    }

    pub fn is_due(&self) -> bool {
        match self.next_run {
            Some(next_run) => next_run <= Utc::now(),
            None => true,
        }
    }

    pub async fn get_status(&self) -> TaskStatus {
        self.status.lock().await.clone()
    }
}
