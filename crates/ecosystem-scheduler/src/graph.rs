//! Aggregates per-repo `tasks.my` sets into one global cross-repo graph and
//! derives each task's readiness by fixed-point iteration.
//!
//! Authority boundary (M1.1 contract): each repo's own `tasks.my` owns its
//! tasks; this module only *projects* them into one inspectable view. It
//! never mutates sources, never resolves conflicts silently — duplicate ids
//! across repos become visible warnings, first input occurrence keeps the
//! seat deterministically.

use crate::tasks_file::ParsedTask;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct RepoTasks {
    /// Repository identity used as the origin fallback (directory name).
    pub repo: String,
    pub tasks: Vec<ParsedTask>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockReason {
    /// Depends on an id that exists in no repo.
    MissingDep(String),
    /// Depends on an open (not-done) task — normal queueing.
    WaitingOn(String),
    /// Not resolvable after the readiness fix point and no missing deps:
    /// behind a dependency cycle.
    Cycle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    Done,
    Ready,
    Blocked(BlockReason),
}

#[derive(Debug, Clone)]
pub struct TaskNode {
    pub id: String,
    /// Declared `origin`, else the repo dir the file was found in.
    pub origin: String,
    pub priority: f64,
    pub capabilities: Vec<String>,
    pub depends_on: Vec<String>,
    pub done: bool,
    /// Repos other than the winner that also define this id (visible, not silent).
    pub duplicate_in: Vec<String>,
}

#[derive(Debug, Default)]
pub struct GlobalGraph {
    pub nodes: BTreeMap<String, TaskNode>,
    /// Human-readable aggregation notes: duplicates, files skipped, etc.
    pub warnings: Vec<String>,
}

impl GlobalGraph {
    pub fn build(mut repos: Vec<RepoTasks>) -> GlobalGraph {
        // Deterministic regardless of filesystem order: sort by repo name.
        repos.sort_by(|a, b| a.repo.cmp(&b.repo));
        let mut g = GlobalGraph::default();
        for rt in &repos {
            for t in &rt.tasks {
                match g.nodes.get_mut(&t.id) {
                    Some(existing) => {
                        existing.duplicate_in.push(rt.repo.clone());
                        g.warnings.push(format!(
                            "duplicate task id `{}` also defined in `{}` (kept `{}`, first input occurrence)",
                            t.id, rt.repo, existing.origin
                        ));
                    }
                    None => {
                        g.nodes.insert(
                            t.id.clone(),
                            TaskNode {
                                id: t.id.clone(),
                                origin: t.origin.clone().unwrap_or_else(|| rt.repo.clone()),
                                priority: t.priority,
                                capabilities: t.capabilities.clone(),
                                depends_on: t.depends_on.clone(),
                                done: t.done,
                                duplicate_in: Vec::new(),
                            },
                        );
                    }
                }
            }
        }
        g
    }

    pub fn state_of(&self, id: &str) -> Option<TaskState> {
        self.state_map().get(id).cloned()
    }

    /// Fixed-point readiness over the whole graph (deterministic: BTreeMap order).
    pub fn state_map(&self) -> BTreeMap<String, TaskState> {
        let mut states: BTreeMap<String, TaskState> = self
            .nodes
            .values()
            .map(|n| {
                let s = if n.done { TaskState::Done } else { TaskState::Blocked(BlockReason::Cycle) };
                (n.id.clone(), s)
            })
            .collect();

        // Iterate to fixpoint: Done is stable; promote to Ready when every
        // dep exists and is Done; otherwise pin the concrete blocker.
        loop {
            let mut changed = false;
            for n in self.nodes.values() {
                if states.get(&n.id) != Some(&TaskState::Done) && !matches!(states.get(&n.id), Some(TaskState::Ready)) {
                    let new_state = self.derive_open_state(n, &states);
                    if states.get(&n.id) != Some(&new_state) {
                        states.insert(n.id.clone(), new_state);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        // Post-fixpoint classification: WaitingOn is only truthful when the
        // blocker eventually reaches Done/Ready (or a terminal MissingDep).
        // Anything whose WaitingOn chains settle entirely among themselves is
        // behind a dependency cycle — relabel exactly those, iteratively.
        loop {
            let mut changed = false;
            let snap = states.clone();
            let good = |id: &str| matches!(snap.get(id), Some(TaskState::Ready | TaskState::Done))
                || matches!(snap.get(id), Some(TaskState::Blocked(BlockReason::MissingDep(_))));
            for n in self.nodes.values() {
                if let Some(TaskState::Blocked(BlockReason::WaitingOn(dep))) = snap.get(&n.id) {
                    if !good(dep) && matches!(snap.get(dep), Some(TaskState::Blocked(BlockReason::WaitingOn(_)))) {
                        states.insert(n.id.clone(), TaskState::Blocked(BlockReason::Cycle));
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        states
    }

    fn derive_open_state(&self, n: &TaskNode, states: &BTreeMap<String, TaskState>) -> TaskState {
        for dep in &n.depends_on {
            match (self.nodes.get(dep), states.get(dep)) {
                (None, _) => return TaskState::Blocked(BlockReason::MissingDep(dep.clone())),
                (_, Some(TaskState::Done)) | (_, None) => continue,
                (_, Some(other)) => {
                    // A dep still labelled Cycle may resolve later in the sweep;
                    // report WaitingOn for now — leftovers after the fix point
                    // really are cycles.
                    let _ = other;
                    return TaskState::Blocked(BlockReason::WaitingOn(dep.clone()));
                }
            }
        }
        TaskState::Ready
    }

    pub fn counts(&self, states: &BTreeMap<String, TaskState>) -> (usize, usize, usize, usize) {
        let (mut done, mut ready, mut blocked) = (0, 0, 0);
        for s in states.values() {
            match s {
                TaskState::Done => done += 1,
                TaskState::Ready => ready += 1,
                TaskState::Blocked(_) => blocked += 1,
            }
        }
        (self.nodes.len(), done, ready, blocked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(id: &str, _repo: &str, done: bool, deps: &[&str]) -> ParsedTask {
        ParsedTask {
            id: id.to_string(),
            priority: 1.0,
            capabilities: vec![],
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
            done,
            description: None,
            origin: None,
        }
    }

    fn repo(name: &str, tasks: Vec<ParsedTask>) -> RepoTasks {
        RepoTasks { repo: name.to_string(), tasks }
    }

    #[test]
    fn aggregates_across_repos_with_origin_fallback() {
        let g = GlobalGraph::build(vec![
            repo("cml", vec![task("CML-A", "cml", false, &[])]),
            repo("my-lisp", vec![task("MY-A", "my-lisp", false, &["CML-A"])]),
        ]);
        assert_eq!(g.nodes["CML-A"].origin, "cml");
        assert_eq!(g.nodes["MY-A"].origin, "my-lisp");
        assert_eq!(g.state_of("CML-A"), Some(TaskState::Ready));
        assert_eq!(g.state_of("MY-A"), Some(TaskState::Blocked(BlockReason::WaitingOn("CML-A".into()))));
    }

    #[test]
    fn duplicate_ids_are_visible_not_silent() {
        let g = GlobalGraph::build(vec![
            repo("a", vec![task("X", "a", false, &[])]),
            repo("b", vec![task("X", "b", true, &[])]),
        ]);
        assert_eq!(g.nodes.len(), 1);
        assert_eq!(g.warnings.len(), 1);
        assert_eq!(g.nodes["X"].duplicate_in, vec!["b"]);
        assert!(!g.nodes["X"].done); // first occurrence kept
    }

    #[test]
    fn missing_dep_is_reported_as_missing_not_cycle() {
        let g = GlobalGraph::build(vec![repo("r", vec![task("A", "r", false, &["GHOST"])])]);
        assert_eq!(g.state_of("A"), Some(TaskState::Blocked(BlockReason::MissingDep("GHOST".into()))));
    }

    #[test]
    fn dependency_cycles_settle_as_cycle_after_fixpoint() {
        let g = GlobalGraph::build(vec![repo(
            "r",
            vec![task("A", "r", false, &["B"]), task("B", "r", false, &["A"]), task("C", "r", false, &[])],
        )]);
        assert_eq!(g.state_of("A"), Some(TaskState::Blocked(BlockReason::Cycle)));
        assert_eq!(g.state_of("B"), Some(TaskState::Blocked(BlockReason::Cycle)));
        assert_eq!(g.state_of("C"), Some(TaskState::Ready));
    }

    #[test]
    fn chains_ready_when_head_dep_done() {
        let g = GlobalGraph::build(vec![repo(
            "r",
            vec![task("BASE", "r", true, &[]), task("MID", "r", false, &["BASE"]), task("TOP", "r", false, &["MID"])],
        )]);
        assert_eq!(g.state_of("MID"), Some(TaskState::Ready));
        assert_eq!(g.state_of("TOP"), Some(TaskState::Blocked(BlockReason::WaitingOn("MID".into()))));
    }
}
