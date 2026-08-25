//! M2.2 — normalized event types.
//!
//! Exactly the 8-event vocabulary from the 2026-08-25 event-contract audit
//! (`docs/STAGE2-RUNTIME-ADAPTER-PLAN.md`). `session_started/resumed`,
//! `input_received`, a separate `tool_failed`, `capability_changed`, and
//! `heartbeat`-as-a-domain-event are deliberately absent — the audit found
//! no proven core need for any of them, and adding them "just in case"
//! is exactly the premature abstraction the plan forbids.
//!
//! `output_chunk` carries a `kind` tag rather than being one untyped blob
//! — the audit's specific finding: OpenCode's own protocol already
//! distinguishes `Part` types (`specs/project.md`), and a single agent's
//! own output is not homogeneous (text vs tool-call vs reasoning are
//! structurally different, not the same thing under different names).

use crate::types::{RuntimeHandle, TaskId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputKind {
    Text,
    Reasoning,
    ToolCall,
    ToolResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolStatus {
    Ok,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Event {
    RuntimeStarted {
        handle: RuntimeHandle,
        pid: Option<u32>,
        cwd: Option<String>,
    },
    RuntimeStopped {
        handle: RuntimeHandle,
        exit_code: Option<i32>,
        reason: Option<String>,
    },
    TaskBound {
        handle: RuntimeHandle,
        task_id: TaskId,
        origin: Option<String>,
    },
    OutputChunk {
        handle: RuntimeHandle,
        task_id: Option<TaskId>,
        output_kind: OutputKind,
        content: String,
    },
    ToolStarted {
        handle: RuntimeHandle,
        task_id: Option<TaskId>,
        /// Opaque — the adapter-contract audit's explicit finding: tool
        /// names are runtime-specific vocabulary, never enumerated here.
        tool_name: String,
    },
    ToolCompleted {
        handle: RuntimeHandle,
        task_id: Option<TaskId>,
        tool_name: String,
        status: ToolStatus,
        /// No `tool_failed` variant — failure is `status: Failed` here,
        /// per the audit's explicit anti-proliferation finding.
        result_ref: Option<String>,
    },
    StatusChanged {
        handle: RuntimeHandle,
        status: crate::types::Status,
    },
    Error {
        handle: RuntimeHandle,
        message: String,
        retryable: Option<bool>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Status;

    fn sample_events() -> Vec<Event> {
        let handle = RuntimeHandle::new("h1");
        let task = TaskId::new("MYLISP-TASK-1");
        vec![
            Event::RuntimeStarted {
                handle: handle.clone(),
                pid: Some(1234),
                cwd: Some("/home/agents/GitHub/tauricode".to_string()),
            },
            Event::TaskBound {
                handle: handle.clone(),
                task_id: task.clone(),
                origin: Some("swarm-cli".to_string()),
            },
            Event::OutputChunk {
                handle: handle.clone(),
                task_id: Some(task.clone()),
                output_kind: OutputKind::Reasoning,
                content: "thinking...".to_string(),
            },
            Event::ToolStarted {
                handle: handle.clone(),
                task_id: Some(task.clone()),
                tool_name: "bash".to_string(),
            },
            Event::ToolCompleted {
                handle: handle.clone(),
                task_id: Some(task.clone()),
                tool_name: "bash".to_string(),
                status: ToolStatus::Ok,
                result_ref: Some("evidence/run-1.log".to_string()),
            },
            Event::StatusChanged {
                handle: handle.clone(),
                status: Status::Fresh,
            },
            Event::Error {
                handle: handle.clone(),
                message: "stream error".to_string(),
                retryable: Some(true),
            },
            Event::RuntimeStopped {
                handle,
                exit_code: Some(0),
                reason: None,
            },
        ]
    }

    #[test]
    fn every_variant_round_trips_through_json() {
        for event in sample_events() {
            let json = serde_json::to_string(&event).unwrap();
            let back: Event = serde_json::from_str(&json).unwrap();
            assert_eq!(event, back);
        }
    }

    /// Exhaustive match with no wildcard arm: if a 9th variant is ever
    /// added, this function fails to compile until whoever added it
    /// comes here and explains why — forcing the "don't make the event
    /// contract richer than core needs" decision to be visible, not
    /// silent.
    #[test]
    fn vocabulary_is_exactly_eight_variants() {
        fn assert_exhaustive(e: &Event) {
            match e {
                Event::RuntimeStarted { .. }
                | Event::RuntimeStopped { .. }
                | Event::TaskBound { .. }
                | Event::OutputChunk { .. }
                | Event::ToolStarted { .. }
                | Event::ToolCompleted { .. }
                | Event::StatusChanged { .. }
                | Event::Error { .. } => {}
            }
        }
        for event in sample_events() {
            assert_exhaustive(&event);
        }
    }
}
