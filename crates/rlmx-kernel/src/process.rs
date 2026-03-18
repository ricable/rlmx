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

    /// Kill a process by setting its status to Failed.
    pub fn kill(&mut self, process_id: ProcessId) -> KernelResult<()> {
        let process = self
            .processes
            .get_mut(&process_id)
            .ok_or(KernelError::ProcessNotFound(process_id))?;

        process.status = ProcessStatus::Failed("killed".into());
        self.senders.remove(&process_id);
        Ok(())
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

        let msg = KernelMessage::new(
            Uuid::new_v4(),
            pid,
            serde_json::json!({"hello": "world"}),
        );
        mgr.send(pid, msg.clone()).await.unwrap();

        let received = mgr
            .recv(pid, Some(std::time::Duration::from_secs(1)))
            .await
            .unwrap();

        assert!(received.is_some());
        assert_eq!(received.unwrap().payload, serde_json::json!({"hello": "world"}));
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
}
