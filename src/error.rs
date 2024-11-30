//! Error types for the scheduler library
//! 
//! This module provides comprehensive error handling for the scheduler,
//! including specific error types for different failure scenarios and
//! recovery strategies.

use thiserror::Error;
use std::time::Duration;
use chrono;

/// Errors that can occur during scheduler operations
/// 
/// This enum represents all possible errors that can occur during
/// task scheduling, execution, and persistence operations.
#[derive(Error, Debug, Clone)]
pub enum SchedulerError {
    /// Error when a task's execution fails
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),
    
    /// Error when attempting to operate on a non-existent task
    #[error("Task with ID {0} not found")]
    TaskNotFound(String),
    
    /// Error when parsing an invalid time format
    #[error("Invalid time format. Expected format: HH:MM or HH:MM:SS, got {0}")]
    InvalidTimeFormat(String),
    
    /// Error when attempting to create an invalid schedule
    #[error("Invalid schedule: {0}")]
    InvalidSchedule(String),
    
    /// Error during I/O operations
    #[error("IO error: {0}")]
    IoError(String),
    
    /// Error when attempting to create a duplicate task
    #[error("Task already exists: {0}")]
    TaskAlreadyExists(String),
    
    /// Error when maximum retry attempts are exceeded
    #[error("Max retries ({max_retries}) exceeded for task. Last error: {last_error}")]
    MaxRetriesExceeded {
        max_retries: u32,
        last_error: String,
    },
    
    /// Error when unable to retrieve task status
    #[error("Failed to retrieve task status")]
    TaskStatusError,

    /// Error when a scheduling conflict occurs
    #[error("Scheduling conflict: Task cannot be scheduled at the specified time")]
    SchedulingConflict,
    
    /// Error during persistence operations
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
    /// 
    /// # Returns
    /// 
    /// * `Some(RecoveryStrategy)` - A suggested recovery strategy for the error
    /// * `None` - If no recovery strategy is available for this error type
    pub fn recovery_strategy(&self) -> Option<RecoveryStrategy> {
        match self {
            SchedulerError::TaskExecutionFailed(_) => Some(RecoveryStrategy::Retry),
            SchedulerError::TaskNotFound(_) => Some(RecoveryStrategy::Skip),
            SchedulerError::InvalidTimeFormat(_) => Some(RecoveryStrategy::Skip),
            SchedulerError::MaxRetriesExceeded { .. } => Some(RecoveryStrategy::Abort),
            SchedulerError::SchedulingConflict => Some(RecoveryStrategy::Pause),
            _ => None
        }
    }
}

/// Suggested recovery strategies for different error types
/// 
/// These strategies provide guidance on how to handle specific error cases
/// in an automated way.
#[derive(Debug, Clone, Copy)]
pub enum RecoveryStrategy {
    /// Retry the failed operation
    Retry,
    /// Skip the failed operation and continue
    Skip,
    /// Abort the current operation
    Abort,
    /// Pause the scheduler
    Pause,
}

/// Error context provides additional information about an error
/// 
/// This struct contains metadata about an error occurrence, including
/// retry delays and context-specific information.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub error: SchedulerError,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub retry_delay: Option<Duration>,
}

impl ErrorContext {
    /// Creates a new error context
    /// 
    /// # Arguments
    /// 
    /// * `error` - The scheduler error to create context for
    /// 
    /// # Returns
    /// 
    /// A new ErrorContext instance
    pub fn new(error: SchedulerError) -> Self {
        Self {
            error,
            timestamp: chrono::Utc::now(),
            retry_delay: None,
        }
    }

    /// Sets a retry delay for the error
    /// 
    /// # Arguments
    /// 
    /// * `delay` - The duration to wait before retrying
    /// 
    /// # Returns
    /// 
    /// The updated ErrorContext instance
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = Some(delay);
        self
    }
}
