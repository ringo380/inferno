//! Priority Queue Infrastructure for Advanced Request Management
//!
//! This module implements a priority queue based on BinaryHeap that supports:
//! - Priority levels: VIP, High, Normal, Low
//! - Deadline-based ordering
//! - FIFO ordering for equal priority requests
//! - Queue statistics and metrics

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Priority levels for request processing
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Hash)]
pub enum Priority {
    /// Payment-backed, highest priority requests
    VIP = 4,
    /// Premium users with higher SLA
    High = 3,
    /// Standard users
    Normal = 2,
    /// Batch/background operations
    Low = 1,
}

impl Priority {
    /// Get the weight for fair scheduling
    pub fn weight(&self) -> u32 {
        match self {
            Priority::VIP => 8,
            Priority::High => 4,
            Priority::Normal => 2,
            Priority::Low => 1,
        }
    }

    /// Convert from numeric representation
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Priority::Low),
            2 => Some(Priority::Normal),
            3 => Some(Priority::High),
            4 => Some(Priority::VIP),
            _ => None,
        }
    }
}

/// Metadata for a queued inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    /// Unique identifier for this request
    pub request_id: String,
    /// User or client identifier
    pub user_id: String,
    /// Priority level for this request
    pub priority: Priority,
    /// When the request was submitted
    pub created_at: u64, // Unix timestamp in milliseconds
    /// Optional deadline in seconds from creation
    pub deadline_secs: Option<u64>,
    /// Estimated token budget for this request
    pub estimated_tokens: u32,
    /// Target model identifier
    pub model_id: String,
    /// User-defined tags for routing
    pub tags: Vec<String>,
    /// Number of retry attempts already made
    pub retry_count: u32,
    /// IDs of other requests this depends on
    pub dependencies: Vec<String>,
}

impl RequestMetadata {
    /// Create a new request metadata
    pub fn new(request_id: String, user_id: String, priority: Priority, model_id: String) -> Self {
        Self {
            request_id,
            user_id,
            priority,
            created_at: Self::current_timestamp(),
            deadline_secs: None,
            estimated_tokens: 256, // Default estimate
            model_id,
            tags: Vec::new(),
            retry_count: 0,
            dependencies: Vec::new(),
        }
    }

    /// Set the deadline for this request (in seconds from now)
    pub fn with_deadline(mut self, deadline_secs: u64) -> Self {
        self.deadline_secs = Some(deadline_secs);
        self
    }

    /// Set the estimated token count
    pub fn with_estimated_tokens(mut self, tokens: u32) -> Self {
        self.estimated_tokens = tokens;
        self
    }

    /// Add a tag to the request
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Add dependency on another request
    pub fn with_dependency(mut self, dep_id: String) -> Self {
        self.dependencies.push(dep_id);
        self
    }

    /// Get the current effective priority, considering age and deadline
    pub fn effective_priority(&self) -> i32 {
        let mut priority_value = self.priority as i32;

        // Age boost: older requests get priority boost
        let age_ms = Self::current_timestamp().saturating_sub(self.created_at);
        let age_secs = age_ms / 1000;
        priority_value += (age_secs / 10) as i32;

        // Deadline escalation
        if let Some(deadline_secs) = self.deadline_secs {
            let elapsed_secs = age_secs;
            let remaining_secs = deadline_secs.saturating_sub(elapsed_secs);

            if remaining_secs < 10 {
                // Critical: boost to VIP + 10
                priority_value = (Priority::VIP as i32) + 10;
            } else if remaining_secs < 30 {
                // Urgent: boost to VIP level
                priority_value = (Priority::VIP as i32) + 5;
            }
        }

        priority_value
    }

    /// Get current Unix timestamp in milliseconds
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Get age in milliseconds
    pub fn age_ms(&self) -> u64 {
        Self::current_timestamp().saturating_sub(self.created_at)
    }
}

/// Wrapper for priority queue ordering
#[derive(Debug, Clone)]
struct QueuedRequest {
    /// The request metadata
    metadata: RequestMetadata,
    /// Position in queue for FIFO ordering when priorities are equal
    sequence: u64,
}

impl PartialEq for QueuedRequest {
    fn eq(&self, other: &Self) -> bool {
        self.metadata.request_id == other.metadata.request_id
    }
}

impl Eq for QueuedRequest {}

impl PartialOrd for QueuedRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        // First, compare effective priority (higher is better, so reverse for BinaryHeap)
        let self_priority = self.metadata.effective_priority();
        let other_priority = other.metadata.effective_priority();

        match other_priority.cmp(&self_priority) {
            Ordering::Equal => {
                // If priority is equal, maintain FIFO order (lower sequence is better)
                other.sequence.cmp(&self.sequence)
            }
            other_ordering => other_ordering,
        }
    }
}

/// Statistics about the queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub queued_count: usize,
    pub total_weight: u32,
    pub estimated_wait_ms: u64,
}

/// Priority queue for managing inference requests
#[derive(Debug)]
pub struct PriorityQueue {
    heap: BinaryHeap<QueuedRequest>,
    sequence_counter: u64,
}

impl PriorityQueue {
    /// Create a new empty priority queue
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            sequence_counter: 0,
        }
    }

    /// Add a request to the queue
    pub fn push(&mut self, metadata: RequestMetadata) {
        let queued = QueuedRequest {
            metadata,
            sequence: self.sequence_counter,
        };
        self.sequence_counter = self.sequence_counter.wrapping_add(1);
        self.heap.push(queued);
    }

    /// Remove and return the highest priority request
    pub fn pop(&mut self) -> Option<RequestMetadata> {
        self.heap.pop().map(|q| q.metadata)
    }

    /// Peek at the highest priority request without removing it
    pub fn peek(&self) -> Option<&RequestMetadata> {
        self.heap.peek().map(|q| &q.metadata)
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Get the number of queued requests
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Get statistics about the queue
    pub fn stats(&self) -> QueueStats {
        let queued_count = self.heap.len();
        let total_weight: u32 = self.heap.iter().map(|q| q.metadata.priority.weight()).sum();

        // Estimate wait based on token counts and weights
        let estimated_tokens: u32 = self.heap.iter().map(|q| q.metadata.estimated_tokens).sum();
        // Assume ~50 tokens/sec average processing speed
        let estimated_wait_ms = ((estimated_tokens as f64 / 50.0) * 1000.0) as u64;

        QueueStats {
            queued_count,
            total_weight,
            estimated_wait_ms,
        }
    }

    /// Find and remove a request by ID
    pub fn remove_by_id(&mut self, request_id: &str) -> Option<RequestMetadata> {
        let mut removed = None;
        let temp: Vec<QueuedRequest> = self
            .heap
            .drain()
            .filter(|q| {
                if q.metadata.request_id == request_id {
                    removed = Some(q.metadata.clone());
                    false
                } else {
                    true
                }
            })
            .collect();

        temp.into_iter().for_each(|q| {
            self.heap.push(q);
        });

        removed
    }

    /// Get all pending requests (for debugging/monitoring)
    pub fn iter(&self) -> impl Iterator<Item = &RequestMetadata> {
        self.heap.iter().map(|q| &q.metadata)
    }

    /// Drain all requests from the queue
    pub fn drain(&mut self) -> impl Iterator<Item = RequestMetadata> + '_ {
        self.heap.drain().map(|q| q.metadata)
    }
}

impl Default for PriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        let mut queue = PriorityQueue::new();

        // Add requests with different priorities
        queue.push(RequestMetadata::new(
            "req1".to_string(),
            "user1".to_string(),
            Priority::Normal,
            "model1".to_string(),
        ));

        queue.push(RequestMetadata::new(
            "req2".to_string(),
            "user2".to_string(),
            Priority::VIP,
            "model1".to_string(),
        ));

        queue.push(RequestMetadata::new(
            "req3".to_string(),
            "user3".to_string(),
            Priority::Low,
            "model1".to_string(),
        ));

        queue.push(RequestMetadata::new(
            "req4".to_string(),
            "user4".to_string(),
            Priority::High,
            "model1".to_string(),
        ));

        // Should pop in priority order: VIP, High, Normal, Low
        assert_eq!(queue.pop().unwrap().request_id, "req2"); // VIP
        assert_eq!(queue.pop().unwrap().request_id, "req4"); // High
        assert_eq!(queue.pop().unwrap().request_id, "req1"); // Normal
        assert_eq!(queue.pop().unwrap().request_id, "req3"); // Low
        assert!(queue.is_empty());
    }

    #[test]
    fn test_fifo_ordering_same_priority() {
        let mut queue = PriorityQueue::new();

        // Add multiple requests with same priority
        queue.push(RequestMetadata::new(
            "req1".to_string(),
            "user1".to_string(),
            Priority::Normal,
            "model1".to_string(),
        ));

        queue.push(RequestMetadata::new(
            "req2".to_string(),
            "user2".to_string(),
            Priority::Normal,
            "model1".to_string(),
        ));

        queue.push(RequestMetadata::new(
            "req3".to_string(),
            "user3".to_string(),
            Priority::Normal,
            "model1".to_string(),
        ));

        // Should pop in FIFO order
        assert_eq!(queue.pop().unwrap().request_id, "req1");
        assert_eq!(queue.pop().unwrap().request_id, "req2");
        assert_eq!(queue.pop().unwrap().request_id, "req3");
    }

    #[test]
    fn test_deadline_handling() {
        let mut queue = PriorityQueue::new();

        // Add a low-priority request
        queue.push(RequestMetadata::new(
            "req1".to_string(),
            "user1".to_string(),
            Priority::Low,
            "model1".to_string(),
        ));

        // Add a high-priority request with deadline approaching
        let mut urgent_req = RequestMetadata::new(
            "req2".to_string(),
            "user2".to_string(),
            Priority::Normal,
            "model1".to_string(),
        );
        urgent_req.deadline_secs = Some(5); // 5 seconds deadline

        queue.push(urgent_req);

        // Even though it's Normal priority, deadline escalation should make it higher
        let first = queue.pop().unwrap();
        assert_eq!(first.request_id, "req2");
    }

    #[test]
    fn test_queue_stats() {
        let mut queue = PriorityQueue::new();

        queue.push(RequestMetadata::new(
            "req1".to_string(),
            "user1".to_string(),
            Priority::VIP,
            "model1".to_string(),
        ));

        queue.push(RequestMetadata::new(
            "req2".to_string(),
            "user2".to_string(),
            Priority::High,
            "model1".to_string(),
        ));

        let stats = queue.stats();
        assert_eq!(stats.queued_count, 2);
        assert_eq!(stats.total_weight, 8 + 4); // VIP weight + High weight
    }

    #[test]
    fn test_remove_by_id() {
        let mut queue = PriorityQueue::new();

        queue.push(RequestMetadata::new(
            "req1".to_string(),
            "user1".to_string(),
            Priority::Normal,
            "model1".to_string(),
        ));

        queue.push(RequestMetadata::new(
            "req2".to_string(),
            "user2".to_string(),
            Priority::Normal,
            "model1".to_string(),
        ));

        queue.push(RequestMetadata::new(
            "req3".to_string(),
            "user3".to_string(),
            Priority::Normal,
            "model1".to_string(),
        ));

        let removed = queue.remove_by_id("req2");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().request_id, "req2");

        // Verify remaining requests are correct
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.pop().unwrap().request_id, "req1");
        assert_eq!(queue.pop().unwrap().request_id, "req3");
    }

    #[test]
    fn test_empty_queue() {
        let queue = PriorityQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert!(queue.peek().is_none());
    }

    #[test]
    fn test_priority_weight() {
        assert_eq!(Priority::VIP.weight(), 8);
        assert_eq!(Priority::High.weight(), 4);
        assert_eq!(Priority::Normal.weight(), 2);
        assert_eq!(Priority::Low.weight(), 1);
    }

    #[test]
    fn test_request_builder() {
        let req = RequestMetadata::new(
            "req1".to_string(),
            "user1".to_string(),
            Priority::Normal,
            "model1".to_string(),
        )
        .with_deadline(60)
        .with_estimated_tokens(512)
        .with_tag("batch".to_string())
        .with_dependency("dep1".to_string());

        assert_eq!(req.deadline_secs, Some(60));
        assert_eq!(req.estimated_tokens, 512);
        assert_eq!(req.tags, vec!["batch".to_string()]);
        assert_eq!(req.dependencies, vec!["dep1".to_string()]);
    }
}
