mod config;
mod cache;
mod process;
mod samples;
mod source;
mod shell;
mod bin;
mod golang;
mod node;
mod python;

use crate::orchestrator::cache::{CacheContext, cache_context};
use crate::orchestrator::config::{Config, ConfigError, validate_config};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

struct RuntimeInfo {
    name: &'static str,
    detect_cmd: &'static str,
    features: &'static [&'static str],
}

const RUNTIMES: &[RuntimeInfo] = &[
    RuntimeInfo {
        name: "node.js",
        detect_cmd: "node",
        features: &["IO"],
    },
    RuntimeInfo {
        name: "python(CPython)",
        detect_cmd: "python3",
        features: &["multiprocess:memory", "multiprocess:cpu"],
    },
    RuntimeInfo {
        name: "golang",
        detect_cmd: "go",
        features: &["multithread:memory", "multithread:cpu"],
    },
    RuntimeInfo {
        name: "jvm",
        detect_cmd: "javac",
        features: &["memory", "cpu", "IO"],
    },
    RuntimeInfo {
        name: "native(Rust)",
        detect_cmd: "rustc",
        features: &["others(TODO)"],
    },
    RuntimeInfo {
        name: "native(bin)",
        detect_cmd: "true",
        features: &["exec"],
    },
    RuntimeInfo {
        name: "shell",
        detect_cmd: "bash",
        features: &["command"],
    },
];

pub fn list_available() -> Result<(), Box<dyn Error>> {
    let mut any = false;
    for runtime in RUNTIMES {
        if is_cmd_available(runtime.detect_cmd) {
            any = true;
            let features = runtime.features.join(", ");
            println!("{}: {}", runtime.name, features);
        }
    }

    if !any {
        println!("No supported runtimes detected on this host.");
    }

    Ok(())
}

pub fn prepare(config_path: &Path) -> Result<(), Box<dyn Error>> {
    let loaded = load_config(config_path)?;
    validate_config(&loaded.config)?;
    for step in &loaded.config.steps {
        ensure_runtime_available(&step.runtime)?;
    }

    for step in &loaded.config.steps {
        println!("prepare: runtime={}", step.runtime);
    }
    println!("prepare: python does not require build steps");

    Ok(())
}

pub fn generate(config_path: &Path) -> Result<(), Box<dyn Error>> {
    let loaded = load_config(config_path)?;
    validate_config(&loaded.config)?;
    for step in &loaded.config.steps {
        ensure_runtime_available(&step.runtime)?;
        let runtime = step.runtime.to_lowercase();
        if runtime == "python" || runtime == "python3" || runtime == "cpython" {
            python::run(step, &loaded.cache)?;
        } else if runtime == "node" || runtime == "node.js" {
            node::run(step, &loaded.cache)?;
        } else if runtime == "golang" || runtime == "go" {
            golang::run(step, &loaded.cache)?;
        } else if runtime == "bin" {
            bin::run(step)?;
        } else if runtime == "shell" {
            shell::run(step)?;
        } else {
            return Err(Box::new(ConfigError(format!(
                "runtime '{}' is not implemented yet",
                step.runtime
            ))));
        }
    }

    Ok(())
}

pub fn samples(output_dir: &Path) -> Result<(), Box<dyn Error>> {
    samples::write_samples(output_dir)
}

struct LoadedConfig {
    config: Config,
    cache: CacheContext,
}

fn load_config(config_path: &Path) -> Result<LoadedConfig, Box<dyn Error>> {
    let content = fs::read_to_string(config_path)?;
    let cache = cache_context(&content);
    fs::create_dir_all(&cache.config_dir)?;
    fs::create_dir_all(&cache.url_dir)?;
    let config: Config = serde_yaml::from_str(&content)?;
    Ok(LoadedConfig { config, cache })
}

fn ensure_runtime_available(runtime: &str) -> Result<(), Box<dyn Error>> {
    let normalized = runtime.to_lowercase();
    let detect_cmd = match normalized.as_str() {
        "node" | "node.js" => Some("node"),
        "python" | "python3" | "cpython" => Some("python3"),
        "golang" | "go" => Some("go"),
        "jvm" | "java" | "javac" => Some("javac"),
        "native(rust)" | "rust" | "rustc" => Some("rustc"),
        "bin" => None,
        "shell" => Some("bash"),
        _ => None,
    };

    match detect_cmd {
        Some(cmd) if is_cmd_available(cmd) => Ok(()),
        Some(cmd) => Err(Box::new(ConfigError(format!(
            "runtime '{}' not detected (missing '{}')",
            runtime, cmd
        )))),
        None if normalized == "bin" => Ok(()),
        None => Err(Box::new(ConfigError(format!(
            "runtime '{}' is not recognized",
            runtime
        )))),
    }
}

fn is_cmd_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}
