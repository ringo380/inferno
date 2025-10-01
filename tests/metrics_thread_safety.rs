/// Test for MetricsCollector thread safety after Arc<T> refactoring
///
/// This test verifies that:
/// 1. MetricsCollector is Send + Sync and can be safely shared across threads
/// 2. Multiple clones can record metrics concurrently without data races
/// 3. The event processor correctly aggregates events from multiple threads
use inferno::metrics::{InferenceEvent, MetricsCollector};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_metrics_collector_concurrent_access() {
    // Create collector and processor
    let (collector, processor) = MetricsCollector::new();
    processor.start();

    // Clone collector for multi-threaded access
    let collector = Arc::new(collector);

    // Spawn multiple tasks that record metrics concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let collector_clone = Arc::clone(&collector);
        let handle = tokio::spawn(async move {
            for j in 0..100 {
                let event = InferenceEvent {
                    model_name: format!("model-{}", i % 3),
                    input_length: 50 + j,
                    output_length: 100 + j,
                    duration: Duration::from_millis(10 + j),
                    success: j % 10 != 0,
                };
                collector_clone.record_inference(event);
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    // Allow event processing to catch up
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify metrics were collected
    let snapshot = collector.get_snapshot().await.expect("Failed to get snapshot");

    // We sent 10 tasks * 100 events = 1000 events
    assert!(snapshot.total_requests >= 1000, "Expected at least 1000 requests, got {}", snapshot.total_requests);

    // Success rate should be around 90% (j % 10 != 0)
    let success_rate = (snapshot.successful_requests as f64) / (snapshot.total_requests as f64) * 100.0;
    assert!(success_rate >= 85.0 && success_rate <= 95.0,
        "Expected ~90% success rate, got {:.2}%", success_rate);

    println!("✅ Thread safety test passed!");
    println!("   Total requests: {}", snapshot.total_requests);
    println!("   Successful: {}", snapshot.successful_requests);
    println!("   Failed: {}", snapshot.failed_requests);
    println!("   Success rate: {:.2}%", success_rate);
}

#[tokio::test]
async fn test_metrics_collector_clone_safety() {
    let (collector, processor) = MetricsCollector::new();
    processor.start();

    // Test that Clone works and produces independent handles to shared state
    let collector1 = collector.clone();
    let collector2 = collector.clone();

    // Record events using different clones
    collector1.record_inference(InferenceEvent {
        model_name: "model-a".to_string(),
        input_length: 10,
        output_length: 20,
        duration: Duration::from_millis(5),
        success: true,
    });

    collector2.record_inference(InferenceEvent {
        model_name: "model-b".to_string(),
        input_length: 15,
        output_length: 25,
        duration: Duration::from_millis(8),
        success: true,
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Both events should be recorded in shared state
    let snapshot = collector.get_snapshot().await.expect("Failed to get snapshot");
    assert_eq!(snapshot.total_requests, 2, "Expected 2 requests");
    assert_eq!(snapshot.successful_requests, 2, "Expected 2 successful requests");

    println!("✅ Clone safety test passed!");
}

#[test]
fn test_metrics_collector_send_sync() {
    // This test verifies at compile-time that MetricsCollector is Send + Sync
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<MetricsCollector>();
    assert_sync::<MetricsCollector>();

    println!("✅ Send + Sync test passed at compile time!");
}
