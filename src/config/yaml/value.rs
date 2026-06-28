//! Generic YAML value tree used by the parser, plus typed-coercion helpers.

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Yaml {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Map(Vec<(String, Yaml)>),
    Seq(Vec<Yaml>),
}

pub(super) struct Line {
    pub indent: usize,
    pub text: String,
}

pub(super) fn tokenize(content: &str) -> Vec<Line> {
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|raw| {
            let indent = raw.len() - raw.trim_start_matches(' ').len();
            Line {
                indent,
                text: raw[indent..].to_string(),
            }
        })
        .collect()
}

pub(super) fn expect_map(y: Yaml, ctx: &str) -> Result<Vec<(String, Yaml)>, String> {
    match y {
        Yaml::Map(m) => Ok(m),
        Yaml::Null => Ok(Vec::new()),
        other => Err(format!("expected map at {}, got {:?}", ctx, other)),
    }
}

pub(super) fn expect_string(y: &Yaml, ctx: &str) -> Result<String, String> {
    match y {
        Yaml::Str(s) => Ok(s.clone()),
        Yaml::Int(n) => Ok(n.to_string()),
        Yaml::Bool(b) => Ok(b.to_string()),
        Yaml::Null => Ok(String::new()),
        other => Err(format!("expected string at {}, got {:?}", ctx, other)),
    }
}

pub(super) fn expect_string_seq(y: &Yaml, ctx: &str) -> Result<Vec<String>, String> {
    match y {
        Yaml::Seq(items) => items
            .iter()
            .enumerate()
            .map(|(i, item)| expect_string(item, &format!("{}[{}]", ctx, i)))
            .collect(),
        Yaml::Null => Ok(Vec::new()),
        other => Err(format!("expected sequence at {}, got {:?}", ctx, other)),
    }
}

pub(super) fn expect_bool(y: &Yaml, ctx: &str) -> Result<bool, String> {
    match y {
        Yaml::Bool(b) => Ok(*b),
        Yaml::Str(s) => match s.to_ascii_lowercase().as_str() {
            "true" | "yes" | "on" => Ok(true),
            "false" | "no" | "off" => Ok(false),
            _ => Err(format!("expected bool at {}, got '{}'", ctx, s)),
        },
        _ => Err(format!("expected bool at {}, got {:?}", ctx, y)),
    }
}

pub(super) fn expect_u64(y: &Yaml, ctx: &str) -> Result<u64, String> {
    match y {
        Yaml::Int(n) if *n >= 0 => Ok(*n as u64),
        Yaml::Str(s) => s
            .parse::<u64>()
            .map_err(|e| format!("expected u64 at {}: {} ({})", ctx, s, e)),
        other => Err(format!("expected u64 at {}, got {:?}", ctx, other)),
    }
}

pub(super) fn expect_u32(y: &Yaml, ctx: &str) -> Result<u32, String> {
    match y {
        Yaml::Int(n) if *n >= 0 && *n <= u32::MAX as i64 => Ok(*n as u32),
        Yaml::Str(s) => s
            .parse::<u32>()
            .map_err(|e| format!("expected u32 at {}: {} ({})", ctx, s, e)),
        other => Err(format!("expected u32 at {}, got {:?}", ctx, other)),
    }
}
