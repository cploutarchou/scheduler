use crate::error::SchedulerError;
use crate::task::{Task, TaskStatus};
use chrono::{DateTime, Datelike, Duration, NaiveTime, Utc, Weekday};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Interval {
    Second(u32),
    Minute(u32),
    Hour(u32),
    Day(u32),
    Week(u32),
    Month(u32),
}

#[derive(Debug, Clone)]
pub struct JobSchedule {
    interval: Interval,
    at_time: Option<NaiveTime>,
    weekday: Option<Weekday>,
    start_time: Option<DateTime<Utc>>,
    next_tick: bool,
}

impl JobSchedule {
    fn new(interval: Interval) -> Self {
        JobSchedule {
            interval,
            at_time: None,
            weekday: None,
            start_time: None,
            next_tick: false,
        }
    }
}

/// The main scheduler struct that manages tasks
pub struct Scheduler {
    tasks: Arc<Mutex<HashMap<String, Task>>>,
    running: Arc<Mutex<bool>>,
}

impl fmt::Debug for Scheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scheduler")
            .field("running", &"<internal state>")
            .finish()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    /// Create a new scheduler instance
    pub fn new() -> Self {
        Scheduler {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Create a job that runs at the specified interval
    pub fn every(&self, count: u32) -> Job {
        Job::new(self.clone(), count)
    }

    /// Start the scheduler in a non-blocking way
    pub async fn start(&self) -> tokio::sync::broadcast::Receiver<()> {
        let mut running = self.running.lock().await;
        if *running {
            let (_, rx) = tokio::sync::broadcast::channel(1);
            return rx;
        }
        *running = true;
        drop(running);

        let (tx, rx) = tokio::sync::broadcast::channel(1);
        let scheduler = self.clone();
        
        tokio::spawn(async move {
            while *scheduler.running.lock().await {
                scheduler.run_pending_tasks().await.ok();
                time::sleep(time::Duration::from_secs(1)).await;
            }
            tx.send(()).ok();
        });

        rx
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

    /// Remove a specific task
    pub async fn remove(&self, id: &str) -> Result<(), SchedulerError> {
        let mut tasks = self.tasks.lock().await;
        tasks.remove(id).ok_or(SchedulerError::TaskNotFound(id.to_string()))?;
        Ok(())
    }

    /// Clear all scheduled tasks
    pub async fn clear(&self) -> Result<(), SchedulerError> {
        let mut tasks = self.tasks.lock().await;
        tasks.clear();
        Ok(())
    }

    /// Get the next run time of any task
    pub async fn next_run(&self) -> Option<DateTime<Utc>> {
        let tasks = self.tasks.lock().await;
        tasks.values()
            .filter_map(|task| task.next_run)
            .min()
    }

    async fn run_pending_tasks(&self) -> Result<(), SchedulerError> {
        let mut tasks = self.tasks.lock().await;
        let mut completed_tasks = Vec::new();

        for (id, task) in tasks.iter_mut() {
            if task.is_due() {
                if let Err(e) = task.execute().await {
                    tracing::error!("Task {} ({}) failed: {:?}", task.get_name(), id, e);
                    if matches!(task.get_status().await, TaskStatus::Failed(_)) {
                        completed_tasks.push(id.clone());
                    }
                }
            }
        }

        for id in completed_tasks {
            tasks.remove(&id);
        }

        Ok(())
    }

    /// Get the status of a specific task by its ID
    pub async fn get_task_status(&self, id: &str) -> Result<TaskStatus, SchedulerError> {
        let tasks = self.tasks.lock().await;
        tasks.get(id)
            .ok_or_else(|| SchedulerError::TaskNotFound(id.to_string()))
            .map(|task| {
                // Use a synchronous method to get the status
                task.status.try_lock().map(|status| status.clone())
                    .unwrap_or(TaskStatus::Running)
            })
    }
}

impl Clone for Scheduler {
    fn clone(&self) -> Self {
        Scheduler {
            tasks: Arc::clone(&self.tasks),
            running: Arc::clone(&self.running),
        }
    }
}

/// Job builder for creating scheduled tasks
pub struct Job {
    scheduler: Scheduler,
    count: u32,
    schedule: Option<JobSchedule>,
    task: Option<Box<dyn Fn() -> Result<(), SchedulerError> + Send + Sync + 'static>>,
    name: Option<String>,
    retries: u32,
}

impl fmt::Debug for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Job")
            .field("scheduler", &"<scheduler>")
            .field("count", &self.count)
            .field("schedule", &self.schedule)
            .field("task", &"<function>")
            .field("name", &self.name)
            .field("retries", &self.retries)
            .finish()
    }
}

impl Job {
    fn new(scheduler: Scheduler, count: u32) -> Self {
        Job {
            scheduler,
            count,
            schedule: None,
            task: None,
            name: None,
            retries: 3,
        }
    }

    pub fn second(self) -> Self {
        self.seconds()
    }

    pub fn seconds(mut self) -> Self {
        self.schedule = Some(JobSchedule::new(Interval::Second(self.count)));
        self
    }

    pub fn minute(self) -> Self {
        self.minutes()
    }

    pub fn minutes(mut self) -> Self {
        self.schedule = Some(JobSchedule::new(Interval::Minute(self.count)));
        self
    }

    pub fn hour(self) -> Self {
        self.hours()
    }

    pub fn hours(mut self) -> Self {
        self.schedule = Some(JobSchedule::new(Interval::Hour(self.count)));
        self
    }

    pub fn day(self) -> Self {
        self.days()
    }

    pub fn days(mut self) -> Self {
        self.schedule = Some(JobSchedule::new(Interval::Day(self.count)));
        self
    }

    pub fn week(self) -> Self {
        self.weeks()
    }

    pub fn weeks(mut self) -> Self {
        self.schedule = Some(JobSchedule::new(Interval::Week(self.count)));
        self
    }

    pub fn monday(mut self) -> Self {
        if let Some(schedule) = &mut self.schedule {
            schedule.weekday = Some(Weekday::Mon);
        } else {
            self.schedule = Some(JobSchedule::new(Interval::Week(1)));
            self.schedule.as_mut().unwrap().weekday = Some(Weekday::Mon);
        }
        self
    }

    pub fn tuesday(mut self) -> Self {
        if let Some(schedule) = &mut self.schedule {
            schedule.weekday = Some(Weekday::Tue);
        } else {
            self.schedule = Some(JobSchedule::new(Interval::Week(1)));
            self.schedule.as_mut().unwrap().weekday = Some(Weekday::Tue);
        }
        self
    }

    pub fn wednesday(mut self) -> Self {
        if let Some(schedule) = &mut self.schedule {
            schedule.weekday = Some(Weekday::Wed);
        } else {
            self.schedule = Some(JobSchedule::new(Interval::Week(1)));
            self.schedule.as_mut().unwrap().weekday = Some(Weekday::Wed);
        }
        self
    }

    pub fn thursday(mut self) -> Self {
        if let Some(schedule) = &mut self.schedule {
            schedule.weekday = Some(Weekday::Thu);
        } else {
            self.schedule = Some(JobSchedule::new(Interval::Week(1)));
            self.schedule.as_mut().unwrap().weekday = Some(Weekday::Thu);
        }
        self
    }

    pub fn friday(mut self) -> Self {
        if let Some(schedule) = &mut self.schedule {
            schedule.weekday = Some(Weekday::Fri);
        } else {
            self.schedule = Some(JobSchedule::new(Interval::Week(1)));
            self.schedule.as_mut().unwrap().weekday = Some(Weekday::Fri);
        }
        self
    }

    pub fn saturday(mut self) -> Self {
        if let Some(schedule) = &mut self.schedule {
            schedule.weekday = Some(Weekday::Sat);
        } else {
            self.schedule = Some(JobSchedule::new(Interval::Week(1)));
            self.schedule.as_mut().unwrap().weekday = Some(Weekday::Sat);
        }
        self
    }

    pub fn sunday(mut self) -> Self {
        if let Some(schedule) = &mut self.schedule {
            schedule.weekday = Some(Weekday::Sun);
        } else {
            self.schedule = Some(JobSchedule::new(Interval::Week(1)));
            self.schedule.as_mut().unwrap().weekday = Some(Weekday::Sun);
        }
        self
    }

    pub fn at<S: AsRef<str>>(mut self, time: S) -> Result<Self, SchedulerError> {
        let time_str = time.as_ref();
        let parts: Vec<&str> = time_str.split(':').collect();
        
        let (hour, minute, second) = match parts.len() {
            2 => {
                let h: u32 = parts[0].parse().map_err(|_| SchedulerError::InvalidTimeFormat(time_str.to_string()))?;
                let m: u32 = parts[1].parse().map_err(|_| SchedulerError::InvalidTimeFormat(time_str.to_string()))?;
                (h, m, 0)
            }
            3 => {
                let h: u32 = parts[0].parse().map_err(|_| SchedulerError::InvalidTimeFormat(time_str.to_string()))?;
                let m: u32 = parts[1].parse().map_err(|_| SchedulerError::InvalidTimeFormat(time_str.to_string()))?;
                let s: u32 = parts[2].parse().map_err(|_| SchedulerError::InvalidTimeFormat(time_str.to_string()))?;
                (h, m, s)
            }
            _ => return Err(SchedulerError::InvalidTimeFormat(time_str.to_string())),
        };

        if let Some(schedule) = &mut self.schedule {
            schedule.at_time = Some(NaiveTime::from_hms_opt(hour, minute, second).unwrap());
        }
        
        Ok(self)
    }

    pub fn from(mut self, time: DateTime<Utc>) -> Self {
        if let Some(schedule) = &mut self.schedule {
            schedule.start_time = Some(time);
        }
        self
    }

    pub fn next_tick(mut self) -> Self {
        if let Some(schedule) = &mut self.schedule {
            schedule.next_tick = true;
        }
        self
    }

    pub async fn do_job<F>(mut self, name: &str, job: F) -> Result<String, SchedulerError>
    where
        F: Fn() -> Result<(), SchedulerError> + Send + Sync + 'static,
    {
        self.name = Some(name.to_string());
        self.task = Some(Box::new(job));
        
        let schedule = self.schedule.ok_or(SchedulerError::InvalidSchedule("No schedule specified".into()))?;
        
        let mut next_run = match (schedule.start_time, schedule.next_tick) {
            (Some(time), _) => time,
            (None, true) => Utc::now(),
            (None, false) => {
                let mut next_run = Utc::now();
                if let Some(weekday) = schedule.weekday {
                    while next_run.weekday() != weekday {
                        next_run = next_run + Duration::days(1);
                    }
                }
                next_run
            },
        };

        if let Some(at_time) = schedule.at_time {
            next_run = next_run.date_naive().and_time(at_time).and_utc();
            if next_run <= Utc::now() {
                next_run = next_run + Duration::days(1);
            }
        }

        let interval = match schedule.interval {
            Interval::Second(n) => Duration::seconds(n as i64),
            Interval::Minute(n) => Duration::minutes(n as i64),
            Interval::Hour(n) => Duration::hours(n as i64),
            Interval::Day(n) => Duration::days(n as i64),
            Interval::Week(n) => Duration::weeks(n as i64),
            Interval::Month(n) => Duration::days(n as i64 * 30), // Approximate
        };

        let task = Task::new(
            name.to_string(),
            format!("{}_{}", name, Uuid::new_v4().to_string()),
            self.task.take().unwrap(),
            Some(next_run),
            Some(interval),
            self.retries,
        );

        self.scheduler.add_task(task).await
    }
}
