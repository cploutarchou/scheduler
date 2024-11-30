use anyhow::Result;
use scheduler::{
    persistence::TaskPersistenceManager, 
    task::{Task, TaskBuilder, TaskStatus}, 
    error::SchedulerError
};
use tempfile::NamedTempFile;
use uuid::Uuid;

#[tokio::test]
async fn test_persistence_manager_basic_operations() -> Result<()> {
    // Create a temporary database file
    let temp_db = NamedTempFile::new()?;
    let persistence_manager = TaskPersistenceManager::new(temp_db.path()).await?;

    // Create multiple test tasks
    let task1 = TaskBuilder::new("backup_task", || Ok(()))
        .every_seconds(3600)
        .build();
    
    let task2 = TaskBuilder::new("cleanup_task", || Ok(()))
        .daily()
        .at("02:00")
        .unwrap()
        .build();

    // Save tasks
    persistence_manager.save_task(&task1).await?;
    persistence_manager.save_task(&task2).await?;

    // List tasks
    let tasks = persistence_manager.list_tasks().await?;
    assert_eq!(tasks.len(), 2, "Should have saved two tasks");

    // Retrieve specific tasks
    let retrieved_task1 = persistence_manager.get_task(&task1.id().to_string()).await?;
    assert!(retrieved_task1.is_some(), "Task 1 should be retrievable");
    assert_eq!(retrieved_task1.unwrap().name, "backup_task");

    let retrieved_task2 = persistence_manager.get_task(&task2.id().to_string()).await?;
    assert!(retrieved_task2.is_some(), "Task 2 should be retrievable");
    assert_eq!(retrieved_task2.unwrap().name, "cleanup_task");

    Ok(())
}

#[tokio::test]
async fn test_persistence_task_update() -> Result<()> {
    let temp_db = NamedTempFile::new()?;
    let persistence_manager = TaskPersistenceManager::new(temp_db.path()).await?;

    // Create a task that will fail to execute
    let mut task = TaskBuilder::new("update_test_task", || {
        Err(SchedulerError::TaskExecutionFailed("Test failure".to_string()))
    })
    .every_seconds(60)
    .build();

    // Save initial task
    persistence_manager.save_task(&task).await?;

    // Execute task to change its status to Failed
    let _ = task.execute().await;

    // Save updated task
    persistence_manager.save_task(&task).await?;

    // Retrieve task and verify status
    let retrieved_task = persistence_manager.get_task(&task.id().to_string()).await?;
    assert!(retrieved_task.is_some());
    let retrieved_task = retrieved_task.unwrap();
    assert!(matches!(retrieved_task.status, TaskStatus::Failed(_)));

    Ok(())
}

#[tokio::test]
async fn test_persistence_task_deletion() -> Result<()> {
    let temp_db = NamedTempFile::new()?;
    let persistence_manager = TaskPersistenceManager::new(temp_db.path()).await?;

    // Create multiple tasks
    let task1 = TaskBuilder::new("delete_task_1", || Ok(()))
        .every_seconds(3600)
        .build();
    
    let task2 = TaskBuilder::new("delete_task_2", || Ok(()))
        .daily()
        .at("03:00")
        .unwrap()
        .build();

    // Save tasks
    persistence_manager.save_task(&task1).await?;
    persistence_manager.save_task(&task2).await?;

    // Initial task count
    let initial_tasks = persistence_manager.list_tasks().await?;
    assert_eq!(initial_tasks.len(), 2);

    // Delete one task
    persistence_manager.delete_task(&task1.id().to_string()).await?;

    // Verify deletion
    let remaining_tasks = persistence_manager.list_tasks().await?;
    assert_eq!(remaining_tasks.len(), 1);
    assert_eq!(remaining_tasks[0].name, "delete_task_2");

    // Clear remaining tasks
    persistence_manager.clear_tasks().await?;
    let final_tasks = persistence_manager.list_tasks().await?;
    assert_eq!(final_tasks.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_persistence_error_handling() -> Result<()> {
    let temp_db = NamedTempFile::new()?;
    let persistence_manager = TaskPersistenceManager::new(temp_db.path()).await?;

    // Try to get a non-existent task
    let non_existent_task = persistence_manager.get_task(&Uuid::new_v4().to_string()).await?;
    assert!(non_existent_task.is_none());

    // Try to delete a non-existent task (should not raise an error)
    persistence_manager.delete_task(&Uuid::new_v4().to_string()).await?;

    Ok(())
}

#[tokio::test]
async fn test_persistence_concurrent_access() -> Result<()> {
    let temp_db = NamedTempFile::new()?;
    let persistence_manager = TaskPersistenceManager::new(temp_db.path()).await?;

    // Create multiple tasks concurrently
    let tasks: Vec<Task> = (0..10)
        .map(|i| {
            TaskBuilder::new(&format!("concurrent_task_{}", i), || Ok(()))
                .every_seconds(60)
                .build()
        })
        .collect();

    // Save tasks concurrently
    let save_futures = tasks.iter().map(|task| persistence_manager.save_task(task));
    futures::future::try_join_all(save_futures).await?;

    // Verify all tasks were saved
    let saved_tasks = persistence_manager.list_tasks().await?;
    assert_eq!(saved_tasks.len(), 10);

    Ok(())
}
