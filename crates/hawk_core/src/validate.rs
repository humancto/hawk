use crate::graph::{Graph, NodeKind};
use std::collections::HashSet;

/// Result of validating a hawk graph.
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Validate a Graph's structural integrity.
///
/// Checks:
/// - All nodes have non-empty `id`, `kind`, and `name`
/// - All edges reference existing node IDs
/// - No duplicate node IDs
/// - Edge `from` and `to` are non-empty
pub fn validate_graph(graph: &Graph) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Check nodes
    let mut seen_ids = HashSet::new();
    for (i, node) in graph.nodes.iter().enumerate() {
        if node.id.is_empty() {
            result
                .errors
                .push(format!("Node at index {i} has empty id"));
        }
        if node.name.is_empty() {
            result
                .warnings
                .push(format!("Node '{}' at index {i} has empty name", node.id));
        }
        if !seen_ids.insert(&node.id) {
            result
                .warnings
                .push(format!("Duplicate node id: '{}'", node.id));
        }
    }

    // Collect all valid node IDs
    let node_ids: HashSet<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();

    // Check edges
    let mut orphan_count = 0;
    for (i, edge) in graph.edges.iter().enumerate() {
        if edge.from.is_empty() {
            result
                .errors
                .push(format!("Edge at index {i} has empty 'from' field"));
        }
        if edge.to.is_empty() {
            result
                .errors
                .push(format!("Edge at index {i} has empty 'to' field"));
        }
        if !node_ids.contains(edge.from.as_str()) {
            orphan_count += 1;
        }
        if !node_ids.contains(edge.to.as_str()) {
            orphan_count += 1;
        }
    }
    if orphan_count > 0 {
        result.warnings.push(format!(
            "{orphan_count} edge endpoint(s) reference nodes not in the graph"
        ));
    }

    // Check for deprecated runtimes
    let deprecated_runtimes: HashSet<&str> = [
        "python2.7",
        "python3.6",
        "python3.7",
        "python3.8",
        "nodejs10.x",
        "nodejs12.x",
        "nodejs14.x",
        "nodejs16.x",
        "dotnetcore2.1",
        "dotnetcore3.1",
        "java8",
        "ruby2.5",
        "ruby2.7",
        "go1.x",
    ]
    .into_iter()
    .collect();

    for node in &graph.nodes {
        if node.kind == NodeKind::Lambda {
            if let Some(rt) = node.props.get("runtime").and_then(|v| v.as_str()) {
                if deprecated_runtimes.contains(rt) {
                    result.warnings.push(format!(
                        "Lambda '{}' uses deprecated runtime: {rt}",
                        node.name
                    ));
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::*;

    #[test]
    fn test_validate_valid_graph() {
        let mut g = Graph::new();
        g.nodes.push(Node {
            id: "a".into(),
            kind: NodeKind::Lambda,
            name: "fn-a".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({"runtime": "nodejs20.x"}),
        });
        g.nodes.push(Node {
            id: "b".into(),
            kind: NodeKind::SqsQueue,
            name: "queue-b".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });
        g.edges.push(Edge {
            from: "b".into(),
            to: "a".into(),
            kind: EdgeKind::Triggers,
            props: serde_json::json!({}),
        });

        let result = validate_graph(&g);
        assert!(result.is_valid());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validate_orphan_edges() {
        let mut g = Graph::new();
        g.nodes.push(Node {
            id: "a".into(),
            kind: NodeKind::Lambda,
            name: "fn-a".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });
        g.edges.push(Edge {
            from: "missing".into(),
            to: "a".into(),
            kind: EdgeKind::Triggers,
            props: serde_json::json!({}),
        });

        let result = validate_graph(&g);
        assert!(result.is_valid()); // orphan edges are warnings, not errors
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("reference nodes")));
    }

    #[test]
    fn test_validate_deprecated_runtime() {
        let mut g = Graph::new();
        g.nodes.push(Node {
            id: "old".into(),
            kind: NodeKind::Lambda,
            name: "old-fn".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({"runtime": "python2.7"}),
        });

        let result = validate_graph(&g);
        assert!(result.is_valid());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("deprecated runtime")));
    }
}
