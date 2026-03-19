use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Duration;
use uuid::Uuid;

use crate::battery::NetworkType;

/// Retry policy for failed offline requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u8,
    pub base_delay: Duration,
    pub backoff_factor: f32,
}

impl RetryPolicy {
    pub fn default_policy() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(5),
            backoff_factor: 2.0,
        }
    }

    /// Calculate delay for the nth retry.
    pub fn delay_for_retry(&self, attempt: u8) -> Duration {
        let factor = self.backoff_factor.powi(attempt as i32);
        Duration::from_secs_f32(self.base_delay.as_secs_f32() * factor)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::default_policy()
    }
}

/// A request queued while the device is offline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedRequest {
    pub id: Uuid,
    pub domain: String,
    pub payload: Vec<u8>,
    pub queued_at: DateTime<Utc>,
    pub priority: u8,
    pub retry_count: u8,
}

/// Transactional outbox for offline-first architecture.
///
/// Invariant 6: maximum 100 queued requests. When full, the oldest is evicted.
#[derive(Debug)]
pub struct OfflineOutbox {
    pub queued_requests: VecDeque<QueuedRequest>,
    pub max_queue_size: usize,
    pub retry_policy: RetryPolicy,
    pub last_sync: Option<DateTime<Utc>>,
}

impl OfflineOutbox {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            queued_requests: VecDeque::new(),
            max_queue_size,
            retry_policy: RetryPolicy::default_policy(),
            last_sync: None,
        }
    }

    /// Enqueue a request. Evicts the oldest if the queue is full.
    pub fn enqueue(&mut self, domain: &str, payload: Vec<u8>, priority: u8) -> Uuid {
        // Enforce bounded queue (invariant 6).
        while self.queued_requests.len() >= self.max_queue_size {
            let evicted = self.queued_requests.pop_front();
            if let Some(evicted) = evicted {
                tracing::warn!(
                    request_id = %evicted.id,
                    domain = %evicted.domain,
                    "evicted oldest request from offline queue (at capacity)"
                );
            }
        }

        let request = QueuedRequest {
            id: Uuid::new_v4(),
            domain: domain.to_string(),
            payload,
            queued_at: Utc::now(),
            priority,
            retry_count: 0,
        };
        let id = request.id;
        self.queued_requests.push_back(request);
        tracing::debug!(request_id = %id, domain, "request queued for offline flush");
        id
    }

    /// Flush queued requests. Returns the number of requests ready to send.
    ///
    /// Only flushes when connectivity is available (not Offline).
    pub fn flush(&mut self, network: NetworkType) -> Vec<QueuedRequest> {
        if network == NetworkType::Offline {
            tracing::debug!("cannot flush: device is offline");
            return Vec::new();
        }

        let flushed: Vec<QueuedRequest> = self.queued_requests.drain(..).collect();
        let count = flushed.len();
        if count > 0 {
            self.last_sync = Some(Utc::now());
            tracing::info!(count, "flushed offline queue");
        }
        flushed
    }

    /// Number of queued requests.
    pub fn queue_len(&self) -> usize {
        self.queued_requests.len()
    }

    /// Whether the queue is at capacity.
    pub fn is_full(&self) -> bool {
        self.queued_requests.len() >= self.max_queue_size
    }
}

impl Default for OfflineOutbox {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_and_flush() {
        let mut outbox = OfflineOutbox::new(100);
        outbox.enqueue("finance", vec![1, 2, 3], 5);
        outbox.enqueue("health", vec![4, 5, 6], 3);
        assert_eq!(outbox.queue_len(), 2);

        let flushed = outbox.flush(NetworkType::WiFi);
        assert_eq!(flushed.len(), 2);
        assert_eq!(outbox.queue_len(), 0);
        assert!(outbox.last_sync.is_some());
    }

    #[test]
    fn test_no_flush_when_offline() {
        let mut outbox = OfflineOutbox::new(100);
        outbox.enqueue("test", vec![], 1);
        let flushed = outbox.flush(NetworkType::Offline);
        assert!(flushed.is_empty());
        assert_eq!(outbox.queue_len(), 1);
    }

    #[test]
    fn test_eviction_at_capacity() {
        let mut outbox = OfflineOutbox::new(2);
        let _id1 = outbox.enqueue("first", vec![1], 1);
        let _id2 = outbox.enqueue("second", vec![2], 2);
        let id3 = outbox.enqueue("third", vec![3], 3);

        assert_eq!(outbox.queue_len(), 2);
        // The first one should have been evicted.
        assert!(outbox
            .queued_requests
            .iter()
            .any(|r| r.id == id3));
    }

    #[test]
    fn test_retry_policy_backoff() {
        let policy = RetryPolicy::default_policy();
        let d0 = policy.delay_for_retry(0);
        let d1 = policy.delay_for_retry(1);
        let d2 = policy.delay_for_retry(2);
        assert!(d1 > d0);
        assert!(d2 > d1);
    }
}
