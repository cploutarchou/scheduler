use criterion::{criterion_group, criterion_main, Criterion};
use tokio_task_scheduler::{Scheduler, TaskBuilder};
use std::time::Duration;

async fn setup_scheduler() -> Scheduler {
    let scheduler = Scheduler::new();
    let _rx = scheduler.start().await;  // Store receiver but don't unwrap
    scheduler
}

async fn add_tasks(scheduler: &Scheduler, count: u32) {
    for i in 0..count {
        let task = TaskBuilder::new(format!("task_{}", i).as_str(), || Ok(()))
            .every_minutes(1)
            .build();
        scheduler.add_task(task).await.unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("add 100 tasks", |b| {
        b.to_async(&rt).iter(|| async {
            let scheduler = setup_scheduler().await;
            add_tasks(&scheduler, 100).await;
        });
    });

    let mut group = c.benchmark_group("scheduler_operations");
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("start_stop", |b| {
        b.to_async(&rt).iter(|| async {
            let scheduler = Scheduler::new();
            let _rx = scheduler.start().await;  // Store receiver but don't unwrap
            scheduler.stop().await.unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
