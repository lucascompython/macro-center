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

#[derive(FromArgs)]
#[argh(subcommand)]
enum SubCommands {
    Release(ReleaseArgs),
    Dev(DevArgs),
}

#[derive(FromArgs)]
/// Build fast to build dev version of Macro Center
#[argh(subcommand, name = "dev")]
struct DevArgs {}

#[derive(FromArgs)]
/// Build release version of Macro Center
#[argh(subcommand, name = "release")]
struct ReleaseArgs {
    /// target triple to build for, e.g. `x86_64-unknown-linux-gnu`
    #[argh(option, short = 't')]
    target: Option<String>,

    /// whether to compress the final binary with upx
    #[argh(switch)]
    upx: bool,
}

/// Build helper for Macro Center
#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help", "help"))]
struct Args {
    #[argh(subcommand)]
    mode: SubCommands,
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
xtask = "run --manifest-path ./xtask/Cargo.toml --"
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

    match args.mode {
        SubCommands::Release(release_args) => build_release(release_args)?,
        SubCommands::Dev(dev_args) => build_dev(dev_args)?,
    }

    Ok(())
}

fn build_dev(_args: DevArgs) -> Result<(), Box<dyn Error>> {
    let project_root = env::current_dir()?;

    let dev_rustflags = "-Clinker=clang -Clink-arg=-fuse-ld=/usr/bin/wild -Zthreads=32 -Zcodegen-backend=cranelift -Zshare-generics=y";

    println!("Building Macro-Center in dev mode (fast build)...");

    let cargo_args = vec![
        "+nightly",
        "tauri",
        "dev",
        "--",
        "--profile",
        "fast-build",
        "-Zcodegen-backend",
    ];

    run_command(
        "cargo",
        &cargo_args,
        &[("RUSTFLAGS", dev_rustflags)],
        &project_root,
    )?;

    println!("Dev build finished successfully.");
    Ok(())
}

fn build_release(args: ReleaseArgs) -> Result<(), Box<dyn Error>> {
    let target = args.target.as_deref().unwrap_or_else(|| {
        if cfg!(target_os = "windows") {
            "x86_64-pc-windows-msvc"
        } else if cfg!(target_os = "linux") {
            "x86_64-unknown-linux-gnu"
        } else if cfg!(target_os = "macos") {
            "x86_64-apple-darwin"
        } else {
            panic!("Unsupported host OS");
        }
    });

    let project_root = env::current_dir()?;
    let config_path = project_root.join(".cargo/config.toml");

    let build_time_content = r#"[alias]
xtask = "run --manifest-path ./xtask/Cargo.toml --"

[unstable]
build-std = ["std", "panic_abort"]
build-std-features = ["optimize_for_size"]
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

    let mut success = true;

    let binary_path = project_root
        .join("src-tauri")
        .join("target")
        .join(target)
        .join("release")
        .join("macro-center")
        .with_extension(std::env::consts::EXE_EXTENSION);

    match build_app(target, &project_root, &binary_path, args.upx) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("App build failed: {}", e);
            success = false;
        }
    };

    if success {
        println!("Release build finished successfully.");

        let metadata = fs::metadata(&binary_path)?;
        let file_size = metadata.len();
        println!("Final binary size: {} bytes", file_size);

        Ok(())
    } else {
        eprintln!("Release build failed.");
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

fn build_app(
    target: &str,
    project_root: &Path,
    binary_path: &Path,
    upx: bool,
) -> Result<(), Box<dyn Error>> {
    let app_rustflags = "-Csymbol-mangling-version=v0 -Zunstable-options -Cdebuginfo=0 -Cpanic=immediate-abort -Zfmt-debug=none -Zlocation-detail=none -Clink-args=-fuse-ld=lld -Clink-args=-Wl,--icf=all,-z,pack-relative-relocs";

    println!("Building Macro-Center for {}...", target);

    let cargo_args = vec!["+nightly", "tauri", "build", "--target", target];

    let build_result = run_command(
        "cargo",
        &cargo_args,
        &[("RUSTFLAGS", app_rustflags)],
        project_root,
    );

    if upx {
        println!("Compressing binary with upx: {:?}", binary_path);

        run_command(
            "upx",
            &["--ultra-brute", "--best", binary_path.to_str().unwrap()],
            &[],
            project_root,
        )
    } else {
        build_result
    }
}
