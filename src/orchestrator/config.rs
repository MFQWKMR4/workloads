use serde::Deserialize;
use std::error::Error;
use std::fmt;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Step {
    pub(crate) id: Option<String>,
    pub(crate) runtime: String,
    pub(crate) parallel: Option<Parallel>,
    pub(crate) task: Option<Task>,
    pub(crate) location: Option<String>,
    pub(crate) stdout: Option<bool>,
    pub(crate) exec: Option<String>,
    pub(crate) args: Option<Vec<String>>,
    pub(crate) command: Option<String>,
    pub(crate) shell: Option<String>,
    pub(crate) depends_on: Option<Vec<String>>,
    pub(crate) when: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Parallel {
    pub(crate) processes: Option<u32>,
    pub(crate) threads: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Task {
    #[serde(rename = "type")]
    pub(crate) task_type: String,
    pub(crate) settings: TaskSettings,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TaskSettings {
    pub(crate) count: Option<u64>,
    pub(crate) duration_ms: Option<u64>,
}

#[derive(Debug)]
pub(crate) struct ConfigError(pub(crate) String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ConfigError {}

pub(crate) fn validate_config(config: &Config) -> Result<(), Box<dyn Error>> {
    if config.steps.is_empty() {
        return Err(Box::new(ConfigError("steps must not be empty".to_string())));
    }

    for step in &config.steps {
        let runtime = step.runtime.trim().to_lowercase();
        if runtime.is_empty() {
            return Err(Box::new(ConfigError("runtime must be set".to_string())));
        }

        if let Some(parallel) = &step.parallel {
            if let Some(processes) = parallel.processes {
                if processes == 0 {
                    return Err(Box::new(ConfigError(
                        "parallel.processes must be > 0".to_string(),
                    )));
                }
            }
            if let Some(threads) = parallel.threads {
                if threads == 0 {
                    return Err(Box::new(ConfigError(
                        "parallel.threads must be > 0".to_string(),
                    )));
                }
            }
        }

        if runtime == "bin" {
            if step.exec.as_deref().unwrap_or("").is_empty() {
                return Err(Box::new(ConfigError(
                    "bin runtime requires 'exec'".to_string(),
                )));
            }
        } else if runtime == "shell" {
            if step.command.as_deref().unwrap_or("").is_empty() {
                return Err(Box::new(ConfigError(
                    "shell runtime requires 'command'".to_string(),
                )));
            }
        } else {
            if step.location.is_none() {
                let task = step
                    .task
                    .as_ref()
                    .ok_or_else(|| ConfigError("task must be set".to_string()))?;
                validate_task(task)?;
            } else if let Some(task) = &step.task {
                validate_task(task)?;
            }
        }
    }

    Ok(())
}

pub(crate) fn task_count(step: &Step) -> Result<Option<u64>, Box<dyn Error>> {
    let task = match &step.task {
        Some(task) => task,
        None => return Ok(None),
    };
    validate_task(task)?;
    Ok(task.settings.count)
}

pub(crate) fn step_processes(step: &Step) -> u32 {
    step.parallel
        .as_ref()
        .and_then(|parallel| parallel.processes)
        .unwrap_or(1)
}

pub(crate) fn step_stdout(step: &Step) -> bool {
    step.stdout.unwrap_or(false)
}

fn validate_task(task: &Task) -> Result<(), Box<dyn Error>> {
    if task.task_type != "counter" {
        return Err(Box::new(ConfigError(format!(
            "task type '{}' is not supported yet",
            task.task_type
        ))));
    }
    let count = task.settings.count.unwrap_or(0);
    if count == 0 {
        return Err(Box::new(ConfigError("task.settings.count must be > 0".to_string())));
    }
    Ok(())
}
