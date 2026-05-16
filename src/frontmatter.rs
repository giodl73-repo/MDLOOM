use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SourceFrontmatter {
    pub tags: Vec<String>,
    pub ops: Vec<String>,
    pub content: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFrontmatter<'a> {
    pub metadata: SourceFrontmatter,
    pub body: &'a str,
    pub body_start_line: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrontmatterTagCounts {
    pub files_with_frontmatter: usize,
    pub files_with_tags: usize,
    pub tags: BTreeMap<String, usize>,
    pub ops: BTreeMap<String, usize>,
    pub content: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrontmatterFilter {
    pub tags: Vec<String>,
    pub ops: Vec<String>,
    pub content: Vec<String>,
}

impl SourceFrontmatter {
    pub fn has_tags(&self) -> bool {
        !(self.tags.is_empty() && self.ops.is_empty() && self.content.is_empty())
    }
}

impl FrontmatterFilter {
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty() && self.ops.is_empty() && self.content.is_empty()
    }

    pub fn matches_path(&self, path: &Path) -> bool {
        if self.is_empty() {
            return true;
        }
        read(path)
            .ok()
            .flatten()
            .is_some_and(|metadata| self.matches_metadata(&metadata))
    }

    pub fn matches_metadata(&self, metadata: &SourceFrontmatter) -> bool {
        contains_all(&metadata.tags, &self.tags)
            && contains_all(&metadata.ops, &self.ops)
            && contains_all(&metadata.content, &self.content)
    }
}

impl FrontmatterTagCounts {
    pub fn add(&mut self, metadata: &SourceFrontmatter) {
        self.files_with_frontmatter += 1;
        if metadata.has_tags() {
            self.files_with_tags += 1;
        }
        increment_all(&mut self.tags, &metadata.tags);
        increment_all(&mut self.ops, &metadata.ops);
        increment_all(&mut self.content, &metadata.content);
    }

    pub fn from_files(files: &[PathBuf]) -> Self {
        let mut counts = Self::default();
        for path in files {
            if let Some(parsed) = read(path).ok().flatten() {
                counts.add(&parsed);
            }
        }
        counts
    }
}

pub fn parse(source: &str) -> Option<ParsedFrontmatter<'_>> {
    let mut lines = source.split_inclusive('\n');
    let first = lines.next()?;
    if first.trim_end_matches(['\r', '\n']) != "---" {
        return None;
    }

    let mut block = String::new();
    let mut consumed = first.len();
    let mut consumed_lines = 1usize;

    for line in lines {
        consumed += line.len();
        consumed_lines += 1;
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return Some(ParsedFrontmatter {
                metadata: parse_block(&block),
                body: &source[consumed..],
                body_start_line: consumed_lines + 1,
            });
        }
        block.push_str(line);
    }

    None
}

pub fn read(path: &Path) -> std::io::Result<Option<SourceFrontmatter>> {
    let source = std::fs::read_to_string(path)?;
    Ok(parse(&source).map(|parsed| parsed.metadata))
}

fn parse_block(block: &str) -> SourceFrontmatter {
    let mut metadata = SourceFrontmatter::default();
    let mut active_key: Option<Field> = None;

    for raw in block.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(item) = line.strip_prefix("- ") {
            if let Some(field) = active_key {
                push_values(&mut metadata, field, parse_values(item));
            }
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            active_key = None;
            continue;
        };
        let Some(field) = Field::from_key(key) else {
            active_key = None;
            continue;
        };

        active_key = Some(field);
        let value = value.trim();
        if !value.is_empty() {
            push_values(&mut metadata, field, parse_values(value));
        }
    }

    metadata
}

fn push_values(metadata: &mut SourceFrontmatter, field: Field, values: Vec<String>) {
    let target = match field {
        Field::Tags => &mut metadata.tags,
        Field::Ops => &mut metadata.ops,
        Field::Content => &mut metadata.content,
    };

    for value in values {
        if !target.contains(&value) {
            target.push(value);
        }
    }
}

fn parse_values(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .unwrap_or(trimmed);

    inner
        .split(',')
        .filter_map(clean_value)
        .filter(|v| !v.is_empty())
        .collect()
}

fn clean_value(value: &str) -> Option<String> {
    let without_comment = value.split_once(" #").map_or(value, |(head, _)| head);
    let cleaned = without_comment
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn increment_all(counts: &mut BTreeMap<String, usize>, values: &[String]) {
    for value in values {
        *counts.entry(value.clone()).or_default() += 1;
    }
}

fn contains_all(haystack: &[String], needles: &[String]) -> bool {
    needles.iter().all(|needle| haystack.contains(needle))
}

#[derive(Clone, Copy)]
enum Field {
    Tags,
    Ops,
    Content,
}

impl Field {
    fn from_key(key: &str) -> Option<Self> {
        match key.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "tags" | "tag" => Some(Self::Tags),
            "ops" | "op" | "operations" => Some(Self::Ops),
            "content" | "content_tags" | "content_tag" => Some(Self::Content),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_and_block_tags() {
        let source = "---\ntags: [ops, runbook]\nops:\n  - lint\ncontent: guide\n---\n# Body\n";

        let parsed = parse(source).expect("frontmatter");

        assert_eq!(parsed.metadata.tags, vec!["ops", "runbook"]);
        assert_eq!(parsed.metadata.ops, vec!["lint"]);
        assert_eq!(parsed.metadata.content, vec!["guide"]);
        assert_eq!(parsed.body, "# Body\n");
        assert_eq!(parsed.body_start_line, 7);
    }

    #[test]
    fn ignores_unclosed_or_absent_frontmatter() {
        assert!(parse("# Body\n").is_none());
        assert!(parse("---\ntags: [ops]\n# Body\n").is_none());
    }

    #[test]
    fn frontmatter_filter_requires_requested_fields() {
        let metadata = SourceFrontmatter {
            tags: vec!["ops".to_string(), "runbook".to_string()],
            ops: vec!["compile".to_string()],
            content: vec!["guide".to_string()],
        };

        assert!(FrontmatterFilter {
            tags: vec!["runbook".to_string()],
            ops: vec!["compile".to_string()],
            content: vec![]
        }
        .matches_metadata(&metadata));
        assert!(!FrontmatterFilter {
            tags: vec!["runbook".to_string()],
            ops: vec!["lint".to_string()],
            content: vec![]
        }
        .matches_metadata(&metadata));
    }
}
