//! Recursive-descent YAML parser for the process-compose schema.

use super::convert::to_config;
use super::scalar::{parse_scalar, split_map_entry};
use super::value::{expect_map, tokenize, Line, Yaml};

pub(super) struct Parser<'a> {
    lines: &'a [Line],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub(super) fn new(lines: &'a [Line]) -> Self {
        Self { lines, pos: 0 }
    }

    fn peek(&self) -> Option<&'a Line> {
        self.lines.get(self.pos)
    }

    fn consume(&mut self) -> Option<&'a Line> {
        let l = self.lines.get(self.pos)?;
        self.pos += 1;
        Some(l)
    }

    /// Parse a block (map or sequence) whose first child sits at or below
    /// `min_indent`. Returns Null when no children exist at that indent.
    pub(super) fn parse_block(&mut self, min_indent: usize) -> Result<Yaml, String> {
        let first = match self.peek() {
            Some(l) if l.indent >= min_indent => l,
            _ => return Ok(Yaml::Null),
        };
        if first.text.trim_start().starts_with('-') {
            self.parse_seq(first.indent)
        } else {
            self.parse_map(first.indent)
        }
    }

    fn parse_map(&mut self, indent: usize) -> Result<Yaml, String> {
        let mut entries: Vec<(String, Yaml)> = Vec::new();
        loop {
            let line = match self.peek() {
                Some(l) if l.indent == indent => self.consume().unwrap(),
                _ => break,
            };
            if line.text.trim_start().starts_with('-') {
                self.pos -= 1;
                break;
            }
            let (key, value_part) = split_map_entry(&line.text)?;
            if value_part.is_empty() {
                if let Some(next) = self.peek() {
                    if next.indent > indent {
                        let val = self.parse_block(next.indent)?;
                        entries.push((key, val));
                        continue;
                    }
                    if next.indent == indent && next.text.trim_start().starts_with('-') {
                        let val = self.parse_seq(next.indent)?;
                        entries.push((key, val));
                        continue;
                    }
                }
                entries.push((key, Yaml::Null));
            } else {
                entries.push((key, parse_scalar(value_part)?));
            }
        }
        Ok(Yaml::Map(entries))
    }

    fn parse_seq(&mut self, indent: usize) -> Result<Yaml, String> {
        let mut items = Vec::new();
        loop {
            let line = match self.peek() {
                Some(l) if l.indent == indent => self.consume().unwrap(),
                _ => break,
            };
            if !line.text.trim_start().starts_with('-') {
                self.pos -= 1;
                break;
            }
            let mut rest = line.text.trim_start();
            debug_assert!(rest.starts_with('-'));
            rest = &rest[1..];
            if rest.starts_with(' ') {
                rest = &rest[1..];
            } else if !rest.is_empty() {
                return Err(format!("expected '- value' at: {}", line.text));
            }
            if rest.is_empty() {
                if let Some(next) = self.peek() {
                    if next.indent > indent {
                        items.push(self.parse_block(next.indent)?);
                        continue;
                    }
                }
                items.push(Yaml::Null);
            } else {
                items.push(parse_scalar(rest)?);
            }
        }
        Ok(Yaml::Seq(items))
    }

    pub(super) fn parse_root(&mut self) -> Result<Yaml, String> {
        self.parse_block(0)
    }
}

/// Parse canonical process-compose YAML into a `ProcessComposeConfig`.
pub fn parse_config(content: &str) -> Result<crate::config::ProcessComposeConfig, String> {
    let lines = tokenize(content);
    let mut p = Parser::new(&lines);
    let root = p.parse_root()?;
    let entries = expect_map(root, "root")?;
    if entries.is_empty() {
        return Ok(crate::config::ProcessComposeConfig {
            version: "0.5".to_string(),
            environment: None,
            log_configuration: None,
            processes: std::collections::HashMap::new(),
        });
    }
    to_config(Yaml::Map(entries))
}
