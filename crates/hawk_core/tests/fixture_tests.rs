use hawk_core::*;
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples/sample_graph.json")
}

fn load_fixture() -> Graph {
    let data = std::fs::read_to_string(fixture_path()).unwrap();
    serde_json::from_str(&data).unwrap()
}

#[test]
fn test_fixture_loads_correctly() {
    let graph = load_fixture();
    assert_eq!(graph.nodes.len(), 16);
    assert_eq!(graph.edges.len(), 12);
    assert_eq!(graph.regions, vec!["us-east-1"]);
    assert_eq!(graph.profile, Some("demo".to_string()));
}

#[test]
fn test_fixture_node_kinds() {
    let graph = load_fixture();
    let lambda_count = graph.nodes.iter().filter(|n| n.kind == NodeKind::Lambda).count();
    let sqs_count = graph.nodes.iter().filter(|n| n.kind == NodeKind::SqsQueue).count();
    let event_count = graph.nodes.iter().filter(|n| n.kind == NodeKind::EventRule).count();
    assert_eq!(lambda_count, 5);
    assert_eq!(sqs_count, 2);
    assert_eq!(event_count, 2);
}

#[test]
fn test_fixture_recompute_stats() {
    let mut graph = load_fixture();
    graph.compute_stats();

    assert_eq!(graph.stats.node_count, 16);
    assert_eq!(graph.stats.edge_count, 12);
    assert_eq!(graph.stats.nodes_by_kind["Lambda"], 5);
    assert_eq!(graph.stats.edges_by_kind["Triggers"], 10);
    assert_eq!(graph.stats.edges_by_kind["Invokes"], 2);

    // Fan-in: order-processor and notification-sender both have 4 incoming edges
    let fan_in_map: std::collections::HashMap<&str, usize> = graph
        .stats
        .top_fan_in
        .iter()
        .map(|(n, c)| (n.as_str(), *c))
        .collect();
    assert_eq!(fan_in_map["order-processor"], 4);
    assert_eq!(fan_in_map["notification-sender"], 4);

    // Fan-out: orders-api and order-fulfillment both have 2
    let fan_out_map: std::collections::HashMap<&str, usize> = graph
        .stats
        .top_fan_out
        .iter()
        .map(|(n, c)| (n.as_str(), *c))
        .collect();
    assert_eq!(fan_out_map["orders-api"], 2);
    assert_eq!(fan_out_map["order-fulfillment"], 2);
}

#[test]
fn test_fixture_dedupe_idempotent() {
    let mut graph = load_fixture();
    let node_count_before = graph.nodes.len();
    let edge_count_before = graph.edges.len();

    graph.dedupe_and_sort();

    assert_eq!(graph.nodes.len(), node_count_before);
    assert_eq!(graph.edges.len(), edge_count_before);
}

#[test]
fn test_fixture_serialization_roundtrip() {
    let graph = load_fixture();
    let json = serde_json::to_string_pretty(&graph).unwrap();
    let parsed: Graph = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.nodes.len(), graph.nodes.len());
    assert_eq!(parsed.edges.len(), graph.edges.len());
    assert_eq!(parsed.regions, graph.regions);
    assert_eq!(parsed.profile, graph.profile);
}

#[test]
fn test_diff_with_added_node() {
    let old = load_fixture();
    let mut new = load_fixture();
    new.nodes.push(Node {
        id: "arn:aws:lambda:us-east-1:123456789012:function:new-func".into(),
        kind: NodeKind::Lambda,
        name: "new-func".into(),
        arn: Some("arn:aws:lambda:us-east-1:123456789012:function:new-func".into()),
        region: Some("us-east-1".into()),
        account_id: Some("123456789012".into()),
        tags: None,
        props: serde_json::json!({}),
    });

    let diff = GraphDiff::compute(&old, &new);
    assert_eq!(diff.added_nodes.len(), 1);
    assert!(diff.added_nodes[0].contains("new-func"));
    assert!(diff.removed_nodes.is_empty());
}

#[test]
fn test_diff_with_removed_edge() {
    let old = load_fixture();
    let mut new = load_fixture();
    new.edges.pop(); // Remove last edge

    let diff = GraphDiff::compute(&old, &new);
    assert!(diff.added_edges.is_empty());
    assert_eq!(diff.removed_edges.len(), 1);
}

#[test]
fn test_diff_identical_graphs() {
    let old = load_fixture();
    let new = load_fixture();
    let diff = GraphDiff::compute(&old, &new);

    assert!(diff.added_nodes.is_empty());
    assert!(diff.removed_nodes.is_empty());
    assert!(diff.added_edges.is_empty());
    assert!(diff.removed_edges.is_empty());
}

#[test]
fn test_merge_discovery_output() {
    let mut graph = Graph::new();
    let output = DiscoveryOutput {
        nodes: vec![Node {
            id: "test-node".into(),
            kind: NodeKind::Lambda,
            name: "test".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        }],
        edges: vec![],
        warnings: vec!["test warning".into()],
    };

    graph.merge(output);
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.warnings.len(), 1);
}

#[test]
fn test_redact_props_on_fixture() {
    let graph = load_fixture();
    // Find a node with env_keys to verify props structure is preserved
    let order_proc = graph
        .nodes
        .iter()
        .find(|n| n.name == "order-processor")
        .unwrap();
    let env_keys = order_proc.props["env_keys"].as_array().unwrap();
    assert!(env_keys.len() > 0);

    // Test redaction on a props object with sensitive data
    let mut sensitive = serde_json::json!({
        "runtime": "nodejs20.x",
        "secret_key": "super-secret",
        "config": {
            "password": "hunter2",
            "host": "example.com"
        }
    });
    hawk_core::redact::redact_props(&mut sensitive);
    assert_eq!(sensitive["runtime"], "nodejs20.x");
    assert_eq!(sensitive["secret_key"], "[REDACTED]");
    assert_eq!(sensitive["config"]["password"], "[REDACTED]");
    assert_eq!(sensitive["config"]["host"], "example.com");
}
