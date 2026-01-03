use crate::orchestrator::config::{ConfigError, Step, step_processes, step_stdout};
use crate::orchestrator::process::{spawn_process, wait_process};
use std::error::Error;
use std::process::Command;

pub(crate) fn run(step: &Step) -> Result<(), Box<dyn Error>> {
    let exec = step
        .exec
        .as_deref()
        .ok_or_else(|| ConfigError("bin runtime requires 'exec'".to_string()))?;
    let processes = step_processes(step);
    let stdout_enabled = step_stdout(step);

    let args: &[String] = step.args.as_deref().unwrap_or(&[]);
    println!("bin: processes={} exec={}", processes, exec);

    let step_id = step.id.as_deref().unwrap_or("unknown");
    let log_label = format!("step={} runtime=bin", step_id);
    let cmd_display = if args.is_empty() {
        exec.to_string()
    } else {
        format!("{} {}", exec, args.join(" "))
    };

    let mut children = Vec::new();
    for _ in 0..processes {
        let mut c = Command::new(exec);
        c.args(args);
        let child = spawn_process(c, &log_label, &cmd_display, stdout_enabled)?;
        children.push(child);
    }

    for child in children {
        wait_process(child, &log_label)?;
    }

    println!("bin: done");

    Ok(())
}
