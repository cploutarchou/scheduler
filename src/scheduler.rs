use crate::error::SchedulerError;
use crate::task::{Task, TaskStatus};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time;

/// The main scheduler struct that manages tasks
#[derive(Clone)]
pub struct Scheduler {
    tasks: Arc<Mutex<HashMap<String, Task>>>,
    running: Arc<Mutex<bool>>,
}

impl Scheduler {
    /// Create a new scheduler instance
    pub fn new() -> Self {
        Scheduler {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the scheduler in a non-blocking way
    pub async fn start(&self) -> Result<(), SchedulerError> {
        let mut running = self.running.lock().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        let scheduler = self.clone();
        tokio::spawn(async move {
            while *scheduler.running.lock().await {
                scheduler.run_pending_tasks().await.ok();
                time::sleep(time::Duration::from_secs(1)).await;
            }
        });

        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) -> Result<(), SchedulerError> {
        let mut running = self.running.lock().await;
        *running = false;
        Ok(())
    }

    /// Add a new task to the scheduler
    pub async fn add_task(&self, task: Task) -> Result<String, SchedulerError> {
        let id = task.id.clone();
        let mut tasks = self.tasks.lock().await;
        tasks.insert(id.clone(), task);
        Ok(id)
    }

    /// Remove a task from the scheduler
    pub async fn remove_task(&self, id: &str) -> Result<(), SchedulerError> {
        let mut tasks = self.tasks.lock().await;
        tasks.remove(id).ok_or(SchedulerError::TaskNotFound(id.to_string()))?;
        Ok(())
    }

    /// Get the status of a specific task
    pub async fn get_task_status(&self, id: &str) -> Result<TaskStatus, SchedulerError> {
        let tasks = self.tasks.lock().await;
        let task = tasks.get(id).ok_or(SchedulerError::TaskNotFound(id.to_string()))?;
        Ok(task.get_status().await)
    }

    /// Run all pending tasks
    async fn run_pending_tasks(&self) -> Result<(), SchedulerError> {
        let mut tasks = self.tasks.lock().await;
        let mut completed_tasks = Vec::new();

        for (id, task) in tasks.iter_mut() {
            if task.is_due() {
                if let Err(e) = task.execute().await {
                    tracing::error!("Task {} failed: {:?}", id, e);
                    if matches!(task.get_status().await, TaskStatus::Failed(_)) {
                        completed_tasks.push(id.clone());
                    }
                }
            }
        }

        // Remove completed one-off tasks and failed tasks that have reached max retries
        for id in completed_tasks {
            tasks.remove(&id);
        }

        Ok(())
    }
}

/// Builder for creating scheduled tasks
pub struct TaskBuilder {
    name: String,
    task: Box<dyn Fn() -> Result<(), SchedulerError> + Send + Sync + 'static>,
    next_run: Option<DateTime<Utc>>,
    interval: Option<Duration>,
    retries: u32,
}

impl TaskBuilder {
    /// Create a new task builder
    pub fn new<F, S>(name: S, task: F) -> Self
    where
        F: Fn() -> Result<(), SchedulerError> + Send + Sync + 'static,
        S: Into<String>,
    {
        TaskBuilder {
            name: name.into(),
            task: Box::new(task),
            next_run: None,
            interval: None,
            retries: 3,
        }
    }

    /// Schedule task to run every n minutes
    pub fn every_minutes(mut self, minutes: u32) -> Self {
        self.interval = Some(Duration::minutes(minutes as i64));
        self
    }

    /// Schedule task to run every n hours
    pub fn every_hours(mut self, hours: u32) -> Self {
        self.interval = Some(Duration::hours(hours as i64));
        self
    }

    /// Schedule task to run daily
    pub fn daily(mut self) -> Self {
        self.interval = Some(Duration::days(1));
        self
    }

    /// Schedule task to run every n days
    pub fn days(mut self, days: u32) -> Self {
        self.interval = Some(Duration::days(days as i64));
        self
    }

    /// Set specific time for the task
    pub fn at<S: AsRef<str>>(mut self, time: S) -> Result<Self, SchedulerError> {
        let time_str = time.as_ref();
        let (hour, minute) = parse_time(time_str)?;
        
        let mut next_run = Utc::now();
        next_run = next_run
            .date_naive()
            .and_hms_opt(hour, minute, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();

        if next_run <= Utc::now() {
            next_run = next_run + Duration::days(1);
        }

        self.next_run = Some(next_run);
        Ok(self)
    }

    /// Set number of retries for failed tasks
    pub fn retries(mut self, count: u32) -> Self {
        self.retries = count;
        self
    }

    /// Build the task
    pub fn build(self) -> Task {
        Task::new(
            self.name,
            self.task,
            self.next_run,
            self.interval,
            self.retries,
        )
    }
}

fn parse_time(time: &str) -> Result<(u32, u32), SchedulerError> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return Err(SchedulerError::InvalidTimeFormat);
    }

    let hour: u32 = parts[0].parse().map_err(|_| SchedulerError::InvalidTimeFormat)?;
    let minute: u32 = parts[1].parse().map_err(|_| SchedulerError::InvalidTimeFormat)?;

    if hour >= 24 || minute >= 60 {
        return Err(SchedulerError::InvalidTimeFormat);
    }

    Ok((hour, minute))
}
