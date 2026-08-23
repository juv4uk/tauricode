//! `swarm-cli` — Level 0 agent adapters for the swarm coordination plane.
//!
//! check / explain / fmt (+ convert) with JSON and YAML output so agents
//! that cannot speak the swarm/1 s-expr protocol can still observe and
//! reason about the mesh. Read-only by design: mutating ops stay with
//! full agents via quorum-guarded claim-task (M1.1 authority boundary).

use swarm_cli::{ops, out, tasks_file, wire};
use swarm_cli::out::Out;
use std::process::ExitCode;

const USAGE: &str = r#"swarm-cli — JSON/YAML adapters over the swarm-node coordination plane

USAGE:
  swarm-cli check [--node HOST:PORT] [--json|--yaml]
  swarm-cli explain <TASK-ID> [--repos DIR] [--json|--yaml]
  swarm-cli fmt <file.tasks.my> [--check]
  swarm-cli nba [--caps a,b]                    (next-best-action, wire-faithful)
  swarm-cli task claim <ID>
  swarm-cli task complete <ID> --gen N
  swarm-cli task release <ID> --gen N
  swarm-cli task define <ID> [--priority P] [--caps c1,c2] [--desc "..."]
  swarm-cli convert <file.my> --json|--yaml     (tasks.my -> structured output)

DEFAULTS: --node 127.0.0.1:9104 (override with SWARM_NODE env or flag).
EXIT CODES: 0 ok · 1 unhealthy (check) · 2 usage · 3 source/parse error · 4 not found
Read-only: claims/dispatch stay with full agents via quorum-guarded claim-task."#;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(e.code)
        }
    }
}

struct CliErr {
    msg: String,
    code: u8,
}

impl std::fmt::Display for CliErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl CliErr {
    fn usage(m: impl Into<String>) -> CliErr {
        CliErr { msg: format!("{}\n\n{}", m.into(), USAGE.trim()), code: 2 }
    }
    fn src(m: impl Into<String>) -> CliErr {
        CliErr { msg: m.into(), code: 3 }
    }
}

impl From<String> for CliErr {
    fn from(m: String) -> Self {
        CliErr { msg: m, code: 1 }
    }
}

fn run(args: Vec<String>) -> Result<(), CliErr> {
    let mut cmd = args.first().cloned().ok_or_else(|| CliErr::usage("missing command"))?;
    let mut rest: Vec<String> = args[1..].to_vec();
    if cmd == "--help" || cmd == "-h" {
        println!("{}", USAGE.trim());
        return Ok(());
    }
    // tolerate global flags before the command
    if cmd.starts_with('-') {
        return Err(CliErr::usage(format!("global flags must follow the command (got `{cmd}`)")));
    }
    let mut node = std::env::var("SWARM_NODE").unwrap_or_else(|_| "127.0.0.1:9104".into());
    let mut format = String::from("json");
    let mut repos = String::from("/home/agents/GitHub");
    let mut check_only = false;

    // allow --node/--format anywhere
    let mut it = rest.into_iter();
    let mut filtered: Vec<String> = Vec::new();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--node" => node = it.next().ok_or_else(|| CliErr::usage("--node needs a value"))?,
            "--json" => format = "json".into(),
            "--yaml" => format = "yaml".into(),
            "--repos" => repos = it.next().ok_or_else(|| CliErr::usage("--repos needs a value"))?,
            "--check" => check_only = true,
            other => filtered.push(other.to_string()),
        }
    }
    rest = filtered;
    let _ = &mut cmd;

    match cmd.as_str() {
        "check" => {
            let client = wire::Client::new(node);
            let (out, healthy) = ops::check(&client).map_err(CliErr::from)?;
            print_output(&out, &format);
            if healthy { Ok(()) } else { Err(CliErr { msg: String::new(), code: 1 }) }
        }
        "explain" => {
            let id = rest.first().ok_or_else(|| CliErr::usage("explain requires a TASK-ID"))?;
            let client = wire::Client::new(node);
            match ops::explain(&client, id, std::path::Path::new(&repos)).map_err(CliErr::from)? {
                Some(out) => {
                    print_output(&out, &format);
                    Ok(())
                }
                None => Err(CliErr { msg: format!("task `{id}` not found"), code: 4 }),
            }
        }
        "fmt" => {
            let file = rest.first().ok_or_else(|| CliErr::usage("fmt requires a file path"))?;
            let (canonical, count) = ops::fmt(std::path::Path::new(file)).map_err(CliErr::src)?;
            if check_only {
                let current = std::fs::read_to_string(file).map_err(|e| CliErr::src(e.to_string()))?;
                if current == canonical {
                    println!("{}: canonical ({} tasks)", file, count);
                    Ok(())
                } else {
                    Err(CliErr { msg: format!("{file}: would reformat ({count} tasks)"), code: 3 })
                }
            } else {
                print!("{canonical}");
                eprintln!("fmt: {} tasks", count);
                Ok(())
            }
        }
        "nba" => {
            // --caps a,b (also accepts leftover positional as caps csv)
            let caps: Vec<String> = rest
                .iter()
                .flat_map(|a| a.trim_start_matches("--caps").split(','))
                .filter(|c| !c.is_empty() && *c != "--caps")
                .map(|c| c.to_string())
                .collect();
            let client = wire::Client::new(node);
            let out = ops::nba(&client, &caps).map_err(CliErr::from)?;
            print_output(&out, &format);
            Ok(())
        }
        "task" => {
            let sub = rest.first().ok_or_else(|| CliErr::usage("task requires a subcommand"))?;
            let id = rest.get(1).ok_or_else(|| CliErr::usage("task subcommands require TASK-ID"))?;
            let mut gen: Option<String> = None;
            let mut prio = String::from("1.0");
            let mut desc = String::new();
            let mut caps = String::new();
            let mut it = rest.iter().skip(2);
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--gen" => gen = Some(it.next().ok_or_else(|| CliErr::usage("--gen needs a value"))?.clone()),
                    "--priority" => prio = it.next().ok_or_else(|| CliErr::usage("--priority needs a value"))?.clone(),
                    "--desc" => desc = it.next().ok_or_else(|| CliErr::usage("--desc needs a value"))?.clone(),
                    "--caps" => caps = it.next().ok_or_else(|| CliErr::usage("--caps needs a value"))?.clone(),
                    other => return Err(CliErr::usage(format!("unknown task flag `{other}`"))),
                }
            }
            let form = match sub.as_str() {
                "claim" => format!("(claim-task (task {id}))"),
                "complete" | "release" => {
                    let g = gen.ok_or_else(|| CliErr::usage(format!("task {sub} requires --gen N")))?;
                    let op = if sub == "complete" { "complete-task" } else { "release-task" };
                    format!("({op} (task {id}) (generation {g}))")
                }
                "define" => {
                    let mut f = format!("(define-task (task {id}) (priority {prio})");
                    if !caps.is_empty() {
                        f.push_str(&format!(" (capabilities ({}))", caps.split(',').collect::<Vec<_>>().join(" ")));
                    }
                    if !desc.is_empty() {
                        f.push_str(&format!(" (description \"{desc}\"))"));
                    } else {
                        f.push(')');
                    }
                    f
                }
                other => return Err(CliErr::usage(format!("unknown task subcommand `{other}` (use claim|complete|release|define)"))),
            };
            let client = wire::Client::new(node);
            let (out, ok) = ops::task_op(&client, form).map_err(CliErr::from)?;
            print_output(&out, &format);
            if ok { Ok(()) } else { Err(CliErr { msg: String::new(), code: 5 }) }
        }
        "convert" => {
            let file = rest.first().ok_or_else(|| CliErr::usage("convert requires a file path"))?;
            let text = std::fs::read_to_string(file).map_err(|e| CliErr::src(format!("cannot read {file}: {e}")))?;
            let tasks = tasks_file::parse_tasks_file(&text).map_err(CliErr::src)?;
            let items: Vec<Out> = tasks
                .iter()
                .map(|t| {
                    Out::m(vec![
                        ("id", Out::S(t.id.clone())),
                        ("priority", Out::N(t.priority.to_string())),
                        ("capabilities", Out::L(t.capabilities.iter().map(|c: &String| Out::S(c.clone())).collect())),
                        ("depends_on", Out::L(t.depends_on.iter().map(|d: &String| Out::S(d.clone())).collect())),
                        ("done", Out::B(t.done)),
                        ("description", Out::S(t.description.clone().unwrap_or_default())),
                        ("origin", Out::S(t.origin.clone().unwrap_or_default())),
                    ])
                })
                .collect();
            let wrapped = Out::m(vec![("tasks", Out::L(items))]);
            print_output(&wrapped, &format);
            Ok(())
        }
        other => Err(CliErr::usage(format!("unknown command `{other}`"))),
    }
}

fn print_output(v: &Out, format: &str) {
    match format {
        "yaml" => print!("{}", out::to_yaml(v)),
        _ => println!("{}", out::to_json(v)),
    }
}
