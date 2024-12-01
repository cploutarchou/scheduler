use tokio_task_scheduler::{Scheduler, TaskBuilder, TaskStatus};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_scheduler_creation() {
    let scheduler = Scheduler::new();
    let _rx = scheduler.start().await;
    assert!(scheduler.stop().await.is_ok());
}

#[tokio::test]
async fn test_task_execution() {
    let scheduler = Scheduler::new();
    let _rx = scheduler.start().await;

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let task = TaskBuilder::new("test_task", move || {
        let counter = counter_clone.clone();
        let mut count = counter.try_lock().unwrap();
        *count += 1;
        Ok(())
    })
    .every_minutes(1)
    .build();

    let task_id = scheduler.add_task(task).await.unwrap();

    // Wait for task execution
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    let count = *counter.lock().await;
    assert!(count > 0, "Task should have executed at least once");

    let status = scheduler.get_task_status(&task_id).await.unwrap();
    assert!(matches!(status, TaskStatus::Completed));

    scheduler.stop().await.unwrap();
}

#[tokio::test]
async fn test_multiple_tasks() {
    let scheduler = Scheduler::new();
    let _rx = scheduler.start().await;

    let task1 = TaskBuilder::new("task1", || Ok(()))
        .every_seconds(1)
        .build();

    let task2 = TaskBuilder::new("task2", || Ok(()))
        .every_minutes(1)
        .build();

    let task1_id = scheduler.add_task(task1).await.unwrap();
    let task2_id = scheduler.add_task(task2).await.unwrap();

    assert!(scheduler.get_task_status(&task1_id).await.is_ok());
    assert!(scheduler.get_task_status(&task2_id).await.is_ok());

    scheduler.stop().await.unwrap();
}

#[tokio::test]
async fn test_task_removal() {
    let scheduler = Scheduler::new();
    let _rx = scheduler.start().await;

    let task = TaskBuilder::new("test_task", || Ok(()))
        .every_minutes(1)
        .build();

    let task_id = scheduler.add_task(task).await.unwrap();
    assert!(scheduler.remove(&task_id).await.is_ok());
    assert!(scheduler.get_task_status(&task_id).await.is_err());

    scheduler.stop().await.unwrap();
}

#[tokio::test]
async fn test_task_scheduling_intervals() {
    let scheduler = Scheduler::new();
    let _rx = scheduler.start().await;

    // Test various intervals
    let task1 = TaskBuilder::new("seconds", || Ok(()))
        .every_seconds(1)
        .build();

    let task2 = TaskBuilder::new("minutes", || Ok(()))
        .every_minutes(1)
        .build();

    let task3 = TaskBuilder::new("hours", || Ok(()))
        .daily()
        .build();

    let task4 = TaskBuilder::new("days", || Ok(()))
        .daily()
        .build();

    scheduler.add_task(task1).await.unwrap();
    scheduler.add_task(task2).await.unwrap();
    scheduler.add_task(task3).await.unwrap();
    scheduler.add_task(task4).await.unwrap();

    // Let tasks run for a bit
    tokio::time::sleep(Duration::from_secs(2)).await;
    scheduler.stop().await.unwrap();
}

#[tokio::test]
async fn test_task_error_handling() {
    let scheduler = Scheduler::new();
    let _rx = scheduler.start().await;

    let task = TaskBuilder::new("failing_task", || {
        Err(tokio_task_scheduler::error::SchedulerError::TaskExecutionFailed(
            "Test error".to_string(),
        ))
    })
    .every_minutes(1)
    .build();

    let task_id = scheduler.add_task(task).await.unwrap();
    
    // Wait for task execution
    tokio::time::sleep(Duration::from_secs(2)).await;

    // The task might have been removed due to failure, so check for either Failed status or TaskNotFound
    match scheduler.get_task_status(&task_id).await {
        Ok(status) => assert!(matches!(status, TaskStatus::Failed(_))),
        Err(tokio_task_scheduler::error::SchedulerError::TaskNotFound(_)) => {},
        Err(e) => panic!("Unexpected error: {:?}", e),
    }

    scheduler.stop().await.unwrap();
}

#[tokio::test]
async fn test_task_with_specific_time() {
    use chrono::{Timelike, Utc};

    let scheduler = Scheduler::new();
    let _rx = scheduler.start().await;

    // Schedule for next minute
    let next_minute = (Utc::now().minute() + 1) % 60;
    let time = format!("{:02}:{:02}", Utc::now().hour(), next_minute);

    let task = TaskBuilder::new("timed_task", || Ok(()))
        .daily()
        .at(&time)
        .unwrap()
        .build();

    scheduler.add_task(task).await.unwrap();
    tokio::time::sleep(Duration::from_secs(2)).await;
    scheduler.stop().await.unwrap();
}

#[tokio::test]
async fn test_clear_all_tasks() {
    let scheduler = Scheduler::new();
    let _rx = scheduler.start().await;

    // Add multiple tasks
    for i in 0..3 {
        let task = TaskBuilder::new(&format!("task_{}", i), || Ok(()))
            .every_minutes(1)
            .build();
        scheduler.add_task(task).await.unwrap();
    }

    // Clear all tasks
    assert!(scheduler.clear().await.is_ok());

    scheduler.stop().await.unwrap();
}
