use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::capability::CapabilityToken;
use crate::types::{KernelError, KernelMessage, KernelResult, ProcessId};

/// Status of a kernel process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Waiting,
    Completed,
    Failed(String),
}

/// A kernel process with its own capability token and message channel.
pub struct Process {
    pub id: ProcessId,
    pub parent: Option<ProcessId>,
    pub capability_token: CapabilityToken,
    pub memory_scope: String,
    pub task: String,
    pub status: ProcessStatus,
    /// Optional agent type label (e.g. "coder", "reviewer", "swarm-worker").
    pub agent_type: Option<String>,
    #[allow(dead_code)]
    tx: mpsc::Sender<KernelMessage>,
    rx: Option<mpsc::Receiver<KernelMessage>>,
}

impl std::fmt::Debug for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("id", &self.id)
            .field("parent", &self.parent)
            .field("memory_scope", &self.memory_scope)
            .field("task", &self.task)
            .field("status", &self.status)
            .field("agent_type", &self.agent_type)
            .finish()
    }
}

/// Manages kernel processes and inter-process message passing.
pub struct ProcessManager {
    processes: HashMap<ProcessId, Process>,
    /// Senders indexed by target process id for message delivery.
    senders: HashMap<ProcessId, mpsc::Sender<KernelMessage>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            senders: HashMap::new(),
        }
    }

    /// Fork (create) a new process with the given capability token.
    pub fn fork(
        &mut self,
        parent: Option<ProcessId>,
        capability_token: CapabilityToken,
        memory_scope: String,
        task: String,
    ) -> ProcessId {
        let id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel(256);

        self.senders.insert(id, tx.clone());

        let process = Process {
            id,
            parent,
            capability_token,
            memory_scope,
            task,
            status: ProcessStatus::Running,
            agent_type: None,
            tx,
            rx: Some(rx),
        };
        self.processes.insert(id, process);
        id
    }

    /// Send a message to the target process.
    pub async fn send(&self, target: ProcessId, message: KernelMessage) -> KernelResult<()> {
        let sender = self
            .senders
            .get(&target)
            .ok_or(KernelError::ProcessNotFound(target))?;

        sender
            .send(message)
            .await
            .map_err(|e| KernelError::ChannelError(e.to_string()))?;

        Ok(())
    }

    /// Receive the next message for a process, with optional timeout.
    pub async fn recv(
        &mut self,
        process_id: ProcessId,
        timeout: Option<std::time::Duration>,
    ) -> KernelResult<Option<KernelMessage>> {
        let process = self
            .processes
            .get_mut(&process_id)
            .ok_or(KernelError::ProcessNotFound(process_id))?;

        let rx = process
            .rx
            .as_mut()
            .ok_or_else(|| KernelError::ChannelError("receiver already taken".into()))?;

        match timeout {
            Some(dur) => match tokio::time::timeout(dur, rx.recv()).await {
                Ok(Some(msg)) => Ok(Some(msg)),
                Ok(None) => Ok(None),
                Err(_) => Ok(None),
            },
            None => Ok(rx.recv().await),
        }
    }

    /// Kill a process and remove it from the process map.
    pub fn kill(&mut self, process_id: ProcessId) -> KernelResult<()> {
        self.processes
            .remove(&process_id)
            .ok_or(KernelError::ProcessNotFound(process_id))?;
        self.senders.remove(&process_id);
        Ok(())
    }

    /// Reap all completed or killed (Failed) processes, removing them from the
    /// process map and freeing associated resources. Returns the number of
    /// reaped processes.
    pub fn reap(&mut self) -> usize {
        let to_reap: Vec<ProcessId> = self
            .processes
            .iter()
            .filter(|(_, p)| {
                matches!(
                    p.status,
                    ProcessStatus::Completed | ProcessStatus::Failed(_)
                )
            })
            .map(|(id, _)| *id)
            .collect();
        let count = to_reap.len();
        for id in to_reap {
            self.processes.remove(&id);
            self.senders.remove(&id);
        }
        count
    }

    /// Mark a process as completed.
    pub fn complete(&mut self, process_id: ProcessId) -> KernelResult<()> {
        let process = self
            .processes
            .get_mut(&process_id)
            .ok_or(KernelError::ProcessNotFound(process_id))?;
        process.status = ProcessStatus::Completed;
        Ok(())
    }

    /// List all process ids and their statuses.
    pub fn list(&self) -> Vec<(ProcessId, &ProcessStatus)> {
        self.processes
            .iter()
            .map(|(id, p)| (*id, &p.status))
            .collect()
    }

    /// Get a reference to a process.
    pub fn get(&self, process_id: &ProcessId) -> Option<&Process> {
        self.processes.get(process_id)
    }

    /// Spawn a new agent process with a specific agent type.
    ///
    /// If `parent_id` is provided and the parent exists, the new agent inherits
    /// the parent's capability token (narrowed to the given capabilities).
    /// Otherwise a default token with the supplied capabilities is created.
    pub fn spawn_agent(
        &mut self,
        agent_type: &str,
        parent_id: Option<Uuid>,
        capabilities: Vec<String>,
    ) -> Result<Uuid, String> {
        // Resolve capability token — inherit from parent when available.
        let cap_token = if let Some(pid) = parent_id {
            let parent = self
                .processes
                .get(&pid)
                .ok_or_else(|| format!("parent process {pid} not found"))?;
            // Derive a child token from the parent's token.
            let mut child_token = parent.capability_token.clone();
            child_token.id = Uuid::new_v4();
            child_token.scope = capabilities.join(",");
            child_token
        } else {
            use crate::types::SyscallPermission;
            use chrono::{Duration, Utc};
            CapabilityToken {
                id: Uuid::new_v4(),
                owner: Uuid::new_v4(),
                granted_syscalls: vec![SyscallPermission::All],
                scope: capabilities.join(","),
                expiry: Utc::now() + Duration::hours(1),
                issuer_signature: "agent-spawn".into(),
            }
        };

        let memory_scope = format!("agent-{agent_type}");
        let task = format!("agent:{agent_type}");

        let id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel(256);
        self.senders.insert(id, tx.clone());

        let process = Process {
            id,
            parent: parent_id,
            capability_token: cap_token,
            memory_scope,
            task,
            status: ProcessStatus::Running,
            agent_type: Some(agent_type.to_string()),
            tx,
            rx: Some(rx),
        };
        self.processes.insert(id, process);
        Ok(id)
    }

    /// List all processes that have an `agent_type` set.
    pub fn list_agents(&self) -> Vec<&Process> {
        self.processes
            .values()
            .filter(|p| p.agent_type.is_some())
            .collect()
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SyscallPermission;
    use chrono::{Duration, Utc};

    fn test_token() -> CapabilityToken {
        CapabilityToken {
            id: Uuid::new_v4(),
            owner: Uuid::new_v4(),
            granted_syscalls: vec![SyscallPermission::All],
            scope: "test".into(),
            expiry: Utc::now() + Duration::hours(1),
            issuer_signature: "test-sig".into(),
        }
    }

    #[test]
    fn test_fork_creates_process() {
        let mut mgr = ProcessManager::new();
        let pid = mgr.fork(None, test_token(), "global".into(), "test-task".into());
        let proc = mgr.get(&pid).unwrap();
        assert_eq!(proc.status, ProcessStatus::Running);
        assert_eq!(proc.task, "test-task");
        assert!(proc.parent.is_none());
    }

    #[tokio::test]
    async fn test_send_and_recv() {
        let mut mgr = ProcessManager::new();
        let pid = mgr.fork(None, test_token(), "global".into(), "task".into());

        let msg = KernelMessage::new(Uuid::new_v4(), pid, serde_json::json!({"hello": "world"}));
        mgr.send(pid, msg.clone()).await.unwrap();

        let received = mgr
            .recv(pid, Some(std::time::Duration::from_secs(1)))
            .await
            .unwrap();

        assert!(received.is_some());
        assert_eq!(
            received.unwrap().payload,
            serde_json::json!({"hello": "world"})
        );
    }

    #[tokio::test]
    async fn test_recv_timeout_returns_none() {
        let mut mgr = ProcessManager::new();
        let pid = mgr.fork(None, test_token(), "global".into(), "task".into());

        let received = mgr
            .recv(pid, Some(std::time::Duration::from_millis(50)))
            .await
            .unwrap();

        assert!(received.is_none());
    }

    #[test]
    fn test_spawn_agent_no_parent() {
        let mut mgr = ProcessManager::new();
        let pid = mgr
            .spawn_agent("coder", None, vec!["read".into(), "write".into()])
            .unwrap();
        let proc = mgr.get(&pid).unwrap();
        assert_eq!(proc.agent_type.as_deref(), Some("coder"));
        assert_eq!(proc.status, ProcessStatus::Running);
        assert!(proc.parent.is_none());
        assert_eq!(proc.task, "agent:coder");
    }

    #[test]
    fn test_spawn_agent_with_parent() {
        let mut mgr = ProcessManager::new();
        let parent_pid = mgr.fork(None, test_token(), "global".into(), "parent".into());
        let child_pid = mgr
            .spawn_agent("reviewer", Some(parent_pid), vec!["review".into()])
            .unwrap();
        let child = mgr.get(&child_pid).unwrap();
        assert_eq!(child.parent, Some(parent_pid));
        assert_eq!(child.agent_type.as_deref(), Some("reviewer"));
    }

    #[test]
    fn test_spawn_agent_invalid_parent() {
        let mut mgr = ProcessManager::new();
        let fake_parent = Uuid::new_v4();
        let result = mgr.spawn_agent("coder", Some(fake_parent), vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_agents() {
        let mut mgr = ProcessManager::new();
        // Regular process — not an agent.
        mgr.fork(None, test_token(), "global".into(), "task".into());
        // Agent processes.
        mgr.spawn_agent("coder", None, vec![]).unwrap();
        mgr.spawn_agent("reviewer", None, vec![]).unwrap();

        let agents = mgr.list_agents();
        assert_eq!(agents.len(), 2);
        let types: Vec<&str> = agents
            .iter()
            .filter_map(|a| a.agent_type.as_deref())
            .collect();
        assert!(types.contains(&"coder"));
        assert!(types.contains(&"reviewer"));
    }

    #[test]
    fn test_fork_has_no_agent_type() {
        let mut mgr = ProcessManager::new();
        let pid = mgr.fork(None, test_token(), "global".into(), "task".into());
        let proc = mgr.get(&pid).unwrap();
        assert!(proc.agent_type.is_none());
    }
}
