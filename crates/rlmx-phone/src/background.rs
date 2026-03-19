use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::Duration;
use uuid::Uuid;

/// Status of a scheduled background task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// A task scheduled for background execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub name: String,
    pub status: TaskStatus,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub max_duration: Duration,
}

/// A task with priority for the binary heap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedTask {
    pub priority: u8, // Higher = more important.
    pub task: ScheduledTask,
}

impl PartialEq for PrioritizedTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
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
        self.priority.cmp(&other.priority)
    }
}

/// Abstracts iOS BGProcessingTask / Android WorkManager scheduling.
///
/// Manages a priority queue of background tasks and respects OS-granted
/// time budgets for background execution.
#[derive(Debug)]
pub struct BackgroundScheduler {
    pub scheduled_tasks: Vec<ScheduledTask>,
    pub next_window: Option<DateTime<Utc>>,
    pub os_budget_remaining: Duration,
    pub priority_queue: BinaryHeap<PrioritizedTask>,
}

impl BackgroundScheduler {
    /// Create a new scheduler with the given OS time budget.
    pub fn new(os_budget: Duration) -> Self {
        Self {
            scheduled_tasks: Vec::new(),
            next_window: None,
            os_budget_remaining: os_budget,
            priority_queue: BinaryHeap::new(),
        }
    }

    /// Schedule a task with a given priority (higher = more important).
    pub fn schedule(&mut self, name: &str, priority: u8, max_duration: Duration) -> Uuid {
        let task = ScheduledTask {
            id: Uuid::new_v4(),
            name: name.to_string(),
            status: TaskStatus::Pending,
            scheduled_at: Utc::now(),
            started_at: None,
            completed_at: None,
            max_duration,
        };
        let id = task.id;
        self.priority_queue.push(PrioritizedTask {
            priority,
            task: task.clone(),
        });
        self.scheduled_tasks.push(task);
        tracing::debug!(task_id = %id, name, priority, "task scheduled");
        id
    }

    /// Pop the highest-priority task that fits within the remaining budget.
    pub fn next_task(&mut self) -> Option<PrioritizedTask> {
        // Peek to check if the top task fits the budget.
        if let Some(top) = self.priority_queue.peek() {
            if top.task.max_duration <= self.os_budget_remaining {
                let task = self.priority_queue.pop().unwrap();
                self.os_budget_remaining = self
                    .os_budget_remaining
                    .saturating_sub(task.task.max_duration);
                return Some(task);
            }
        }
        None
    }

    /// Number of pending tasks.
    pub fn pending_count(&self) -> usize {
        self.priority_queue.len()
    }

    /// Mark a task as completed by id.
    pub fn complete_task(&mut self, task_id: Uuid) {
        if let Some(task) = self.scheduled_tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Completed;
            task.completed_at = Some(Utc::now());
        }
    }

    /// Cancel a task by id.
    pub fn cancel_task(&mut self, task_id: Uuid) -> bool {
        if let Some(task) = self.scheduled_tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Cancelled;
            return true;
        }
        false
    }

    /// Reset the OS budget (called when a new background window opens).
    pub fn reset_budget(&mut self, budget: Duration) {
        self.os_budget_remaining = budget;
        self.next_window = Some(Utc::now());
        tracing::debug!(budget_ms = budget.as_millis(), "background budget reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_and_pop() {
        let mut sched = BackgroundScheduler::new(Duration::from_secs(60));
        sched.schedule("low", 1, Duration::from_secs(10));
        sched.schedule("high", 10, Duration::from_secs(10));

        let task = sched.next_task().unwrap();
        assert_eq!(task.task.name, "high");
        assert_eq!(task.priority, 10);
    }

    #[test]
    fn test_budget_enforcement() {
        let mut sched = BackgroundScheduler::new(Duration::from_secs(5));
        sched.schedule("big", 10, Duration::from_secs(30));

        // Task doesn't fit the budget.
        assert!(sched.next_task().is_none());
    }

    #[test]
    fn test_complete_task() {
        let mut sched = BackgroundScheduler::new(Duration::from_secs(60));
        let id = sched.schedule("task", 5, Duration::from_secs(10));
        sched.complete_task(id);

        let completed = sched.scheduled_tasks.iter().find(|t| t.id == id).unwrap();
        assert_eq!(completed.status, TaskStatus::Completed);
        assert!(completed.completed_at.is_some());
    }
}
