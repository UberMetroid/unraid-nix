//! Scalar parsing helpers — used by the YAML parser to interpret individual
//! key/value text fragments.

pub(super) fn find_unquoted_colon(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut in_single = false;
    let mut in_double = false;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\'' && !in_double {
            in_single = !in_single;
        } else if c == b'"' && !in_single {
            in_double = !in_double;
        } else if c == b':' && !in_single && !in_double {
            let next = bytes.get(i + 1).copied();
            if matches!(next, None | Some(b' ') | Some(b'\t')) {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

pub(super) fn split_map_entry(text: &str) -> Result<(String, &str), String> {
    let trimmed = text.trim_start();
    let colon_rel = find_unquoted_colon(trimmed)
        .ok_or_else(|| format!("expected 'key:' in '{}'", trimmed))?;
    let key = trimmed[..colon_rel].trim().to_string();
    let after = trimmed[colon_rel + 1..].trim_start();
    Ok((key, after))
}

pub(super) fn strip_trailing_comment(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    for (i, &c) in bytes.iter().enumerate() {
        match c {
            b'\'' if !in_double => in_single = !in_single,
            b'"' if !in_single => in_double = !in_double,
            b'#' if !in_single && !in_double && (i == 0 || bytes[i - 1] == b' ') => {
                return &s[..i];
            }
            _ => {}
        }
    }
    s
}

pub(super) fn unquote_single(rest: &str) -> Result<String, String> {
    if !rest.ends_with('\'') {
        return Err("unterminated single-quoted string".to_string());
    }
    let body = &rest[..rest.len() - 1];
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\'' {
            if chars.peek() == Some(&'\'') {
                out.push('\'');
                chars.next();
            } else {
                return Err("unterminated single-quoted string".to_string());
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

pub(super) fn unquote_double(rest: &str) -> Result<String, String> {
    if !rest.ends_with('"') {
        return Err("unterminated double-quoted string".to_string());
    }
    let body = &rest[..rest.len() - 1];
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('/') => out.push('/'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => return Err("dangling backslash in double-quoted string".to_string()),
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

pub(super) fn parse_scalar(s: &str) -> Result<super::value::Yaml, String> {
    use super::value::Yaml;
    let s = s.trim();
    if s.is_empty() {
        return Ok(Yaml::Null);
    }
    let stripped = strip_trailing_comment(s);
    if stripped.is_empty() {
        return Ok(Yaml::Null);
    }
    let bytes = stripped.as_bytes();
    if bytes[0] == b'\'' {
        return Ok(Yaml::Str(unquote_single(&stripped[1..])?));
    }
    if bytes[0] == b'"' {
        return Ok(Yaml::Str(unquote_double(&stripped[1..])?));
    }
    match stripped.to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" => return Ok(Yaml::Bool(true)),
        "false" | "no" | "off" => return Ok(Yaml::Bool(false)),
        "null" | "~" => return Ok(Yaml::Null),
        _ => {}
    }
    if let Ok(n) = stripped.parse::<i64>() {
        return Ok(Yaml::Int(n));
    }
    Ok(Yaml::Str(stripped.to_string()))
}