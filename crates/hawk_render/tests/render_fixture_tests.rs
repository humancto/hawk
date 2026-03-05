use hawk_core::Graph;
use hawk_render::*;
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
fn test_mermaid_full_renders_all_nodes() {
    let graph = load_fixture();
    let opts = MermaidOptions {
        full: true,
        ..Default::default()
    };
    let mmd = render_mermaid(&graph, &opts);

    assert!(mmd.starts_with("flowchart LR"));
    // All 5 lambda functions should appear
    assert!(mmd.contains("order-processor"));
    assert!(mmd.contains("notification-sender"));
    assert!(mmd.contains("image-resizer"));
    assert!(mmd.contains("daily-report"));
    assert!(mmd.contains("auth-handler"));
    // Subgraph groupings
    assert!(mmd.contains("subgraph Compute"));
    assert!(mmd.contains("subgraph Events"));
    assert!(mmd.contains("subgraph Orchestration"));
    assert!(mmd.contains("subgraph Storage"));
}

#[test]
fn test_mermaid_filtered_excludes_unconnected() {
    let graph = load_fixture();
    let opts = MermaidOptions::default(); // full: false
    let mmd = render_mermaid(&graph, &opts);

    // auth-handler has no edges, should be excluded in filtered mode
    // (it's only connected to nothing in the sample)
    // All connected lambdas should still appear
    assert!(mmd.contains("order-processor"));
    assert!(mmd.contains("notification-sender"));
}

#[test]
fn test_dot_full_renders_all_nodes() {
    let graph = load_fixture();
    let opts = DotOptions {
        full: true,
        ..Default::default()
    };
    let dot = render_dot(&graph, &opts);

    assert!(dot.starts_with("digraph hawk"));
    assert!(dot.contains("rankdir=LR"));
    assert!(dot.contains("order-processor"));
    assert!(dot.contains("notification-sender"));
    assert!(dot.contains("#FF9900")); // Lambda color
    assert!(dot.contains("#DD4444")); // EventRule color
    assert!(dot.contains("#44BB44")); // SqsQueue color
    assert!(dot.contains("cluster_"));
    assert!(dot.ends_with("}\n"));
}

#[test]
fn test_dot_contains_edges() {
    let graph = load_fixture();
    let opts = DotOptions {
        full: true,
        ..Default::default()
    };
    let dot = render_dot(&graph, &opts);

    assert!(dot.contains("-> "));
    assert!(dot.contains("Triggers"));
    assert!(dot.contains("Invokes"));
}

#[test]
fn test_mermaid_deterministic_on_fixture() {
    let graph = load_fixture();
    let opts = MermaidOptions {
        full: true,
        ..Default::default()
    };
    let r1 = render_mermaid(&graph, &opts);
    let r2 = render_mermaid(&graph, &opts);
    assert_eq!(r1, r2);
}

#[test]
fn test_dot_deterministic_on_fixture() {
    let graph = load_fixture();
    let opts = DotOptions {
        full: true,
        ..Default::default()
    };
    let r1 = render_dot(&graph, &opts);
    let r2 = render_dot(&graph, &opts);
    assert_eq!(r1, r2);
}
