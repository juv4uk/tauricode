//! Stage 2 runtime-adapter boundary. See `docs/STAGE2-RUNTIME-ADAPTER-PLAN.md`
//! for the audits this crate executes and the increment this file belongs to.

mod adapter;
mod events;
mod lifecycle;
#[cfg(any(test, feature = "mock"))]
mod mock;
mod types;

pub use adapter::{AdapterError, AgentRuntimeAdapter};
pub use events::{Event, OutputKind, ToolStatus};
pub use lifecycle::{HandleRegistry, LifecycleState, TransitionError};
#[cfg(any(test, feature = "mock"))]
pub use mock::MockAdapter;
pub use types::{Capabilities, Identity, RuntimeHandle, Status, TaskId, Workspace};
