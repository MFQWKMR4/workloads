use crate::orchestrator::config::{ConfigError, Step};
use std::collections::HashMap;

pub(crate) fn apply_placeholders(
    step: &Step,
    pid_map: &HashMap<String, Vec<u32>>,
) -> Result<Step, Box<dyn std::error::Error>> {
    let mut updated = step.clone();

    if let Some(command) = &step.command {
        updated.command = Some(expand_value(command, pid_map)?);
    }
    if let Some(wrapper) = &step.wrapper {
        updated.wrapper = Some(expand_value(wrapper, pid_map)?);
    }
    if let Some(env) = &step.env {
        let mut new_env = env.clone();
        for (key, value) in env {
            new_env.insert(key.clone(), expand_value(value, pid_map)?);
        }
        updated.env = Some(new_env);
    }

    Ok(updated)
}

fn expand_value(
    value: &str,
    pid_map: &HashMap<String, Vec<u32>>,
) -> Result<String, Box<dyn std::error::Error>> {
    if !value.starts_with("p\"") {
        return Ok(value.to_string());
    }

    let trimmed = value.strip_prefix("p\"").unwrap_or("");
    let rest = trimmed.strip_suffix('"').ok_or_else(|| {
        ConfigError("placeholder missing closing '\"'".to_string())
    })?;

    let mut output = String::new();
    let mut rest = rest;
    while let Some(start) = rest.find('{') {
        output.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after.find('}').ok_or_else(|| {
            ConfigError("placeholder missing closing '}'".to_string())
        })?;
        let token = &after[..end];
        output.push_str(&expand_token(token, pid_map)?);
        rest = &after[end + 1..];
    }
    output.push_str(rest);
    Ok(output)
}

fn expand_token(
    token: &str,
    pid_map: &HashMap<String, Vec<u32>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut parts = token.splitn(2, ':');
    let id = parts
        .next()
        .ok_or_else(|| ConfigError("placeholder missing id".to_string()))?;
    let key = parts
        .next()
        .ok_or_else(|| ConfigError("placeholder missing key".to_string()))?;

    if key == "pid" {
        let pids = pid_map
            .get(id)
            .ok_or_else(|| ConfigError(format!("unknown id '{}'", id)))?;
        let first = pids
            .first()
            .ok_or_else(|| ConfigError(format!("no pid for id '{}'", id)))?;
        return Ok(first.to_string());
    }

    if key == "pid," {
        let pids = pid_map
            .get(id)
            .ok_or_else(|| ConfigError(format!("unknown id '{}'", id)))?;
        let joined = pids
            .iter()
            .map(|pid| pid.to_string())
            .collect::<Vec<_>>()
            .join(",");
        return Ok(joined);
    }

    Err(Box::new(ConfigError(format!(
        "unknown placeholder key '{}'",
        key
    ))))
}
