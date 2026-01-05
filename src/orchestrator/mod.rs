mod bin;
mod cache;
mod config;
mod golang;
mod node;
mod process;
mod python;
mod samples;
mod shell;
mod source;
mod templating;
mod wrapper;

use crate::orchestrator::cache::{CacheContext, cache_context};
use crate::orchestrator::config::{Config, ConfigError, Dependency, validate_config};
use crate::orchestrator::templating::apply_placeholders;
use std::collections::HashMap;
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

pub fn generate(config_path: &Path) -> Result<(), Box<dyn Error>> {
    let loaded = load_config(config_path)?;
    validate_config(&loaded.config)?;
    for step in &loaded.config.steps {
        ensure_runtime_available(&step.runtime)?;
    }

    let steps = loaded.config.steps;
    let shared = SharedState::new();

    let mut handles = Vec::new();
    for step in steps {
        let cache = loaded.cache.clone();
        let shared = shared.clone();
        handles.push(std::thread::spawn(move || {
            run_step_with_deps(step, cache, shared).map_err(|err| err.to_string())
        }));
    }

    for handle in handles {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(message)) => return Err(Box::new(ConfigError(message))),
            Err(_) => return Err(Box::new(ConfigError("worker thread panicked".to_string()))),
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

#[derive(Clone)]
struct SharedState {
    inner: std::sync::Arc<(std::sync::Mutex<StateMap>, std::sync::Condvar)>,
}

type StateMap = HashMap<String, StepState>;

#[derive(Clone, Default)]
struct StepState {
    started: bool,
    finished: bool,
    pids: Vec<u32>,
    exit_codes: Vec<i32>,
}

impl SharedState {
    fn new() -> Self {
        SharedState {
            inner: std::sync::Arc::new((
                std::sync::Mutex::new(HashMap::new()),
                std::sync::Condvar::new(),
            )),
        }
    }

    fn update_started(&self, id: &str, pids: Vec<u32>) {
        let (lock, cv) = &*self.inner;
        let mut map = lock.lock().unwrap();
        let entry = map.entry(id.to_string()).or_default();
        entry.started = true;
        entry.pids = pids;
        cv.notify_all();
    }

    fn update_finished(&self, id: &str, exit_codes: Vec<i32>) {
        let (lock, cv) = &*self.inner;
        let mut map = lock.lock().unwrap();
        let entry = map.entry(id.to_string()).or_default();
        entry.finished = true;
        entry.exit_codes = exit_codes;
        cv.notify_all();
    }

    fn snapshot_pids(&self) -> HashMap<String, Vec<u32>> {
        let (lock, _) = &*self.inner;
        let map = lock.lock().unwrap();
        map.iter()
            .map(|(id, state)| (id.clone(), state.pids.clone()))
            .collect()
    }

    fn wait_for(&self, dep: &Dependency) -> Result<(), Box<dyn Error>> {
        let (lock, cv) = &*self.inner;
        let mut map = lock.lock().unwrap();
        loop {
            let state = map.get(&dep.id);
            if dependency_satisfied(dep, state) {
                return Ok(());
            }
            map = cv.wait(map).unwrap();
        }
    }
}

fn dependency_satisfied(dep: &Dependency, state: Option<&StepState>) -> bool {
    let when = dep.when.as_deref().unwrap_or("started");
    let state = match state {
        Some(state) => state,
        None => return false,
    };
    match when {
        "started" => state.started,
        "exited" => {
            if !state.finished {
                return false;
            }
            if let Some(codes) = &dep.exit_codes {
                return state.exit_codes.iter().all(|code| codes.contains(code));
            }
            true
        }
        _ => false,
    }
}

fn run_step_with_deps(
    step: config::Step,
    cache: CacheContext,
    shared: SharedState,
) -> Result<(), Box<dyn Error>> {
    if let Some(deps) = &step.depends_on {
        if step.id.is_none() {
            return Err(Box::new(ConfigError(
                "step id is required when using depends_on".to_string(),
            )));
        }
        for dep in deps {
            shared.wait_for(dep)?;
        }
    }

    let pid_map = shared.snapshot_pids();
    let step = apply_placeholders(&step, &pid_map)?;

    let step_id = step.id.clone();
    let on_start = |pids: &[u32]| {
        if let Some(id) = &step_id {
            shared.update_started(id, pids.to_vec());
        }
    };

    let outcome = run_step(step, cache, &on_start)?;
    if let Some(id) = &step_id {
        shared.update_finished(id, outcome.exit_codes);
    }
    Ok(())
}

pub(crate) struct StepOutcome {
    pids: Vec<u32>,
    exit_codes: Vec<i32>,
}

fn run_step(
    step: config::Step,
    cache: CacheContext,
    on_start: &dyn Fn(&[u32]),
) -> Result<StepOutcome, Box<dyn Error>> {
    let runtime = step.runtime.to_lowercase();
    if runtime == "python" || runtime == "python3" || runtime == "cpython" {
        return python::run(&step, &cache, on_start);
    }
    if runtime == "node" || runtime == "node.js" {
        return node::run(&step, &cache, on_start);
    }
    if runtime == "golang" || runtime == "go" {
        return golang::run(&step, &cache, on_start);
    }
    if runtime == "bin" {
        return bin::run(&step, on_start);
    }
    if runtime == "shell" {
        return shell::run(&step, on_start);
    }

    Err(Box::new(ConfigError(format!(
        "runtime '{}' is not implemented yet",
        step.runtime
    ))))
}
