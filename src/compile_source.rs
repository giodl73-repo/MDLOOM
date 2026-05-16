use anyhow::Result;
use std::path::Path;

pub(crate) fn resolve_source_for_compile(src: &str, root: &Path) -> Result<String> {
    let (clean_src, query) = split_md_query(src);

    let raw = if clean_src.starts_with("md://") {
        if let Ok(parsed) = mdpath::parse(&clean_src) {
            if let Ok(element) =
                mdpath::resolve_with_classifier(&parsed, root, &mdpath::DefaultClassifier)
            {
                element.content
            } else {
                let path_part = clean_src.strip_prefix("md://").unwrap_or(&clean_src);
                let path = root.join(path_part);
                if !path.exists() {
                    anyhow::bail!(
                        "cannot resolve md:// URI {:?} — file not found and no addressed element",
                        src
                    );
                }
                std::fs::read_to_string(&path)
                    .map_err(|e| anyhow::anyhow!("reading {:?}: {}", path, e))?
            }
        } else {
            let path_part = clean_src.strip_prefix("md://").unwrap_or(&clean_src);
            let path = root.join(path_part);
            std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("reading {:?}: {}", path, e))?
        }
    } else {
        let path = root.join(&clean_src);
        std::fs::read_to_string(&path).map_err(|e| anyhow::anyhow!("reading {:?}: {}", path, e))?
    };

    if query.is_empty() {
        return Ok(raw);
    }
    apply_md_query(&raw, &query)
}

pub(crate) fn split_md_query(src: &str) -> (String, Vec<(String, String)>) {
    if let Some((head, tail)) = src.split_once('?') {
        let pairs: Vec<(String, String)> = tail
            .split('&')
            .filter(|kv| !kv.is_empty())
            .map(|kv| {
                if let Some((k, v)) = kv.split_once('=') {
                    (k.trim().to_string(), v.trim().to_string())
                } else {
                    (kv.trim().to_string(), String::new())
                }
            })
            .collect();
        (head.to_string(), pairs)
    } else {
        (src.to_string(), Vec::new())
    }
}

pub(crate) fn apply_md_query(raw: &str, query: &[(String, String)]) -> Result<String> {
    use crate::tree::schema::parse_md_table;
    let (headers, rows) = parse_md_table(raw)?;
    let mut headers = headers;
    let mut rows = rows;

    for (_, v) in query.iter().filter(|(k, _)| k == "filter") {
        let (col, op, target) = parse_filter_term(v).ok_or_else(|| {
            anyhow::anyhow!(
                "invalid ?filter term {:?} — expected col=val, col!=val, col>val, or col<val",
                v
            )
        })?;
        if !headers.iter().any(|h| h == col) {
            anyhow::bail!("?filter references unknown column {:?}", col);
        }
        rows.retain(|r| {
            let cell = r.get(col).cloned().unwrap_or_default();
            match op {
                FilterOp::Eq => cell == target,
                FilterOp::Neq => cell != target,
                FilterOp::Gt => cell
                    .parse::<f64>()
                    .ok()
                    .zip(target.parse::<f64>().ok())
                    .map_or(false, |(a, b)| a > b),
                FilterOp::Lt => cell
                    .parse::<f64>()
                    .ok()
                    .zip(target.parse::<f64>().ok())
                    .map_or(false, |(a, b)| a < b),
            }
        });
    }

    if let Some((_, n)) = query.iter().find(|(k, _)| k == "skip") {
        let n: usize = n
            .parse()
            .map_err(|_| anyhow::anyhow!("?skip value must be a non-negative integer"))?;
        if n >= rows.len() {
            rows.clear();
        } else {
            rows.drain(0..n);
        }
    }

    if let Some((_, n)) = query.iter().find(|(k, _)| k == "top") {
        let n: usize = n
            .parse()
            .map_err(|_| anyhow::anyhow!("?top value must be a non-negative integer"))?;
        rows.truncate(n);
    }

    if let Some((_, cols_csv)) = query.iter().find(|(k, _)| k == "select") {
        let want: Vec<String> = cols_csv.split(',').map(|s| s.trim().to_string()).collect();
        for c in &want {
            if !headers.iter().any(|h| h == c) {
                anyhow::bail!("?select references unknown column {:?}", c);
            }
        }
        headers = want;
    }

    if query.iter().any(|(k, _)| k == "count") {
        let n = rows.len();
        return Ok(format!("| count |\n|-------|\n| {} |\n", n));
    }

    let mut out = String::new();
    out.push('|');
    for h in &headers {
        out.push_str(&format!(" {} |", h));
    }
    out.push('\n');
    out.push('|');
    for _ in &headers {
        out.push_str("---|");
    }
    out.push('\n');
    for r in &rows {
        out.push('|');
        for h in &headers {
            let cell = r.get(h).cloned().unwrap_or_default();
            out.push_str(&format!(" {} |", cell));
        }
        out.push('\n');
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy)]
enum FilterOp {
    Eq,
    Neq,
    Gt,
    Lt,
}

fn parse_filter_term(term: &str) -> Option<(&str, FilterOp, String)> {
    if let Some((c, t)) = term.split_once("!=") {
        return Some((c.trim(), FilterOp::Neq, t.trim().to_string()));
    }
    if let Some((c, t)) = term.split_once('>') {
        return Some((c.trim(), FilterOp::Gt, t.trim().to_string()));
    }
    if let Some((c, t)) = term.split_once('<') {
        return Some((c.trim(), FilterOp::Lt, t.trim().to_string()));
    }
    if let Some((c, t)) = term.split_once('=') {
        return Some((c.trim(), FilterOp::Eq, t.trim().to_string()));
    }
    None
}
