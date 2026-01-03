use std::error::Error;
use std::io::{self, BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::thread::JoinHandle;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub(crate) struct ChildTracker {
    child: Child,
    started_at: Instant,
    pid: u32,
    stdout: Option<JoinHandle<io::Result<()>>>,
}

pub(crate) fn spawn_process(
    mut command: Command,
    log_label: &str,
    cmd_display: &str,
    stdout_enabled: bool,
) -> Result<ChildTracker, Box<dyn Error>> {
    if stdout_enabled {
        command.stdout(Stdio::piped());
    }
    let mut child = command.spawn()?;
    let pid = child.id();

    let started_at = Instant::now();
    let ts = unix_millis();
    println!(
        "start pid={} ts={} {} cmd=\"{}\"",
        pid, ts, log_label, cmd_display
    );

    let stdout = if stdout_enabled {
        let handle = child.stdout.take().map(|stdout| {
            let prefix = format!("[pid={} {}]", pid, log_label);
            std::thread::spawn(move || stream_stdout(stdout, &prefix))
        });
        handle
    } else {
        None
    };

    Ok(ChildTracker {
        child,
        started_at,
        pid,
        stdout,
    })
}

pub(crate) fn wait_process(
    mut tracker: ChildTracker,
    log_label: &str,
) -> Result<(), Box<dyn Error>> {
    let status = tracker.child.wait()?;
    let duration_ms = tracker.started_at.elapsed().as_millis();
    let ts = unix_millis();
    let code = status.code().map(|value| value.to_string()).unwrap_or_else(|| "signal".to_string());
    println!(
        "end pid={} ts={} {} duration_ms={} exit={}",
        tracker.pid, ts, log_label, duration_ms, code
    );
    if let Some(handle) = tracker.stdout.take() {
        let _ = handle.join();
    }
    if !status.success() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("process pid {} exited with {}", tracker.pid, code),
        )));
    }
    Ok(())
}

fn stream_stdout(stdout: impl std::io::Read, prefix: &str) -> io::Result<()> {
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        let line = line?;
        println!("{} {}", prefix, line);
    }
    Ok(())
}

fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
