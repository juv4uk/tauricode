//! M2.0 — contract types only. No behavior, no I/O.
//!
//! Every shape here is either reused byte-for-byte from `ecosystem-observer`'s
//! already-proven-generic fields (per the 2026-08-25 dependency-boundary and
//! minimal-stable-core audits, `docs/STAGE2-RUNTIME-ADAPTER-PLAN.md`), or
//! newly introduced only where the audits found a genuine gap (`RuntimeHandle`
//! as an opaque wrapper; `TaskId` kept structurally distinct from any
//! runtime-internal session identifier).

use serde::{Deserialize, Serialize};

/// Opaque handle to a launched runtime instance. Deliberately carries no
/// runtime-specific fields (no port, no session id, no process handle) —
/// the adapter-contract audit's explicit finding: "runtime handle must be
/// opaque". Only the adapter that issued it knows what's inside.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeHandle(String);

impl RuntimeHandle {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Tauricode's own task identifier (`tasks.my`-scoped), structurally
/// distinct from any runtime-internal session id (e.g. OpenCode's
/// `ses_...`). The adapter-contract audit's explicit finding: task-id and
/// session-id are two different concepts and must never be merged into one
/// field — the ecosystem task stays the primary key, a runtime's own
/// session concept (if it has one) is an adapter-internal detail.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(String);

impl TaskId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Self-reported identity. Field shape copied from
/// `ecosystem-observer::snapshot::SelfReportedIdentity` rather than
/// reinvented — the minimal-stable-core audit found that shape already
/// generic (freeform, not hardcoded to any runtime; its own test fixtures
/// use "Claude Sonnet 5" as an example value with no special-casing).
/// Self-reported, therefore never authoritative — same caveat as the
/// source type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub model: Option<String>,
    pub role: Option<String>,
    pub repository: Option<String>,
    pub instance: Option<String>,
    pub task: Option<String>,
}

/// What the adapter/runtime *claims* it can do — not a grant, not a
/// verified fact. Same deliberate framing as
/// `ecosystem-observer::SelfReportedIdentity::declared_capabilities`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    pub declared: Vec<String>,
}

/// OS-level workspace association. Field shape copied from
/// `ecosystem-observer::snapshot::OsObservedFacts` (`cwd`,
/// `repo_association`) — proven generic because it's OS-level (any
/// process has a cwd), not runtime-specific.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub cwd: Option<String>,
    pub repo_association: Option<String>,
}

/// Correlation/liveness status. Deliberately the same four states as
/// `ecosystem-observer::snapshot::IdentityStatus` — reusing the proven
/// enum rather than diverging it, per the plan's explicit instruction not
/// to silently fork an already-working concept. This is a
/// freshness/liveness axis, not a trust/authenticity one (same caveat the
/// source type documents) and not the same axis as `LifecycleState`
/// (M2.1) — process lifecycle and identity freshness are kept separate on
/// purpose, matching how `ecosystem-observer` already keeps them
/// separate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Fresh,
    Stale,
    Orphaned,
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_handle_roundtrips_as_str() {
        let h = RuntimeHandle::new("abc-123");
        assert_eq!(h.as_str(), "abc-123");
    }

    #[test]
    fn task_id_roundtrips_as_str() {
        let t = TaskId::new("MYLISP-TASK-42");
        assert_eq!(t.as_str(), "MYLISP-TASK-42");
    }

    #[test]
    fn identity_default_is_all_none() {
        let id = Identity::default();
        assert_eq!(id.model, None);
        assert_eq!(id.role, None);
        assert_eq!(id.repository, None);
        assert_eq!(id.instance, None);
        assert_eq!(id.task, None);
    }

    #[test]
    fn capabilities_default_is_empty() {
        assert!(Capabilities::default().declared.is_empty());
    }

    #[test]
    fn workspace_default_is_all_none() {
        let w = Workspace::default();
        assert_eq!(w.cwd, None);
        assert_eq!(w.repo_association, None);
    }

    #[test]
    fn status_serde_roundtrip() {
        for status in [Status::Fresh, Status::Stale, Status::Orphaned, Status::NotFound] {
            let json = serde_json::to_string(&status).unwrap();
            let back: Status = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn identity_serde_roundtrip() {
        let id = Identity {
            model: Some("Claude Sonnet 5".to_string()),
            role: Some("Ecosystem Lead".to_string()),
            repository: Some("tauricode".to_string()),
            instance: None,
            task: Some("M2.0".to_string()),
        };
        let json = serde_json::to_string(&id).unwrap();
        let back: Identity = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }
}
