# Hawk

AWS infrastructure graph analyzer. Discovers Lambda functions and their triggers across AWS services, exports connectivity graphs, and provides an interactive Bevy-based viewer.

## Quick Start

```bash
# Discover all Lambda triggers in your AWS account
hawk analyze aws all --profile myprofile --region us-east-1 --out hawk.json --pretty

# Discover Lambda only (faster)
hawk analyze aws lambda --profile myprofile --region us-east-1 --out hawk.json

# Print summary statistics
hawk summary --in hawk.json

# Export as Mermaid diagram
hawk export mermaid --in hawk.json --out hawk.mmd

# Show full graph (not just Lambda-centric)
hawk export mermaid --in hawk.json --out hawk.mmd --full

# Diff two snapshots
hawk diff --old old.json --new new.json
```

## AWS Coverage

| Source                  | Target | Edge Kind |
|------------------------|--------|-----------|
| SQS / DynamoDB / Kinesis (event source mappings) | Lambda | Triggers |
| EventBridge rules      | Lambda | Triggers  |
| S3 notifications       | Lambda | Triggers  |
| SNS subscriptions      | Lambda | Triggers  |
| CloudWatch Logs subscription filters | Lambda | Triggers |
| Step Functions         | Lambda | Invokes   |
| API Gateway v2 (HTTP)  | Lambda | Triggers  |

## Viewer

```bash
# Launch the interactive viewer
hawk_viewer hawk.json
```

- Click nodes to see details
- Use left panel to search, filter by kind, and toggle layers
- Connected edges highlight on selection
- Zoom and pan with mouse

## Building

```bash
cargo build --release
```

## License

MIT
