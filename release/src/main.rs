use argh::FromArgs;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
struct CommandError {
    command: String,
    status: std::process::ExitStatus,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Command '{}' failed with status: {}",
            self.command, self.status
        )
    }
}

impl Error for CommandError {}

/// Build helper for Macro Center
#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help"))]
struct Args {
    /// target triple to build for, e.g. `x86_64-unknown-linux-gnu`
    #[argh(option, short = 't')]
    target: Option<String>,
}

struct ConfigGuard {
    path: PathBuf,
}

impl ConfigGuard {
    fn new(path: PathBuf) -> Self {
        ConfigGuard { path }
    }
}

impl Drop for ConfigGuard {
    fn drop(&mut self) {
        let final_content = r#"[alias]
release = "run --manifest-path ./release/Cargo.toml --"
"#;

        println!("Writing minimal config (aliases only) to {:?}", self.path);
        match fs::write(&self.path, final_content) {
            Ok(_) => (),
            Err(e) => eprintln!("Error writing final config to {:?}: {}", self.path, e),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = argh::from_env();

    if args.target.is_none() {
        eprintln!("Error: At least --target=<triple> is required.");
        eprintln!("Usage: cargo release [OPTIONS]\nSee: cargo release --help");
        std::process::exit(1);
    }

    let project_root = env::current_dir()?;
    let config_path = project_root.join(".cargo/config.toml");

    let build_time_content = r#"[alias]
release = "run --manifest-path ./release/Cargo.toml --"

[unstable]
build-std = ["std", "panic_abort"]
build-std-features = ["panic_immediate_abort"]
trim-paths = true
"#;

    println!("Writing build-time config to {:?}", config_path);
    fs::write(&config_path, build_time_content).map_err(|e| {
        format!(
            "Failed to write build-time config to {:?}: {}",
            config_path, e
        )
    })?;

    let _config_guard = ConfigGuard::new(config_path);

    let base_rustflags = "-Csymbol-mangling-version=v0 -Zlocation-detail=none ";

    let mut success = true;

    match build_frontend(&project_root, base_rustflags) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Frontend build failed: {}", e);
            success = false;
        }
    }

    if success {
        if let Some(target) = args.target.as_ref() {
            match build_app(target, &project_root, base_rustflags) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("App build failed: {}", e);
                    success = false;
                }
            }
        }
    }

    if success {
        println!("Build finished successfully.");
        Ok(())
    } else {
        eprintln!("Build failed.");
        drop(_config_guard);
        std::process::exit(1);
    }
}

fn run_command(
    cmd_path: &str,
    args: &[&str],
    env_vars: &[(&str, &str)],
    cwd: &Path,
) -> Result<(), Box<dyn Error>> {
    println!("Running: {} {}", cmd_path, args.join(" "));
    for (key, val) in env_vars {
        println!("  Env: {}={}", key, val);
    }

    let mut command = Command::new(cmd_path);
    command
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    for (key, val) in env_vars {
        command.env(key, val);
    }

    let status = command
        .spawn()
        .map_err(|e| format!("Failed to spawn command '{}': {}", cmd_path, e))?
        .wait()
        .map_err(|e| format!("Failed to wait for command '{}': {}", cmd_path, e))?;

    if !status.success() {
        Err(Box::new(CommandError {
            command: format!("{} {}", cmd_path, args.join(" ")),
            status,
        }))
    } else {
        Ok(())
    }
}

fn build_frontend(project_root: &PathBuf, base_rustflags: &str) -> Result<(), Box<dyn Error>> {
    println!("Building Macro-Center frontend...");
    let frontend_rustflags = format!("{} -C target-feature=+bulk-memory", base_rustflags);
    let trunk_args = vec!["build", "--release"];

    run_command(
        "trunk",
        &trunk_args,
        &[("RUSTFLAGS", &frontend_rustflags)],
        project_root,
    )
}

fn build_app(
    target: &str,
    project_root: &PathBuf,
    base_rustflags: &str,
) -> Result<(), Box<dyn Error>> {
    let app_rustflags = format!(
        "{} -Zfmt-debug=none -Clink-args=-fuse-ld=lld -Clink-args=-Wl,--icf=all",
        base_rustflags
    );

    println!("Building Macro-Center for {}...", target);

    let cargo_args = vec!["+nightly", "tauri", "build", "--target", target];

    run_command(
        "cargo",
        &cargo_args,
        &[("RUSTFLAGS", &app_rustflags)],
        project_root,
    )
}
