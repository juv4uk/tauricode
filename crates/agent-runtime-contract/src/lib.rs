//! Stage 2 runtime-adapter boundary. See `docs/STAGE2-RUNTIME-ADAPTER-PLAN.md`
//! for the audits this crate executes and the increment this file belongs to.

mod events;
mod lifecycle;
mod types;

pub use events::{Event, OutputKind, ToolStatus};
pub use lifecycle::{HandleRegistry, LifecycleState, TransitionError};
pub use types::{Capabilities, Identity, RuntimeHandle, Status, TaskId, Workspace};
