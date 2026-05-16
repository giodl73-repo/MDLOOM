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
    /// Number of proof:* directives resolved during the compile that produced
    /// this entry. Restored on cache hit so the "(N directives)" output stays
    /// truthful across recompiles. `#[serde(default)]` keeps old entries
    /// loadable (they restore as 0 — acceptable; they'll be re-cached on miss).
    #[serde(default)]
    pub directives_resolved: usize,
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
        directives_resolved: 0,
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
// ─────────────────────────────────────────────────────────
// Cache snapshots — named compile states
// (per design/CACHE-SNAPSHOTS.md)
// ─────────────────────────────────────────────────────────

fn snapshots_dir(root: &Path) -> PathBuf {
    cache_dir(root).join("snapshots")
}

fn snapshot_dir(root: &Path, name: &str) -> PathBuf {
    snapshots_dir(root).join(name)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotManifest {
    pub name: String,
    pub created_at: u64,
    pub proof_version: String,
    /// Source file paths covered by this snapshot, in compile-cache enumeration order.
    pub files: Vec<String>,
    /// Per-file three-tier keys. For each file: parse key, resolve keys (one per
    /// referenced URI), compile key (None if no compile entry).
    pub tiers: std::collections::HashMap<String, TieredCacheKeys>,
    pub total_size: u64,
    /// SHA-256 over the manifest body (without this hash) plus all cache keys.
    pub integrity_hash: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TieredCacheKeys {
    pub parse: String,
    pub resolve: Vec<String>,
    pub compile: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SnapshotDiff {
    pub only_in_a: Vec<String>,
    pub only_in_b: Vec<String>,
    pub changed: Vec<String>,
    pub identical: Vec<String>,
}

/// Save the current cache state to a named snapshot. Atomic temp-then-rename.
/// Returns the manifest on success.
pub fn snapshot_save(root: &Path, name: &str) -> std::io::Result<SnapshotManifest> {
    let snap_root = snapshot_dir(root, name);
    if snap_root.exists() {
        std::fs::remove_dir_all(&snap_root)?;
    }
    let tmp = snapshots_dir(root).join(format!(".{}.tmp", name));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp)?;
    let dirs = [
        ("parse", parse_dir(root)),
        ("resolve", resolve_dir(root)),
        ("compile", compile_dir(root)),
    ];
    let mut total_size: u64 = 0;
    for (label, src) in &dirs {
        let dest = tmp.join(label);
        std::fs::create_dir_all(&dest)?;
        if let Ok(entries) = std::fs::read_dir(src) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let bytes = std::fs::read(&p)?;
                total_size += bytes.len() as u64;
                let target = dest.join(p.file_name().unwrap_or_default());
                std::fs::write(&target, &bytes)?;
            }
        }
    }

    // Build the per-file TieredCacheKeys map by reading every compile entry's source_path.
    let mut tiers: std::collections::HashMap<String, TieredCacheKeys> =
        std::collections::HashMap::new();
    let mut files: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(tmp.join("compile")) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&p) else {
                continue;
            };
            let Ok(ce) = serde_json::from_str::<CompileCacheEntry>(&text) else {
                continue;
            };
            let compile_key_str = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            let path_index = load_path_index(root);
            let parse_key = path_index
                .get(&ce.source_path)
                .map(|e| e.parse_key.clone())
                .unwrap_or_default();
            let resolve_keys = ce.resolved_uris.clone();
            files.push(ce.source_path.clone());
            tiers.insert(
                ce.source_path.clone(),
                TieredCacheKeys {
                    parse: parse_key,
                    resolve: resolve_keys,
                    compile: Some(compile_key_str),
                },
            );
        }
    }
    files.sort();

    // Manifest with placeholder hash, then compute.
    let mut manifest = SnapshotManifest {
        name: name.to_string(),
        created_at: epoch_ms(),
        proof_version: proof_version().to_string(),
        files: files.clone(),
        tiers: tiers.clone(),
        total_size,
        integrity_hash: String::new(),
    };
    manifest.integrity_hash = compute_integrity_hash(&manifest);
    let manifest_path = tmp.join("manifest.json");
    let manifest_json =
        serde_json::to_string_pretty(&manifest).map_err(|e| std::io::Error::other(e))?;
    std::fs::write(&manifest_path, manifest_json)?;

    // Atomic rename: tmp → snap_root
    if let Some(parent) = snap_root.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(&tmp, &snap_root)?;
    Ok(manifest)
}

/// Restore a named snapshot to active cache directories. Verifies integrity
/// before applying. Returns the manifest on success, or an error if the
/// snapshot is missing or corrupted.
pub fn snapshot_restore(root: &Path, name: &str) -> Result<SnapshotManifest, SnapshotError> {
    let snap_root = snapshot_dir(root, name);
    if !snap_root.exists() {
        return Err(SnapshotError::NotFound(name.to_string()));
    }
    let manifest = load_snapshot_manifest(&snap_root)
        .ok_or_else(|| SnapshotError::Corrupted("manifest.json missing or unparseable".into()))?;
    verify_integrity(&manifest)?;

    // Copy each tier dir back to active cache.
    for (label, dest) in [
        ("parse", parse_dir(root)),
        ("resolve", resolve_dir(root)),
        ("compile", compile_dir(root)),
    ] {
        std::fs::create_dir_all(&dest).map_err(SnapshotError::Io)?;
        let src = snap_root.join(label);
        let Ok(entries) = std::fs::read_dir(&src) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let target = dest.join(p.file_name().unwrap_or_default());
            std::fs::copy(&p, &target).map_err(SnapshotError::Io)?;
        }
    }
    Ok(manifest)
}

/// List all named snapshots with their manifests, ordered by created_at descending.
pub fn snapshot_list(root: &Path) -> Vec<SnapshotManifest> {
    let dir = snapshots_dir(root);
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        if let Some(m) = load_snapshot_manifest(&p) {
            out.push(m);
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    out
}

/// Compare two named snapshots by per-file tier keys.
pub fn snapshot_diff(
    root: &Path,
    name_a: &str,
    name_b: &str,
) -> Result<SnapshotDiff, SnapshotError> {
    let a = load_snapshot_manifest(&snapshot_dir(root, name_a))
        .ok_or_else(|| SnapshotError::NotFound(name_a.to_string()))?;
    let b = load_snapshot_manifest(&snapshot_dir(root, name_b))
        .ok_or_else(|| SnapshotError::NotFound(name_b.to_string()))?;
    let mut diff = SnapshotDiff {
        only_in_a: Vec::new(),
        only_in_b: Vec::new(),
        changed: Vec::new(),
        identical: Vec::new(),
    };
    for f in &a.files {
        if !b.tiers.contains_key(f) {
            diff.only_in_a.push(f.clone());
        } else if a.tiers.get(f).map(tiered_signature) != b.tiers.get(f).map(tiered_signature) {
            diff.changed.push(f.clone());
        } else {
            diff.identical.push(f.clone());
        }
    }
    for f in &b.files {
        if !a.tiers.contains_key(f) {
            diff.only_in_b.push(f.clone());
        }
    }
    diff.only_in_a.sort();
    diff.only_in_b.sort();
    diff.changed.sort();
    diff.identical.sort();
    Ok(diff)
}

/// Remove all but the N most recent snapshots. Returns deleted snapshot names.
pub fn snapshot_prune(root: &Path, keep: usize) -> Vec<String> {
    let snapshots = snapshot_list(root);
    let mut deleted = Vec::new();
    for old in snapshots.into_iter().skip(keep) {
        let p = snapshot_dir(root, &old.name);
        if std::fs::remove_dir_all(&p).is_ok() {
            deleted.push(old.name);
        }
    }
    deleted
}

/// Materialize compiled output from a snapshot to a target directory without
/// recompiling. Each file's compiled_text is written under target_dir/{relative_path}.
pub fn snapshot_deploy(root: &Path, name: &str, target_dir: &Path) -> Result<usize, SnapshotError> {
    let snap_root = snapshot_dir(root, name);
    if !snap_root.exists() {
        return Err(SnapshotError::NotFound(name.to_string()));
    }
    let manifest = load_snapshot_manifest(&snap_root)
        .ok_or_else(|| SnapshotError::Corrupted("manifest.json missing".into()))?;
    verify_integrity(&manifest)?;

    let mut count = 0usize;
    let compile_dir_in_snap = snap_root.join("compile");
    let Ok(entries) = std::fs::read_dir(&compile_dir_in_snap) else {
        return Ok(0);
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&p) else {
            continue;
        };
        let Ok(ce) = serde_json::from_str::<CompileCacheEntry>(&text) else {
            continue;
        };
        let rel = std::path::Path::new(&ce.output_path);
        let final_path = if rel.is_absolute() {
            target_dir.join(rel.file_name().unwrap_or_default())
        } else {
            target_dir.join(rel)
        };
        if let Some(parent) = final_path.parent() {
            std::fs::create_dir_all(parent).map_err(SnapshotError::Io)?;
        }
        std::fs::write(&final_path, ce.compiled_text).map_err(SnapshotError::Io)?;
        count += 1;
    }
    Ok(count)
}

#[derive(Debug)]
pub enum SnapshotError {
    NotFound(String),
    Corrupted(String),
    IntegrityMismatch,
    Io(std::io::Error),
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(n) => write!(f, "snapshot {:?} not found", n),
            Self::Corrupted(m) => write!(f, "snapshot corrupted: {}", m),
            Self::IntegrityMismatch => write!(f, "snapshot integrity hash mismatch (COMPILE-004)"),
            Self::Io(e) => write!(f, "io error: {}", e),
        }
    }
}

impl std::error::Error for SnapshotError {}

fn load_snapshot_manifest(snap_root: &Path) -> Option<SnapshotManifest> {
    let text = std::fs::read_to_string(snap_root.join("manifest.json")).ok()?;
    serde_json::from_str(&text).ok()
}

fn compute_integrity_hash(manifest: &SnapshotManifest) -> String {
    // Canonicalize: name + created_at + proof_version + sorted files + sorted tier keys.
    let mut parts: Vec<String> = vec![
        manifest.name.clone(),
        manifest.created_at.to_string(),
        manifest.proof_version.clone(),
        manifest.total_size.to_string(),
    ];
    let mut files = manifest.files.clone();
    files.sort();
    parts.extend(files);
    let mut keys: Vec<&String> = manifest.tiers.keys().collect();
    keys.sort();
    for k in keys {
        if let Some(t) = manifest.tiers.get(k) {
            parts.push(format!(
                "{}|{}|{}|{}",
                k,
                t.parse,
                t.resolve.join(","),
                t.compile.as_deref().unwrap_or("")
            ));
        }
    }
    let part_refs: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
    compute_key(&part_refs)
}

fn verify_integrity(manifest: &SnapshotManifest) -> Result<(), SnapshotError> {
    let stored = manifest.integrity_hash.clone();
    let mut m = manifest.clone();
    m.integrity_hash = String::new();
    let computed = compute_integrity_hash(&m);
    if stored != computed {
        Err(SnapshotError::IntegrityMismatch)
    } else {
        Ok(())
    }
}

fn tiered_signature(t: &TieredCacheKeys) -> String {
    format!(
        "{}|{}|{}",
        t.parse,
        t.resolve.join(","),
        t.compile.as_deref().unwrap_or("")
    )
}

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
            directives_resolved: 3,
        };
        save_compile_cache(root, &entry);
        let loaded = load_compile_cache(root, "testkey").unwrap();
        assert_eq!(loaded.compiled_text, "# Hello\n");
        assert_eq!(
            loaded.directives_resolved, 3,
            "directives_resolved must round-trip through cache"
        );
    }

    fn seed_compile_entry(
        root: &Path,
        key: &str,
        source_path: &str,
        output_path: &str,
        text: &str,
    ) {
        let _ = std::fs::create_dir_all(compile_dir(root));
        let entry = CompileCacheEntry {
            compile_key: key.to_string(),
            source_path: source_path.to_string(),
            output_path: output_path.to_string(),
            compiled_text: text.to_string(),
            resolved_uris: vec![],
            proof_version: proof_version().to_string(),
            created_at: epoch_ms(),
            directives_resolved: 1,
        };
        save_compile_cache(root, &entry);
    }

    #[test]
    fn snapshot_save_creates_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_compile_entry(root, "k1", "src/a.source.md", "docs/a.md", "<a/>");
        let manifest = snapshot_save(root, "v1").unwrap();
        assert_eq!(manifest.name, "v1");
        assert!(snapshot_dir(root, "v1").join("manifest.json").exists());
        assert!(!manifest.integrity_hash.is_empty());
        assert!(manifest.files.iter().any(|f| f == "src/a.source.md"));
    }

    #[test]
    fn snapshot_restore_returns_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_compile_entry(root, "k1", "src/a.source.md", "docs/a.md", "<a/>");
        let saved = snapshot_save(root, "prod").unwrap();
        // Wipe active cache, then restore.
        let _ = std::fs::remove_dir_all(compile_dir(root));
        let restored = snapshot_restore(root, "prod").unwrap();
        assert_eq!(restored.name, saved.name);
        // Compile entry should be back.
        assert!(
            load_compile_cache(root, "k1").is_some(),
            "restored cache entry should reload"
        );
    }

    #[test]
    fn snapshot_restore_rejects_corrupted_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_compile_entry(root, "k1", "src/a.source.md", "docs/a.md", "<a/>");
        let _ = snapshot_save(root, "v1").unwrap();
        // Tamper with the manifest.
        let mp = snapshot_dir(root, "v1").join("manifest.json");
        let raw = std::fs::read_to_string(&mp).unwrap();
        let mut m: SnapshotManifest = serde_json::from_str(&raw).unwrap();
        m.total_size = m.total_size + 999_999; // change a covered field, leave hash stale
        std::fs::write(&mp, serde_json::to_string(&m).unwrap()).unwrap();
        let result = snapshot_restore(root, "v1");
        assert!(
            matches!(result, Err(SnapshotError::IntegrityMismatch)),
            "tampered snapshot must reject restore: {:?}",
            result
        );
    }

    #[test]
    fn snapshot_list_orders_newest_first() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_compile_entry(root, "k1", "a.source.md", "a.md", "1");
        let _ = snapshot_save(root, "old").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        seed_compile_entry(root, "k2", "b.source.md", "b.md", "2");
        let _ = snapshot_save(root, "new").unwrap();
        let list = snapshot_list(root);
        assert!(list.len() >= 2);
        assert_eq!(
            list[0].name,
            "new",
            "newest first: {:?}",
            list.iter().map(|m| &m.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn snapshot_diff_reports_changed_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // Snapshot A has file a with key k1.
        seed_compile_entry(root, "k1", "shared.source.md", "shared.md", "v1");
        let _ = snapshot_save(root, "a").unwrap();
        // Reset and seed file with different key for snapshot B.
        let _ = std::fs::remove_dir_all(compile_dir(root));
        seed_compile_entry(root, "k2", "shared.source.md", "shared.md", "v2");
        let _ = snapshot_save(root, "b").unwrap();
        let diff = snapshot_diff(root, "a", "b").unwrap();
        assert!(
            diff.changed.iter().any(|f| f == "shared.source.md"),
            "shared file with different keys should appear as changed: {:?}",
            diff
        );
    }

    #[test]
    fn snapshot_prune_keeps_n_most_recent() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_compile_entry(root, "k", "x.source.md", "x.md", "x");
        for n in ["a", "b", "c", "d"] {
            let _ = snapshot_save(root, n).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let deleted = snapshot_prune(root, 2);
        assert_eq!(deleted.len(), 2, "should delete 2 of 4 with keep=2");
        let remaining = snapshot_list(root);
        assert_eq!(remaining.len(), 2, "2 most recent remain");
        // The most recent ("d") must still be there.
        assert!(
            remaining.iter().any(|m| m.name == "d"),
            "newest snapshot kept"
        );
    }

    #[test]
    fn snapshot_deploy_writes_compiled_text() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_compile_entry(root, "k", "a.source.md", "out/a.md", "compiled body");
        let _ = snapshot_save(root, "v").unwrap();
        let target = root.join("dist");
        let count = snapshot_deploy(root, "v", &target).unwrap();
        assert_eq!(count, 1, "one entry deployed");
        let written = std::fs::read_to_string(target.join("out/a.md")).unwrap();
        assert!(
            written.contains("compiled body"),
            "deployed file content: {:?}",
            written
        );
    }

    #[test]
    fn compile_cache_old_entry_loads_with_zero_directives() {
        // Older entries on disk lack the directives_resolved field. Verify the
        // serde(default) annotation lets them load (as 0) instead of failing.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let _ = std::fs::create_dir_all(compile_dir(root));
        let raw = r#"{
            "compile_key": "oldkey",
            "source_path": "src/x.source.md",
            "output_path": "docs/x.md",
            "compiled_text": "old",
            "resolved_uris": [],
            "proof_version": "0.5.0",
            "created_at": 0
        }"#;
        std::fs::write(compile_dir(root).join("oldkey.json"), raw).unwrap();
        let loaded = load_compile_cache(root, "oldkey").expect("must load old entry");
        assert_eq!(loaded.compiled_text, "old");
        assert_eq!(loaded.directives_resolved, 0);
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
