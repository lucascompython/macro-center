use clap::{Args, Parser, Subcommand};
use std::env;
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::process::{Command, Stdio};

// To change the build commands (e.g., to use Tauri), update these constants:
//
// Example for default Cargo project:
// const RUN_CMD: &[&str] = &["cargo", "run"];
// const BUILD_CMD: &[&str] = &["cargo", "build", "--release"];
//
// Example for Tauri:
// const RUN_CMD: &[&str] = &["cargo", "tauri", "dev"];
// const BUILD_CMD: &[&str] = &["cargo", "tauri", "build"];
//
// Example for custom profiles:
// const RUN_CMD: &[&str] = &["cargo", "run", "--profile", "fast-dev"];
// const BUILD_CMD: &[&str] = &["cargo", "build", "--profile", "fast-dev"];

const RUN_CMD: &[&str] = &["cargo", "tauri", "dev"];
const BUILD_CMD: &[&str] = &["cargo", "tauri", "build"];

/// Update this with your actual binary name to locate the executable correctly for things like UPX and size reporting.
const BINARY_NAME: &str = "macro-center";

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

#[derive(Parser, Debug)]
#[command(
    name = "xtask",
    author,
    version,
    about = "Build helper for project",
    after_help = "EXTRA ARGS SYNTAX:\n  cargo xtask <PROFILE> [profile-flags] -- [cargo-args] -- [program-args]\n\nEXAMPLES:\n  cargo xtask fast-dev -- --features foo -- --verbose\n  cargo xtask min-size --upx -- --features bar --package my-crate\n  cargo xtask speed --native -- --features foo -- --ignored",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    mode: SubCommands,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    /// Run dev build with fast linking. Syntax: -- [cargo-args] -- [program-args]
    FastDev(FastDevArgs),
    /// Build release optimized for minimum binary size. Syntax: -- [cargo-args]
    MinSize(MinSizeArgs),
    /// Build release optimized for execution speed. Syntax: -- [cargo-args]
    Speed(SpeedArgs),
    /// Run clippy with fast-dev flags
    Clippy(ClippyArgs),
}

/// Splits a `Vec<String>` on the first `"--"` token into `(before, after)`.
///
/// In practice this means the shell invocation:
///   `cargo xtask <profile> [profile-flags] -- [cargo-args] -- [program-args]`
/// has the outer `--` consumed by clap's `last = true`, giving us a flat vec of
/// `[cargo-args..., "--", program-args...]` which this function then splits.
fn split_extra_args(raw: Vec<String>) -> (Vec<String>, Vec<String>) {
    match raw.iter().position(|a| a == "--") {
        Some(pos) => (raw[..pos].to_vec(), raw[pos + 1..].to_vec()),
        None => (raw, vec![]),
    }
}

#[derive(Args, Debug)]
struct FastDevArgs {
    /// Pass `-- [cargo-args] -- [program-args]` after the profile flags
    #[arg(last = true)]
    raw_args: Vec<String>,
}

#[derive(Args, Debug)]
struct ClippyArgs {}

#[derive(Args, Debug)]
struct MinSizeArgs {
    /// Target triple to build for, e.g. `x86_64-unknown-linux-gnu`
    #[arg(short, long)]
    target: Option<String>,

    /// Whether to compress the final binary with upx
    #[arg(long)]
    upx: bool,

    /// Pass `-- [cargo-args]` after the profile flags (program-args are ignored for build-only profiles)
    #[arg(last = true)]
    raw_args: Vec<String>,
}

#[derive(Args, Debug)]
struct SpeedArgs {
    /// Target triple to build for, e.g. `x86_64-unknown-linux-gnu`
    #[arg(short, long)]
    target: Option<String>,

    /// Compile with -C target-cpu=native
    #[arg(long)]
    native: bool,

    /// Pass `-- [cargo-args]` after the profile flags (program-args are ignored for build-only profiles)
    #[arg(last = true)]
    raw_args: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.mode {
        SubCommands::FastDev(args) => build_fast_dev(args)?,
        SubCommands::MinSize(args) => build_min_size(args)?,
        SubCommands::Speed(args) => build_speed(args)?,
        SubCommands::Clippy(args) => run_clippy(args)?,
    }

    Ok(())
}

fn get_fast_dev_rustflags() -> String {
    let linker_arg = if cfg!(target_os = "windows") {
        "-Clinker=rust-lld.exe"
    } else if cfg!(target_os = "linux") {
        "-Clinker=clang -Clink-arg=--ld-path=mold" // use mold while https://github.com/wild-linker/wild/issues/1576 isnt fixed
    } else {
        ""
    };

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    // disabled cranelift backend because of: https://github.com/tauri-apps/tauri/issues/14928
    format!(
        "{} -Zthreads={} -Zshare-generics=y",
        linker_arg, num_threads
    )
}

fn build_fast_dev(args: FastDevArgs) -> Result<(), Box<dyn Error>> {
    let project_root = env::current_dir()?;
    let dev_rustflags = get_fast_dev_rustflags();

    let (cargo_args, program_args) = split_extra_args(args.raw_args);

    println!("Building in dev mode (fast build)...");

    let cmd = RUN_CMD[0];
    let mut full_args = vec!["+nightly"];
    full_args.extend_from_slice(&RUN_CMD[1..]);
    full_args.extend(cargo_args.iter().map(|s| s.as_str()));
    if !program_args.is_empty() {
        full_args.push("--");
        full_args.extend(program_args.iter().map(|s| s.as_str()));
    }

    run_command(
        cmd,
        &full_args,
        &[("RUSTFLAGS", &dev_rustflags)],
        &project_root,
    )?;

    println!("Fast-dev build finished successfully.");
    Ok(())
}

fn run_clippy(_args: ClippyArgs) -> Result<(), Box<dyn Error>> {
    let project_root = env::current_dir()?;
    let dev_rustflags = get_fast_dev_rustflags();

    println!("Running clippy with fast-dev flags...");

    run_command(
        "cargo",
        &["+nightly", "clippy"],
        &[("RUSTFLAGS", &dev_rustflags)],
        &project_root,
    )?;

    println!("Clippy finished successfully.");
    Ok(())
}

fn build_min_size(args: MinSizeArgs) -> Result<(), Box<dyn Error>> {
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

    let app_rustflags = "-Csymbol-mangling-version=v0 -Zunstable-options -Cdebuginfo=0 -Cpanic=immediate-abort -Zfmt-debug=none -Zlocation-detail=none -Clink-args=-fuse-ld=lld -Clink-args=-Wl,--icf=all,-z,pack-relative-relocs -Copt-level=z";

    let env_vars = vec![
        ("RUSTFLAGS", app_rustflags),
        ("CARGO_UNSTABLE_BUILD_STD", "std,panic_abort"),
        ("CARGO_UNSTABLE_BUILD_STD_FEATURES", "optimize_for_size"),
        ("CARGO_UNSTABLE_TRIM_PATHS", "true"),
    ];

    let binary_path = project_root
        .join("target")
        .join(target)
        .join("release")
        .join(BINARY_NAME)
        .with_extension(std::env::consts::EXE_EXTENSION);

    let (cargo_args, _) = split_extra_args(args.raw_args);

    build_app(
        target,
        &project_root,
        &binary_path,
        args.upx,
        &env_vars,
        &cargo_args,
    )
}

fn build_speed(args: SpeedArgs) -> Result<(), Box<dyn Error>> {
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

    let mut rustflags = "-Copt-level=3 -Csymbol-mangling-version=v0 -Zunstable-options -Cdebuginfo=0 -Cpanic=immediate-abort -Zfmt-debug=none -Zlocation-detail=none -Clink-args=-fuse-ld=lld -Clink-args=-Wl,--icf=all,-z,pack-relative-relocs".to_string();
    if args.native {
        rustflags.push_str(" -Ctarget-cpu=native");
    }

    let env_vars = vec![
        ("RUSTFLAGS", rustflags.as_str()),
        ("CARGO_UNSTABLE_BUILD_STD", "std,panic_abort"),
        ("CARGO_UNSTABLE_TRIM_PATHS", "true"),
    ];

    let binary_path = project_root
        .join("target")
        .join(target)
        .join("release")
        .join(BINARY_NAME)
        .with_extension(std::env::consts::EXE_EXTENSION);

    let (cargo_args, _) = split_extra_args(args.raw_args);

    build_app(
        target,
        &project_root,
        &binary_path,
        false,
        &env_vars,
        &cargo_args,
    )
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
    env_vars: &[(&str, &str)],
    extra_cargo_args: &[String],
) -> Result<(), Box<dyn Error>> {
    println!("Building for {}...", target);

    let cmd = BUILD_CMD[0];
    let mut args = vec!["+nightly"];
    args.extend_from_slice(&BUILD_CMD[1..]);
    args.extend_from_slice(&["--target", target]);
    args.extend(extra_cargo_args.iter().map(|s| s.as_str()));

    let build_result = run_command(cmd, &args, env_vars, project_root);

    let build_ok = build_result.is_ok();

    if build_ok && upx {
        println!("Compressing binary with upx: {:?}", binary_path);
        let upx_result = run_command(
            "upx",
            &["--ultra-brute", "--best", binary_path.to_str().unwrap()],
            &[],
            project_root,
        );
        if let Err(e) = upx_result {
            eprintln!("UPX compression failed: {}", e);
        }
    } else if let Err(e) = &build_result {
        eprintln!("App build failed: {}", e);
        std::process::exit(1);
    }

    if build_ok {
        println!("Release build finished successfully.");
        if let Ok(metadata) = std::fs::metadata(binary_path) {
            println!("Final binary size: {} bytes", metadata.len());
        } else {
            println!(
                "Could not read final binary size (path might be incorrect: {:?}).",
                binary_path
            );
        }
    }

    build_result
}
