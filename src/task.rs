use crate::error::SchedulerError;
use chrono::{DateTime, Duration, Utc};
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

impl fmt::Debug for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::Running => write!(f, "Running"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Failed(err) => write!(f, "Failed({})", err),
        }
    }
}

// Wrapper type for our task function that implements Debug
#[derive(Clone)]
pub(crate) struct TaskFn(Arc<dyn Fn() -> Result<(), SchedulerError> + Send + Sync + 'static>);

impl fmt::Debug for TaskFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use the Arc's pointer address for a unique identifier
        write!(f, "<function@{:p}>", self.0.as_ref())
    }
}

/// A scheduled task that can be executed
pub struct Task {
    pub(crate) id: String,
    pub(crate) name: String,
    task_name: String,
    task: TaskFn,
    pub(crate) next_run: Option<DateTime<Utc>>,
    pub(crate) interval: Option<Duration>,
    pub(crate) retries: u32,
    pub(crate) retry_count: u32,
    pub(crate) status: Arc<Mutex<TaskStatus>>,
    pub(crate) last_run: Option<DateTime<Utc>>,
}

impl Clone for Task {
    fn clone(&self) -> Self {
        Task {
            id: self.id.clone(),
            name: self.name.clone(),
            task_name: self.task_name.clone(),
            task: self.task.clone(),
            next_run: self.next_run,
            interval: self.interval,
            retries: self.retries,
            retry_count: self.retry_count,
            status: Arc::clone(&self.status),
            last_run: self.last_run,
        }
    }
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("task_name", &self.task_name)
            .field("next_run", &self.next_run)
            .field("interval", &self.interval)
            .field("retries", &self.retries)
            .field("retry_count", &self.retry_count)
            .field("last_run", &self.last_run)
            .field("status", &format_args!("<mutex>"))
            .field("task", &self.task)
            .finish()
    }
}

impl Task {
    pub(crate) fn new<F>(
        name: String,
        task_name: String,
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
            task_name,
            task: TaskFn(Arc::new(task)),
            next_run,
            interval,
            retries,
            retry_count: 0,
            status: Arc::new(Mutex::new(TaskStatus::Pending)),
            last_run: None,
        }
    }

    pub async fn execute(&mut self) -> Result<(), SchedulerError> {
        let mut status = self.status.lock().await;
        *status = TaskStatus::Running;
        drop(status);

        let result = (self.task.0)();
        let mut status = self.status.lock().await;

        match result {
            Ok(()) => {
                *status = TaskStatus::Completed;
                self.last_run = Some(Utc::now());
                if let Some(interval) = self.interval {
                    self.next_run = Some(Utc::now() + interval);
                }
                Ok(())
            }
            Err(err) => {
                self.retry_count += 1;
                if self.retry_count >= self.retries {
                    *status = TaskStatus::Failed(err.to_string());
                    Err(SchedulerError::MaxRetriesExceeded)
                } else {
                    *status = TaskStatus::Failed(format!("Retry {}/{}", self.retry_count, self.retries));
                    Err(err)
                }
            }
        }
    }

    pub fn is_due(&self) -> bool {
        if let Some(next_run) = self.next_run {
            Utc::now() >= next_run
        } else {
            false
        }
    }

    pub async fn get_status(&self) -> TaskStatus {
        self.status.lock().await.clone()
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_task_name(&self) -> &str {
        &self.task_name
    }
}
