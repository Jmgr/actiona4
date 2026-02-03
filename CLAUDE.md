# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Actiona-ng is a cross-platform desktop automation tool written in Rust. It provides a TypeScript/JavaScript scripting interface (via QuickJS) to control mouse, keyboard, windows, clipboard, audio, and other system functions. The UI layer uses Tauri.

## Build Commands

```bash
# Install dependencies (first time setup)
cargo install cargo-make
sudo apt install pkg-config libopencv-dev clang libclang-dev libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev libxkbcommon-x11-dev llvm libclang-dev libgtk-3-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev libwebkit2gtk-4.1-dev libopencv-dev libxkbcommon-x11-dev

# Build the project
cargo build

# Run the CLI
cargo run -p actiona-ng-cli -- run <script.ts>
cargo run -p actiona-ng-cli -- eval "console.log('hello')"
cargo run -p actiona-ng-cli -- repl

# Run tests (standard tests)
cargo test

# Run UI tests (custom harness, single-threaded)
cargo test -p actiona-ng --test ui

# Format code (requires nightly rustfmt, cargo-derivefmt, cargo-sort)
cargo make format

# Lint
cargo make lint
```

## Architecture

### Workspace Structure

- **actiona-ng**: Core library containing the runtime, JavaScript bindings, and all automation functionality
- **actiona-ng-cli**: CLI application with run/eval/repl commands
- **macros**: Proc macros for JavaScript/serde interop (`FromJsObject`, `IntoSerde`, `FromSerde`)
- **doc-generator**: Tool for generating documentation from rustdoc JSON

### Core Library (actiona-ng/src)

- **runtime/mod.rs**: Main `Runtime` struct that orchestrates everything. Creates the QuickJS engine, registers all JS classes, manages cancellation tokens and task tracking. `run_with_ui()` starts Tauri + async runtime together.
- **scripting/**: TypeScript-to-JavaScript transpilation (via SWC), sourcemap handling for error translation, and the `Engine` wrapper around QuickJS's `AsyncRuntime`/`AsyncContext`
- **core/**: All automation modules (mouse, keyboard, clipboard, screenshot, system, etc.). Each module typically has:
  - `mod.rs`: Platform-agnostic Rust implementation
  - `js.rs`: JavaScript bindings using rquickjs
  - `platform/`: OS-specific implementations (x11.rs, win.rs)
- **platform/**: Low-level platform utilities (X11 connection management, Windows handles)
- **types/**: Shared types like Point, Size, Rect, Pid

### JavaScript Class Registration Pattern

Classes exposed to JavaScript follow one of three patterns:

1. **SingletonClass**: Single global instance (e.g., `mouse`, `keyboard`, `clipboard`)
2. **ValueClass**: User-instantiable classes (e.g., `Point`, `Size`, `Image`, `File`)
3. **HostClass**: Classes returned by API but not user-constructible (e.g., `system.cpu`)

Register classes in `Runtime::register_classes()`. The `Js` prefix is stripped from Rust struct names (e.g., `JsMouse` becomes `mouse` in JS).

### Platform Abstraction

Platform-specific code uses cfg attributes:
- `#[cfg(linux)]` for Linux code
- `#[cfg(unix)]` for X11/Unix code
- `#[cfg(windows)]` for Windows code

Platform-specific code is implemented in `platform` directories.

### Async Model

- Tokio for async runtime
- QuickJS async context for JavaScript Promises
- swc for TypeScript to JavaScript conversion
- `CancellationToken` and `TaskTracker` from tokio-util for graceful shutdown
- Background tasks increment `background_tasks_counter` to determine if runtime should wait at end

## Key Conventions

- Use `#[instrument(skip_all)]` from tracing for function-level instrumentation
- Errors use `color_eyre::Result` and `thiserror` for custom error types
- JavaScript methods use camelCase naming via `#[rquickjs::methods(rename_all = "camelCase")]`
- Clippy lints: `#![warn(clippy::all, clippy::nursery)]` and `#![deny(unsafe_code)]`
- Rust edition 2024 with stable toolchain (nightly only for rustfmt)
