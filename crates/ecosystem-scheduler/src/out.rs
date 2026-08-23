//! Output emitters: deterministic JSON and human-readable text.
//! Hand-rolled (zero-dep convention); the shapes are fully known here.

use crate::graph::BlockReason;
use crate::route::AgentRoute;

pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn str_array(items: &[String]) -> String {
    let inner: Vec<String> = items.iter().map(|i| format!("\"{}\"", json_escape(i))).collect();
    format!("[{}]", inner.join(","))
}

pub struct PlanOut<'a> {
    pub sources: &'a [(String, String, usize)], // (repo, path, task_count)
    pub warnings: &'a [String],
    pub counts: (usize, usize, usize, usize), // total, done, ready, blocked
    pub routes: &'a [AgentRoute],
    pub blocked_detail: Vec<(String, String, BlockReason)>, // id, origin, reason
}

pub fn to_json(p: &PlanOut) -> String {
    let sources: Vec<String> = p
        .sources
        .iter()
        .map(|(r, path, n)| {
            format!(
                "{{\"repo\":\"{}\",\"path\":\"{}\",\"tasks\":{}}}",
                json_escape(r),
                json_escape(path),
                n
            )
        })
        .collect();
    let warnings: Vec<String> = p.warnings.iter().map(|w| format!("\"{}\"", json_escape(w))).collect();
    let routing: Vec<String> = p
        .routes
        .iter()
        .map(|ar| {
            let entries: Vec<String> = ar
                .ranked
                .iter()
                .map(|e| {
                    format!(
                        "{{\"id\":\"{}\",\"origin\":\"{}\",\"score\":{},\"priority\":{},\"unblock_impact\":{},\"capabilities\":{},\"depends_on\":{}}}",
                        json_escape(&e.id),
                        json_escape(&e.origin),
                        e.score,
                        e.priority,
                        e.unblock_impact,
                        str_array(&e.capabilities),
                        str_array(&e.depends_on)
                    )
                })
                .collect();
            format!(
                "{{\"agent\":\"{}\",\"capabilities\":{},\"ranked\":[{}]}}",
                json_escape(&ar.agent),
                str_array(&ar.capabilities),
                entries.join(",")
            )
        })
        .collect();
    let blocked: Vec<String> = p
        .blocked_detail
        .iter()
        .map(|(id, origin, reason)| {
            let r = match reason {
                BlockReason::MissingDep(d) => format!("{{\"kind\":\"missing-dep\",\"dep\":\"{}\"}}", json_escape(d)),
                BlockReason::WaitingOn(d) => format!("{{\"kind\":\"waiting-on\",\"dep\":\"{}\"}}", json_escape(d)),
                BlockReason::Cycle => "{\"kind\":\"cycle\"}".to_string(),
            };
            format!("{{\"id\":\"{}\",\"origin\":\"{}\",\"reason\":{}}}", json_escape(id), json_escape(origin), r)
        })
        .collect();
    format!(
        "{{\"scheduler_version\":\"0.1.0\",\"sources\":[{}],\"warnings\":[{}],\"graph\":{{\"total\":{},\"done\":{},\"ready\":{},\"blocked\":{}}},\"routing\":[{}],\"blocked_detail\":[{}]}}",
        sources.join(","),
        warnings.join(","),
        p.counts.0,
        p.counts.1,
        p.counts.2,
        p.counts.3,
        routing.join(","),
        blocked.join(",")
    )
}

pub fn to_text(p: &PlanOut) -> String {
    let mut s = String::new();
    for w in p.warnings {
        s.push_str(&format!("WARN: {w}\n"));
    }
    s.push_str(&format!(
        "graph: {} tasks | {} done | {} ready | {} blocked\n\n",
        p.counts.0, p.counts.1, p.counts.2, p.counts.3
    ));
    for ar in p.routes {
        s.push_str(&format!(
            "== {} (caps: {}) ==\n",
            ar.agent,
            if ar.capabilities.is_empty() { "any".to_string() } else { ar.capabilities.join(", ") }
        ));
        if ar.ranked.is_empty() {
            s.push_str("  (nothing actionable)\n");
        }
        for (i, e) in ar.ranked.iter().enumerate() {
            s.push_str(&format!(
                "  {:>2}. [{:<12}] {:<28} score {:>6.1} = prio {:>4.1} x (1+{}){}\n",
                i + 1,
                e.origin,
                e.id,
                e.score,
                e.priority,
                e.unblock_impact,
                if e.capabilities.is_empty() { String::new() } else { format!(" caps({})", e.capabilities.join(",")) }
            ));
        }
        s.push('\n');
    }
    if !p.blocked_detail.is_empty() {
        s.push_str("== blocked ==\n");
        for (id, origin, reason) in &p.blocked_detail {
            let r = match reason {
                BlockReason::MissingDep(d) => format!("missing dep `{d}`"),
                BlockReason::WaitingOn(d) => format!("waiting on `{d}`"),
                BlockReason::Cycle => "dependency cycle".to_string(),
            };
            s.push_str(&format!("  [{origin}] {id}: {r}\n"));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GlobalGraph;

    #[test]
    fn escapes_json_control_chars() {
        assert_eq!(json_escape("a\"b\\c\nd"), "a\\\"b\\\\c\\nd");
    }

    #[test]
    fn text_output_lists_counts_and_routes() {
        let g = GlobalGraph::build(vec![crate::graph::RepoTasks {
            repo: "r".into(),
            tasks: vec![crate::tasks_file::ParsedTask {
                id: "A".into(),
                priority: 2.0,
                capabilities: vec![],
                depends_on: vec![],
                done: false,
                description: None,
                origin: None,
            }],
        }]);
        let states = g.state_map();
        let routes = crate::route::plan(&g, &states, &[]);
        let out = PlanOut {
            sources: &[("r".into(), "/tmp/r/tasks.my".into(), 1)],
            warnings: &[],
            counts: g.counts(&states),
            routes: &routes,
            blocked_detail: vec![],
        };
        let t = to_text(&out);
        assert!(t.contains("graph: 1 tasks | 0 done | 1 ready | 0 blocked"));
        assert!(t.contains("A"));
        assert!(to_json(&out).contains("\"total\":1"));
    }
}
