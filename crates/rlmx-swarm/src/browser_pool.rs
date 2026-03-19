use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use uuid::Uuid;

/// Status of a browser compute worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    Idle,
    Computing,
    Disconnected,
}

/// Capabilities reported by a browser worker on connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserCapabilities {
    pub wasm: bool,
    pub simd: bool,
    pub webgpu: bool,
    pub shared_array_buffer: bool,
    pub max_workers: usize,
}

impl Default for BrowserCapabilities {
    fn default() -> Self {
        Self {
            wasm: true,
            simd: false,
            webgpu: false,
            shared_array_buffer: false,
            max_workers: 1,
        }
    }
}

/// A browser-based compute worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserWorker {
    pub id: String,
    pub connected_at: DateTime<Utc>,
    pub tasks_completed: u64,
    pub status: WorkerStatus,
    pub capabilities: Option<BrowserCapabilities>,
}

/// Type of compute task to offload to browser workers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeTaskType {
    /// Embedding generation for VecInsert syscalls.
    Embedding { text: String, dim: usize },
    /// Matrix multiplication offload.
    MatMul {
        a_shape: [usize; 2],
        b_shape: [usize; 2],
    },
    /// Cosine similarity between a query vector and candidate set.
    CosineSimilarity {
        query: Vec<f32>,
        candidates_count: usize,
    },
    /// TRM neural network forward pass.
    TrmForward { stream: String, input_dim: usize },
}

/// A compute task to be executed by a browser worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeTask {
    pub id: Uuid,
    pub task_type: ComputeTaskType,
    pub data: Vec<u8>,
    pub created_at: DateTime<Utc>,
    /// Priority level: 0 = highest (embedding), 1 = TRM, 2 = experiment, 3 = background.
    pub priority: u8,
}

/// Wrapper for BinaryHeap ordering (min-heap by priority, then earliest created_at).
#[derive(Debug, Clone)]
struct PrioritizedTask(ComputeTask);

impl PartialEq for PrioritizedTask {
    fn eq(&self, other: &Self) -> bool {
        self.0.priority == other.0.priority && self.0.created_at == other.0.created_at
    }
}

impl Eq for PrioritizedTask {}

impl PartialOrd for PrioritizedTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max-heap, so reverse priority (lower number = higher urgency).
        other
            .0
            .priority
            .cmp(&self.0.priority)
            .then_with(|| other.0.created_at.cmp(&self.0.created_at))
    }
}

/// An entry tracking a task currently being computed by a worker.
#[derive(Debug, Clone)]
pub struct InFlightEntry {
    pub task: ComputeTask,
    pub worker_id: String,
    pub assigned_at: DateTime<Utc>,
}

/// Pool of browser-based compute workers with priority queue and fault tolerance.
pub struct BrowserComputePool {
    pub workers: HashMap<String, BrowserWorker>,
    pending_tasks: BinaryHeap<PrioritizedTask>,
    completed_results: HashMap<Uuid, Vec<u8>>,
    in_flight: HashMap<Uuid, InFlightEntry>,
    /// Timeout in milliseconds for in-flight tasks before they are re-queued.
    pub timeout_ms: u64,
}

impl BrowserComputePool {
    /// Default task timeout: 30 seconds.
    const DEFAULT_TIMEOUT_MS: u64 = 30_000;

    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
            pending_tasks: BinaryHeap::new(),
            completed_results: HashMap::new(),
            in_flight: HashMap::new(),
            timeout_ms: Self::DEFAULT_TIMEOUT_MS,
        }
    }

    /// Create a pool with a custom timeout.
    pub fn with_timeout_ms(timeout_ms: u64) -> Self {
        Self {
            timeout_ms,
            ..Self::new()
        }
    }

    /// Register a new browser worker with optional capabilities.
    pub fn register_worker(
        &mut self,
        id: impl Into<String>,
        capabilities: Option<BrowserCapabilities>,
    ) {
        let id = id.into();
        self.workers.insert(
            id.clone(),
            BrowserWorker {
                id,
                connected_at: Utc::now(),
                tasks_completed: 0,
                status: WorkerStatus::Idle,
                capabilities,
            },
        );
    }

    /// Unregister a browser worker. All in-flight tasks for this worker are re-queued.
    pub fn unregister_worker(&mut self, id: &str) {
        // Collect task IDs assigned to this worker.
        let requeue_ids: Vec<Uuid> = self
            .in_flight
            .iter()
            .filter(|(_, entry)| entry.worker_id == id)
            .map(|(task_id, _)| *task_id)
            .collect();

        for task_id in requeue_ids {
            if let Some(entry) = self.in_flight.remove(&task_id) {
                tracing::info!(
                    worker_id = id,
                    task_id = %task_id,
                    "re-queuing in-flight task from disconnected worker"
                );
                self.pending_tasks.push(PrioritizedTask(entry.task));
            }
        }

        self.workers.remove(id);
    }

    /// Submit a compute task. Returns the task ID.
    pub fn submit_task(&mut self, task: ComputeTask) -> Uuid {
        let id = task.id;
        self.pending_tasks.push(PrioritizedTask(task));
        id
    }

    /// Poll for the next available task (called by workers). Respects priority ordering.
    pub fn poll_task(&mut self, worker_id: &str) -> Option<ComputeTask> {
        if let Some(worker) = self.workers.get_mut(worker_id) {
            if worker.status == WorkerStatus::Disconnected {
                return None;
            }
            if let Some(PrioritizedTask(task)) = self.pending_tasks.pop() {
                worker.status = WorkerStatus::Computing;
                self.in_flight.insert(
                    task.id,
                    InFlightEntry {
                        task: task.clone(),
                        worker_id: worker_id.to_string(),
                        assigned_at: Utc::now(),
                    },
                );
                return Some(task);
            }
        }
        None
    }

    /// Mark a task as complete with its result.
    ///
    /// Returns `true` if the result was accepted, `false` if the task was not
    /// in-flight (duplicate or already timed out and re-queued).
    pub fn complete_task(&mut self, task_id: Uuid, worker_id: &str, result: Vec<u8>) -> bool {
        // Duplicate detection: only accept if still in in_flight.
        let Some(entry) = self.in_flight.remove(&task_id) else {
            tracing::warn!(
                task_id = %task_id,
                worker_id = worker_id,
                "ignoring duplicate or expired task result"
            );
            return false;
        };

        self.completed_results.insert(task_id, result);

        if let Some(worker) = self.workers.get_mut(&entry.worker_id) {
            worker.tasks_completed += 1;
            worker.status = WorkerStatus::Idle;
        }

        true
    }

    /// Reap expired in-flight tasks (those exceeding `timeout_ms`) back to the pending queue.
    /// Returns the number of tasks re-queued.
    pub fn reap_expired(&mut self) -> usize {
        let now = Utc::now();
        let timeout = chrono::Duration::milliseconds(self.timeout_ms as i64);

        let expired_ids: Vec<Uuid> = self
            .in_flight
            .iter()
            .filter(|(_, entry)| now.signed_duration_since(entry.assigned_at) > timeout)
            .map(|(id, _)| *id)
            .collect();

        let count = expired_ids.len();
        for task_id in expired_ids {
            if let Some(entry) = self.in_flight.remove(&task_id) {
                tracing::warn!(
                    task_id = %task_id,
                    worker_id = %entry.worker_id,
                    "reaping expired in-flight task"
                );
                // Mark worker idle since the task timed out.
                if let Some(worker) = self.workers.get_mut(&entry.worker_id) {
                    worker.status = WorkerStatus::Idle;
                }
                self.pending_tasks.push(PrioritizedTask(entry.task));
            }
        }

        count
    }

    /// Retrieve a completed result by task ID.
    pub fn get_result(&self, task_id: &Uuid) -> Option<&Vec<u8>> {
        self.completed_results.get(task_id)
    }

    /// Number of registered workers.
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    /// Number of pending tasks in the queue.
    pub fn pending_count(&self) -> usize {
        self.pending_tasks.len()
    }

    /// Number of tasks currently in flight.
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }
}

impl Default for BrowserComputePool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(task_type: ComputeTaskType, priority: u8) -> ComputeTask {
        ComputeTask {
            id: Uuid::new_v4(),
            task_type,
            data: vec![],
            created_at: Utc::now(),
            priority,
        }
    }

    fn embedding_task(priority: u8) -> ComputeTask {
        make_task(
            ComputeTaskType::Embedding {
                text: "hello".into(),
                dim: 64,
            },
            priority,
        )
    }

    fn matmul_task(priority: u8) -> ComputeTask {
        make_task(
            ComputeTaskType::MatMul {
                a_shape: [4, 4],
                b_shape: [4, 4],
            },
            priority,
        )
    }

    #[test]
    fn test_register_worker() {
        let mut pool = BrowserComputePool::new();
        pool.register_worker("w1", None);
        assert_eq!(pool.worker_count(), 1);
        assert_eq!(pool.workers["w1"].status, WorkerStatus::Idle);
        assert!(pool.workers["w1"].capabilities.is_none());
    }

    #[test]
    fn test_register_worker_with_capabilities() {
        let mut pool = BrowserComputePool::new();
        let caps = BrowserCapabilities {
            wasm: true,
            simd: true,
            webgpu: false,
            shared_array_buffer: true,
            max_workers: 4,
        };
        pool.register_worker("w1", Some(caps));
        assert_eq!(pool.worker_count(), 1);
        let w = &pool.workers["w1"];
        let caps = w.capabilities.as_ref().unwrap();
        assert!(caps.simd);
        assert_eq!(caps.max_workers, 4);
    }

    #[test]
    fn test_unregister_worker() {
        let mut pool = BrowserComputePool::new();
        pool.register_worker("w1", None);
        pool.unregister_worker("w1");
        assert_eq!(pool.worker_count(), 0);
    }

    #[test]
    fn test_submit_task() {
        let mut pool = BrowserComputePool::new();
        let task = matmul_task(2);
        let id = pool.submit_task(task);
        assert_eq!(pool.pending_count(), 1);
        assert!(!id.is_nil());
    }

    #[test]
    fn test_poll_task() {
        let mut pool = BrowserComputePool::new();
        pool.register_worker("w1", None);
        let task = embedding_task(0);
        pool.submit_task(task);

        let polled = pool.poll_task("w1");
        assert!(polled.is_some());
        assert_eq!(pool.pending_count(), 0);
        assert_eq!(pool.in_flight_count(), 1);
        assert_eq!(pool.workers["w1"].status, WorkerStatus::Computing);
    }

    #[test]
    fn test_poll_task_no_worker() {
        let mut pool = BrowserComputePool::new();
        let task = embedding_task(0);
        pool.submit_task(task);
        let polled = pool.poll_task("nonexistent");
        assert!(polled.is_none());
    }

    #[test]
    fn test_complete_task() {
        let mut pool = BrowserComputePool::new();
        pool.register_worker("w1", None);
        let task = matmul_task(1);
        let task_id = task.id;
        pool.submit_task(task);
        pool.poll_task("w1");

        let accepted = pool.complete_task(task_id, "w1", vec![1, 2, 3]);
        assert!(accepted);
        assert_eq!(pool.workers["w1"].status, WorkerStatus::Idle);
        assert_eq!(pool.workers["w1"].tasks_completed, 1);
        assert_eq!(pool.in_flight_count(), 0);
        assert_eq!(pool.get_result(&task_id).unwrap(), &vec![1, 2, 3]);
    }

    #[test]
    fn test_priority_ordering_embedding_before_background() {
        let mut pool = BrowserComputePool::new();
        pool.register_worker("w1", None);

        // Submit background task first (priority 3), then embedding (priority 0).
        let bg = make_task(
            ComputeTaskType::MatMul {
                a_shape: [2, 2],
                b_shape: [2, 2],
            },
            3,
        );
        let emb = make_task(
            ComputeTaskType::Embedding {
                text: "urgent".into(),
                dim: 64,
            },
            0,
        );
        let emb_id = emb.id;
        pool.submit_task(bg);
        pool.submit_task(emb);

        // Embedding should come out first despite being submitted second.
        let polled = pool.poll_task("w1").unwrap();
        assert_eq!(polled.id, emb_id);
        assert_eq!(polled.priority, 0);
    }

    #[test]
    fn test_priority_ordering_four_levels() {
        let mut pool = BrowserComputePool::new();
        pool.register_worker("w1", None);

        let t3 = make_task(
            ComputeTaskType::CosineSimilarity {
                query: vec![1.0],
                candidates_count: 10,
            },
            3,
        );
        let t1 = make_task(
            ComputeTaskType::TrmForward {
                stream: "x".into(),
                input_dim: 16,
            },
            1,
        );
        let t2 = matmul_task(2);
        let t0 = embedding_task(0);

        let ids = [t0.id, t1.id, t2.id, t3.id];
        pool.submit_task(t3);
        pool.submit_task(t1);
        pool.submit_task(t2);
        pool.submit_task(t0);

        // Should dequeue in priority order: 0, 1, 2, 3.
        for (i, expected_id) in ids.iter().enumerate() {
            // Complete previous task so worker is idle again.
            if i > 0 {
                let prev_id = ids[i - 1];
                pool.complete_task(prev_id, "w1", vec![]);
            }
            let polled = pool.poll_task("w1").unwrap();
            assert_eq!(polled.id, *expected_id, "Expected priority {} first", i);
        }
    }

    #[test]
    fn test_fault_tolerance_worker_disconnect_requeues() {
        let mut pool = BrowserComputePool::new();
        pool.register_worker("w1", None);
        pool.register_worker("w2", None);

        let task = embedding_task(0);
        let task_id = task.id;
        pool.submit_task(task);

        // w1 picks up the task.
        let polled = pool.poll_task("w1");
        assert!(polled.is_some());
        assert_eq!(pool.in_flight_count(), 1);
        assert_eq!(pool.pending_count(), 0);

        // w1 disconnects -- task should be re-queued.
        pool.unregister_worker("w1");
        assert_eq!(pool.in_flight_count(), 0);
        assert_eq!(pool.pending_count(), 1);
        assert_eq!(pool.worker_count(), 1);

        // w2 picks up the re-queued task.
        let polled = pool.poll_task("w2").unwrap();
        assert_eq!(polled.id, task_id);
    }

    #[test]
    fn test_timeout_reaping() {
        let mut pool = BrowserComputePool::with_timeout_ms(0); // immediate timeout
        pool.register_worker("w1", None);

        let task = matmul_task(2);
        let task_id = task.id;
        pool.submit_task(task);
        pool.poll_task("w1");

        assert_eq!(pool.in_flight_count(), 1);
        assert_eq!(pool.pending_count(), 0);

        // Reap should move the expired task back.
        let reaped = pool.reap_expired();
        assert_eq!(reaped, 1);
        assert_eq!(pool.in_flight_count(), 0);
        assert_eq!(pool.pending_count(), 1);

        // Worker should be marked idle after timeout.
        assert_eq!(pool.workers["w1"].status, WorkerStatus::Idle);

        // Task can be re-polled.
        let polled = pool.poll_task("w1").unwrap();
        assert_eq!(polled.id, task_id);
    }

    #[test]
    fn test_duplicate_result_handling() {
        let mut pool = BrowserComputePool::with_timeout_ms(0);
        pool.register_worker("w1", None);
        pool.register_worker("w2", None);

        let task = embedding_task(0);
        let task_id = task.id;
        pool.submit_task(task);

        // w1 polls the task.
        pool.poll_task("w1");

        // Task times out, gets re-queued.
        pool.reap_expired();

        // w2 polls the re-queued task.
        pool.poll_task("w2");

        // w2 completes first -- accepted.
        let accepted = pool.complete_task(task_id, "w2", vec![10, 20]);
        assert!(accepted);

        // w1 also "completes" -- should be rejected (duplicate).
        let accepted = pool.complete_task(task_id, "w1", vec![99, 99]);
        assert!(!accepted);

        // Only the first result is stored.
        assert_eq!(pool.get_result(&task_id).unwrap(), &vec![10, 20]);
    }

    #[test]
    fn test_compute_task_types_serialize() {
        let types = vec![
            ComputeTaskType::Embedding {
                text: "hello".into(),
                dim: 64,
            },
            ComputeTaskType::MatMul {
                a_shape: [4, 4],
                b_shape: [4, 4],
            },
            ComputeTaskType::CosineSimilarity {
                query: vec![0.1, 0.2],
                candidates_count: 100,
            },
            ComputeTaskType::TrmForward {
                stream: "x".into(),
                input_dim: 16,
            },
        ];

        for tt in types {
            let json = serde_json::to_string(&tt).unwrap();
            let roundtrip: ComputeTaskType = serde_json::from_str(&json).unwrap();
            // Verify roundtrip doesn't panic.
            let _ = serde_json::to_string(&roundtrip).unwrap();
        }
    }

    #[test]
    fn test_browser_capabilities_default() {
        let caps = BrowserCapabilities::default();
        assert!(caps.wasm);
        assert!(!caps.simd);
        assert!(!caps.webgpu);
        assert!(!caps.shared_array_buffer);
        assert_eq!(caps.max_workers, 1);
    }

    #[test]
    fn test_multiple_workers_in_flight() {
        let mut pool = BrowserComputePool::new();
        pool.register_worker("w1", None);
        pool.register_worker("w2", None);

        let t1 = embedding_task(0);
        let t2 = matmul_task(1);
        let t1_id = t1.id;
        let t2_id = t2.id;
        pool.submit_task(t1);
        pool.submit_task(t2);

        let p1 = pool.poll_task("w1").unwrap();
        let p2 = pool.poll_task("w2").unwrap();

        assert_eq!(p1.id, t1_id);
        assert_eq!(p2.id, t2_id);
        assert_eq!(pool.in_flight_count(), 2);
        assert_eq!(pool.pending_count(), 0);

        pool.complete_task(t1_id, "w1", vec![1]);
        pool.complete_task(t2_id, "w2", vec![2]);
        assert_eq!(pool.in_flight_count(), 0);
    }
}
