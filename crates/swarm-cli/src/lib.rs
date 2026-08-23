//! `swarm-cli` library — Level 0 agent adapters over the swarm coordination
//! plane (TAURICODE-SWARM-CLI-ADAPTERS).
//!
//! Read-mostly by design: check/explain/nba/fmt/convert observe the mesh;
//! the task-op passthrough transports claim/release/complete/define with
//! full response read-back while the server-side quorum keeps guarding
//! authority (M1.1). Nothing here invents state or silently retries.

pub mod ops;
pub mod out;
pub mod sexpr;
pub mod tasks_file;
pub mod wire;
