/// Queue Performance Benchmarks for Phase 4A
///
/// This benchmark suite measures queue performance under various load conditions

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use inferno::operations::queue::*;

fn benchmark_priority_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("priority_queue");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut queue = PriorityQueue::new();

                for i in 0..size {
                    let priority = Priority::from_u8(((i % 4) + 1) as u8).unwrap();
                    queue.push(RequestMetadata::new(
                        format!("req_{}", i),
                        "user".to_string(),
                        priority,
                        "model".to_string(),
                    ));
                }

                while let Some(_) = queue.pop() {}
            });
        });
    }

    group.finish();
}

fn benchmark_fair_scheduler(c: &mut Criterion) {
    let mut group = c.benchmark_group("fair_scheduler");

    for size in [100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut scheduler = FairScheduler::new();

                for i in 0..size {
                    let priority = Priority::from_u8(((i % 4) + 1) as u8).unwrap();
                    scheduler.enqueue(RequestMetadata::new(
                        format!("req_{}", i),
                        format!("user_{}", i % 10),
                        priority,
                        "model".to_string(),
                    ));
                }

                while let Some(_) = scheduler.dequeue() {}
            });
        });
    }

    group.finish();
}

fn benchmark_load_balancer(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_balancer");

    group.bench_function("least_loaded_1000_requests", |b| {
        b.iter(|| {
            let mut lb = LoadBalancer::new(AssignmentStrategy::LeastLoaded);

            for i in 1..=20 {
                lb.register_worker(i);
            }

            for i in 0..1000 {
                lb.update_worker_metrics(
                    ((i % 20) + 1) as u32,
                    (i % 10) as u32,
                    1000 * (i as u64),
                    4096,
                );

                let req = RequestMetadata::new(
                    format!("req_{}", i),
                    "user".to_string(),
                    Priority::Normal,
                    "model".to_string(),
                );

                let _ = lb.assign_request(&req, 10_000);
            }
        });
    });

    group.finish();
}

fn benchmark_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics");

    group.bench_function("metrics_1000_samples", |b| {
        b.iter(|| {
            let mut metrics = QueueMetricsCollector::new();

            for i in 0..1000 {
                let priority = ((i % 4) + 1) as u8;
                metrics.record_queued(priority);
                metrics.record_queue_depth(i % 100);
                metrics.record_processed(priority, (i as u64 * 50) % 5000);
            }

            let _ = metrics.snapshot(50);
        });
    });

    group.finish();
}

fn benchmark_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");
    group.sample_size(10);

    group.bench_function("realistic_1000_requests", |b| {
        b.iter(|| {
            let mut scheduler = FairScheduler::new();
            let mut lb = LoadBalancer::new(AssignmentStrategy::LeastLoaded);
            let mut metrics = QueueMetricsCollector::new();

            for i in 1..=20 {
                lb.register_worker(i);
            }

            for i in 0..1000 {
                let priority = match i % 100 {
                    0..=10 => Priority::VIP,
                    11..=30 => Priority::High,
                    31..=80 => Priority::Normal,
                    _ => Priority::Low,
                };

                let req = RequestMetadata::new(
                    format!("req_{}", i),
                    format!("user_{}", i % 20),
                    priority,
                    if i % 3 == 0 { "model1" } else { "model2" }.to_string(),
                );

                scheduler.enqueue(req.clone());
                metrics.record_queued(priority as u8);

                let _ = lb.assign_request(&req, 10_000);
            }

            while let Some(req) = scheduler.dequeue() {
                metrics.record_processed(req.priority as u8, 100);
            }

            let _ = metrics.snapshot(0);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_priority_queue,
    benchmark_fair_scheduler,
    benchmark_load_balancer,
    benchmark_metrics,
    benchmark_mixed_workload
);

criterion_main!(benches);
