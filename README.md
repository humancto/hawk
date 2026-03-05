# Hawk

**Map your AWS serverless architecture in seconds.**

Hawk is a CLI tool and interactive viewer that automatically discovers AWS Lambda functions, their triggers, and connected services — then renders the entire architecture as a navigable graph.

Point it at an AWS account, get a complete picture of what triggers what.

```
                ┌─────────────┐     ┌─────────────┐
                │ EventBridge │────▶│   Lambda A   │
                └─────────────┘     └──────┬──────┘
                                           │
┌─────────────┐     ┌─────────────┐        ▼
│  S3 Bucket  │────▶│   Lambda B   │◀── SNS Topic
└─────────────┘     └─────────────┘
                                     ▲
                ┌─────────────┐      │
                │ Step Function├─────┘
                └─────────────┘
```

---

## About

Hawk scans your AWS account using the AWS SDK for Rust, builds a directed graph of Lambda functions and every service that triggers or invokes them, and outputs the result as structured JSON. From there you can generate Mermaid diagrams, diff snapshots over time, or explore the graph interactively in a Bevy-powered 2D viewer.

**Why Hawk?**
- You inherited an AWS account and need to understand what's connected to what
- You want to audit Lambda trigger chains before making changes
- You need a visual map for architecture reviews or onboarding
- You want to track infrastructure drift by diffing snapshots

**Tags:** `#aws` `#lambda` `#serverless` `#infrastructure-as-graph` `#architecture-visualization` `#rust` `#bevy` `#cloud-discovery` `#devops` `#aws-sdk-rust` `#mermaid` `#infrastructure-mapping`

---

## Features

- **Auto-discovery** — scans 7 AWS services for Lambda connectivity
- **Deterministic output** — sorted, deduped JSON for stable diffs
- **Mermaid export** — paste into GitHub, Notion, or any Markdown renderer
- **Snapshot diffing** — compare two scans to see what changed
- **Interactive viewer** — Bevy 2D app with search, filters, and layer toggles
- **Security-conscious** — env var values, secrets, and tokens are never exported

---

## Quick Start

### Prerequisites

- **Rust 1.75+** — [install via rustup](https://rustup.rs)
- **AWS credentials** — configured via `~/.aws/credentials`, environment variables, or SSO
- **AWS permissions** — read-only access to Lambda, EventBridge, S3, SNS, CloudWatch Logs, Step Functions, API Gateway v2 (see [IAM Policy](#iam-policy) below)

### Install

```bash
# Clone and build
git clone https://github.com/humancto/hawk.git
cd hawk
cargo build --release

# The binary is at target/release/hawk
# Optionally copy to your PATH:
cp target/release/hawk /usr/local/bin/
```

### Run Your First Scan

```bash
# Discover everything Hawk supports
hawk analyze aws all \
  --profile my-aws-profile \
  --region us-east-1 \
  --out hawk.json \
  --pretty

# Or just Lambda + event source mappings (faster)
hawk analyze aws lambda \
  --profile my-aws-profile \
  --region us-east-1 \
  --out hawk.json
```

### Explore the Output

```bash
# Print a summary to the terminal
hawk summary --in hawk.json

# Export as a Mermaid diagram
hawk export mermaid --in hawk.json --out hawk.mmd

# Include all node types (not just Lambda-centric)
hawk export mermaid --in hawk.json --out hawk.mmd --full

# Compare two snapshots
hawk diff --old baseline.json --new current.json
```

### Launch the Viewer

```bash
cargo run --release -p hawk_viewer -- hawk.json
```

---

## CLI Reference

### `hawk analyze aws <scope>`

Discover AWS resources and write a graph JSON file.

| Scope    | Description |
|----------|-------------|
| `lambda` | Lambda functions + event source mappings only |
| `all`    | All supported AWS services |

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--profile <name>` | env default | AWS profile name |
| `--region <name>` | env default | AWS region |
| `--out <file>` | `hawk.json` | Output file path |
| `--pretty` | off | Pretty-print JSON |
| `--verbose` | off | Enable debug logging |

### `hawk summary`

Print human-readable stats from a scan.

```
=== Hawk Summary ===

Generated: 2026-03-05T14:30:00Z
Profile:   production
Regions:   us-east-1

Nodes: 47
  Lambda: 23
  SqsQueue: 8
  EventRule: 6
  S3Bucket: 4
  SnsTopic: 3
  StepFunction: 2
  ApiGateway: 1

Edges: 38
  Triggers: 31
  Invokes: 7

Top fan-in (most triggered):
  order-processor: 5
  notification-handler: 4

Top fan-out (most connections):
  main-event-bus: 6
```

### `hawk export mermaid`

Generate a Mermaid flowchart diagram.

| Flag | Default | Description |
|------|---------|-------------|
| `--in <file>` | `hawk.json` | Input graph file |
| `--out <file>` | `hawk.mmd` | Output Mermaid file |
| `--full` | off | Show all node types |

The output can be pasted directly into GitHub Markdown, Notion, or rendered with the [Mermaid CLI](https://github.com/mermaid-js/mermaid-cli).

### `hawk diff`

Compare two graph snapshots.

```bash
hawk diff --old monday.json --new friday.json
```

```
=== Graph Diff ===

Added nodes (2):
  + arn:aws:lambda:us-east-1:123:function:new-handler
  + arn:aws:sqs:us-east-1:123:new-queue

Removed nodes (1):
  - arn:aws:lambda:us-east-1:123:function:deprecated-fn

Added edges (2):
  + new-queue --Triggers--> new-handler
  + main-bus --Triggers--> new-handler
```

---

## AWS Coverage

| Source | Target | Edge Kind | How |
|--------|--------|-----------|-----|
| **SQS / DynamoDB / Kinesis** | Lambda | Triggers | `ListEventSourceMappings` |
| **EventBridge rules** | Lambda | Triggers | `ListRules` + `ListTargetsByRule` |
| **S3 notifications** | Lambda | Triggers | `GetBucketNotificationConfiguration` |
| **SNS subscriptions** | Lambda | Triggers | `ListSubscriptionsByTopic` |
| **CloudWatch Logs** | Lambda | Triggers | `DescribeSubscriptionFilters` |
| **Step Functions** | Lambda | Invokes | `DescribeStateMachine` (definition parse) |
| **API Gateway v2** | Lambda | Triggers | `GetRoutes` + `GetIntegrations` |

---

## Graph Schema

The output JSON follows a stable schema:

```jsonc
{
  "generated_at": "2026-03-05T14:30:00Z",
  "profile": "production",
  "regions": ["us-east-1"],
  "nodes": [
    {
      "id": "arn:aws:lambda:us-east-1:123456789012:function:my-fn",
      "kind": "Lambda",
      "name": "my-fn",
      "arn": "arn:aws:lambda:us-east-1:123456789012:function:my-fn",
      "region": "us-east-1",
      "account_id": "123456789012",
      "props": {
        "runtime": "nodejs20.x",
        "memory_size": 256,
        "timeout": 30,
        "handler": "index.handler",
        "env_keys": ["DATABASE_URL", "API_KEY"]  // values redacted
      }
    }
  ],
  "edges": [
    {
      "from": "arn:aws:sqs:us-east-1:123456789012:my-queue",
      "to": "arn:aws:lambda:us-east-1:123456789012:function:my-fn",
      "kind": "Triggers",
      "props": { "batch_size": 10 }
    }
  ],
  "warnings": [],
  "stats": { "node_count": 2, "edge_count": 1, "..." : "..." }
}
```

**Node kinds:** Lambda, ApiGateway, ApiRoute, EventRule, SqsQueue, SnsTopic, S3Bucket, DynamoStream, StepFunction, LogGroup, EcsService, Ec2Instance, LoadBalancer, Unknown

**Edge kinds:** Triggers, Invokes, Consumes, Publishes, ReadsFrom, WritesTo

---

## Interactive Viewer

The Bevy-based viewer renders the graph as a 2D node-and-edge map.

**Controls:**
| Action | Input |
|--------|-------|
| Select node | Click |
| Pan | Drag |
| Zoom | Scroll wheel |

**UI panels:**
- **Left panel** — search bar, layer toggles (Compute / Events / Storage / Orchestration)
- **Right panel** — selected node details (name, kind, ARN, region, properties)

**Layers:**
| Layer | Node kinds |
|-------|-----------|
| Compute | Lambda, ECS Service, EC2 Instance |
| Events | EventBridge Rule, API Gateway, API Route, SNS Topic, SQS Queue, Log Group |
| Storage | S3 Bucket, DynamoDB Stream |
| Orchestration | Step Function |

---

## IAM Policy

Hawk requires **read-only** access. Here's a minimal IAM policy:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "HawkReadOnly",
      "Effect": "Allow",
      "Action": [
        "lambda:ListFunctions",
        "lambda:ListEventSourceMappings",
        "events:ListRules",
        "events:ListTargetsByRule",
        "s3:ListAllMyBuckets",
        "s3:GetBucketNotificationConfiguration",
        "sns:ListTopics",
        "sns:ListSubscriptionsByTopic",
        "logs:DescribeLogGroups",
        "logs:DescribeSubscriptionFilters",
        "states:ListStateMachines",
        "states:DescribeStateMachine",
        "apigateway:GET"
      ],
      "Resource": "*"
    }
  ]
}
```

---

## Project Structure

```
hawk/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── hawk_core/              # Graph model, stats, dedupe, redaction
│   ├── hawk_aws/               # AWS SDK discovery modules (7 connectors)
│   ├── hawk_cli/               # CLI binary (clap-based)
│   └── hawk_render/            # Mermaid renderer
├── apps/
│   └── hawk_viewer/            # Bevy 2D interactive viewer
├── examples/
│   └── sample_graph.json       # Example output for testing
└── assets/
    ├── sprites/                # Node sprite assets
    └── fonts/                  # Font assets
```

---

## Development

```bash
# Check compilation
cargo check --workspace

# Run tests
cargo test --workspace --exclude hawk_viewer

# Run with verbose logging
hawk analyze aws all --profile dev --region us-east-1 --verbose

# Run clippy
cargo clippy --workspace --exclude hawk_viewer

# Format code
cargo fmt --all
```

### Running Tests Without AWS Credentials

The unit tests don't require AWS credentials — they test ARN parsing, graph operations, Mermaid rendering, and data redaction. Integration tests use fixture JSON files in `examples/`.

---

## Security & Data Safety

Hawk is designed to be safe to run against production accounts:

- **Environment variable values are never exported** — only keys are recorded
- **Secrets, tokens, and auth data are redacted** from all output
- **No write operations** — Hawk only calls read/list/describe APIs
- **No data leaves your machine** — output is written to local files only
- **Inline policy documents are excluded** unless explicitly requested

---

## Roadmap

- [ ] API Gateway v1 (REST APIs) discovery
- [ ] Multi-region scanning in a single run
- [ ] Multi-account scanning (AWS Organizations)
- [ ] Force-directed graph layout in the viewer
- [ ] HTML export with interactive SVG
- [ ] Cost annotations via Cost Explorer API
- [ ] CloudFormation / CDK stack grouping
- [ ] Terraform state file import

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, coding standards, and pull request guidelines.

---

## License

[MIT](LICENSE) — Archith Rapaka
