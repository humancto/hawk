# Contributing to Hawk

Thanks for your interest in contributing! This guide will help you get set up.

## Development Setup

1. **Install Rust 1.75+** via [rustup](https://rustup.rs)

2. **Clone and build:**
   ```bash
   git clone https://github.com/humancto/hawk.git
   cd hawk
   cargo build
   ```

3. **Run tests:**
   ```bash
   cargo test --workspace --exclude hawk_viewer
   ```

4. **Run lints:**
   ```bash
   cargo clippy --workspace --exclude hawk_viewer -- -D warnings
   cargo fmt --all -- --check
   ```

## Project Structure

| Crate | Purpose |
|-------|---------|
| `hawk_core` | Graph model, stats, deduplication, redaction |
| `hawk_aws` | AWS SDK discovery modules (one per service) |
| `hawk_cli` | CLI binary with clap |
| `hawk_render` | Mermaid diagram renderer |
| `hawk_viewer` | Bevy 2D interactive viewer |

## Adding a New AWS Connector

1. Create a new file in `crates/hawk_aws/src/` (e.g., `kinesis.rs`)
2. Implement `pub async fn discover(ctx: &AwsCtx) -> DiscoveryOutput`
3. Add the module to `crates/hawk_aws/src/lib.rs`
4. Call it from `crates/hawk_aws/src/discover.rs` in the `Scope::All` branch
5. Add the AWS SDK dependency to `crates/hawk_aws/Cargo.toml`
6. Add tests for any ARN parsing or data extraction logic
7. Update `README.md` AWS Coverage table
8. Update the IAM policy in the README

## Coding Standards

- **Format:** Run `cargo fmt --all` before committing
- **Lints:** Code must pass `cargo clippy -- -D warnings`
- **Tests:** Add unit tests for parsing, graph operations, and rendering logic
- **Error handling:** Use `anyhow` for CLI, `thiserror` for library errors
- **Determinism:** Output must be sorted and deduped for stable diffs
- **Security:** Never export env var values, secrets, or tokens

## Commit Messages

Use conventional commit prefixes:

- `feat:` new feature
- `fix:` bug fix
- `refactor:` code restructuring
- `test:` adding or updating tests
- `docs:` documentation changes
- `chore:` tooling, dependencies, CI

## Pull Requests

1. Create a feature branch from `main`
2. Keep PRs focused — one feature or fix per PR
3. Ensure all tests pass and clippy is clean
4. Update documentation if behavior changes
5. Add entries to the AWS Coverage table if adding connectors

## Testing Without AWS Credentials

Unit tests don't require AWS credentials. They test:

- ARN parsing and resource kind detection
- Graph deduplication, sorting, and stats
- Mermaid rendering output
- Data redaction

For manual testing with fixture data:

```bash
# Use the sample graph
hawk summary --in examples/sample_graph.json
hawk export mermaid --in examples/sample_graph.json --out test.mmd --full
```

## Reporting Issues

When filing a bug report, please include:

- Hawk version (`hawk --version` once implemented, or commit hash)
- Rust version (`rustc --version`)
- AWS SDK error output (if applicable)
- Sanitized graph JSON (redact account IDs and ARNs if needed)
