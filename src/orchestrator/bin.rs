use crate::orchestrator::config::{
    ConfigError, Step, step_duration_ms, step_env, step_processes, step_stdout,
};
use crate::orchestrator::process::{kill_process, spawn_process, wait_process};
use crate::orchestrator::wrapper::wrap_command;
use std::error::Error;
use std::thread;
use std::time::Duration;

pub(crate) fn run(step: &Step) -> Result<(), Box<dyn Error>> {
    let exec = step
        .exec
        .as_deref()
        .ok_or_else(|| ConfigError("bin runtime requires 'exec'".to_string()))?;
    let processes = step_processes(step);
    let stdout_enabled = step_stdout(step);
    let duration_ms = step_duration_ms(step);
    let envs = step_env(step);

    let args: &[String] = step.args.as_deref().unwrap_or(&[]);
    println!("bin: processes={} exec={}", processes, exec);

    let step_id = step.id.as_deref().unwrap_or("unknown");
    let log_label = format!("step={} runtime=bin", step_id);
    let mut base = vec![exec.to_string()];
    base.extend(args.iter().cloned());

    let mut children = Vec::new();
    let mut pids = Vec::new();
    for _ in 0..processes {
        let wrapped = wrap_command(step, &base);
        let mut command = wrapped.command;
        for (key, value) in &envs {
            command.env(key, value);
        }
        let child = spawn_process(command, &log_label, &wrapped.display, stdout_enabled)?;
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
    println!("bin: done");

    Ok(())
}

fn join_pids(pids: &[u32]) -> String {
    pids.iter()
        .map(|pid| pid.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
