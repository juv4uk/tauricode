//! Stage 2 runtime-adapter boundary. See `docs/STAGE2-RUNTIME-ADAPTER-PLAN.md`
//! for the audits this crate executes and the increment this file belongs to.

mod adapter;
mod events;
mod lifecycle;
#[cfg(any(test, feature = "mock"))]
mod mock;
#[cfg(feature = "opencode")]
mod opencode_adapter;
#[cfg(feature = "opencode")]
mod opencode_log_normalizer;
mod types;

pub use adapter::{AdapterError, AgentRuntimeAdapter};
pub use events::{Event, OutputKind, ToolStatus};
pub use lifecycle::{HandleRegistry, LifecycleState, TransitionError};
#[cfg(any(test, feature = "mock"))]
pub use mock::MockAdapter;
#[cfg(feature = "opencode")]
pub use opencode_adapter::OpenCodeAdapter;
#[cfg(feature = "opencode")]
pub use opencode_log_normalizer::normalize_opencode_log_line;
pub use types::{Capabilities, Identity, RuntimeHandle, Status, TaskId, Workspace};
