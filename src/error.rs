use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),
    
    #[error("Task with ID {0} not found")]
    TaskNotFound(String),
    
    #[error("Invalid time format")]
    InvalidTimeFormat,
    
    #[error("Invalid schedule: {0}")]
    InvalidSchedule(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Task already exists: {0}")]
    TaskAlreadyExists(String),
    
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
    
    #[error("Failed to retrieve task status")]
    TaskStatusError,
}
