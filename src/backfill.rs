use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::checks::markdown_table::parse_tables;
use crate::compile::compile_file;
use crate::GlintConfig;

#[derive(Debug, Clone)]
pub struct BackfillOptions {
    pub paths: Vec<PathBuf>,
    pub output_source: PathBuf,
    pub report: PathBuf,
    pub literal_first: bool,
    pub extract_tables: bool,
    pub check_roundtrip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackfillReport {
    pub schema_version: String,
    pub generated_by: String,
    pub summary: BackfillSummary,
    pub files: Vec<BackfillFileReport>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackfillSummary {
    pub files_scanned: usize,
    pub files_generated: usize,
    pub roundtrip_checked: usize,
    pub roundtrip_passed: usize,
    pub roundtrip_failed: usize,
    pub tables_extracted: usize,
    pub blocks: BackfillBlockCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackfillFileReport {
    pub original_path: PathBuf,
    pub generated_path: PathBuf,
    pub classification: String,
    pub confidence: String,
    pub blocks: BackfillBlockCounts,
    pub evidence: Vec<String>,
    pub extractions: Vec<BackfillExtractionReport>,
    pub roundtrip: Option<BackfillRoundTrip>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackfillExtractionReport {
    pub kind: String,
    pub generated_path: PathBuf,
    pub confidence: String,
    pub line: usize,
    pub columns: usize,
    pub rows: usize,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackfillTableDataset {
    pub schema_version: String,
    pub source_markdown: PathBuf,
    pub tables: Vec<BackfillExtractedTable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackfillExtractedTable {
    pub id: String,
    pub line: usize,
    pub heading_context: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackfillBlockCounts {
    pub total: usize,
    pub prose: usize,
    pub fenced: usize,
    pub markdown_tables: usize,
    pub ascii_table_candidates: usize,
    pub chart_like: usize,
    pub diagram_like: usize,
    pub ambiguous: usize,
}

impl BackfillBlockCounts {
    fn add(&mut self, other: &Self) {
        self.total += other.total;
        self.prose += other.prose;
        self.fenced += other.fenced;
        self.markdown_tables += other.markdown_tables;
        self.ascii_table_candidates += other.ascii_table_candidates;
        self.chart_like += other.chart_like;
        self.diagram_like += other.diagram_like;
        self.ambiguous += other.ambiguous;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackfillRoundTrip {
    pub passed: bool,
    pub diff_summary: String,
}

pub fn run(options: BackfillOptions) -> Result<BackfillReport> {
    let mut report = BackfillReport {
        schema_version: "1".to_string(),
        generated_by: "proof backfill".to_string(),
        summary: BackfillSummary::default(),
        files: Vec::new(),
    };

    let inputs = collect_markdown_inputs(&options.paths, &options.output_source)?;
    report.summary.files_scanned = inputs.len();

    for input in inputs {
        let generated_path =
            generated_source_path(&input.base, &input.path, &options.output_source)?;
        let original = std::fs::read_to_string(&input.path)
            .with_context(|| format!("reading {}", input.path.display()))?;
        let relative_original = input
            .path
            .strip_prefix(&input.base)
            .unwrap_or(&input.path)
            .to_path_buf();
        let source = literal_source(&relative_original, &original);
        let inventory = classify_blocks(&original);

        if let Some(parent) = generated_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(&generated_path, source)
            .with_context(|| format!("writing {}", generated_path.display()))?;
        report.summary.files_generated += 1;

        let mut file_report = BackfillFileReport {
            original_path: input.path.clone(),
            generated_path: generated_path.clone(),
            classification: "literal_markdown".to_string(),
            confidence: if options.literal_first {
                "high"
            } else {
                "medium"
            }
            .to_string(),
            blocks: inventory.counts.clone(),
            evidence: inventory.evidence,
            extractions: Vec::new(),
            roundtrip: None,
            notes: vec!["preserved literal markdown body".to_string()],
        };
        report.summary.blocks.add(&file_report.blocks);

        if options.extract_tables {
            let extraction = extract_markdown_tables(&original, &input.path, &generated_path)?;
            report.summary.tables_extracted += extraction.reports.len();
            file_report.extractions.extend(extraction.reports);
        }

        if options.check_roundtrip {
            let roundtrip = check_roundtrip(&generated_path, &original)?;
            report.summary.roundtrip_checked += 1;
            if roundtrip.passed {
                report.summary.roundtrip_passed += 1;
            } else {
                report.summary.roundtrip_failed += 1;
            }
            file_report.roundtrip = Some(roundtrip);
        }

        report.files.push(file_report);
    }

    if let Some(parent) = options.report.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
    }
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(&options.report, json)
        .with_context(|| format!("writing {}", options.report.display()))?;

    Ok(report)
}

#[derive(Debug, Default)]
struct TableExtraction {
    reports: Vec<BackfillExtractionReport>,
}

fn extract_markdown_tables(
    original: &str,
    original_path: &Path,
    generated_path: &Path,
) -> Result<TableExtraction> {
    let lines: Vec<&str> = original.lines().collect();
    let in_code = code_block_mask(&lines);
    let parsed = parse_tables(&lines, &in_code);
    if parsed.is_empty() {
        return Ok(TableExtraction::default());
    }

    let table_path = table_sidecar_path(generated_path)?;
    if let Some(parent) = table_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }

    let mut tables = Vec::new();
    let mut reports = Vec::new();
    for (index, table) in parsed.into_iter().enumerate() {
        let id = format!("table-{}", index + 1);
        let rows = table.body_rows.len();
        let columns = table.headers.len();
        tables.push(BackfillExtractedTable {
            id,
            line: table.line,
            heading_context: table.heading_context,
            headers: trim_cells(table.headers),
            rows: table.body_rows.into_iter().map(trim_cells).collect(),
        });
        reports.push(BackfillExtractionReport {
            kind: "markdown_table".to_string(),
            generated_path: table_path.clone(),
            confidence: "high".to_string(),
            line: table.line,
            columns,
            rows,
            notes: vec!["extracted from markdown pipe table".to_string()],
        });
    }

    let dataset = BackfillTableDataset {
        schema_version: "1".to_string(),
        source_markdown: original_path.to_path_buf(),
        tables,
    };
    let json = serde_json::to_string_pretty(&dataset)?;
    std::fs::write(&table_path, json)
        .with_context(|| format!("writing {}", table_path.display()))?;

    Ok(TableExtraction { reports })
}

fn trim_cells(cells: Vec<String>) -> Vec<String> {
    cells
        .into_iter()
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn table_sidecar_path(generated_path: &Path) -> Result<PathBuf> {
    let Some(file_name) = generated_path.file_name().and_then(|name| name.to_str()) else {
        anyhow::bail!(
            "cannot derive table sidecar name for {}",
            generated_path.display()
        );
    };
    let table_name = file_name
        .strip_suffix(".source.md")
        .map(|stem| format!("{}.tables.json", stem))
        .unwrap_or_else(|| format!("{}.tables.json", file_name));
    let mut path = generated_path.to_path_buf();
    path.set_file_name(table_name);
    Ok(path)
}

fn code_block_mask(lines: &[&str]) -> Vec<bool> {
    let mut mask = vec![false; lines.len()];
    let mut in_block = false;
    let mut fence_char = '`';
    let mut fence_len = 0usize;

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !in_block {
            let ch = trimmed.chars().next();
            if matches!(ch, Some('`') | Some('~')) {
                let c = ch.unwrap();
                let run = trimmed.chars().take_while(|&x| x == c).count();
                if run >= 3 {
                    in_block = true;
                    fence_char = c;
                    fence_len = run;
                }
            }
        } else {
            let ch = trimmed.chars().next();
            if ch == Some(fence_char) {
                let run = trimmed.chars().take_while(|&x| x == fence_char).count();
                if run >= fence_len {
                    in_block = false;
                    continue;
                }
            }
            mask[index] = true;
        }
    }

    mask
}

#[derive(Debug, Default)]
struct BlockInventory {
    counts: BackfillBlockCounts,
    evidence: Vec<String>,
}

fn classify_blocks(markdown: &str) -> BlockInventory {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut inventory = BlockInventory::default();
    let mut index = 0usize;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            index += 1;
            continue;
        }

        if let Some(fence) = fence_marker(trimmed) {
            let start = index + 1;
            let mut body = Vec::new();
            index += 1;
            let mut closed = false;
            while index < lines.len() {
                let inner = lines[index];
                if inner.trim_start().starts_with(fence) {
                    closed = true;
                    index += 1;
                    break;
                }
                body.push(inner);
                index += 1;
            }
            inventory.counts.total += 1;
            inventory.counts.fenced += 1;
            classify_visual_block(&body, start, &mut inventory);
            if !closed {
                inventory.counts.ambiguous += 1;
                inventory
                    .evidence
                    .push(format!("line {}: unclosed fenced block", start));
            }
            continue;
        }

        if is_markdown_table_start(&lines, index) {
            inventory.counts.total += 1;
            inventory.counts.markdown_tables += 1;
            inventory
                .evidence
                .push(format!("line {}: markdown table candidate", index + 1));
            index += 2;
            while index < lines.len() && looks_like_pipe_row(lines[index]) {
                index += 1;
            }
            continue;
        }

        if looks_like_ascii_table_line(line) {
            inventory.counts.total += 1;
            inventory.counts.ascii_table_candidates += 1;
            inventory
                .evidence
                .push(format!("line {}: ASCII table candidate", index + 1));
            index += 1;
            while index < lines.len() && looks_like_ascii_table_line(lines[index]) {
                index += 1;
            }
            continue;
        }

        if looks_chart_like(line) {
            inventory.counts.total += 1;
            inventory.counts.chart_like += 1;
            inventory
                .evidence
                .push(format!("line {}: chart-like block", index + 1));
            index += 1;
            while index < lines.len() && looks_chart_like(lines[index]) {
                index += 1;
            }
            continue;
        }

        if looks_diagram_like(line) {
            inventory.counts.total += 1;
            inventory.counts.diagram_like += 1;
            inventory
                .evidence
                .push(format!("line {}: diagram-like block", index + 1));
            index += 1;
            while index < lines.len() && looks_diagram_like(lines[index]) {
                index += 1;
            }
            continue;
        }

        inventory.counts.total += 1;
        inventory.counts.prose += 1;
        index += 1;
        while index < lines.len()
            && !lines[index].trim().is_empty()
            && fence_marker(lines[index].trim()).is_none()
            && !is_markdown_table_start(&lines, index)
            && !looks_like_ascii_table_line(lines[index])
            && !looks_chart_like(lines[index])
            && !looks_diagram_like(lines[index])
        {
            index += 1;
        }
    }

    inventory
}

fn fence_marker(trimmed: &str) -> Option<&'static str> {
    if trimmed.starts_with("```") {
        Some("```")
    } else if trimmed.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

fn classify_visual_block(body: &[&str], start_line: usize, inventory: &mut BlockInventory) {
    if body.iter().any(|line| looks_like_ascii_table_line(line)) {
        inventory.counts.ascii_table_candidates += 1;
        inventory
            .evidence
            .push(format!("line {}: fenced ASCII table candidate", start_line));
    } else if body.iter().any(|line| looks_chart_like(line)) {
        inventory.counts.chart_like += 1;
        inventory
            .evidence
            .push(format!("line {}: fenced chart-like block", start_line));
    } else if body.iter().any(|line| looks_diagram_like(line)) {
        inventory.counts.diagram_like += 1;
        inventory
            .evidence
            .push(format!("line {}: fenced diagram-like block", start_line));
    }
}

fn is_markdown_table_start(lines: &[&str], index: usize) -> bool {
    index + 1 < lines.len()
        && looks_like_pipe_row(lines[index])
        && looks_like_table_separator(lines[index + 1])
}

fn looks_like_pipe_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.matches('|').count() >= 2
}

fn looks_like_table_separator(line: &str) -> bool {
    let trimmed = line.trim();
    looks_like_pipe_row(trimmed)
        && trimmed.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
        && trimmed.contains("---")
}

fn looks_like_ascii_table_line(line: &str) -> bool {
    let trimmed = line.trim();
    (trimmed.contains('+') && trimmed.contains('-') && trimmed.matches('+').count() >= 2)
        || (trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.matches('|').count() >= 2)
}

fn looks_chart_like(line: &str) -> bool {
    let trimmed = line.trim();
    let bar_count = trimmed
        .chars()
        .filter(|c| matches!(c, '█' | '▓' | '▒' | '░' | '#'))
        .count();
    bar_count >= 3 && trimmed.chars().any(|c| c.is_ascii_digit())
}

fn looks_diagram_like(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains("->")
        || trimmed.contains("-->")
        || trimmed.contains("=>")
        || trimmed.chars().any(|c| {
            matches!(
                c,
                '│' | '─'
                    | '┌'
                    | '┐'
                    | '└'
                    | '┘'
                    | '├'
                    | '┤'
                    | '┬'
                    | '┴'
                    | '┼'
                    | '▶'
                    | '◀'
                    | '→'
                    | '←'
            )
        })
}

fn literal_source(original_path: &Path, original: &str) -> String {
    format!(
        "---\ntags: [backfill]\nops: [backfill]\ncontent_tags: [markdown]\nproof_original: \"{}\"\n---\n{}",
        original_path.display().to_string().replace('\\', "/"),
        original
    )
}

fn check_roundtrip(source_path: &Path, original: &str) -> Result<BackfillRoundTrip> {
    let temp_dir = unique_temp_dir()?;
    let output_path = temp_dir.join("roundtrip.md");
    let cfg = GlintConfig::default();
    compile_file(source_path, &output_path, &temp_dir, &cfg)
        .with_context(|| format!("round-trip compiling {}", source_path.display()))?;
    let compiled = std::fs::read_to_string(&output_path)
        .with_context(|| format!("reading {}", output_path.display()))?;
    let passed = compiled == original;
    let diff_summary = if passed {
        "identical".to_string()
    } else {
        format!(
            "compiled output differs: original {} bytes, compiled {} bytes",
            original.len(),
            compiled.len()
        )
    };
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(BackfillRoundTrip {
        passed,
        diff_summary,
    })
}

fn unique_temp_dir() -> Result<PathBuf> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("proof-backfill-{}-{}", std::process::id(), nanos));
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    Ok(dir)
}

#[derive(Debug)]
struct MarkdownInput {
    base: PathBuf,
    path: PathBuf,
}

fn collect_markdown_inputs(paths: &[PathBuf], output_source: &Path) -> Result<Vec<MarkdownInput>> {
    let mut inputs = Vec::new();
    for path in paths {
        if path.is_dir() {
            for entry in walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_file())
            {
                let candidate = entry.path();
                if is_backfillable_markdown(candidate) && !candidate.starts_with(output_source) {
                    inputs.push(MarkdownInput {
                        base: path.clone(),
                        path: candidate.to_path_buf(),
                    });
                }
            }
        } else if is_backfillable_markdown(path) {
            inputs.push(MarkdownInput {
                base: path.parent().unwrap_or_else(|| Path::new("")).to_path_buf(),
                path: path.clone(),
            });
        }
    }
    inputs.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(inputs)
}

fn is_backfillable_markdown(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    path.extension().and_then(|ext| ext.to_str()) == Some("md") && !name.ends_with(".source.md")
}

fn generated_source_path(base: &Path, original: &Path, output_source: &Path) -> Result<PathBuf> {
    let relative = original.strip_prefix(base).unwrap_or(original);
    let mut generated = output_source.join(relative);
    let Some(name) = generated.file_name().and_then(|name| name.to_str()) else {
        anyhow::bail!(
            "cannot derive generated source name for {}",
            original.display()
        );
    };
    let source_name = name
        .strip_suffix(".md")
        .map(|stem| format!("{}.source.md", stem))
        .unwrap_or_else(|| format!("{}.source.md", name));
    generated.set_file_name(source_name);
    Ok(generated)
}
