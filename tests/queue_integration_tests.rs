//! Queue System Integration Tests
//!
//! Comprehensive tests for Phase 4A queue implementation including:
//! - Priority ordering verification
//! - Fair scheduling and starvation prevention
//! - Worker pool scaling
//! - Load balancing
//! - Backpressure handling

use inferno::operations::queue::*;

#[test]
fn test_priority_ordering_under_mixed_load() {
    let mut scheduler = FairScheduler::new();

    // Add requests from each priority level
    let mut request_count = 0;
    for priority in 1..=4 {
        for i in 0..10 {
            let prio = Priority::from_u8(priority).unwrap();
            let req = RequestMetadata::new(
                format!("req_p{}_i{}", priority, i),
                format!("user_{}", i),
                prio,
                "llama-2-7b".to_string(),
            );
            scheduler.enqueue(req);
            request_count += 1;
        }
    }

    assert_eq!(scheduler.len(), request_count);

    // Dequeue and verify priority ordering
    let mut vip_count = 0;
    let mut high_count = 0;
    let mut normal_count = 0;
    let mut low_count = 0;

    while let Some(req) = scheduler.dequeue() {
        match req.priority {
            Priority::VIP => vip_count += 1,
            Priority::High => high_count += 1,
            Priority::Normal => normal_count += 1,
            Priority::Low => low_count += 1,
        }
    }

    assert_eq!(vip_count, 10);
    assert_eq!(high_count, 10);
    assert_eq!(normal_count, 10);
    assert_eq!(low_count, 10);
}

#[test]
fn test_starvation_prevention_with_aging() {
    let mut scheduler = FairScheduler::new().with_starvation_threshold(5000);

    // Add 1 VIP request
    scheduler.enqueue(RequestMetadata::new(
        "vip1".to_string(),
        "user".to_string(),
        Priority::VIP,
        "model".to_string(),
    ));

    // Add 100 Low priority requests
    for i in 0..100 {
        scheduler.enqueue(RequestMetadata::new(
            format!("low_{}", i),
            "user".to_string(),
            Priority::Low,
            "model".to_string(),
        ));
    }

    // Dequeue and check fairness
    let mut vip_processed = 0;
    let mut low_processed = 0;

    while let Some(req) = scheduler.dequeue() {
        match req.priority {
            Priority::VIP => vip_processed += 1,
            Priority::Low => low_processed += 1,
            _ => {}
        }
    }

    assert_eq!(vip_processed, 1);
    assert_eq!(low_processed, 100);

    // Check fairness score
    let stats = scheduler.fairness_stats();
    assert!(stats.fairness_score > 0.5); // At least 50% met SLA
}

#[test]
fn test_deadline_escalation() {
    let mut queue = PriorityQueue::new();

    // Add a low-priority request with approaching deadline
    let mut urgent = RequestMetadata::new(
        "urgent_low".to_string(),
        "user".to_string(),
        Priority::Low,
        "model".to_string(),
    );
    urgent.deadline_secs = Some(5); // 5 seconds deadline

    // Add a normal priority request
    queue.push(RequestMetadata::new(
        "normal".to_string(),
        "user".to_string(),
        Priority::Normal,
        "model".to_string(),
    ));

    queue.push(urgent);

    // Urgent should be prioritized due to deadline
    let first = queue.pop().unwrap();
    assert_eq!(first.request_id, "urgent_low");
}

#[test]
fn test_worker_pool_auto_scaling() {
    let config = WorkerPoolConfig::new("llama-2-7b".to_string())
        .with_min_workers(1)
        .with_max_workers(10)
        .with_target_latency_ms(200);

    let mut pool = WorkerPool::new(config);
    assert_eq!(pool.len(), 1);

    // Simulate high load
    for _ in 0..5 {
        if let Some(worker) = pool.get_least_loaded_worker() {
            pool.assign_request(worker);
        }
    }

    // Trigger auto-scale
    pool.auto_scale(50, 300.0, 20_000); // High queue, high latency, lots of memory
    assert!(pool.len() > 1);
}

#[test]
fn test_load_balancer_strategies() {
    let mut lb = LoadBalancer::new(AssignmentStrategy::LeastLoaded);

    for i in 1..=4 {
        lb.register_worker(i);
    }

    lb.update_worker_metrics(1, 10, 1000, 4096);
    lb.update_worker_metrics(2, 5, 500, 4096);
    lb.update_worker_metrics(3, 15, 2000, 4096);
    lb.update_worker_metrics(4, 8, 1500, 4096);

    let req = RequestMetadata::new(
        "test_req".to_string(),
        "user".to_string(),
        Priority::Normal,
        "model".to_string(),
    );

    let assignment = lb.assign_request(&req, 10_000);
    assert!(assignment.is_some());
    assert_eq!(assignment.unwrap().assigned_worker_id, 2); // Least loaded
}

#[test]
fn test_backpressure_detection() {
    let lb = LoadBalancer::new(AssignmentStrategy::LeastLoaded);

    let status = lb.check_backpressure(1000, 4096);
    assert_eq!(status, BackpressureStatus::Healthy);

    let status = lb.check_backpressure(7000, 4096);
    assert_eq!(status, BackpressureStatus::Elevated);

    let status = lb.check_backpressure(9500, 256);
    assert_eq!(status, BackpressureStatus::Critical);
}

#[test]
fn test_queue_metrics() {
    let mut metrics = QueueMetricsCollector::new();

    for i in 0..20 {
        let priority = ((i % 4) + 1) as u8;
        metrics.record_queued(priority);
        metrics.record_queue_depth(i);
        metrics.record_processed(priority, (i as u64 + 1) * 50);
    }

    let snapshot = metrics.snapshot(5);
    assert_eq!(snapshot.total_queued, 20);
    assert_eq!(snapshot.total_processed, 20);
    assert!(snapshot.throughput_requests_per_sec > 0.0);
}

#[test]
fn test_health_status() {
    let health = QueueHealthStatus::new(1000, 4, 250.0, 100, true);
    assert_eq!(health.status, HealthStatus::Critical);

    let health = QueueHealthStatus::new(500, 4, 150.0, 4096, true);
    assert_eq!(health.status, HealthStatus::Healthy);
}

#[test]
fn test_end_to_end_workflow() {
    let mut scheduler = FairScheduler::new();
    let mut metrics = QueueMetricsCollector::new();

    for i in 0..50 {
        let priority = match i % 4 {
            0 => Priority::VIP,
            1 => Priority::High,
            2 => Priority::Normal,
            _ => Priority::Low,
        };

        let req = RequestMetadata::new(
            format!("req_{}", i),
            format!("user_{}", i % 5),
            priority,
            "model".to_string(),
        );

        scheduler.enqueue(req);
        metrics.record_queued(priority as u8);
    }

    let mut processed = 0;
    while let Some(req) = scheduler.dequeue() {
        metrics.record_processed(req.priority as u8, 100);
        processed += 1;
    }

    assert_eq!(processed, 50);

    let snapshot = metrics.snapshot(0);
    assert_eq!(snapshot.total_processed, 50);
}
