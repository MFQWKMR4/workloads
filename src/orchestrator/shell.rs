use crate::orchestrator::config::{ConfigError, Step, step_processes, step_stdout};
use crate::orchestrator::process::{spawn_process, wait_process};
use std::error::Error;
use std::process::Command;

pub(crate) fn run(step: &Step) -> Result<(), Box<dyn Error>> {
    let command = step
        .command
        .as_deref()
        .ok_or_else(|| ConfigError("shell runtime requires 'command'".to_string()))?;
    let shell = step.shell.as_deref().unwrap_or("bash");
    let processes = step_processes(step);
    let stdout_enabled = step_stdout(step);

    println!("shell: processes={} shell={}", processes, shell);

    let step_id = step.id.as_deref().unwrap_or("unknown");
    let log_label = format!("step={} runtime=shell", step_id);
    let cmd_display = format!("{} -lc {}", shell, command);

    let mut children = Vec::new();
    for _ in 0..processes {
        let mut cmd = Command::new(shell);
        cmd.arg("-lc").arg(command);
        let child = spawn_process(cmd, &log_label, &cmd_display, stdout_enabled)?;
        children.push(child);
    }

    for child in children {
        wait_process(child, &log_label)?;
    }

    println!("shell: done");

    Ok(())
}
