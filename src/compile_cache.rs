use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::cache::{self, PathIndex};
use crate::compile_output;
use crate::compile_types::CompileResult;

#[allow(clippy::too_many_arguments)]
pub(crate) fn restore_compile_cache(
    root: &Path,
    source_path: &Path,
    output_path: &Path,
    source_text: &str,
    compile_attrs: &str,
    resolved_files: &[PathBuf],
    dependency_parse_keys: &[String],
    path_index: &mut PathIndex,
) -> Result<Option<CompileResult>> {
    let source_parse_key = cache::get_or_compute_parse_key(source_path, source_text, path_index);
    let cache_key = cache::compile_key(&source_parse_key, dependency_parse_keys, compile_attrs);
    let Some(entry) = cache::load_compile_cache(root, &cache_key) else {
        return Ok(None);
    };

    let current = std::fs::read_to_string(output_path).unwrap_or_default();
    let written = current != entry.compiled_text;
    if written {
        compile_output::atomic_write(output_path, &entry.compiled_text)?;
    }
    cache::save_path_index(root, path_index);

    Ok(Some(CompileResult {
        output_path: output_path.to_path_buf(),
        directives_resolved: entry.directives_resolved,
        violations: vec![],
        from_cache: true,
        resolved_files: resolved_files.to_vec(),
        written,
    }))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn store_compile_cache(
    root: &Path,
    source_path: &Path,
    output_path: &Path,
    source_text: &str,
    compiled_text: &str,
    compile_attrs: &str,
    resolved_files: &[PathBuf],
    dependency_parse_keys: &[String],
    directives_resolved: usize,
    path_index: &mut PathIndex,
) {
    let source_parse_key = cache::get_or_compute_parse_key(source_path, source_text, path_index);
    let cache_key = cache::compile_key(&source_parse_key, dependency_parse_keys, compile_attrs);
    let entry = cache::CompileCacheEntry {
        compile_key: cache_key,
        source_path: source_path.to_string_lossy().to_string(),
        output_path: output_path.to_string_lossy().to_string(),
        compiled_text: compiled_text.to_string(),
        resolved_uris: resolved_files
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        proof_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        directives_resolved,
    };
    cache::save_compile_cache(root, &entry);
    cache::save_path_index(root, path_index);
}
