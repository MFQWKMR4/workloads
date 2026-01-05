use crate::orchestrator::config::Step;
use std::process::Command;

pub(crate) struct WrappedCommand {
    pub(crate) command: Command,
    pub(crate) display: String,
}

pub(crate) fn wrap_command(step: &Step, base: &[String]) -> WrappedCommand {
    if let Some(wrapper) = step.wrapper.as_deref() {
        if wrapper.trim().is_empty() {
            return base_command(base);
        }
        let mut wrapper_parts = split_args(wrapper);
        let wrapper_display = wrapper_parts.join(" ");
        let mut command = Command::new(
            wrapper_parts
                .get(0)
                .map(|value| value.as_str())
                .unwrap_or(wrapper),
        );
        if wrapper_parts.len() > 1 {
            command.args(&wrapper_parts[1..]);
        }
        command.arg("--");
        command.args(base);
        let display = format!("{} -- {}", wrapper_display, base.join(" "));
        return WrappedCommand { command, display };
    }

    base_command(base)
}

fn base_command(base: &[String]) -> WrappedCommand {
    let mut command = Command::new(&base[0]);
    if base.len() > 1 {
        command.args(&base[1..]);
    }
    WrappedCommand {
        command,
        display: base.join(" "),
    }
}

fn split_args(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
}
