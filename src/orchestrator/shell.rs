use crate::orchestrator::config::{
    ConfigError, Step, step_duration_ms, step_env, step_processes, step_stdout,
};
use crate::orchestrator::process::{kill_process, spawn_process, wait_process};
use crate::orchestrator::wrapper::wrap_command;
use std::error::Error;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub(crate) fn run(step: &Step) -> Result<(), Box<dyn Error>> {
    let command = step
        .command
        .as_deref()
        .ok_or_else(|| ConfigError("shell runtime requires 'command'".to_string()))?;
    let shell = step.shell.as_deref().unwrap_or("bash");
    let processes = step_processes(step);
    let stdout_enabled = step_stdout(step);
    let duration_ms = step_duration_ms(step);
    let envs = step_env(step);

    println!("shell: processes={} shell={}", processes, shell);

    let step_id = step.id.as_deref().unwrap_or("unknown");
    let log_label = format!("step={} runtime=shell", step_id);
    let mut base = vec![shell.to_string(), "-lc".to_string(), command.to_string()];

    let mut children = Vec::new();
    let mut pids = Vec::new();
    for _ in 0..processes {
        let wrapped = wrap_command(step, &base);
        let mut cmd = wrapped.command;
        for (key, value) in &envs {
            cmd.env(key, value);
        }
        let child = spawn_process(cmd, &log_label, &wrapped.display, stdout_enabled)?;
        pids.push(child.pid());
        children.push(child);
    }

    if let Some(duration) = duration_ms {
        thread::sleep(Duration::from_millis(duration));
        for child in &mut children {
            let _ = kill_process(child);
        }
    }

    for child in children {
        wait_process(child, &log_label, duration_ms.is_some())?;
    }

    println!("pids={}", join_pids(&pids));
    println!("shell: done");

    Ok(())
}

fn join_pids(pids: &[u32]) -> String {
    pids.iter()
        .map(|pid| pid.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
