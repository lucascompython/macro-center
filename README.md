# Macro Center

A Crossplatform Visual Programming/Scripting App for keyboard and mouse macros

## Features

- Visual programming interface for creating macros
- Support for multiple platforms: Windows, macOS, Linux (x11 and wayland as soon as [162](https://github.com/tauri-apps/global-hotkey/pull/162) gets merged)
- Submacro support for reusable code blocks
- Data nodes for storing and manipulating variables and control flow
- Record and playback functionality for easy macro creation

## How to build

You need the nightly toolchain of Rust installed.  
For development builds, you also need to install the cranelift backend for better compile times.

```bash
git clone https://github.com/lucascompython/macro-center
cd macro-center
# install deps
cd src && bun install

# build
cd ..
cargo xtask dev
# or
cargo xtask release
```

## Demo

![macro-center-demo](demo/demo.png)
