//! Stage 2 runtime-adapter boundary. See `docs/STAGE2-RUNTIME-ADAPTER-PLAN.md`
//! for the audits this crate executes and the increment this file belongs to.

mod adapter;
mod events;
mod lifecycle;
#[cfg(any(test, feature = "mock"))]
mod mock;
#[cfg(feature = "opencode")]
mod opencode_adapter;
mod types;

pub use adapter::{AdapterError, AgentRuntimeAdapter};
pub use events::{Event, OutputKind, ToolStatus};
pub use lifecycle::{HandleRegistry, LifecycleState, TransitionError};
#[cfg(any(test, feature = "mock"))]
pub use mock::MockAdapter;
#[cfg(feature = "opencode")]
pub use opencode_adapter::OpenCodeAdapter;
pub use types::{Capabilities, Identity, RuntimeHandle, Status, TaskId, Workspace};
