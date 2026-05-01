# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Purpose

A timer display for a 24-hour endurance event. Shows elapsed/remaining time and displays messages sent by families/friends that appear on screen for the runner to see.

## Build & Run

```bash
cargo build          # debug build
cargo build --release # release build
cargo run            # run the program
cargo test           # run all tests
cargo test <name>    # run a single test by name
cargo clippy         # lint
cargo fmt            # format code
```

## Architecture

Rust project using edition 2024. Structure is minimal — single binary crate with entry point at `src/main.rs`.
