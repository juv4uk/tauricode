//! Format adapters between the durable `tasks.my` records and interchange
//! formats used by Level 0 tools: JSON, GitHub Issues export, Markdown
//! frontmatter notes, and our own emitted YAML dialect.
//!
//! Mapping contract (deliberate, documented — not guessed per-file):
//!
//! | canonical field | JSON/YAML key   | GH issue source        | MD frontmatter |
//! |-----------------|-----------------|------------------------|----------------|
//! | id              | id              | `GH-<number>`          | task-id/title  |
//! | priority        | priority (num)  | --priority flag (3.0)  | priority       |
//! | capabilities    | capabilities[]  | labels[].name          | tags/capabilities|
//! | depends_on      | depends_on[]    | —                      | depends-on     |
//! | done            | done (bool)     | state == "closed"      | done           |
//! | description     | description     | title + "\n\n" + body  | body text      |

use crate::minijson::{self, Json};
use crate::tasks_file::ParsedTask;
use std::path::Path;

fn base_task(id: String) -> ParsedTask {
    ParsedTask {
        id,
        priority: 1.0,
        capabilities: vec![],
        depends_on: vec![],
        done: false,
        description: None,
        origin: None,
    }
}

// ---------- JSON ----------

pub fn tasks_from_json(text: &str) -> Result<Vec<ParsedTask>, String> {
    let j = minijson::parse(text)?;
    let arr = match &j {
        Json::Arr(a) => a.clone(),
        Json::Obj(_) => match j.get("tasks").and_then(|t| t.as_arr()) {
            Some(a) => a.to_vec(),
            None => return Err("JSON object must contain a `tasks` array".into()),
        },
        _ => return Err("top-level JSON must be an array or {tasks:[...]}".into()),
    };
    arr.iter().map(json_task).collect()
}

fn json_task(v: &Json) -> Result<ParsedTask, String> {
    if !matches!(v, Json::Obj(_)) {
        return Err("task entry must be an object".into());
    }
    let mut t = base_task(
        v.get("id")
            .and_then(|x| x.as_str())
            .ok_or_else(|| "task object needs `id`".to_string())?
            .to_string(),
    );
    if let Some(p) = v.get("priority").and_then(|x| x.as_f64()) {
        t.priority = p;
    }
    if let Some(cs) = v.get("capabilities").and_then(|x| x.as_arr()) {
        t.capabilities = cs.iter().filter_map(|c| c.as_str().map(String::from)).collect();
    }
    if let Some(ds) = v.get("depends_on").and_then(|x| x.as_arr()) {
        t.depends_on = ds.iter().filter_map(|d| d.as_str().map(String::from)).collect();
    }
    if let Some(d) = v.get("done").and_then(|x| x.as_bool()) {
        t.done = d;
    }
    t.description = v.get("description").and_then(|x| x.as_str()).map(String::from);
    t.origin = v.get("origin").and_then(|x| x.as_str()).map(String::from);
    Ok(t)
}

/// One GH issue object -> task. Id is `GH-<number>` unless --prefix given.
pub fn task_from_gh_issue(v: &Json, prefix: Option<&str>, default_priority: f64) -> Result<ParsedTask, String> {
    let number = v
        .get("number")
        .and_then(|x| x.as_f64())
        .ok_or_else(|| "issue object needs `number`".to_string())? as u64;
    let title = v.get("title").and_then(|x| x.as_str()).unwrap_or("");
    let body = v.get("body").and_then(|x| x.as_str()).unwrap_or("");
    let state = v.get("state").and_then(|x| x.as_str()).unwrap_or("open");
    let mut t = base_task(format!("{}GH-{number}", prefix.unwrap_or("")));
    t.priority = default_priority;
    if let Some(labels) = v.get("labels").and_then(|x| x.as_arr()) {
        t.capabilities = labels
            .iter()
            .filter_map(|l| {
                l.get("name")
                    .and_then(|n| n.as_str().map(String::from))
                    .or_else(|| l.as_str().map(String::from))
            })
            .collect();
    }
    t.done = state == "closed";
    t.description = Some(if body.is_empty() {
        title.to_string()
    } else {
        format!("{title}\n\n{body}")
    });
    Ok(t)
}

pub fn tasks_from_gh_json(text: &str, prefix: Option<&str>, default_priority: f64) -> Result<Vec<ParsedTask>, String> {
    let j = minijson::parse(text)?;
    let arr = j.as_arr().ok_or("expected a JSON array of issues")?;
    arr.iter().map(|v| task_from_gh_issue(v, prefix, default_priority)).collect()
}

// ---------- Markdown frontmatter ----------

/// Parses ONE note with `---` frontmatter into a task. Missing fields fall
/// back per the contract table above; the note body becomes the description.
pub fn task_from_md_note(text: &str) -> Result<ParsedTask, String> {
    let rest = text.strip_prefix("---").ok_or("no frontmatter block")?;
    let end = rest.find("\n---").ok_or("unterminated frontmatter")?;
    let fm = &rest[..end];
    let body = rest[end + 4..].trim();
    let mut t = base_task(String::new());
    let mut slug_title: Option<String> = None;
    let mut title_fallback: Option<String> = None;
    for line in fm.lines() {
        let Some((k, v)) = line.split_once(':') else { continue };
        let k = k.trim();
        let v = v.trim().trim_matches('"');
        match k {
            "task-id" | "id" => t.id = v.to_string(),
            "title" => {
                if t.id.is_empty() {
                    slug_title = Some(v.to_string());
                }
                title_fallback = Some(v.to_string());
            }
            "priority" => {
                t.priority = v.parse().unwrap_or(1.0);
            }
            "tags" | "capabilities" => {
                let cleaned = v.trim_start_matches('[').trim_end_matches(']');
                t.capabilities = cleaned
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            "depends-on" | "depends_on" => {
                let cleaned = v.trim_start_matches('[').trim_end_matches(']');
                t.depends_on = cleaned
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            "done" => t.done = matches!(v, "true" | "t" | "yes"),
            "origin" => t.origin = Some(v.to_string()),
            _ => {}
        }
    }
    if t.id.is_empty() {
        if let Some(title) = slug_title.or(title_fallback) {
            t.id = slugify(&title);
        }
    }
    if t.id.is_empty() {
        return Err("note has neither task-id/id nor title to derive one from".into());
    }
    if t.description.is_none() && !body.is_empty() {
        // first non-empty body line as short description
        t.description = body.lines().map(str::trim).find(|l| !l.is_empty()).map(String::from);
    }
    Ok(t)
}

fn slugify(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| c.to_ascii_uppercase())
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let trimmed = cleaned.trim_matches('-');
    let mut compact = String::new();
    let mut prev_dash = false;
    for c in trimmed.chars() {
        if c == '-' {
            if !prev_dash {
                compact.push('-');
            }
            prev_dash = true;
        } else {
            compact.push(c);
            prev_dash = false;
        }
    }
    compact
}

/// Every `.md` file in a directory (non-recursive) becomes one task.
pub fn tasks_from_md_dir(dir: &Path) -> Result<Vec<ParsedTask>, String> {
    let rd = std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    let mut files: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().map(|x| x == "md").unwrap_or(false)).collect();
    files.sort();
    files.iter().map(|p| {
        let text = std::fs::read_to_string(p).map_err(|e| format!("cannot read {}: {e}", p.display()))?;
        task_from_md_note(&text).map_err(|e| format!("{}: {e}", p.display()))
    }).collect()
}

// ---------- renderers: tasks -> external formats ----------

pub fn task_to_gh_json(t: &ParsedTask) -> Json {
    Json::Obj(vec![
        ("title".into(), Json::Str(t.description.clone().unwrap_or_else(|| t.id.clone()))),
        ("labels".into(), Json::Arr(t.capabilities.iter().map(|c| Json::Str(c.clone())).collect())),
        ("state".into(), Json::Str(if t.done { "closed".into() } else { "open".into() })),
    ])
}

/// Converts a parsed JSON tree into the output IR so any Json value can be
/// rendered through the same emitters as everything else.
pub fn json_to_out(j: &Json) -> crate::out::Out {
    use crate::out::Out;
    match j {
        // N renders raw text in both emitters -> JSON/YAML `null`, not a string
        Json::Null => Out::N("null".into()),
        Json::Bool(b) => Out::B(*b),
        Json::Num(n) => Out::N(n.to_string()),
        Json::Str(s) => Out::S(s.clone()),
        Json::Arr(a) => Out::L(a.iter().map(json_to_out).collect()),
        Json::Obj(pairs) => Out::M(pairs.iter().map(|(k, v)| (k.clone(), json_to_out(v))).collect()),
    }
}

pub fn tasks_to_md_notes(tasks: &[ParsedTask]) -> String {
    let mut out = String::new();
    for t in tasks {
        out.push_str("---\n");
        out.push_str(&format!("task-id: {}\n", t.id));
        out.push_str(&format!("priority: {}\n", t.priority));
        if !t.capabilities.is_empty() {
            out.push_str(&format!("capabilities: [{}]\n", t.capabilities.join(", ")));
        }
        if !t.depends_on.is_empty() {
            out.push_str(&format!("depends-on: [{}]\n", t.depends_on.join(", ")));
        }
        out.push_str(&format!("done: {}\n", if t.done { "true" } else { "false" }));
        if let Some(o) = &t.origin {
            out.push_str(&format!("origin: {o}\n"));
        }
        out.push_str("---\n\n");
        out.push_str(&format!("# {}\n\n", t.id));
        if let Some(d) = &t.description {
            out.push_str(d);
            out.push('\n');
        }
        out.push('\n');
    }
    out
}
