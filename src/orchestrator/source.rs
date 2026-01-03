use crate::orchestrator::cache::CacheContext;
use crate::orchestrator::config::{ConfigError, Step};
use std::error::Error;
use std::io::Read;
use std::path::{Path, PathBuf};

pub(crate) struct ResolvedSource {
    pub(crate) path: PathBuf,
    cleanup: bool,
}

impl Drop for ResolvedSource {
    fn drop(&mut self) {
        if self.cleanup {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

pub(crate) fn resolve_source(
    step: &Step,
    default_path: &Path,
    extension: &str,
    cache: &CacheContext,
) -> Result<ResolvedSource, Box<dyn Error>> {
    let location = match &step.location {
        Some(location) => location,
        None => {
            return Ok(ResolvedSource {
                path: default_path.to_path_buf(),
                cleanup: false,
            })
        }
    };

    if is_http_url(location) {
        let url_cache_path = cache.url_source_path(location, extension);
        if !url_cache_path.exists() {
            if let Some(parent) = url_cache_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let bytes = download_bytes(location)?;
            std::fs::write(&url_cache_path, bytes)?;
        }

        let config_cache_path = cache.config_source_path(location, extension);
        if !config_cache_path.exists() {
            if let Some(parent) = config_cache_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&url_cache_path, &config_cache_path)?;
        }

        Ok(ResolvedSource {
            path: config_cache_path,
            cleanup: false,
        })
    } else {
        let path = PathBuf::from(location);
        if !path.exists() {
            return Err(Box::new(ConfigError(format!(
                "location path '{}' does not exist",
                location
            ))));
        }
        Ok(ResolvedSource {
            path,
            cleanup: false,
        })
    }
}

pub(crate) fn is_http_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn download_bytes(url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let response = ureq::get(url).call()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(Box::new(ConfigError(format!(
            "failed to download '{}': status {}",
            url, status
        ))));
    }
    let mut bytes = Vec::new();
    response.into_reader().read_to_end(&mut bytes)?;
    Ok(bytes)
}
