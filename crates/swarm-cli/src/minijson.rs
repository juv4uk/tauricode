//! Minimal strict JSON reader for adapter inputs (GitHub Issues export,
//! generic task records). External data JUSTIFIES a real parser here, but
//! the crate keeps zero dependencies by scoping it hard: objects, arrays,
//! strings with standard escapes, numbers (f64), true/false/null. Anything
//! else is an Err, never silently accepted.

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_arr(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }
}

pub fn parse(input: &str) -> Result<Json, String> {
    let b: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    skip_ws(&b, &mut i);
    let v = value(&b, &mut i)?;
    skip_ws(&b, &mut i);
    if i != b.len() {
        return Err(format!("trailing content at char {i}"));
    }
    Ok(v)
}

fn skip_ws(b: &[char], i: &mut usize) {
    while *i < b.len() && matches!(b[*i], ' ' | '\t' | '\r' | '\n') {
        *i += 1;
    }
}

const ERR_END: &str = "unexpected end of JSON";

fn value(b: &[char], i: &mut usize) -> Result<Json, String> {
    skip_ws(b, i);
    if *i >= b.len() {
        return Err(ERR_END.to_string());
    }
    match b[*i] {
        '{' => {
            *i += 1;
            let mut pairs = Vec::new();
            skip_ws(b, i);
            if *i < b.len() && b[*i] == '}' {
                *i += 1;
                return Ok(Json::Obj(pairs));
            }
            loop {
                skip_ws(b, i);
                let k = match string(b, i)? {
                    Json::Str(s) => s,
                    _ => unreachable!(),
                };
                skip_ws(b, i);
                if *i >= b.len() || b[*i] != ':' {
                    return Err("expected ':'".into());
                }
                *i += 1;
                let v = value(b, i)?;
                pairs.push((k, v));
                skip_ws(b, i);
                match b.get(*i) {
                    Some(',') => *i += 1,
                    Some('}') => {
                        *i += 1;
                        return Ok(Json::Obj(pairs));
                    }
                    _ => return Err("expected ',' or '}'".into()),
                }
            }
        }
        '[' => {
            *i += 1;
            let mut items = Vec::new();
            skip_ws(b, i);
            if *i < b.len() && b[*i] == ']' {
                *i += 1;
                return Ok(Json::Arr(items));
            }
            loop {
                items.push(value(b, i)?);
                skip_ws(b, i);
                match b.get(*i) {
                    Some(',') => *i += 1,
                    Some(']') => {
                        *i += 1;
                        return Ok(Json::Arr(items));
                    }
                    _ => return Err("expected ',' or ']'".into()),
                }
            }
        }
        '"' => string(b, i),
        't' => lit(b, i, "true", Json::Bool(true)),
        'f' => lit(b, i, "false", Json::Bool(false)),
        'n' => lit(b, i, "null", Json::Null),
        _ => number(b, i),
    }
}

fn lit(b: &[char], i: &mut usize, word: &str, v: Json) -> Result<Json, String> {
    for c in word.chars() {
        if b.get(*i) != Some(&c) {
            return Err(format!("expected `{word}`"));
        }
        *i += 1;
    }
    Ok(v)
}

fn string(b: &[char], i: &mut usize) -> Result<Json, String> {
    if b.get(*i) != Some(&'"') {
        return Err("expected string".into());
    }
    *i += 1;
    let mut out = String::new();
    loop {
        if *i >= b.len() {
            return Err(ERR_END.to_string());
        }
        match b[*i] {
            '"' => {
                *i += 1;
                return Ok(Json::Str(out));
            }
            '\\' => {
                *i += 1;
                match b.get(*i) {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('/') => out.push('/'),
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('b') => out.push('\u{8}'),
                    Some('f') => out.push('\u{c}'),
                    Some('u') => {
                        let hex: String = b[*i + 1..*i + 5].iter().collect();
                        let cp = u32::from_str_radix(&hex, 16).map_err(|_| "bad \\u escape")?;
                        out.push(char::from_u32(cp).unwrap_or('\u{fffd}'));
                        *i += 4;
                    }
                    _ => return Err("bad escape".into()),
                }
                *i += 1;
            }
            c => {
                out.push(c);
                *i += 1;
            }
        }
    }
}

fn number(b: &[char], i: &mut usize) -> Result<Json, String> {
    let start = *i;
    if b.get(*i) == Some(&'-') || b.get(*i) == Some(&'+') {
        *i += 1;
    }
    while *i < b.len() && (b[*i].is_ascii_digit() || matches!(b[*i], '.' | 'e' | 'E' | '+' | '-')) {
        *i += 1;
    }
    if start == *i {
        return Err(format!("unexpected character at {start}"));
    }
    let s: String = b[start..*i].iter().collect();
    s.parse::<f64>().map(Json::Num).map_err(|_| format!("bad number `{s}`"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gh_issue_like_object() {
        let j = parse(
            r#"{"number":42,"title":"fix \"it\"","labels":[{"name":"rust"},{"name":"docs"}],"state":"open","body":"line1\nline2"}"#,
        )
        .unwrap();
        assert_eq!(j.get("number").and_then(|v| v.as_f64()), Some(42.0));
        assert_eq!(j.get("title").and_then(|v| v.as_str()), Some("fix \"it\""));
        assert_eq!(j.get("labels").unwrap().as_arr().unwrap().len(), 2);
        assert_eq!(
            j.get("body").and_then(|v| v.as_str()),
            Some("line1\nline2")
        );
    }

    #[test]
    fn rejects_garbage_strictly() {
        assert!(parse("{").is_err());
        assert!(parse("[1,]").is_err());
        assert!(parse("{} trailing").is_err());
        assert!(parse("{'a':1}").is_err());
    }

    #[test]
    fn scalars_and_null() {
        assert_eq!(parse("null").unwrap(), Json::Null);
        assert_eq!(parse("true").unwrap(), Json::Bool(true));
        assert_eq!(parse("-2.5e2").unwrap(), Json::Num(-250.0));
    }
}
