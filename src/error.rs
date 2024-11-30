use thiserror::Error;
use std::time::Duration;
use chrono;

#[derive(Error, Debug, Clone)]
pub enum SchedulerError {
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),
    
    #[error("Task with ID {0} not found")]
    TaskNotFound(String),
    
    #[error("Invalid time format. Expected format: HH:MM or HH:MM:SS, got {0}")]
    InvalidTimeFormat(String),
    
    #[error("Invalid schedule: {0}")]
    InvalidSchedule(String),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Task already exists: {0}")]
    TaskAlreadyExists(String),
    
    #[error("Max retries ({max_retries}) exceeded for task. Last error: {last_error}")]
    MaxRetriesExceeded {
        max_retries: u32,
        last_error: String,
    },
    
    #[error("Failed to retrieve task status")]
    TaskStatusError,

    #[error("Scheduling conflict: Task cannot be scheduled at the specified time")]
    SchedulingConflict,
    
    #[error("Persistence error: {0}")]
    PersistenceError(String),
}

impl From<std::io::Error> for SchedulerError {
    fn from(error: std::io::Error) -> Self {
        SchedulerError::IoError(error.to_string())
    }
}

impl SchedulerError {
    /// Suggests a recovery strategy based on the error type
    pub fn recovery_strategy(&self) -> Option<RecoveryStrategy> {
        match self {
            SchedulerError::TaskExecutionFailed(_) => Some(RecoveryStrategy::Retry),
            SchedulerError::TaskNotFound(_) => Some(RecoveryStrategy::Ignore),
            SchedulerError::InvalidTimeFormat(_) => Some(RecoveryStrategy::Correct),
            SchedulerError::MaxRetriesExceeded { .. } => Some(RecoveryStrategy::Abort),
            SchedulerError::SchedulingConflict => Some(RecoveryStrategy::Reschedule),
            _ => None
        }
    }
}

/// Suggested recovery strategies for different error types
#[derive(Debug, Clone, Copy)]
pub enum RecoveryStrategy {
    /// Automatically retry the operation
    Retry,
    /// Skip the error and continue
    Ignore,
    /// Attempt to fix the error
    Correct,
    /// Stop further execution
    Abort,
    /// Attempt to reschedule
    Reschedule,
}

/// Error context provides additional information about an error
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub error: SchedulerError,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub retry_delay: Option<Duration>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(error: SchedulerError) -> Self {
        Self {
            error,
            timestamp: chrono::Utc::now(),
            retry_delay: None,
        }
    }

    /// Set a retry delay
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = Some(delay);
        self
    }
}
