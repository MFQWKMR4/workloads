use crate::orchestrator::cache::CacheContext;
use crate::orchestrator::config::{Step, step_processes, step_stdout, task_count};
use crate::orchestrator::process::{spawn_process, wait_process};
use crate::orchestrator::source::resolve_source;
use std::error::Error;
use std::path::Path;
use std::process::Command;

pub(crate) fn run(step: &Step, cache: &CacheContext) -> Result<(), Box<dyn Error>> {
    let count = task_count(step)?;
    let processes = step_processes(step);
    let stdout_enabled = step_stdout(step);
    let source = resolve_source(step, Path::new("runtimes/node/main.js"), "js", cache)?;

    let count_label = count
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());
    println!("node: processes={} count={}", processes, count_label);

    let step_id = step.id.as_deref().unwrap_or("unknown");
    let log_label = format!("step={} runtime=node", step_id);

    let mut children = Vec::new();
    for _ in 0..processes {
        let mut command = Command::new("node");
        command.arg(&source.path);
        if let Some(value) = count {
            command.arg("--count").arg(value.to_string());
        }
        let cmd_display = if let Some(value) = count {
            format!("node {} --count {}", source.path.display(), value)
        } else {
            format!("node {}", source.path.display())
        };
        let child = spawn_process(command, &log_label, &cmd_display, stdout_enabled)?;
        children.push(child);
    }

    for child in children {
        wait_process(child, &log_label)?;
    }

    println!("node: done");

    Ok(())
}
