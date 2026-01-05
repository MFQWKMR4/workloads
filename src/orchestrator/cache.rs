use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) struct CacheContext {
    pub(crate) config_hash: String,
    pub(crate) base_dir: PathBuf,
    pub(crate) source_dir: PathBuf,
    pub(crate) url_dir: PathBuf,
}

impl CacheContext {
    pub(crate) fn url_source_path(&self, url: &str, extension: &str) -> PathBuf {
        let url_hash = hash_string(url);
        self.url_dir
            .join(url_hash)
            .join(format!("source.{extension}"))
    }

    pub(crate) fn config_source_path(&self, url: &str, extension: &str) -> PathBuf {
        let url_hash = hash_string(url);
        self.source_dir
            .join(url_hash)
            .join(format!("source.{extension}"))
    }

    pub(crate) fn build_path_for_source(&self, source_path: &Path) -> PathBuf {
        let key = source_path.to_string_lossy();
        let hash = hash_string(&key);
        self.source_dir.join(hash).join("build")
    }
}

pub(crate) fn cache_context(config_content: &str) -> CacheContext {
    let config_hash = hash_string(config_content);
    let base_dir = cache_base_dir().join("tmp_workspace");
    let source_dir = base_dir.join("source");
    let url_dir = base_dir.join("url");

    CacheContext {
        config_hash,
        base_dir,
        source_dir,
        url_dir,
    }
}

fn cache_base_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir())
}

fn hash_string(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    bytes_to_hex(&hasher.finalize())
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        let _ = write!(out, "{:02x}", byte);
    }
    out
}
