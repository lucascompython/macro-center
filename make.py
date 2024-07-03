#!/usr/bin/env python3

import argparse
import os
import sys
import subprocess
from time import perf_counter
from shutil import rmtree
from typing import Sequence

OS = sys.platform
if OS == "win32":
    OS = "windows"
VERBOSE = False
USE_MINGW = False


class Colors:
    """ANSI color codes."""

    RED = "\033[91m"
    GREEN = "\033[92m"
    YELLOW = "\033[93m"
    BLUE = "\033[94m"
    MAGENTA = "\033[95m"
    CYAN = "\033[96m"
    BOLD = "\033[1m"
    UNDERLINE = "\033[4m"
    ITALIC = "\033[3m"
    END = "\033[0m"

    @classmethod
    def warning_message(cls, message: str) -> str:
        return f"{cls.YELLOW}{cls.BOLD}WARNING:{cls.END} {message}"

    @classmethod
    def error_message(cls, message: str) -> str:
        return f"{cls.RED}{cls.BOLD}ERROR:{cls.END} {message}"

    @classmethod
    def success_message(cls, message: str) -> str:
        return f"{cls.GREEN}{cls.BOLD}SUCCESS:{cls.END} {message}"

    @classmethod
    def info_message(cls, message: str) -> str:
        return f"{cls.BLUE}{cls.BOLD}INFO:{cls.END} {message}"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=f"{Colors.UNDERLINE}Auto-Spammer{Colors.END} build script."
    )

    parser.add_argument(
        "-d", "--dev", action="store_true", help="Build in development mode."
    )
    parser.add_argument(
        "-r", "--release", action="store_true", help="Build in release mode."
    )
    parser.add_argument(
        "-c",
        "--clean",
        action="store_true",
        help="Cleans the build directories and dependencies.",
    )
    parser.add_argument(
        "-v", "--verbose", action="store_true", help="Prints verbose output."
    )
    parser.add_argument(
        "-u",
        "--upx",
        action="store_true",
        help=Colors.warning_message(
            f"Compress the executable using {Colors.ITALIC}UPX{Colors.END}. Might cause false positives in some antivirus software."
        ),
    )
    parser.add_argument(
        "-t",
        "--target",
        type=str,
        help=f"The target platform to build for. Default: {Colors.BOLD + OS + Colors.END} (current OS)",
        choices=("windows", "linux"),
        default=OS,
    )
    parser.add_argument(
        "-n",
        "--nightly",
        action="store_true",
        help=Colors.warning_message(
            f"Build using the {Colors.UNDERLINE}nightly{Colors.END} toolchain for a smaller binary size. See {Colors.UNDERLINE}https://github.com/johnthagen/min-sized-rust{Colors.END}"
        ),
    )
    parser.add_argument(
        "--run", action="store_true", help="Run the executable after building."
    )
    parser.add_argument(
        "--native",
        action="store_true",
        help=f"Build using the {Colors.ITALIC}'target-cpu=native'{Colors.END} flag. This will optimize the binary for the current CPU architecture. {Colors.YELLOW}Not recommended for cross-compilation{Colors.END}.",
    )
    parser.add_argument(
        "-m",
        "--mold",
        action="store_true",
        help=f"Use the {Colors.ITALIC}mold{Colors.END} linker. {Colors.UNDERLINE}Only works on Linux.{Colors.END}",
    )
    parser.add_argument(
        "-s",
        "--smallest",
        action="store_true",
        help=f"Build the smallest possible binary size. Enables {Colors.ITALIC}--release{Colors.END}, {Colors.ITALIC}--upx{Colors.END}, and {Colors.ITALIC}--nightly{Colors.END}.",
    )
    parser.add_argument(
        "--mingw",
        action="store_true",
        help=f"Use {Colors.BOLD}MinGW{Colors.END} on Windows.",
    )

    return parser.parse_args()


def verbose_print(message: str) -> None:
    if VERBOSE:
        print(message)


def run_command(command: Sequence, **kwargs) -> None:
    try:
        if VERBOSE:
            subprocess.run(command, check=True, **kwargs)
        else:
            with open(os.devnull, "w") as devnull:
                p = subprocess.Popen(
                    command, stdout=devnull, stderr=devnull, **kwargs
                ).wait()
                if p != 0:
                    raise subprocess.CalledProcessError(p, command)

    except subprocess.CalledProcessError as e:
        if not VERBOSE:
            e = f"{e}\nRun with {Colors.ITALIC}--verbose{Colors.END} flag for more information."
        sys.stderr.write(f"{Colors.error_message(f'{e}')}\n")
        sys.exit(1)


def clean() -> None:
    print(Colors.info_message("Cleaning build directories and dependencies..."))
    if VERBOSE:
        start = perf_counter()

    run_command(("cargo", "clean"), cwd="src-tauri")

    try:
        rmtree("src-frontend/dist")
    except FileNotFoundError:
        pass

    try:
        rmtree("src-frontend/node_modules")
    except FileNotFoundError:
        pass

    if VERBOSE:
        end = perf_counter()
        print(
            f"{Colors.success_message(f'Cleaned build directories and dependencies in {end - start:.2f} seconds.')}\n"
        )  # type: ignore


def check_node_modules() -> None:
    verbose_print(Colors.info_message("Checking for frontend dependencies..."))
    run_command(("bun", "install"), cwd="src-frontend")


def get_target(target: str) -> str:
    if target == "linux":
        return "x86_64-unknown-linux-gnu"
    elif target == "linux" and OS == "windows":
        if USE_MINGW:
            return "x86_64-unknown-linux-gnu"
        return "x86_64-unknown-linux-musl"

    elif target == "windows" and OS == "linux":
        return "x86_64-pc-windows-gnu"

    elif target == "windows" and OS == "windows":
        if USE_MINGW:
            return "x86_64-pc-windows-gnu"

        return "x86_64-pc-windows-msvc"

    else:
        sys.stderr.write(f"{Colors.error_message(f'Invalid target: {target}.')}\n")
        sys.exit(1)


def get_app_name(target: str) -> str:
    app_name = "auto-spammer"
    if target == "windows":
        app_name += ".exe"

    return app_name


def build_release(args: argparse.Namespace) -> float | None:
    print(Colors.info_message("Building in release mode..."))
    check_node_modules()

    target = get_target(args.target)

    command = ["cargo", "tauri", "build", "--target", target]
    rustflags = []

    if args.mold:
        if OS != "linux":
            print(
                Colors.warning_message(
                    "The mold linker only works on Linux. Ignoring --mold flag."
                )
            )
        else:
            rustflags.append("-C link-arg=-fuse-ld=mold")

    if "msvc" in target:
        rustflags.append(
            "-C target-feature=+crt-static"
        )  # Make sure the binary is statically linked
        # rustflags.append("-C linker=rust-lld")

    if args.nightly:
        command.insert(1, "+nightly")
        rustflags.append("-Zlocation-detail=none")

        command.extend(
            [
                "--",
                "-Z",
                "build-std=std,panic_abort",
                "-Z",
                "build-std-features=panic_immediate_abort",
                "-Z",
                "trim-paths",
            ]
        )

    if args.native:
        rustflags.append("-C target-cpu=native")

    env = os.environ.copy()
    run_command(command, cwd="src-tauri", env={**env, "RUSTFLAGS": " ".join(rustflags)})

    if args.upx:
        upx_start = perf_counter()
        print(Colors.info_message("Compressing executable using UPX..."))

        run_command(
            (
                "upx",
                "--ultra-brute",
                f"src-tauri/target/{target}/release/{get_app_name(args.target)}",
            )
        )
        upx_time = perf_counter() - upx_start

        return upx_time


def convert_bytes(num: int | float | str) -> str:
    if isinstance(num, str):
        num = float(num)
    """
    this function will convert bytes to MB.... GB... etc
    """
    for x in ["bytes", "KB", "MB", "GB", "TB"]:
        if num < 1024.0:
            return "%3.1f %s" % (num, x)
        num /= 1024.0
    return f"{num:.1f} PB"


def get_size(target: str) -> str:
    app_name = get_app_name(target)
    target = get_target(target)

    try:
        size = os.path.getsize(f"src-tauri/target/{target}/release/{app_name}")
        return convert_bytes(size)
    except FileNotFoundError:
        print(
            Colors.warning_message(
                f"Executable not found at {Colors.UNDERLINE}src-tauri/target/{target}/release/{app_name}{Colors.END}."
            )
        )
        return "N/A"


def run_app(target: str) -> None:
    if OS != target:
        print(
            Colors.warning_message(
                f"Cannot run {Colors.BOLD + target.capitalize() + Colors.END} executable on {Colors.BOLD + OS.capitalize() + Colors.END}."
            )
        )
        return

    app_path = f"./src-tauri/target/{get_target(target)}/release/{get_app_name(target)}"

    if OS == "linux":
        command = ["/usr/bin/time", "-f", "%M", app_path]
    elif OS == "windows":
        command = [
            "powershell",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            "get_peak_mem.ps1",
            app_path,
        ]
    else:
        sys.stderr.write(f"{Colors.error_message(f'Unsupported OS: {OS}.')}\n")
        sys.exit(1)

    print(Colors.info_message(f"Running {Colors.UNDERLINE + app_path + Colors.END}..."))
    proc = subprocess.run(command, capture_output=True)

    if proc.returncode != 0:
        sys.stderr.write(f"{Colors.error_message(f'Error running {app_path}.')}\n")
        sys.exit(1)

    if OS == "linux":
        peak_mem = proc.stderr.decode().strip().split("\n")[-1]
    else:
        peak_mem = proc.stdout.decode().strip().split("\n")[-1]

    peak_mem = convert_bytes(peak_mem + "000" if OS == "linux" else peak_mem)

    print(
        Colors.info_message(
            f"Peak Memory Usage: {Colors.BOLD}{peak_mem}{Colors.END}. {f'{Colors.ITALIC}Note: This {Colors.BOLD}IS{Colors.END} not really accurate, because, well, Windows sucks.{Colors.END}' if OS == 'windows' else ''}"
        )
    )


def main() -> None:
    args = parse_args()

    if len(sys.argv) == 1:
        sys.argv.append("-h")
        args = parse_args()

    global VERBOSE
    VERBOSE = args.verbose
    if args.smallest:
        args.release = True
        args.upx = True
        args.nightly = True

    if args.dev and args.release:
        sys.stderr.write(
            f"{Colors.error_message(f'Cannot use both {Colors.ITALIC}--dev{Colors.END} and {Colors.ITALIC}--release{Colors.END} flags at the same time.')}\n"
        )
        sys.exit(1)

    if args.clean:
        clean()

    if args.dev:
        print(Colors.info_message("Building in development mode..."))
        VERBOSE = True  # Always print output in development mode
        check_node_modules()
        run_command(("cargo", "tauri", "dev"), cwd="src-tauri")
        VERBOSE = args.verbose  # Reset verbosity to user's preference

    elif args.release:
        start = perf_counter()
        if OS == "windows" and args.mingw:
            global USE_MINGW
            USE_MINGW = True
        upx_time = build_release(args)
        total_time = perf_counter() - start
        if upx_time:
            print(
                "\n"
                + Colors.info_message(
                    f"Compilation Time: {total_time - upx_time:.2f} seconds."
                )
            )
            print(Colors.info_message(f"UPX Compression Time: {upx_time:.2f} seconds."))

        print(
            "\n"
            + Colors.success_message(
                f"{Colors.BOLD}Finished building in {perf_counter() - start:.2f} seconds.{Colors.END}"
            )
        )
        print(
            Colors.info_message(
                f"{Colors.BOLD}Executable Size: {get_size(args.target)}"
            )
        )

    if args.run:
        run_app(args.target)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print(f"\n{Colors.MAGENTA}Exiting...{Colors.END}")
else:
    sys.stderr.write(f"{Colors.error_message('This script cannot be imported.')}\n")
    sys.exit(1)
