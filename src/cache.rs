/// Three-tier content-addressed build cache for proof compile.
///
/// Tier 1 (parse): ParsedDocument per .md file, keyed by content hash + proof version
/// Tier 2 (resolve): ResolvedElement per md:// URI, keyed by target parse_key + URI + version
/// Tier 3 (compile): Full compiled output per source document
///
/// All tiers live in `.proof/cache/` at the proof root.
/// See design/THREE-TIER-CACHE.md for full spec.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// ─────────────────────────────────────────────────────────
// Cache root
// ─────────────────────────────────────────────────────────

pub fn cache_dir(root: &Path) -> PathBuf {
    root.join(".proof").join("cache")
}

fn parse_dir(root: &Path) -> PathBuf {
    cache_dir(root).join("parse")
}
fn resolve_dir(root: &Path) -> PathBuf {
    cache_dir(root).join("resolve")
}
fn compile_dir(root: &Path) -> PathBuf {
    cache_dir(root).join("compile")
}

// ─────────────────────────────────────────────────────────
// Hashing utilities
// ─────────────────────────────────────────────────────────

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Stable hex key from multiple string inputs (length-prefixed concatenation).
/// Not cryptographic — but stable and collision-resistant enough for a build cache.
pub fn compute_key(parts: &[&str]) -> String {
    let mut h = DefaultHasher::new();
    for part in parts {
        (part.len() as u64).hash(&mut h);
        part.hash(&mut h);
    }
    format!("{:016x}", h.finish())
}

/// Hash file content to a hex string.
pub fn hash_file_content(content: &str) -> String {
    let mut h = DefaultHasher::new();
    content.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn proof_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ─────────────────────────────────────────────────────────
// Path index (Tier 1 reverse lookup)
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PathIndexEntry {
    pub parse_key: String,
    pub mtime_ms: u64,
    pub size: u64,
    pub content_hash: String,
}

pub type PathIndex = HashMap<String, PathIndexEntry>;

pub fn load_path_index(root: &Path) -> PathIndex {
    let path = cache_dir(root).join("parse-index.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_path_index(root: &Path, index: &PathIndex) {
    let path = cache_dir(root).join("parse-index.json");
    if let Ok(json) = serde_json::to_string_pretty(index) {
        let _ = std::fs::create_dir_all(cache_dir(root));
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, &json).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

/// Get or compute the parse_key for a file.
/// Uses mtime+size as a fast-path check before re-hashing.
pub fn get_or_compute_parse_key(file_path: &Path, content: &str, index: &mut PathIndex) -> String {
    let rel = file_path.to_string_lossy().to_string();
    let content_hash = hash_file_content(content);
    let parse_key = compute_key(&[&content_hash, proof_version()]);

    // Check if index entry matches
    if let Some(entry) = index.get(&rel) {
        if entry.content_hash == content_hash {
            return entry.parse_key.clone();
        }
    }

    // Update index
    let mtime_ms = file_path
        .metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let size = file_path.metadata().map(|m| m.len()).unwrap_or(0);

    index.insert(
        rel,
        PathIndexEntry {
            parse_key: parse_key.clone(),
            mtime_ms,
            size,
            content_hash,
        },
    );

    parse_key
}

// ─────────────────────────────────────────────────────────
// Tier 3: Compile cache
// ─────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CompileCacheEntry {
    pub compile_key: String,
    pub source_path: String,
    pub output_path: String,
    pub compiled_text: String,
    pub resolved_uris: Vec<String>,
    pub proof_version: String,
    pub created_at: u64,
}

/// Compute the Tier 3 compile key.
/// `source_parse_key`: parse_key of the source document
/// `resolve_keys`: parse_keys of all referenced data/figure files, in source order, NOT deduplicated
/// `directive_attrs_json`: stable JSON of all directive attributes affecting output
pub fn compile_key(
    source_parse_key: &str,
    resolve_keys: &[String],
    directive_attrs_json: &str,
) -> String {
    let mut parts: Vec<&str> = vec![source_parse_key, directive_attrs_json, proof_version()];
    let resolve_joined: String = resolve_keys.join("|");
    parts.push(&resolve_joined);
    // Re-borrow to avoid lifetime issues
    compute_key(&[
        source_parse_key,
        &resolve_joined,
        directive_attrs_json,
        proof_version(),
    ])
}

pub fn load_compile_cache(root: &Path, key: &str) -> Option<CompileCacheEntry> {
    let path = compile_dir(root).join(format!("{}.json", key));
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_compile_cache(root: &Path, entry: &CompileCacheEntry) {
    let _ = std::fs::create_dir_all(compile_dir(root));
    let path = compile_dir(root).join(format!("{}.json", entry.compile_key));
    if let Ok(json) = serde_json::to_string(entry) {
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, &json).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

/// Try to serve a compile from Tier 3 cache.
/// Returns the compiled text if hit, None if miss.
pub fn try_compile_cache_hit(
    root: &Path,
    source_path: &Path,
    source_content: &str,
    resolved_file_contents: &[(String, String)], // (rel_path, content)
    directive_attrs_json: &str,
    index: &mut PathIndex,
) -> Option<String> {
    let source_parse_key = get_or_compute_parse_key(source_path, source_content, index);
    let resolve_keys: Vec<String> = resolved_file_contents
        .iter()
        .map(|(p, c)| {
            let path = root.join(p);
            get_or_compute_parse_key(&path, c, index)
        })
        .collect();
    let key = compile_key(&source_parse_key, &resolve_keys, directive_attrs_json);
    let entry = load_compile_cache(root, &key)?;
    Some(entry.compiled_text)
}

/// Store a compile result to Tier 3 cache.
pub fn store_compile_cache(
    root: &Path,
    source_path: &Path,
    output_path: &Path,
    source_content: &str,
    resolved_file_contents: &[(String, String)],
    directive_attrs_json: &str,
    compiled_text: &str,
    resolved_uris: Vec<String>,
    index: &mut PathIndex,
) {
    let source_parse_key = get_or_compute_parse_key(source_path, source_content, index);
    let resolve_keys: Vec<String> = resolved_file_contents
        .iter()
        .map(|(p, c)| {
            let path = root.join(p);
            get_or_compute_parse_key(&path, c, index)
        })
        .collect();
    let key = compile_key(&source_parse_key, &resolve_keys, directive_attrs_json);
    let entry = CompileCacheEntry {
        compile_key: key,
        source_path: source_path.to_string_lossy().to_string(),
        output_path: output_path.to_string_lossy().to_string(),
        compiled_text: compiled_text.to_string(),
        resolved_uris,
        proof_version: proof_version().to_string(),
        created_at: epoch_ms(),
    };
    save_compile_cache(root, &entry);
}

// ─────────────────────────────────────────────────────────
// Tier 2: Resolve cache
// ─────────────────────────────────────────────────────────
//
// Caches the resolved content for a `md://` URI.
// Key: (parse_key_of_target_file, uri_string, proof_version)
// Value: the resolved figure content string.
//
// When the same figure is referenced by multiple source files in one
// `proof compile` run, each call after the first is a disk-cache hit — the
// figure file is read and parsed only once across the entire build.

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ResolveCacheEntry {
    pub resolve_key: String,
    pub uri: String,
    pub target_parse_key: String,
    pub content: String, // resolved figure content
    pub proof_version: String,
    pub created_at: u64,
}

/// Compute the Tier 2 resolve key.
pub fn resolve_key(target_parse_key: &str, uri: &str) -> String {
    compute_key(&[target_parse_key, uri, proof_version()])
}

/// Load a Tier 2 resolve cache entry. Returns `None` on miss.
pub fn load_resolve_cache(root: &Path, key: &str) -> Option<ResolveCacheEntry> {
    let path = resolve_dir(root).join(format!("{}.json", key));
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Store a Tier 2 resolve cache entry.
pub fn save_resolve_cache(root: &Path, entry: &ResolveCacheEntry) {
    let _ = std::fs::create_dir_all(resolve_dir(root));
    let path = resolve_dir(root).join(format!("{}.json", entry.resolve_key));
    if let Ok(json) = serde_json::to_string(entry) {
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, &json).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

/// Try to serve a resolve from Tier 2 cache.
/// Returns the resolved content string if hit.
pub fn try_resolve_cache_hit(
    root: &Path,
    target_path: &Path,
    target_content: &str,
    uri: &str,
    index: &mut PathIndex,
) -> Option<String> {
    let target_parse_key = get_or_compute_parse_key(target_path, target_content, index);
    let key = resolve_key(&target_parse_key, uri);
    let entry = load_resolve_cache(root, &key)?;
    Some(entry.content)
}

/// Store a resolve result to Tier 2 cache.
pub fn store_resolve_cache(
    root: &Path,
    target_path: &Path,
    target_content: &str,
    uri: &str,
    content: &str,
    index: &mut PathIndex,
) {
    let target_parse_key = get_or_compute_parse_key(target_path, target_content, index);
    let key = resolve_key(&target_parse_key, uri);
    let entry = ResolveCacheEntry {
        resolve_key: key,
        uri: uri.to_string(),
        target_parse_key,
        content: content.to_string(),
        proof_version: proof_version().to_string(),
        created_at: epoch_ms(),
    };
    save_resolve_cache(root, &entry);
}

// ─────────────────────────────────────────────────────────
// Cache pruning
// ─────────────────────────────────────────────────────────

/// Remove cache entries older than `max_age_days`. Returns count removed.
pub fn prune_cache(root: &Path, max_age_days: u64) -> usize {
    let cutoff = epoch_ms().saturating_sub(max_age_days * 24 * 3600 * 1000);
    let mut removed = 0;
    for tier_dir in [parse_dir(root), resolve_dir(root), compile_dir(root)] {
        let Ok(entries) = std::fs::read_dir(&tier_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            // Read created_at from the JSON
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if val
                        .get("created_at")
                        .and_then(|v| v.as_u64())
                        .map(|ts| ts < cutoff)
                        .unwrap_or(true)
                    {
                        let _ = std::fs::remove_file(&path);
                        removed += 1;
                    }
                }
            }
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_key_stable() {
        let k1 = compute_key(&["hello", "world"]);
        let k2 = compute_key(&["hello", "world"]);
        assert_eq!(k1, k2);
    }

    #[test]
    fn compute_key_order_matters() {
        let k1 = compute_key(&["hello", "world"]);
        let k2 = compute_key(&["world", "hello"]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn compute_key_length_prefix_prevents_collision() {
        // "ab" + "c" should differ from "a" + "bc"
        let k1 = compute_key(&["ab", "c"]);
        let k2 = compute_key(&["a", "bc"]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn path_index_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let _ = std::fs::create_dir_all(cache_dir(root));
        let mut index = PathIndex::new();
        index.insert(
            "foo.md".to_string(),
            PathIndexEntry {
                parse_key: "abc".to_string(),
                mtime_ms: 123,
                size: 456,
                content_hash: "def".to_string(),
            },
        );
        save_path_index(root, &index);
        let loaded = load_path_index(root);
        assert_eq!(loaded.get("foo.md").unwrap().parse_key, "abc");
    }

    #[test]
    fn compile_cache_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let entry = CompileCacheEntry {
            compile_key: "testkey".to_string(),
            source_path: "src/foo.source.md".to_string(),
            output_path: "docs/foo.md".to_string(),
            compiled_text: "# Hello\n".to_string(),
            resolved_uris: vec!["md://data.md".to_string()],
            proof_version: proof_version().to_string(),
            created_at: epoch_ms(),
        };
        save_compile_cache(root, &entry);
        let loaded = load_compile_cache(root, "testkey").unwrap();
        assert_eq!(loaded.compiled_text, "# Hello\n");
    }

    #[test]
    fn compile_key_not_deduplicated() {
        // Same URI twice should produce a different key than once
        let k1 = compile_key("parse1", &["r1".to_string(), "r1".to_string()], "{}");
        let k2 = compile_key("parse1", &["r1".to_string()], "{}");
        assert_ne!(k1, k2, "resolve_keys must not be deduplicated (spec F20)");
    }

    #[test]
    fn compile_key_order_matters() {
        let k1 = compile_key("p", &["a".to_string(), "b".to_string()], "{}");
        let k2 = compile_key("p", &["b".to_string(), "a".to_string()], "{}");
        assert_ne!(k1, k2, "resolve_key order matters");
    }

    // ── Tier 2: resolve cache ─────────────────────────────────────────────────

    #[test]
    fn resolve_cache_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let entry = ResolveCacheEntry {
            resolve_key: "rkey1".to_string(),
            uri: "md://fig.md#:0".to_string(),
            target_parse_key: "pkey1".to_string(),
            content: "```\nfigure\n```".to_string(),
            proof_version: proof_version().to_string(),
            created_at: epoch_ms(),
        };
        save_resolve_cache(root, &entry);
        let loaded = load_resolve_cache(root, "rkey1").unwrap();
        assert_eq!(loaded.content, "```\nfigure\n```");
        assert_eq!(loaded.uri, "md://fig.md#:0");
    }

    #[test]
    fn resolve_cache_miss_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let result = load_resolve_cache(dir.path(), "nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn try_resolve_cache_hit_on_miss_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("fig.md");
        std::fs::write(&target, "```\ncontent\n```").unwrap();
        let mut index = PathIndex::new();
        let result = try_resolve_cache_hit(
            dir.path(),
            &target,
            "```\ncontent\n```",
            "md://fig.md#:0",
            &mut index,
        );
        assert!(result.is_none(), "fresh cache should be a miss");
    }

    #[test]
    fn try_resolve_cache_hit_after_store() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("fig.md");
        let figure_content = "```\ncontent\n```";
        std::fs::write(&target, figure_content).unwrap();
        let mut index = PathIndex::new();

        // Store
        store_resolve_cache(
            dir.path(),
            &target,
            figure_content,
            "md://fig.md#:0",
            "resolved content",
            &mut index,
        );

        // Hit
        let hit = try_resolve_cache_hit(
            dir.path(),
            &target,
            figure_content,
            "md://fig.md#:0",
            &mut index,
        );
        assert_eq!(
            hit.as_deref(),
            Some("resolved content"),
            "should hit after store"
        );
    }

    #[test]
    fn resolve_cache_miss_when_content_changes() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("fig.md");
        let mut index = PathIndex::new();

        // Store with v1
        std::fs::write(&target, "v1").unwrap();
        store_resolve_cache(
            dir.path(),
            &target,
            "v1",
            "md://fig.md#:0",
            "result v1",
            &mut index,
        );

        // Try with v2 — different content → miss
        let hit = try_resolve_cache_hit(dir.path(), &target, "v2", "md://fig.md#:0", &mut index);
        assert!(hit.is_none(), "different content should be a cache miss");
    }
}
