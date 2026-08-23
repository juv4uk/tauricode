//! `ecosystem-scheduler` — read-only cross-repo task aggregation and
//! routing-plan computation (TAURICODE-SCHEDULER-01).
//!
//! Reads each repo's durable `tasks.my`, projects them into one global graph
//! (M1.1: sources stay the authority; this is a materialized view), computes
//! readiness/scoring with the same convention as swarm `next-best-action`,
//! and emits a per-agent routing plan as JSON or text.
//!
//! Deliberately OUT of scope: claiming tasks in the mesh, writing to any
//! repo or to the swarm journal. Dispatch *execution* stays with agents and
//! swarm-node's quorum-guarded claims; this tool produces the routing plan
//! only. Merging it into the live claim path would duplicate authority that
//! M1.1 assigns elsewhere.

use ecosystem_scheduler::{graph, out, route, tasks_file, SCHEDULER_VERSION};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Usage(msg)) => {
            eprintln!("{msg}\n\n{}", USAGE.trim());
            ExitCode::from(2)
        }
        Err(CliError::Source(msg)) => {
            eprintln!("error: {msg}");
            ExitCode::from(3)
        }
    }
}

enum CliError {
    Usage(String),
    Source(String),
}

const USAGE: &str = r#"ecosystem-scheduler — cross-repo task routing plan (read-only)

USAGE:
  ecosystem-scheduler --github-root <DIR> [options]
  ecosystem-scheduler --repo <DIR> [--repo <DIR> ...] [options]

OPTIONS:
  --github-root <DIR>   Scan DIR's immediate subdirectories for tasks.my.
  --repo <DIR>          Explicit repo root containing tasks.my (repeatable).
  --agent <ID>=<c1,c2>  Declare an agent's capabilities for per-agent routing
                        (repeatable). Without it: one "_any" ranked plan.
  --format <text|json>  Output format (default: text).
  --origin <NAME>       Only include tasks whose origin == NAME.

SCOPE:
  Read-only. Produces a routing PLAN; claims/dispatch execution remain
  with agents via swarm-node quorum-guarded claim-task."#;

fn run(args: Vec<String>) -> Result<(), CliError> {
    let mut github_root: Option<String> = None;
    let mut repos: Vec<String> = Vec::new();
    let mut agents: Vec<(String, Vec<String>)> = Vec::new();
    let mut format = String::from("text");
    let mut origin_filter: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!("{}", USAGE.trim());
                return Ok(());
            }
            "--version" => {
                println!("ecosystem-scheduler {SCHEDULER_VERSION}");
                return Ok(());
            }
            "--github-root" => {
                i += 1;
                github_root = Some(args.get(i).ok_or_else(|| CliError::Usage("--github-root needs a value".into()))?.clone());
            }
            "--repo" => {
                i += 1;
                repos.push(args.get(i).ok_or_else(|| CliError::Usage("--repo needs a value".into()))?.clone());
            }
            "--agent" => {
                i += 1;
                let spec = args.get(i).ok_or_else(|| CliError::Usage("--agent needs ID=caps".into()))?;
                let (id, caps) = spec.split_once('=').ok_or_else(|| {
                    CliError::Usage(format!("--agent must be ID=c1,c2 (got `{spec}`)"))
                })?;
                if id.is_empty() {
                    return Err(CliError::Usage("--agent id must not be empty".into()));
                }
                agents.push((
                    id.to_string(),
                    caps.split(',').filter(|c| !c.is_empty()).map(|c| c.to_string()).collect(),
                ));
            }
            "--format" => {
                i += 1;
                format = args.get(i).ok_or_else(|| CliError::Usage("--format needs a value".into()))?.clone();
                if format != "text" && format != "json" {
                    return Err(CliError::Usage(format!("--format must be text|json (got `{format}`)")));
                }
            }
            "--origin" => {
                i += 1;
                origin_filter = Some(args.get(i).ok_or_else(|| CliError::Usage("--origin needs a value".into()))?.clone());
            }
            other => return Err(CliError::Usage(format!("unknown argument `{other}`"))),
        }
        i += 1;
    }

    // Resolve source repos: explicit --repo entries + subdirs of --github-root.
    let mut roots: Vec<(String, PathBuf)> = Vec::new();
    for r in &repos {
        let p = PathBuf::from(r);
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or(r).to_string();
        roots.push((name, p));
    }
    if let Some(root) = github_root {
        let rd = std::fs::read_dir(root.clone())
            .map_err(|e| CliError::Source(format!("cannot read --github-root `{root}`: {e}")))?;
        let mut found: Vec<(String, PathBuf)> = Vec::new();
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() && p.join("tasks.my").is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                found.push((name, p));
            }
        }
        found.sort();
        roots.extend(found);
    }
    if roots.is_empty() {
        return Err(CliError::Usage("no repos given: use --github-root and/or --repo".into()));
    }

    let mut sources: Vec<(String, String, usize)> = Vec::new();
    let mut repo_tasks: Vec<graph::RepoTasks> = Vec::new();
    for (name, path) in &roots {
        let file = path.join("tasks.my");
        let text = std::fs::read_to_string(&file)
            .map_err(|e| CliError::Source(format!("cannot read {}: {e}", file.display())))?;
        let parsed = tasks_file::parse_tasks_file(&text)
            .map_err(|e| CliError::Source(format!("parse error in {}: {e}", file.display())))?;
        sources.push((name.clone(), file.display().to_string(), parsed.len()));
        repo_tasks.push(graph::RepoTasks { repo: name.clone(), tasks: parsed });
    }

    let g = graph::GlobalGraph::build(repo_tasks);
    let states = g.state_map();

    // Blocked detail is derived from the final states; Cycle labels are exact
    // after the fix point because WaitingOn leftovers were re-derived until stable.
    let blocked_detail: Vec<(String, String, graph::BlockReason)> = g
        .nodes
        .values()
        .filter_map(|n| match states.get(&n.id) {
            Some(graph::TaskState::Blocked(reason)) => Some((n.id.clone(), n.origin.clone(), reason.clone())),
            _ => None,
        })
        .collect();

    let routes_all = route::plan(&g, &states, &agents);
    let routes: Vec<_> = match &origin_filter {
        None => routes_all,
        Some(o) => routes_all
            .into_iter()
            .map(|mut ar| {
                ar.ranked.retain(|e| &e.origin == o);
                ar
            })
            .collect(),
    };

    let counts = g.counts(&states);
    let plan_out = out::PlanOut {
        sources: &sources,
        warnings: &g.warnings,
        counts,
        routes: &routes,
        blocked_detail,
    };

    match format.as_str() {
        "json" => println!("{}", out::to_json(&plan_out)),
        _ => print!("{}", out::to_text(&plan_out)),
    }
    Ok(())
}
