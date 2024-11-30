use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Task execution failed: {0}")]
    ExecutionError(String),
    
    #[error("Invalid time format. Expected HH:MM")]
    InvalidTimeFormat,
    
    #[error("Task scheduling failed: {0}")]
    SchedulingError(String),
    
    #[error("Maximum retry attempts reached")]
    MaxRetriesReached,
    
    #[error("Queue error: {0}")]
    QueueError(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Invalid schedule configuration: {0}")]
    InvalidSchedule(String),
}
