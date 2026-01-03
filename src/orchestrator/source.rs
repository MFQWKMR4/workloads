use crate::orchestrator::config::{ConfigError, Step};
use std::error::Error;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
        let bytes = download_bytes(location)?;
        let temp_path = temp_file_path(extension);
        std::fs::write(&temp_path, bytes)?;
        Ok(ResolvedSource {
            path: temp_path,
            cleanup: true,
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

fn temp_file_path(extension: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let filename = format!("wl_remote_{}_{}.{}", pid, nanos, extension);
    std::env::temp_dir().join(filename)
}
