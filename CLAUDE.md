# CLAUDE.md

## Project Overview

Hawk is a Rust CLI tool and interactive viewer that discovers AWS Lambda functions, their triggers, and connected services, then renders the architecture as a navigable graph.

## Repository Structure

- `crates/hawk_core` - Core graph data structures and types
- `crates/hawk_aws` - AWS SDK integration and service discovery
- `crates/hawk_cli` - CLI entry point and argument parsing
- `crates/hawk_render` - Graph rendering (Mermaid, DOT, JSON output)
- `apps/hawk_viewer` - Bevy-powered interactive 2D graph viewer

## Build & Test Commands

- `cargo build` - Build all crates
- `cargo test` - Run all tests
- `cargo clippy` - Run linter
- `cargo fmt --check` - Check formatting
- `cargo run -p hawk_cli -- --help` - Run the CLI

## Code Conventions

- Rust edition 2021, MSRV 1.75
- Formatting enforced via `rustfmt.toml`
- Linting enforced via `clippy.toml`
- Workspace uses resolver v2
