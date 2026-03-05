# CLAUDE.md

## Hawk Implementation Guide (Rust Workspace + AWS Discovery + Graph + Viewer)

Owner: HumanCTO
Language: Rust
Primary Outputs: CLI analyzer + JSON graph exporter + Bevy sprite viewer

This file is an implementation guide for Claude Code. It should produce working code, not just design.

---

# 0. What to build first

Build Hawk in 3 milestones.

## Milestone A (CLI + Graph + JSON export) — DONE

- Created a Rust workspace with crates:
  - `hawk_core` (graph model, merge, stats)
  - `hawk-cloud` (CLI binary, published as `hawk-cloud` on crates.io, binary name `hawk`)
  - `hawk_aws` (AWS discovery for Lambda + triggers)
- Implemented:
  - `hawk analyze aws lambda --profile X --region Y --out hawk.json`
  - `hawk export mermaid --in hawk.json --out hawk.mmd`
  - `hawk summary --in hawk.json`

## Milestone B (Add more AWS connectors) — DONE

Added:

- EventBridge rules -> Lambda
- S3 notifications -> Lambda
- SNS topics -> Lambda
- CloudWatch Logs subscription filters -> Lambda
- Step Functions -> Lambda (parse definition)
- API Gateway v2 (HTTP APIs) -> Lambda

## Milestone C (Bevy sprite viewer) — PARTIALLY DONE

- `hawk_viewer` reads `hawk.json`
- Renders colored square sprites for nodes
- Renders edges as lines
- Search and layer filter toggles
- **Missing**: pan/zoom, force-directed layout, node dragging, AWS-specific icons

---

# 1. Repo and workspace layout

```
hawk/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── hawk_core/              # Graph model, stats, dedupe, redaction
│   ├── hawk_aws/               # AWS SDK discovery modules (7 connectors)
│   ├── hawk-cloud/             # CLI binary (clap-based), crate name: hawk-cloud
│   └── hawk_render/            # Mermaid + Graphviz DOT renderer
├── apps/
│   └── hawk_viewer/            # Bevy 2D interactive viewer
├── assets/
│   └── screenshots/            # README screenshots (fake data only!)
├── docs/
│   └── index.html              # GitHub Pages landing page
├── .github/workflows/
│   ├── ci.yml                  # CI: check, test, clippy, fmt
│   ├── release.yml             # Cross-platform binary builds + crates.io publish
│   └── pages.yml               # GitHub Pages deploy
└── examples/
    └── sample_graph.json       # Example output for testing
```

## Build & Test Commands

```bash
cargo check --workspace --exclude hawk_viewer   # Check compilation
cargo test --workspace --exclude hawk_viewer     # Run all tests
cargo clippy --workspace --exclude hawk_viewer   # Run clippy
cargo fmt --all                                  # Format code
cargo run -p hawk-cloud -- --help                # Run the CLI
cargo run --release -p hawk_viewer -- hawk.json  # Launch viewer
```

Note: `hawk_viewer` is excluded from default builds due to Bevy version conflicts.

## Code Conventions

- Rust edition 2021, MSRV 1.75
- Workspace resolver v2
- Authors: HumanCTO
- License: MIT
- Published crates: hawk_core, hawk_aws, hawk_render, hawk-cloud (all on crates.io)
- Homebrew: `brew install humancto/tap/hawk`

---

# 2. Dependencies

## hawk_core

- `serde`, `serde_json`
- `thiserror`
- `indexmap` (stable ordering for deterministic diffs)

## hawk-cloud (CLI)

- `clap` (derive)
- `anyhow`
- `tokio`
- `tracing`, `tracing-subscriber`
- depends on `hawk_core`, `hawk_aws`, `hawk_render`

## hawk_aws

- AWS SDK for Rust crates:
  - `aws-config`
  - `aws-sdk-lambda`
  - `aws-sdk-events` (EventBridge)
  - `aws-sdk-s3`
  - `aws-sdk-sns`
  - `aws-sdk-logs`
  - `aws-sdk-sfn` (Step Functions)
  - `aws-sdk-apigatewayv2`
  - `aws-sdk-sqs` (optional, mostly for metadata)
- `tokio`
- `regex` (parsing ARNs and integration URIs)
- `serde_json` (Step Functions definition parse)

## hawk_render

- `serde_json`
- `anyhow`

## hawk_viewer (Bevy)

- `bevy`
- `bevy_egui` (UI overlay)
- `serde`, `serde_json`
- depends on `hawk_core`

---

# 3. Graph model (hawk_core)

## NodeKind

Enum:

- Lambda, ApiGateway, ApiRoute, EventRule, SqsQueue, SnsTopic, S3Bucket
- DynamoStream, StepFunction, LogGroup, EcsService, Ec2Instance, LoadBalancer, Unknown

## EdgeKind

Enum:

- Triggers, Invokes, Consumes, Publishes, ReadsFrom, WritesTo

## Node

Fields:

- `id: String` (stable key, prefer ARN)
- `kind: NodeKind`
- `name: String`
- `arn: Option<String>`
- `region: Option<String>`
- `account_id: Option<String>`
- `tags: Option<IndexMap<String, String>>`
- `props: serde_json::Value`

## Edge

Fields:

- `from: String`, `to: String`
- `kind: EdgeKind`
- `props: serde_json::Value`

## Graph

Fields:

- `generated_at: String` (ISO8601)
- `profile: Option<String>`
- `regions: Vec<String>`
- `nodes: Vec<Node>`, `edges: Vec<Edge>`
- `warnings: Vec<String>`
- `stats: GraphStats`

Implement: `Graph::dedupe_and_sort()`, `Graph::compute_stats()`

---

# 4-12. AWS Discovery Modules

See sections 4-12 of the original spec. All 7 connectors are implemented:

- Lambda + event source mappings (SQS, DynamoDB, Kinesis)
- EventBridge rules -> Lambda
- S3 notifications -> Lambda
- SNS subscriptions -> Lambda
- CloudWatch Logs subscription filters -> Lambda
- Step Functions -> Lambda (definition parse)
- API Gateway v2 -> Lambda

---

# 13. CLI commands (hawk-cloud)

```bash
hawk analyze aws lambda --profile X --region Y --out hawk.json
hawk analyze aws all --profile X --region Y --out hawk.json --pretty
hawk summary --in hawk.json [--format text|json]
hawk export mermaid --in hawk.json --out hawk.mmd [--full]
hawk export dot --in hawk.json --out hawk.dot [--full]
hawk diff --old a.json --new b.json [--format text|json] [--exit_code]
```

---

# 14. Bevy viewer (apps/hawk_viewer)

### Current state

- 2D camera with colored square sprites per node type
- Edges as gray lines (yellow when highlighted)
- Click to select, search bar, layer toggles
- Band-based deterministic layout (triggers → orchestration → compute → storage)

### Missing / TODO

- Pan (drag) and zoom (scroll wheel) — documented in README but NOT implemented
- Force-directed layout — roadmap item
- AWS-specific icons/sprites — currently just colored rectangles
- Node dragging
- Interactive HTML/SVG export
- Direct image export (screenshot/render-to-file)

---

# 15. Data safety and redaction rules

Do not output: env var values, secrets, raw auth tokens, inline policy documents.
Allowed: env var keys only, ARNs, non-secret config fields.

---

# 16. Screenshots

All README screenshots use completely fake/generic data. NEVER use real AWS account data in screenshots.

---

# 17. Release & Distribution

- **crates.io**: hawk_core, hawk_render, hawk_aws, hawk-cloud (publish in dependency order)
- **GitHub Releases**: Cross-platform binaries (x86_64/aarch64 for Linux/macOS) via release.yml
- **Homebrew**: `brew install humancto/tap/hawk` (formula auto-updated on release)
- **GitHub Pages**: Landing page at https://humancto.github.io/hawk

---

# 18. Roadmap

- [ ] API Gateway v1 (REST APIs) discovery
- [ ] Multi-region scanning in a single run
- [ ] Multi-account scanning (AWS Organizations)
- [ ] Force-directed graph layout in the viewer
- [ ] Pan/zoom in viewer
- [ ] AWS-specific node icons/sprites
- [ ] HTML export with interactive SVG
- [ ] Cost annotations via Cost Explorer API
- [ ] CloudFormation / CDK stack grouping
- [ ] Terraform state file import
- [ ] GCP and Azure support (future cloud providers)
