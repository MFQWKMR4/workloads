use crate::orchestrator::cache::CacheContext;
use crate::orchestrator::config::{Step, step_duration_ms, step_env, step_processes, step_stdout};
use crate::orchestrator::process::{kill_process, spawn_process, wait_process};
use crate::orchestrator::StepOutcome;
use crate::orchestrator::source::resolve_source;
use crate::orchestrator::wrapper::wrap_command;
use std::error::Error;
use std::path::Path;
use std::thread;
use std::time::Duration;

pub(crate) fn run(
    step: &Step,
    cache: &CacheContext,
    on_start: &dyn Fn(&[u32]),
) -> Result<StepOutcome, Box<dyn Error>> {
    let processes = step_processes(step);
    let stdout_enabled = step_stdout(step);
    let duration_ms = step_duration_ms(step);
    let source = resolve_source(step, Path::new("runtimes/python/main.py"), "py", cache)?;
    let args = step.args.as_deref().unwrap_or(&[]);
    let envs = step_env(step);

    println!("python: processes={} args={}", processes, args.join(" "));

    let step_id = step.id.as_deref().unwrap_or("unknown");
    let log_label = format!("step={} runtime=python", step_id);

    let mut children = Vec::new();
    let mut pids = Vec::new();
    for _ in 0..processes {
        let mut base = vec![
            "python3".to_string(),
            source.path.to_string_lossy().to_string(),
        ];
        base.extend(args.iter().cloned());
        let wrapped = wrap_command(step, &base);
        let mut command = wrapped.command;
        for (key, value) in &envs {
            command.env(key, value);
        }
        let child = spawn_process(command, &log_label, &wrapped.display, stdout_enabled)?;
        pids.push(child.pid());
        children.push(child);
    }

    on_start(&pids);

    if let Some(duration) = duration_ms {
        thread::sleep(Duration::from_millis(duration));
        for child in &mut children {
            let _ = kill_process(child);
        }
    }

    let mut exit_codes = Vec::new();
    for child in children {
        let code = wait_process(child, &log_label, duration_ms.is_some())?;
        exit_codes.push(code);
    }

    println!("pids={}", join_pids(&pids));
    println!("python: done");

    Ok(StepOutcome { pids, exit_codes })
}

fn join_pids(pids: &[u32]) -> String {
    pids.iter()
        .map(|pid| pid.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
