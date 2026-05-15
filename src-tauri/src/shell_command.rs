use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellCommandOutput {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn parse_command_line(command: &str) -> Result<Vec<String>, String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut in_token = false;

    for char_ in command.chars() {
        if let Some(quote_char) = quote {
            if char_ == quote_char {
                quote = None;
            } else {
                current.push(char_);
            }
            in_token = true;
            continue;
        }

        if char_ == '"' || char_ == '\'' {
            quote = Some(char_);
            in_token = true;
            continue;
        }

        if char_.is_whitespace() {
            if in_token {
                parts.push(std::mem::take(&mut current));
                in_token = false;
            }
            continue;
        }

        current.push(char_);
        in_token = true;
    }

    if quote.is_some() {
        return Err("Unclosed quote in command.".to_string());
    }
    if in_token {
        parts.push(current);
    }

    Ok(parts)
}

fn parse_shell_env(raw: &str) -> Result<Vec<(String, String)>, String> {
    let env = raw.trim();
    if env.is_empty() {
        return Ok(Vec::new());
    }

    if env.starts_with('{') {
        let parsed: HashMap<String, String> =
            serde_json::from_str(env).map_err(|error| error.to_string())?;
        let mut entries = Vec::with_capacity(parsed.len());
        for (key, value) in parsed {
            if !key.is_empty() {
                entries.push((key, value));
            }
        }
        return Ok(entries);
    }

    let split_on_whitespace = !env.as_bytes().contains(&b'\n') && !env.as_bytes().contains(&b';');
    let bytes = env.as_bytes();
    let mut entries = Vec::new();
    let mut segment_start = 0;

    for index in 0..=bytes.len() {
        let at_end = index == bytes.len();
        let separator = if at_end {
            true
        } else if split_on_whitespace {
            bytes[index].is_ascii_whitespace()
        } else {
            bytes[index] == b'\n' || bytes[index] == b'\r' || bytes[index] == b';'
        };

        if !separator {
            continue;
        }

        let segment = env[segment_start..index].trim();
        segment_start = index + 1;
        if segment.is_empty() {
            continue;
        }

        if let Some(separator_index) = segment.find('=') {
            if separator_index == 0 {
                continue;
            }
            entries.push((
                segment[..separator_index].trim().to_string(),
                segment[separator_index + 1..].to_string(),
            ));
        }
    }

    Ok(entries)
}

async fn collect_shell_command_output(
    command: tauri_plugin_shell::process::Command,
    timeout_ms: u64,
) -> Result<ShellCommandOutput, String> {
    let (mut rx, child) = command
        .set_raw_out(true)
        .spawn()
        .map_err(|error| error.to_string())?;
    let mut exit_code = None;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    if timeout_ms > 0 {
        let timeout = tokio::time::sleep(Duration::from_millis(timeout_ms));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                _ = &mut timeout => {
                    let _ = child.kill();
                    return Ok(ShellCommandOutput {
                        exit_code: Some(124),
                        stdout: String::from_utf8_lossy(&stdout).into_owned(),
                        stderr: format!("Command timed out after {timeout_ms}ms"),
                    });
                }
                event = rx.recv() => {
                    match event {
                        Some(CommandEvent::Stdout(bytes)) => stdout.extend(bytes),
                        Some(CommandEvent::Stderr(bytes)) => stderr.extend(bytes),
                        Some(CommandEvent::Terminated(payload)) => {
                            exit_code = payload.code;
                            break;
                        }
                        Some(CommandEvent::Error(error)) => return Err(error),
                        Some(_) => {}
                        None => break,
                    }
                }
            }
        }
    } else {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(bytes) => stdout.extend(bytes),
                CommandEvent::Stderr(bytes) => stderr.extend(bytes),
                CommandEvent::Terminated(payload) => {
                    exit_code = payload.code;
                    break;
                }
                CommandEvent::Error(error) => return Err(error),
                _ => {}
            }
        }
    }

    Ok(ShellCommandOutput {
        exit_code,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    })
}

fn spawn_background_command(
    command: tauri_plugin_shell::process::Command,
    timeout_ms: u64,
) -> Result<(), String> {
    let (_rx, child) = command.spawn().map_err(|error| error.to_string())?;
    if timeout_ms > 0 {
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
            let _ = child.kill();
        });
    }
    Ok(())
}

fn background_shell_output() -> ShellCommandOutput {
    ShellCommandOutput {
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
    }
}

#[tauri::command]
pub async fn execute_shell_command(
    app: tauri::AppHandle,
    command: String,
    timeout_ms: u64,
    env: String,
    cwd: String,
    run_in_background: bool,
) -> Result<ShellCommandOutput, String> {
    let mut parts = parse_command_line(&command)?;
    if parts.is_empty() {
        return Ok(ShellCommandOutput {
            exit_code: Some(1),
            stdout: String::new(),
            stderr: "No command specified.".to_string(),
        });
    }

    let program = std::mem::take(&mut parts[0]);
    let envs = parse_shell_env(&env)?;
    let cwd = cwd.trim();
    let mut shell_command = app.shell().command(program);

    if parts.len() > 1 {
        shell_command = shell_command.args(parts.into_iter().skip(1));
    }
    if !envs.is_empty() {
        shell_command = shell_command.envs(envs);
    }
    if !cwd.is_empty() {
        shell_command = shell_command.current_dir(cwd);
    }

    if run_in_background {
        spawn_background_command(shell_command, timeout_ms)?;
        return Ok(background_shell_output());
    }

    collect_shell_command_output(shell_command, timeout_ms).await
}
