use crate::orchestrator::cache::CacheContext;
use crate::orchestrator::config::{Step, step_duration_ms, step_processes, step_stdout};
use crate::orchestrator::process::{kill_process, spawn_process, wait_process};
use crate::orchestrator::source::resolve_source;
use std::error::Error;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub(crate) fn run(step: &Step, cache: &CacheContext) -> Result<(), Box<dyn Error>> {
    let processes = step_processes(step);
    let stdout_enabled = step_stdout(step);
    let duration_ms = step_duration_ms(step);
    let source = resolve_source(step, Path::new("runtimes/python/main.py"), "py", cache)?;
    let args = step.args.as_deref().unwrap_or(&[]);

    println!("python: processes={} args={}", processes, args.join(" "));

    let step_id = step.id.as_deref().unwrap_or("unknown");
    let log_label = format!("step={} runtime=python", step_id);

    let mut children = Vec::new();
    let mut pids = Vec::new();
    for _ in 0..processes {
        let mut command = Command::new("python3");
        command.arg(&source.path);
        if !args.is_empty() {
            command.args(args);
        }
        let cmd_display = if args.is_empty() {
            format!("python3 {}", source.path.display())
        } else {
            format!(
                "python3 {} {}",
                source.path.display(),
                args.join(" ")
            )
        };
        let child = spawn_process(command, &log_label, &cmd_display, stdout_enabled)?;
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
    println!("python: done");

    Ok(())
}

fn join_pids(pids: &[u32]) -> String {
    pids.iter()
        .map(|pid| pid.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
