<p align="center">
  <h1 align="center">Hawk</h1>
  <p align="center">
    <strong>Map your AWS serverless architecture in seconds.</strong>
  </p>
  <p align="center">
    <a href="https://github.com/humancto/hawk/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/humancto/hawk/ci.yml?branch=main&style=flat-square&logo=github&label=CI" alt="CI"></a>
    <a href="https://crates.io/crates/hawk-cloud"><img src="https://img.shields.io/crates/v/hawk-cloud?style=flat-square&logo=rust&color=orange" alt="crates.io"></a>
    <a href="https://github.com/humancto/hawk/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License"></a>
    <a href="https://github.com/humancto/hawk/releases"><img src="https://img.shields.io/github/v/release/humancto/hawk?style=flat-square&logo=github&color=green" alt="Release"></a>
    <img src="https://img.shields.io/badge/rust-1.75%2B-orange?style=flat-square&logo=rust" alt="MSRV">
  </p>
  <p align="center">
    <a href="#quick-start">Quick Start</a> &middot;
    <a href="#cli-reference">CLI Reference</a> &middot;
    <a href="#interactive-viewer">Viewer</a> &middot;
    <a href="#aws-coverage">AWS Coverage</a> &middot;
    <a href="#contributing">Contributing</a>
  </p>
</p>

---

Hawk is a CLI tool and interactive viewer that automatically discovers AWS Lambda functions, their triggers, and connected services — then renders the entire architecture as a navigable graph.

Point it at an AWS account, get a complete picture of what triggers what.

<p align="center">
  <img src="assets/screenshots/hawk-analyze.png" alt="hawk analyze" width="800">
</p>

## Why Hawk?

- **You inherited an AWS account** and need to understand what's connected to what
- **You want to audit Lambda trigger chains** before making changes
- **You need a visual map** for architecture reviews or onboarding
- **You want to track infrastructure drift** by diffing snapshots over time

## Features

| Feature                  | Description                                                              |
| ------------------------ | ------------------------------------------------------------------------ |
| **Auto-discovery**       | Scans 7 AWS services for Lambda connectivity                             |
| **Deterministic output** | Sorted, deduped JSON for stable diffs                                    |
| **Mermaid export**       | Paste into GitHub, Notion, or any Markdown renderer                      |
| **Snapshot diffing**     | Compare two scans to see what changed                                    |
| **Web viewer**           | Cytoscape.js app with force layout, search, focus mode, clustering       |
| **Risk assessment**      | Real-time health scoring, SPOF detection, blast radius, what-if analysis |
| **Graph insights**       | Auto-generated badges: WTF hubs, fossils, ghost rules, naming tribes     |
| **Schema validation**    | Validates hawk.json on load and via `hawk validate` CLI command          |
| **Advanced search**      | Property-based filtering: `kind:Lambda runtime:python* timeout:>300`     |
| **Performance mode**     | Auto-scales rendering for 500+ node graphs (textures, haystack edges)    |
| **Native viewer**        | Bevy 2D app with force layout, pan/zoom, search, filters                 |
| **Security-conscious**   | Env var values, secrets, and tokens are never exported                   |

---

## Quick Start

### Prerequisites

- **Rust 1.75+** — [install via rustup](https://rustup.rs)
- **AWS credentials** — configured via `~/.aws/credentials`, environment variables, or SSO
- **AWS permissions** — read-only access ([see IAM policy](#iam-policy))

### Install from source

```bash
git clone https://github.com/humancto/hawk.git
cd hawk
cargo install --path crates/hawk-cloud
```

### Install from crates.io

```bash
cargo install hawk-cloud
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

### Launch the Web Viewer

```bash
# Copy your scan output alongside the viewer
cp hawk.json apps/hawk_web/hawk.json

# Open in your browser
open apps/hawk_web/index.html
# Or serve it
cd apps/hawk_web && python3 -m http.server 8080
```

The web viewer features force-directed layout, search, layer toggles, focus mode, clustering, minimap, and handles 1000+ nodes smoothly. You can also drag-and-drop any `hawk.json` file onto the page.

### Launch the Native Viewer (Bevy)

```bash
cargo run --release -p hawk_viewer -- hawk.json
```

---

## CLI Reference

### `hawk analyze aws <scope>`

Discover AWS resources and write a graph JSON file.

| Scope    | Description                                   |
| -------- | --------------------------------------------- |
| `lambda` | Lambda functions + event source mappings only |
| `all`    | All supported AWS services                    |

**Flags:**

| Flag               | Default     | Description          |
| ------------------ | ----------- | -------------------- |
| `--profile <name>` | env default | AWS profile name     |
| `--region <name>`  | env default | AWS region           |
| `--out <file>`     | `hawk.json` | Output file path     |
| `--pretty`         | off         | Pretty-print JSON    |
| `--verbose`        | off         | Enable debug logging |

### `hawk validate`

Validate a hawk.json file for schema correctness.

```bash
hawk validate hawk-output.json
```

Checks for:
- Required top-level fields (`nodes`, `edges`)
- Node structure (`id`, `kind`, `name` required)
- Edge structure (`from`, `to`, `kind` required)
- Dangling edge references
- Detects real AWS data vs demo data

### `hawk summary`

Print human-readable stats from a scan.

<p align="center">
  <img src="assets/screenshots/hawk-summary.png" alt="hawk summary" width="800">
</p>

<details>
<summary>Example output</summary>

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

</details>

### `hawk export mermaid`

Generate a Mermaid flowchart diagram.

| Flag           | Default     | Description         |
| -------------- | ----------- | ------------------- |
| `--in <file>`  | `hawk.json` | Input graph file    |
| `--out <file>` | `hawk.mmd`  | Output Mermaid file |
| `--full`       | off         | Show all node types |

Output works with GitHub Markdown, Notion, or the [Mermaid CLI](https://github.com/mermaid-js/mermaid-cli).

<p align="center">
  <img src="assets/screenshots/hawk-mermaid.png" alt="hawk mermaid export" width="800">
</p>

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

| Source                       | Target | Edge Kind | Discovery Method                          |
| ---------------------------- | ------ | --------- | ----------------------------------------- |
| **SQS / DynamoDB / Kinesis** | Lambda | Triggers  | `ListEventSourceMappings`                 |
| **EventBridge rules**        | Lambda | Triggers  | `ListRules` + `ListTargetsByRule`         |
| **S3 notifications**         | Lambda | Triggers  | `GetBucketNotificationConfiguration`      |
| **SNS subscriptions**        | Lambda | Triggers  | `ListSubscriptionsByTopic`                |
| **CloudWatch Logs**          | Lambda | Triggers  | `DescribeSubscriptionFilters`             |
| **Step Functions**           | Lambda | Invokes   | `DescribeStateMachine` (definition parse) |
| **API Gateway v2**           | Lambda | Triggers  | `GetRoutes` + `GetIntegrations`           |

---

## Graph Schema

Output JSON follows a stable, documented schema:

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
        "env_keys": ["DATABASE_URL", "API_KEY"], // values redacted
      },
    },
  ],
  "edges": [
    {
      "from": "arn:aws:sqs:us-east-1:123456789012:my-queue",
      "to": "arn:aws:lambda:us-east-1:123456789012:function:my-fn",
      "kind": "Triggers",
      "props": { "batch_size": 10 },
    },
  ],
  "warnings": [],
  "stats": { "node_count": 2, "edge_count": 1 },
}
```

**Node kinds:** `Lambda` `ApiGateway` `ApiRoute` `EventRule` `SqsQueue` `SnsTopic` `S3Bucket` `DynamoStream` `StepFunction` `LogGroup` `EcsService` `Ec2Instance` `LoadBalancer` `Unknown`

**Edge kinds:** `Triggers` `Invokes` `Consumes` `Publishes` `ReadsFrom` `WritesTo`

---

## Web Viewer (Recommended)

The web-based viewer uses Cytoscape.js to render the graph as an interactive force-directed map. It handles 1000+ nodes smoothly and requires no build step — just open the HTML file.

```bash
open apps/hawk_web/index.html
```

**Core features:**

- **Force-directed, hierarchical, and circular layouts** — switch between them in the toolbar
- **Search with autocomplete** — type to find nodes, results highlight in the graph
- **Advanced property search** — filter by any field: `kind:Lambda runtime:python* timeout:>300`
- **Focus mode** — click a node then press 1/2/3 to show only its N-hop neighborhood
- **Cluster grouping** — auto-group nodes by service type
- **Layer toggles** — show/hide Compute, Events, Storage, Orchestration layers
- **Isolated node filter** — hide orphan nodes with zero connections
- **Edge bundling** — duplicate edges collapsed with count badges
- **Minimap** — overview with viewport indicator
- **Node detail panel** — ARN, region, properties, clickable connection list, Explain tab
- **Context menu** — right-click any node for quick actions
- **Export to PNG** — full-resolution graph export
- **Export to HTML** — self-contained shareable HTML file with embedded graph
- **Drag-and-drop** — drop any hawk.json file onto the page with schema validation
- **Toast notifications** — color-coded feedback for all actions
- **Keyboard shortcuts** — Esc, F (fit), / (search), 1-3 (focus), C (cluster), I (isolate), R (risk)

**Risk assessment:**

- **Health score** — composite graph health (0-100) based on SPOFs, redundancy, connectivity
- **Risk heatmap** — toggle risk view to color nodes by risk level
- **Top risks** — ranked list of riskiest nodes with reasons (bridge node, high fan-in, etc.)
- **Blast radius** — visualize downstream impact if a node fails
- **What-if analysis** — right-click → "What if this node fails?" shows before/after comparison
- **Recommendations** — actionable suggestions grouped by severity

**Graph insights & badges** (auto-generated):

| Badge | Trigger |
| --- | --- |
| 🤯 **WTF** | Node with extreme number of connections |
| 💀 **SPOF** | Single points of failure (articulation points) |
| 🏙️ **Ghost Town** | High percentage of isolated/orphan nodes |
| 👻 **Phantom Rules** | EventRules with zero edges |
| 🦕 **Fossil** | Lambdas on deprecated runtimes |
| 🏋️ **Heavy Lifter** | Lambdas with 1GB+ RAM or 5min+ timeout |
| 🕷️ **Spider** | Highest fan-out node |
| 💣 **Nuke Zone** | Highest blast radius |
| ⛓️ **Chain Gang** | Longest dependency chain |
| 🏷️ **Naming Tribe** | Largest naming convention cluster |

**Performance mode** (auto-enabled for 200+ nodes):

- Disables shadows, dashed edges, and edge animations
- Uses haystack (straight) edges for 500+ nodes
- Texture caching during pan/zoom
- Draft-quality force layout with reduced iterations
- Throttled minimap rendering (10fps)

**UI panels:**

- **Left panel** — search, layer toggles, stats bars, graph insights badges, keyboard shortcuts
- **Right panel** — selected node details, Explain tab, Risk tab
- **Top toolbar** — layout selector, focus depth, cluster toggle, risk view, blast radius, export
- **Bottom bar** — node/edge counts, health score, severity badges, zoom level

## Native Viewer (Bevy)

The Bevy-based viewer renders the graph as a native 2D application with force-directed layout.

| Action      | Input            |
| ----------- | ---------------- |
| Select node | Left click       |
| Pan         | Right-click drag |
| Zoom        | Scroll wheel     |

**Features:**

- **Force-directed layout** — physics simulation with band constraints
- **Node icons** — service type labels (fn, Q, S3, EB, SNS, SF, API, CW, DB)
- **Directional edges** — arrows show trigger/invocation direction
- **Connection details** — selected nodes show incoming/outgoing edges

| Layer         | Node Kinds                                                                |
| ------------- | ------------------------------------------------------------------------- |
| Compute       | Lambda, ECS Service, EC2 Instance                                         |
| Events        | EventBridge Rule, API Gateway, API Route, SNS Topic, SQS Queue, Log Group |
| Storage       | S3 Bucket, DynamoDB Stream                                                |
| Orchestration | Step Function                                                             |

---

## IAM Policy

Hawk requires **read-only** access. Minimal policy:

<details>
<summary>Click to expand IAM policy JSON</summary>

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

</details>

---

## Security & Data Safety

Hawk is designed to be safe to run against production accounts:

- Environment variable **values are never exported** — only keys are recorded
- Secrets, tokens, and auth data are **redacted** using an allowlist approach (only structural keys pass through)
- **No write operations** — Hawk only calls read/list/describe APIs
- **No data leaves your machine** — output is written to local files only
- Inline policy documents are excluded unless explicitly requested
- **CDN integrity** — all external scripts use SRI (Subresource Integrity) hashes
- **Content Security Policy** — CSP meta tag restricts script sources
- **Schema validation** — loaded files are validated before rendering

---

## Project Structure

```
hawk/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── hawk_core/              # Graph model, stats, dedupe, redaction
│   ├── hawk_aws/               # AWS SDK discovery modules (7 connectors)
│   ├── hawk-cloud/               # CLI binary (clap-based)
│   └── hawk_render/            # Mermaid renderer
├── apps/
│   ├── hawk_web/               # Web-based graph viewer (Cytoscape.js)
│   └── hawk_viewer/            # Native Bevy 2D viewer
└── examples/
    └── sample_graph.json       # Example output for testing
```

---

## Development

```bash
cargo check --workspace            # Check compilation
cargo test --workspace             # Run all tests
cargo clippy --workspace           # Run clippy
cargo fmt --all                    # Format code
```

Unit tests don't require AWS credentials — they test ARN parsing, graph operations, Mermaid rendering, and data redaction. Integration tests use fixture JSON files in `examples/`.

---

## Roadmap

- [x] Force-directed graph layout in the viewer
- [x] Risk assessment and health scoring
- [x] What-if analysis and blast radius
- [x] Graph insights and auto-badges
- [x] Advanced property-based search
- [x] HTML export with embedded graph
- [x] Schema validation (CLI + viewer)
- [x] Performance mode for large graphs
- [ ] API Gateway v1 (REST APIs) discovery
- [ ] Multi-region scanning in a single run
- [ ] Multi-account scanning (AWS Organizations)
- [ ] Cost annotations via Cost Explorer API
- [ ] CloudFormation / CDK stack grouping
- [ ] Terraform state file import

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, coding standards, and pull request guidelines.

## License

[MIT](LICENSE) — HumanCTO
