//! The three adapter ops. Each one is a deterministic translation between
//! the swarm/1 wire world and plain data an L0 agent can consume.

use crate::out::Out;
use crate::tasks_file;
use crate::wire::{field, Client};
use std::path::{Path, PathBuf};

pub fn check(client: &Client) -> Result<(Out, bool), String> {
    let metrics = client.call("(metrics)")?;
    let status = client.call("(status)")?;

    if metrics.head() != Some("metrics") {
        return Err(format!("unexpected response to metrics: {}", metrics.to_text()));
    }
    let get = |r: &crate::sexpr::Sexp, k: &str| field(r, k).map(|v| v.to_text());

    let synced = get(&status, "synced").as_deref() == Some("t");
    let peers: usize = get(&metrics, "peer-count").and_then(|s| s.parse().ok()).unwrap_or(0);
    let healthy = synced && peers > 0;

    // presence: (presence (node-1 ganaka-1)) -> list of names after head
    let presence: Vec<Out> = match status.field("presence") {
        Some(items) => items.iter().map(|s| Out::S(s.to_text())).collect(),
        None => vec![],
    };
    let members = status
        .field("members")
        .map(|m| m.len())
        .unwrap_or(0);

    let out = Out::m(vec![
        ("node", Out::S(get(&metrics, "node").unwrap_or_default())),
        ("epoch", Out::N(get(&metrics, "epoch").unwrap_or_else(|| "0".into()))),
        ("uptime_secs", Out::N(get(&metrics, "uptime-secs").unwrap_or_else(|| "0".into()))),
        ("event_count", Out::N(get(&metrics, "event-count").unwrap_or_else(|| "0".into()))),
        ("peer_count", Out::N(peers.to_string())),
        ("synced", Out::B(synced)),
        ("healthy", Out::B(healthy)),
        ("presence", Out::L(presence)),
        ("member_count", Out::N(members.to_string())),
    ]);
    Ok((out, healthy))
}

pub fn explain(client: &Client, task_id: &str, repos_root: &Path) -> Result<Option<Out>, String> {
    let resp = client.call(&format!("(task-status (task {}))", task_id))?;
    if resp.head() != Some("task-status") {
        return Err(format!("unexpected response: {}", resp.to_text()));
    }
    // Response shape: (task-status ((task X) (generation N) ...)) — the
    // payload is a single nested alist, so positional access, not head-field.
    let crate::sexpr::Sexp::List(items) = &resp else {
        return Err(format!("unexpected response: {}", resp.to_text()));
    };
    let inner = items.get(1).ok_or("malformed task-status payload")?;
    let get = |k: &str| field(inner, k).map(|v| v.to_text());

    let generation = get("generation").unwrap_or_else(|| "0".into());
    let holder = get("holder").unwrap_or_default();
    let completed = get("completed").as_deref() == Some("t");

    // Enrich from origin repo files when present locally.
    let mut file_priority = None;
    let mut file_caps: Vec<Out> = vec![];
    let mut file_deps: Vec<Out> = vec![];
    let mut file_desc: Option<String> = None;
    let mut origin_repo = None;
    for entry in discover_tasks_files(repos_root) {
        let Ok(text) = std::fs::read_to_string(&entry) else { continue };
        let Ok(tasks) = tasks_file::parse_tasks_file(&text) else { continue };
        if let Some(t) = tasks.into_iter().find(|t| t.id == task_id) {
            if file_desc.is_none() { file_desc = t.description.clone(); }
            origin_repo.get_or_insert_with(|| repo_name(&entry));
            file_priority.get_or_insert_with(|| t.priority.to_string());
            if file_caps.is_empty() {
                file_caps = t.capabilities.iter().map(|c| Out::S(c.clone())).collect();
            }
            if file_deps.is_empty() {
                file_deps = t.depends_on.iter().map(|d| Out::S(d.clone())).collect();
            }

        }
    }

    let recommended: String = if completed {
        "no action — task is completed".to_string()
    } else if !holder.is_empty() && holder != "()" {
        format!("wait on `{}` or coordinate a handoff", holder)
    } else if get("ready").as_deref() == Some("t") {
        "claimable now via (claim-task (task ID))".to_string()
    } else {
        "blocked — inspect blocked-by".to_string()
    };

    let out = Out::m(vec![
        ("task", Out::S(task_id.to_string())),
        ("generation", Out::N(generation)),
        (
            "holder",
            Out::S(if holder == "()" { String::new() } else { holder }),
        ),
        ("completed", Out::B(completed)),
        ("origin_repo", Out::S(origin_repo.unwrap_or_default())),
        // Wire priority is authoritative (server state); repo file is a
        // local convenience fallback.
        (
            "priority",
            Out::N(
                get("priority")
                    .or(file_priority)
                    .unwrap_or_else(|| "1".to_string()),
            ),
        ),
        ("capabilities", Out::L(file_caps)),
        ("depends_on", Out::L(file_deps)),
        ("description", Out::S(file_desc.take().unwrap_or_default())),
        ("recommended", Out::S(recommended)),
    ]);
    Ok(Some(out))
}

/// `(repo.my -> context card)`: instant LLM-friendly view of a repository's
/// declared scope (Swarm Contract v0.1). Pure file reading — no mesh needed.
pub fn context(repo_dir: &Path) -> Result<Out, String> {
    use crate::sexpr::{self, Sexp};
    let file = repo_dir.join("repo.my");
    let text = std::fs::read_to_string(&file)
        .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
    let form = sexpr::parse(&text).map_err(|e| format!("parse error in {file:?}: {e}"))?;

    // Locate the (repository ...) form anywhere in the document.
    fn find_repo(s: &Sexp) -> Option<&Sexp> {
        match s {
            Sexp::List(items) => {
                if matches!(items.first(), Some(Sexp::Atom(a)) if a == "repository") {
                    return Some(s);
                }
                items.iter().find_map(find_repo)
            }
            _ => None,
        }
    }
    let repo = find_repo(&form).ok_or("no (repository ...) form found")?;

    // Field extractor: (field v1 v2 ...) -> ["v1","v2",...]
    let field_values = |name: &str| -> Vec<String> {
        match repo.field(name) {
            Some(tail) => tail
                .iter()
                .map(|v| match v {
                    Sexp::Atom(a) => a.clone(),
                    Sexp::Str(s) => s.clone(),
                    other => other.to_text(),
                })
                .collect(),
            None => vec![],
        }
    };
    let one = |name: &str| field_values(name).into_iter().next().unwrap_or_default();

    // Local tasks.my summary when the repo has one (counts only).
    let (tasks_total, tasks_done) = match std::fs::read_to_string(repo_dir.join("tasks.my")) {
        Ok(t) => match tasks_file::parse_tasks_file(&t) {
            Ok(list) => (
                list.len().to_string(),
                list.iter().filter(|x| x.done).count().to_string(),
            ),
            Err(_) => ("0".into(), "0".into()),
        },
        Err(_) => ("0".into(), "0".into()),
    };

    Ok(Out::m(vec![
        ("repo", Out::S(one("id"))),
        ("role", Out::S(one("role"))),
        ("exports", Out::L(field_values("exports").into_iter().map(Out::S).collect())),
        ("imports", Out::L(field_values("imports").into_iter().map(Out::S).collect())),
        (
            "capabilities",
            Out::L(field_values("capabilities").into_iter().map(Out::S).collect()),
        ),
        (
            "authorities",
            Out::L(field_values("authorities").into_iter().map(Out::S).collect()),
        ),
        (
            "non_authorities",
            Out::L(field_values("non-authorities").into_iter().map(Out::S).collect()),
        ),
        ("tasks_total", Out::N(tasks_total)),
        ("tasks_done", Out::N(tasks_done)),
        ("source", Out::S(file.display().to_string())),
    ]))
}

/// Every direct subdirectory of `root` that declares a repo.my.
pub fn all_contexts(root: &Path) -> Result<Out, String> {
    let rd = std::fs::read_dir(root).map_err(|e| format!("cannot read root: {e}"))?;
    let mut dirs: Vec<std::path::PathBuf> = Vec::new();
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() && p.join("repo.my").is_file() {
            dirs.push(p);
        }
    }
    dirs.sort();
    let cards: Vec<Out> = dirs
        .iter()
        .filter_map(|d| context(d).ok())
        .collect();
    let count = cards.len();
    Ok(Out::m(vec![("repos", Out::L(cards)), ("count", Out::N(count.to_string()))]))
}

/// `(next-best-action ...)`: wire-faithful passthrough. The mesh's ranking
/// convention is documented in swarm-node itself; this adapter does not
/// re-derive it, it transports the answer verbatim into structured data.
pub fn nba(client: &Client, caps: &[String]) -> Result<Out, String> {
    let caps_list = if caps.is_empty() {
        String::new()
    } else {
        format!("(capabilities ({}))", caps.join(" "))
    };
    let resp = client.call(&format!("(next-best-action (from l0-agent) {})", caps_list))?;
    if resp.head() != Some("next-best-action") {
        return Err(format!("unexpected response: {}", resp.to_text()));
    }
    let fields = crate::out::sexpr_list_to_out(&resp);
    Ok(Out::m(vec![("head", Out::S("next-best-action".into())), ("fields", fields)]))
}

/// Mutating task ops with full response read-back — the safe alternative to
/// fire-and-forget scripts. The server-side quorum still guards claims;
/// this adapter only makes ok/error visible as structured data.
pub fn task_op(client: &Client, form: String) -> Result<(Out, bool), String> {
    let resp = client.call(&form)?;
    let head = resp.head().unwrap_or("?").to_string();
    let ok = head == "ok";
    let mut pairs = vec![
        ("ok".to_string(), Out::B(ok)),
        ("op_response_head".to_string(), Out::S(head)),
    ];
    if let crate::sexpr::Sexp::List(rest) = &resp {
        for item in rest.iter().skip(1) {
            if let crate::sexpr::Sexp::List(kv) = item {
                if let [crate::sexpr::Sexp::Atom(k), v] = &kv[..] {
                    pairs.push((k.clone(), Out::S(v.to_text())));
                }
                // error responses carry (error "...") style strings too
                else if let [other] = &kv[..] {
                    pairs.push(("detail".to_string(), Out::S(other.to_text())));
                }
            }
        }
    }
    Ok((Out::M(pairs), ok))
}

/// Returns (canonical_text, task_count). Parse errors are reported as Err.
pub fn fmt(file: &Path) -> Result<(String, usize), String> {
    let text = std::fs::read_to_string(file).map_err(|e| format!("cannot read {}: {e}", file.display()))?;
    let tasks = tasks_file::parse_tasks_file(&text)?;
    let mut out = String::from("((kind . tasks-my)\n (tasks . (\n");
    for t in &tasks {
        out.push_str(&format!("  (\"{}\" . (\n", t.id));
        out.push_str(&format!("    (priority . {})\n", trim_float(t.priority)));
        if !t.capabilities.is_empty() {
            out.push_str(&format!("    (capabilities . ({}))\n", t.capabilities.join(" ")));
        }
        if !t.depends_on.is_empty() {
            let deps: Vec<String> = t.depends_on.iter().map(|d| format!("{d:?}")).collect();
            out.push_str(&format!("    (depends-on . ({}))\n", deps.join(" ")));
        }
        out.push_str(&format!("    (done . {})\n", if t.done { "t" } else { "()" }));
        if let Some(d) = &t.description {
            out.push_str(&format!("    (description . {:?})\n", d));
        }
        if let Some(o) = &t.origin {
            out.push_str(&format!("    (origin . {o})\n"));
        }
        out.push_str("  ))\n");
    }
    out.push_str(" ))\n)");
    Ok((out, tasks.len()))
}

fn trim_float(f: f64) -> String {
    f.to_string()
}

fn repo_name(path: &Path) -> String {
    path.parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn discover_tasks_files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else { return found };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            let cand = p.join("tasks.my");
            if cand.is_file() {
                found.push(cand);
            }
        }
    }
    found.sort();
    found
}
