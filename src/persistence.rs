//! Task persistence and storage
//! 
//! This module provides functionality for persisting tasks to a SQLite database,
//! allowing tasks to survive application restarts and system reboots. It handles:
//! 
//! - Task serialization and deserialization
//! - Database operations (create, read, update, delete)
//! - Error handling for persistence operations
//! 
//! The module uses SQLite as its storage backend for reliability and simplicity.

use std::path::Path;
use std::sync::Arc;
use rusqlite::{Connection, params};
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::task::{Task, TaskStatus};
use crate::error::SchedulerError;

/// A serializable representation of a task for database storage
/// 
/// This struct contains all task data that needs to be persisted,
/// with appropriate serialization support for database storage.
/// 
/// # Fields
/// 
/// * `id` - Unique identifier for the task
/// * `name` - Human-readable name
/// * `status` - Current execution status
/// * `created_at` - Creation timestamp
/// * `last_executed` - Last execution timestamp
/// * `next_execution` - Next scheduled execution
/// * `interval_seconds` - Execution interval in seconds
/// * `daily_time` - Specific time for daily tasks
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistableTask {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub last_executed: Option<DateTime<Utc>>,
    pub next_execution: Option<DateTime<Utc>>,
    pub interval_seconds: Option<u64>,
    pub daily_time: Option<String>,
}

/// Manages task persistence using a SQLite database
/// 
/// This struct provides methods for saving and loading tasks from a
/// SQLite database, ensuring task data survives between application
/// restarts.
pub struct TaskPersistenceManager {
    conn: Arc<Mutex<Connection>>,
}

impl TaskPersistenceManager {
    /// Creates a new persistence manager with the specified database
    /// 
    /// This method initializes the database connection and creates the
    /// necessary tables if they don't exist.
    /// 
    /// # Arguments
    /// 
    /// * `database_path` - Path to the SQLite database file
    /// 
    /// # Returns
    /// 
    /// * `Ok(TaskPersistenceManager)` - Successfully created manager
    /// * `Err(SchedulerError)` - If database initialization fails
    pub async fn new<P: AsRef<Path>>(database_path: P) -> Result<Self, SchedulerError> {
        let conn = Connection::open(database_path)
            .map_err(|e| SchedulerError::PersistenceError(e.to_string()))?;
        
        // Create tasks table if not exists
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_executed TEXT,
                next_execution TEXT,
                interval_seconds INTEGER,
                daily_time TEXT
            )
            "#,
            [],
        )
        .map_err(|e| SchedulerError::PersistenceError(e.to_string()))?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Saves a task to the database
    /// 
    /// This method serializes and stores a task in the database. If a task
    /// with the same ID already exists, it will be updated.
    /// 
    /// # Arguments
    /// 
    /// * `task` - The task to save
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Task was saved successfully
    /// * `Err(SchedulerError)` - If the save operation fails
    pub async fn save_task(&self, task: &Task) -> Result<(), SchedulerError> {
        let persistable_task = PersistableTask {
            id: task.id().to_string(),
            name: task.name().to_string(),
            status: task.get_status().await,  
            created_at: task.created_at,
            last_executed: task.last_run,
            next_execution: task.next_run,
            interval_seconds: task.interval.map(|d| d.num_seconds() as u64),
            daily_time: None,
        };

        let conn = self.conn.lock().await;
        
        conn.execute(
            r#"
            INSERT OR REPLACE INTO tasks 
            (id, name, status, created_at, last_executed, next_execution, interval_seconds, daily_time)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                &persistable_task.id,
                &persistable_task.name,
                persistable_task.status.to_string(),
                persistable_task.created_at.to_rfc3339(),
                persistable_task.last_executed.map(|dt| dt.to_rfc3339()),
                persistable_task.next_execution.map(|dt| dt.to_rfc3339()),
                persistable_task.interval_seconds,
                persistable_task.daily_time,
            ],
        )
        .map_err(|e| SchedulerError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    /// Retrieves a task from the database by its ID
    /// 
    /// # Arguments
    /// 
    /// * `task_id` - The ID of the task to retrieve
    /// 
    /// # Returns
    /// 
    /// * `Ok(Some(PersistableTask))` - Task found and retrieved
    /// * `Ok(None)` - No task found with the given ID
    /// * `Err(SchedulerError)` - If the retrieval operation fails
    pub async fn get_task(&self, task_id: &str) -> Result<Option<PersistableTask>, SchedulerError> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                id, name, status, created_at, last_executed, 
                next_execution, interval_seconds, daily_time 
            FROM tasks 
            WHERE id = ?1
            "#,
        )
        .map_err(|e| SchedulerError::PersistenceError(e.to_string()))?;

        let task = stmt.query_row(
            params![task_id],
            |row| {
                Ok(PersistableTask {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: TaskStatus::from(row.get::<_, String>(2)?),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?.with_timezone(&Utc),
                    last_executed: row.get::<_, Option<String>>(4)?
                        .and_then(|s| DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()),
                    next_execution: row.get::<_, Option<String>>(5)?
                        .and_then(|s| DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()),
                    interval_seconds: row.get(6)?,
                    daily_time: row.get(7)?,
                })
            },
        );

        match task {
            Ok(task) => Ok(Some(task)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(SchedulerError::PersistenceError(e.to_string())),
        }
    }

    /// Lists all tasks stored in the database
    /// 
    /// This method retrieves all tasks currently stored in the database.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<PersistableTask>)` - List of all stored tasks
    /// * `Err(SchedulerError)` - If the retrieval operation fails
    pub async fn list_tasks(&self) -> Result<Vec<PersistableTask>, SchedulerError> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                id, name, status, created_at, last_executed, 
                next_execution, interval_seconds, daily_time 
            FROM tasks
            "#,
        )
        .map_err(|e| SchedulerError::PersistenceError(e.to_string()))?;

        let tasks = stmt.query_map([], |row| {
            Ok(PersistableTask {
                id: row.get(0)?,
                name: row.get(1)?,
                status: TaskStatus::from(row.get::<_, String>(2)?),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?.with_timezone(&Utc),
                last_executed: row.get::<_, Option<String>>(4)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()),
                next_execution: row.get::<_, Option<String>>(5)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()),
                interval_seconds: row.get(6)?,
                daily_time: row.get(7)?,
            })
        })
        .map_err(|e| SchedulerError::PersistenceError(e.to_string()))?;

        let mut result = Vec::new();
        for task in tasks {
            result.push(task.map_err(|e| SchedulerError::PersistenceError(e.to_string()))?);
        }

        Ok(result)
    }

    /// Deletes a task from the database
    /// 
    /// # Arguments
    /// 
    /// * `task_id` - The ID of the task to delete
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Task was deleted or didn't exist
    /// * `Err(SchedulerError)` - If the deletion operation fails
    pub async fn delete_task(&self, task_id: &str) -> Result<(), SchedulerError> {
        let conn = self.conn.lock().await;
        
        conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id])
            .map_err(|e| SchedulerError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    /// Removes all tasks from the database
    /// 
    /// This method deletes all stored tasks, effectively resetting
    /// the persistence store.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - All tasks were cleared successfully
    /// * `Err(SchedulerError)` - If the clear operation fails
    pub async fn clear_tasks(&self) -> Result<(), SchedulerError> {
        let conn = self.conn.lock().await;
        
        conn.execute("DELETE FROM tasks", [])
            .map_err(|e| SchedulerError::PersistenceError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use crate::task::TaskBuilder;

    #[tokio::test]
    async fn test_task_persistence() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary database file
        let temp_db = NamedTempFile::new()?;
        let persistence_manager = TaskPersistenceManager::new(temp_db.path()).await?;

        // Create a sample task
        let task = TaskBuilder::new("test_task", || Ok(()))
            .every_seconds(10)
            .build();

        // Save the task
        persistence_manager.save_task(&task).await?;

        // Retrieve the task
        let retrieved_task = persistence_manager.get_task(&task.id().to_string()).await?;
        assert!(retrieved_task.is_some());
        assert_eq!(retrieved_task.unwrap().name, "test_task");

        // List tasks
        let tasks = persistence_manager.list_tasks().await?;
        assert_eq!(tasks.len(), 1);

        // Delete task
        persistence_manager.delete_task(&task.id().to_string()).await?;
        let tasks = persistence_manager.list_tasks().await?;
        assert_eq!(tasks.len(), 0);

        Ok(())
    }
}
