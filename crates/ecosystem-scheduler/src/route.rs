//! Routing-plan computation: deterministic scoring and per-agent matching.
//!
//! Scoring relation note (C11 discipline): `score = priority ×
//! (1 + unblock_impact)` and "unblock_impact = how many open tasks list this
//! one in depends-on" deliberately MIRROR the ecosystem's existing
//! `next-best-action` convention (`:9999` / swarm-node) so agents see one
//! consistent ranking everywhere. This is a shared-convention witness, not an
//! independent invention.

use crate::graph::{GlobalGraph, TaskNode, TaskState};
use std::collections::BTreeMap;

pub fn unblock_impact(graph: &GlobalGraph, id: &str) -> usize {
    graph
        .nodes
        .values()
        .filter(|n| !n.done && n.depends_on.iter().any(|d| d == id))
        .count()
}

pub fn score(task: &TaskNode, impact: usize) -> f64 {
    task.priority * (1.0 + impact as f64)
}

#[derive(Debug, Clone)]
pub struct RouteEntry {
    pub id: String,
    pub origin: String,
    pub score: f64,
    pub priority: f64,
    pub unblock_impact: usize,
    pub capabilities: Vec<String>,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AgentRoute {
    /// Agent node identity, e.g. "ganaka-1"; "_any" when no agents given.
    pub agent: String,
    pub capabilities: Vec<String>,
    pub ranked: Vec<RouteEntry>,
}

/// A task is routable to an agent when it is Ready and either declares no
/// capabilities or shares at least one with the agent.
fn matches(agent_caps: &[String], task: &TaskNode) -> bool {
    task.capabilities.is_empty() || task.capabilities.iter().any(|c| agent_caps.contains(c))
}

pub fn plan(
    graph: &GlobalGraph,
    states: &BTreeMap<String, TaskState>,
    agents: &[(String, Vec<String>)],
) -> Vec<AgentRoute> {
    let mut routes = Vec::new();
    if agents.is_empty() {
        routes.push(collect(graph, states, "_any", &[]));
    } else {
        for (node, caps) in agents {
            routes.push(collect(graph, states, node, caps));
        }
    }
    routes
}

fn collect(graph: &GlobalGraph, states: &BTreeMap<String, TaskState>, agent: &str, caps: &[String]) -> AgentRoute {
    let mut ranked: Vec<RouteEntry> = graph
        .nodes
        .values()
        .filter(|n| states.get(&n.id) == Some(&TaskState::Ready))
        .filter(|n| matches(caps, n))
        .map(|n| {
            let impact = unblock_impact(graph, &n.id);
            RouteEntry {
                id: n.id.clone(),
                origin: n.origin.clone(),
                score: score(n, impact),
                priority: n.priority,
                unblock_impact: impact,
                capabilities: n.capabilities.clone(),
                depends_on: n.depends_on.clone(),
            }
        })
        .collect();
    // Deterministic order: score desc, then id asc for ties.
    ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal).then(a.id.cmp(&b.id)));
    AgentRoute { agent: agent.to_string(), capabilities: caps.to_vec(), ranked }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GlobalGraph;
    use crate::tasks_file::ParsedTask;

    fn task(id: &str, prio: f64, caps: &[&str], deps: &[&str], done: bool) -> ParsedTask {
        ParsedTask {
            id: id.to_string(),
            priority: prio,
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
            done,
            description: None,
            origin: None,
        }
    }

    #[test]
    fn scores_like_next_best_action_and_ranks_deterministically() {
        let g = GlobalGraph::build(vec![crate::graph::RepoTasks {
            repo: "r".into(),
            tasks: vec![
                task("HIGH-NO-FOLLOW", 9.0, &[], &[], false),
                task("MED-TWO-FOLLOWERS", 5.0, &[], &[], false),
                task("F1", 1.0, &[], &["MED-TWO-FOLLOWERS"], false),
                task("F2", 1.0, &[], &["MED-TWO-FOLLOWERS"], false),
                task("DONE", 10.0, &[], &[], true),
                task("BLOCKED", 10.0, &[], &["GHOST"], false),
            ],
        }]);
        let states = g.state_map();
        let routes = plan(&g, &states, &[]);
        assert_eq!(routes.len(), 1);
        let ids: Vec<&str> = routes[0].ranked.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["MED-TWO-FOLLOWERS", "HIGH-NO-FOLLOW"]); // 15 > 9; DONE/BLOCKED excluded
        assert_eq!(routes[0].ranked[0].unblock_impact, 2);
        assert_eq!(routes[0].ranked[0].score, 15.0);
    }

    #[test]
    fn capability_matching_includes_unclaimed_generic_tasks() {
        let g = GlobalGraph::build(vec![crate::graph::RepoTasks {
            repo: "r".into(),
            tasks: vec![
                task("GENERIC", 1.0, &[], &[], false),
                task("RUSTY", 9.0, &["rust"], &[], false),
                task("SANSKRIT", 9.0, &["sanskrit"], &[], false),
            ],
        }]);
        let states = g.state_map();
        let routes = plan(&g, &states, &[("ganaka-1".to_string(), vec!["rust".to_string(), "lisp".to_string()])]);
        let ids: Vec<&str> = routes[0].ranked.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["RUSTY", "GENERIC"]); // SANSKRIT filtered out
    }

    #[test]
    fn ties_break_by_id() {
        let g = GlobalGraph::build(vec![crate::graph::RepoTasks {
            repo: "r".into(),
            tasks: vec![task("B", 1.0, &[], &[], false), task("A", 1.0, &[], &[], false)],
        }]);
        let states = g.state_map();
        let routes = plan(&g, &states, &[]);
        let ids: Vec<&str> = routes[0].ranked.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["A", "B"]);
    }
}
