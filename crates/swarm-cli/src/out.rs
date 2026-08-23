//! Tiny output IR shared by both renderers, so JSON and YAML can never
//! disagree about content — only about syntax.

pub enum Out {
    S(String),
    N(String), // pre-rendered number (keeps integers integer-looking)
    B(bool),
    L(Vec<Out>),
    M(Vec<(String, Out)>),
}

impl Out {
    pub fn m(pairs: Vec<(&str, Out)>) -> Out {
        Out::M(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }
}

/// Wire-faithful conversion of a parsed s-expr response into the output IR:
/// atoms/strings become S, lists become L. No interpretation, no loss —
/// Level 0 agents see exactly what swarm-node said.
pub fn sexpr_list_to_out(s: &crate::sexpr::Sexp) -> Out {
    match s {
        crate::sexpr::Sexp::Atom(a) => Out::S(a.clone()),
        crate::sexpr::Sexp::Str(v) => Out::S(v.clone()),
        crate::sexpr::Sexp::List(items) => {
            // A (key v1 ...) form renders as a one-pair map for readability.
            if let [crate::sexpr::Sexp::Atom(head), rest @ ..] = &items[..] {
                let vals: Vec<Out> = rest.iter().map(sexpr_list_to_out).collect();
                return Out::M(vec![(
                    head.clone(),
                    if vals.len() == 1 { vals.into_iter().next().unwrap() } else { Out::L(vals) },
                )]);
            }
            Out::L(items.iter().map(sexpr_list_to_out).collect())
        }
    }
}

pub fn to_json(v: &Out) -> String {    match v {
        Out::S(s) => format!("\"{}\"", json_escape(s)),
        Out::N(n) => n.clone(),
        Out::B(b) => b.to_string(),
        Out::L(items) => {
            let inner: Vec<String> = items.iter().map(to_json).collect();
            format!("[{}]", inner.join(","))
        }
        Out::M(pairs) => {
            let inner: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("\"{}\":{}", json_escape(k), to_json(v)))
                .collect();
            format!("{{{}}}", inner.join(","))
        }
    }
}

fn json_escape(s: &str) -> String {
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

fn yaml_needs_quotes(s: &str) -> bool {
    s.is_empty()
        || s.starts_with(' ')
        || s.ends_with(' ')
        || s.starts_with('#')
        || s.starts_with('-')
        || s.starts_with('&')
        || s.starts_with('*')
        || s.starts_with('?')
        || s.starts_with('!')
        || s.starts_with('%')
        || s.starts_with('@')
        || s.contains(": ")
        || s.ends_with(':')
        || s.contains(" #")
        || s.contains('\n')
        || matches!(s, "true" | "false" | "null" | "~" | "yes" | "no" | "on" | "off")
}

fn yaml_scalar(s: &str) -> String {
    if yaml_needs_quotes(s) {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"))
    } else {
        s.to_string()
    }
}

pub fn to_yaml(v: &Out) -> String {
    let mut out = String::new();
    render_yaml(v, 0, &mut out, false);
    out
}

fn render_yaml(v: &Out, indent: usize, out: &mut String, inline_key: bool) {
    let pad = "  ".repeat(indent);
    match v {
        Out::S(s) => {
            if inline_key {
                out.push_str(&yaml_scalar(s));
                out.push('\n');
            } else {
                out.push_str(&format!("{pad}{}\n", yaml_scalar(s)));
            }
        }
        Out::N(n) => {
            if inline_key {
                out.push_str(n);
                out.push('\n');
            } else {
                out.push_str(&format!("{pad}{n}\n"));
            }
        }
        Out::B(b) => {
            let s = if *b { "true" } else { "false" };
            if inline_key {
                out.push_str(s);
                out.push('\n');
            } else {
                out.push_str(&format!("{pad}{s}\n"));
            }
        }
        Out::L(items) => {
            if items.is_empty() {
                out.push_str(&format!("{pad}[]\n"));
                return;
            }
            for item in items {
                match item {
                    Out::M(_) => {
                        out.push_str(&format!("{pad}- "));
                        // first pair inline after "- "
                        if let Out::M(pairs) = item {
                            for (i, (k, val)) in pairs.iter().enumerate() {
                                if i == 0 {
                                    out.push_str(&format!("{k}: "));
                                    render_yaml(val, indent + 1, out, true);
                                } else {
                                    out.push_str(&format!("{pad}  {k}: "));
                                    render_yaml(val, indent + 1, out, true);
                                }
                            }
                        }
                    }
                    scalar @ (Out::S(_) | Out::N(_) | Out::B(_)) => {
                        out.push_str(&format!("{pad}- "));
                        render_yaml(scalar, 0, out, true);
                    }
                    Out::L(_) => {
                        out.push_str(&format!("{pad}-\n"));
                        render_yaml(item, indent + 1, out, false);
                    }
                }
            }
        }
        Out::M(pairs) => {
            for (k, val) in pairs {
                match val {
                    Out::M(_) | Out::L(_) if !matches!(val, Out::L(l) if l.is_empty()) => {
                        out.push_str(&format!("{pad}{k}:\n"));
                        render_yaml(val, indent + 1, out, false);
                    }
                    Out::L(l) if l.is_empty() => out.push_str(&format!("{pad}{k}: []\n")),
                    scalar => {
                        out.push_str(&format!("{pad}{k}: "));
                        render_yaml(scalar, 0, out, true);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip_shapes() {
        let v = Out::m(vec![
            ("a", Out::S("x\"y".into())),
            ("n", Out::N("3".into())),
            ("ok", Out::B(true)),
            ("list", Out::L(vec![Out::N("1".into()), Out::S("b".into())])),
            ("empty", Out::L(vec![])),
        ]);
        assert_eq!(
            to_json(&v),
            r#"{"a":"x\"y","n":3,"ok":true,"list":[1,"b"],"empty":[]}"#
        );
    }

    #[test]
    fn yaml_maps_lists_scalars() {
        let v = Out::m(vec![
            ("node", Out::S("ganaka-1".into())),
            ("synced", Out::B(true)),
            ("caps", Out::L(vec![Out::S("rust".into()), Out::S("lisp".into())])),
            ("none", Out::L(vec![])),
            ("sub", Out::m(vec![("k", Out::S("v: 1".into()))])),
        ]);
        let y = to_yaml(&v);
        assert!(y.contains("node: ganaka-1"));
        assert!(y.contains("synced: true"));
        assert!(y.contains("- rust"));
        assert!(y.contains("none: []"));
        assert!(y.contains("k: \"v: 1\""));
    }
}
