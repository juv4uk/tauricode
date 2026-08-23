//! `ecosystem-scheduler` library — read-only cross-repo task aggregation
//! and routing-plan computation for the my-lisp ecosystem.
//!
//! Authority boundary (M1.1): each repo's own `tasks.my` owns its tasks;
//! this crate only *projects* them into one inspectable global view and
//! computes a deterministic routing plan. It never claims tasks, never
//! writes to repositories or the swarm journal — claim execution stays
//! with agents via swarm-node's quorum-guarded `(claim-task ...)`.
//!
//! Provenance: the `.my` reader is ported from
//! `my-lisp/crates/swarm-node/src/{sexpr,tasks_file}.rs` (M1.1 era) so both
//! sides read the same format the same way — a shared-format implementation
//! pair, not an independent format definition. Scoring mirrors the
//! ecosystem's `next-best-action` convention deliberately (shared
//! convention witness).

pub mod graph;
pub mod out;
pub mod route;
pub mod sexpr;
pub mod tasks_file;

pub const SCHEDULER_VERSION: &str = "0.1.0";
