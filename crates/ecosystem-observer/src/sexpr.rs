//! Minimal S-expression reader for `ecosystem-observer` (Stage 1
//! acceptance criteria's "Ecosystem contracts" section: `repo.my`,
//! `language-contract.my`, `isa-contract.my`, `compatibility.my`,
//! `tasks.my`, `evidence/` — all plain S-expr/alist data files, not
//! executable my-lisp code).
//!
//! Per ECO-DECISION-2026-08-19-TAURICODE-TAURI-ARCHITECTURE point 3:
//! a subset (symbols, strings, numbers, lists, dotted pairs,
//! `t`/`()`-truthiness), no eval, no macro-expansion, no dependency on
//! the my-lisp runtime — closer in size and spirit to `cml/src/parser.rs`
//! (read directly before writing this, same tokenize/parse_expr/
//! parse_list shape) than to my-lisp's own full 24KB parser, which also
//! serves executable code. `cml`'s own doctrine note applies here too:
//! this crate deliberately does not depend on my-lisp or any sibling
//! repo's parser — "neighbor is an external authority not a file to
//! edit" (`docs/agent-doctrine.md`), so ecosystem-observer reads the
//! same S-expr *syntax* independently rather than importing a parser
//! from a repo it must never author code changes to.

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Symbol(String),
    String(String),
    Integer(i64),
    List(Vec<Expr>),
    DottedList(Vec<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedEof,
    UnexpectedToken(String),
}

impl Expr {
    /// `t`/`()`-truthiness (per the ACCEPTED decision's own spec): every
    /// value is truthy except the empty list, matching the Lisp
    /// convention `tasks.my` itself already uses for `done` fields.
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Expr::List(items) if items.is_empty())
    }

    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Expr::Symbol(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Expr::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Expr]> {
        match self {
            Expr::List(items) => Some(items.as_slice()),
            _ => None,
        }
    }

    /// A "tagged list" is the ecosystem's own alist-of-alists
    /// convention (e.g. `(role observer-ide)`, `(name "tauricode")`
    /// inside a `repo.my`): a list whose first element is a symbol
    /// naming the entry, matched against `tag`.
    pub fn tagged_list(&self, tag: &str) -> Option<&[Expr]> {
        let items = self.as_list()?;
        let (head, rest) = items.split_first()?;
        if head.as_symbol() == Some(tag) {
            Some(rest)
        } else {
            None
        }
    }

    /// Alist lookup: finds the first `(key ...)`-tagged element of this
    /// list and returns its rest, matching my-idea's own `assoc`
    /// convention (FACT section of ECO-DECISION-2026-08-19-TAURICODE-
    /// TAURI-ARCHITECTURE) — read about, not imported from, per the
    /// same cross-repo non-dependency note above.
    pub fn assoc(&self, key: &str) -> Option<&[Expr]> {
        let items = self.as_list()?;
        items.iter().find_map(|item| item.tagged_list(key))
    }
}

pub fn parse(input: &str) -> Result<Vec<Expr>, ParseError> {
    let tokens = tokenize(input);
    let mut it = tokens.into_iter().peekable();
    let mut exprs = Vec::new();
    while it.peek().is_some() {
        exprs.push(parse_expr(&mut it)?);
    }
    Ok(exprs)
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut in_comment = false;

    for c in input.chars() {
        if in_comment {
            if c == '\n' {
                in_comment = false;
            }
            continue;
        }
        if in_string {
            current.push(c);
            if c == '"' {
                tokens.push(current.clone());
                current.clear();
                in_string = false;
            }
            continue;
        }
        match c {
            ';' => in_comment = true,
            '(' | ')' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(c.to_string());
            }
            '"' => {
                in_string = true;
                current.push(c);
            }
            c if c.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn parse_expr(
    tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
) -> Result<Expr, ParseError> {
    let token = tokens.next().ok_or(ParseError::UnexpectedEof)?;
    match token.as_str() {
        "(" => parse_list(tokens),
        ")" => Err(ParseError::UnexpectedToken(")".to_string())),
        _ => {
            if token.starts_with('"') && token.ends_with('"') && token.len() >= 2 {
                Ok(Expr::String(token[1..token.len() - 1].to_string()))
            } else if let Ok(n) = token.parse::<i64>() {
                Ok(Expr::Integer(n))
            } else {
                Ok(Expr::Symbol(token))
            }
        }
    }
}

fn parse_list(
    tokens: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
) -> Result<Expr, ParseError> {
    let mut list = Vec::new();
    while let Some(peeked) = tokens.peek() {
        if peeked == ")" {
            tokens.next();
            return Ok(Expr::List(list));
        } else if peeked == "." {
            tokens.next();
            let dotted = parse_expr(tokens)?;
            let closing = tokens.next().ok_or(ParseError::UnexpectedEof)?;
            if closing != ")" {
                return Err(ParseError::UnexpectedToken(closing));
            }
            return Ok(Expr::DottedList(list, Box::new(dotted)));
        }
        list.push(parse_expr(tokens)?);
    }
    Err(ParseError::UnexpectedEof)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_symbol_string_integer() {
        let exprs = parse(r#"foo "bar" 42"#).unwrap();
        assert_eq!(
            exprs,
            vec![
                Expr::Symbol("foo".to_string()),
                Expr::String("bar".to_string()),
                Expr::Integer(42),
            ]
        );
    }

    #[test]
    fn parses_nested_lists() {
        let exprs = parse("(role observer-ide)").unwrap();
        assert_eq!(
            exprs,
            vec![Expr::List(vec![
                Expr::Symbol("role".to_string()),
                Expr::Symbol("observer-ide".to_string()),
            ])]
        );
    }

    #[test]
    fn parses_dotted_pair() {
        let exprs = parse("(a . b)").unwrap();
        assert_eq!(
            exprs,
            vec![Expr::DottedList(
                vec![Expr::Symbol("a".to_string())],
                Box::new(Expr::Symbol("b".to_string())),
            )]
        );
    }

    #[test]
    fn skips_comments() {
        let exprs = parse("; a comment\n(x 1) ; trailing\n(y 2)").unwrap();
        assert_eq!(exprs.len(), 2);
    }

    #[test]
    fn empty_list_is_falsy_nonempty_is_truthy() {
        assert!(!Expr::List(vec![]).is_truthy());
        assert!(Expr::Symbol("t".to_string()).is_truthy());
        assert!(Expr::List(vec![Expr::Integer(1)]).is_truthy());
    }

    #[test]
    fn assoc_finds_tagged_entry() {
        let repo_my = parse(
            "(repo (name \"tauricode\") (role observer-ide) (authorities ide-ux))",
        )
        .unwrap();
        let root = &repo_my[0];
        let name = root.assoc("name").unwrap();
        assert_eq!(name[0].as_string(), Some("tauricode"));
        let role = root.assoc("role").unwrap();
        assert_eq!(role[0].as_symbol(), Some("observer-ide"));
        assert!(root.assoc("does-not-exist").is_none());
    }

    #[test]
    fn unclosed_list_is_a_parse_error_not_a_panic() {
        let result = parse("(a b");
        assert_eq!(result, Err(ParseError::UnexpectedEof));
    }

    #[test]
    fn unexpected_close_paren_is_a_parse_error_not_a_panic() {
        let result = parse(")");
        assert_eq!(result, Err(ParseError::UnexpectedToken(")".to_string())));
    }
}
