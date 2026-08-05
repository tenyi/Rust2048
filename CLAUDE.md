# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust implementation of the 2048 game, intended as a starting point for building a GUI-based version with cross-platform source compatibility and custom font support (to avoid Chinese character rendering issues).

## Current State

This is an empty scaffolding project:
- `src/main.rs` contains only `println!("Hello, world!")`
- No dependencies configured in Cargo.toml yet
- No game logic, GUI framework, or fonts implemented

## When Building the 2048 Game

### Key Requirements (from user request)
1. **GUI Support**: Use a cross-platform Rust GUI framework such as `egui`, `iced`, or `winit` + `gtk-rs`
2. **Cross-Platform Source Code**: Ensure all source code compiles on Windows, Linux, and macOS
3. **Custom Font Specifier**: Allow users to specify fonts explicitly — this prevents Chinese character garbling (亂碼) in tile numbers/labels

### Recommended Architecture
- `src/main.rs` → Entry point, framework setup
- `src/game/` → Game logic (grid, tiles, merge algorithm, score tracking)
- `src/gui/` or `src/ui/` → Rendering layer (separate from game logic for cross-platform clarity)
- `src/config/` → Font settings and platform detection

### Common Dependencies to Consider
```toml
[dependencies]
egui = "0.29"        # lightweight immediate-mode GUI
eframe = "0.29"       # egui framework application wrapper
serde = { version = "1", features = ["derive"] }  # config serialization
```

## Commands

- **Run**: `cargo run`
- **Build**: `cargo build --release`
- **Lint**: `cargo clippy -- -D warnings`
- **Format**: `cargo fmt`
- **Test**: `cargo test` (once tests are added)

## Development Guidelines

1. Keep game logic separate from GUI rendering — this enables cross-platform source code reuse and makes testing easier
2. When adding font support, load custom fonts via the GUI framework's resource system rather than relying on system defaults
3. Test on your target platforms early; GUI frameworks sometimes behave differently across OSes
4. Add tests for game logic (merge algorithm, win/lose conditions) — these are framework-agnostic and easy to verify
