use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// NodeKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub enum NodeKind {
    Lambda,
    ApiGateway,
    ApiRoute,
    EventRule,
    SqsQueue,
    SnsTopic,
    S3Bucket,
    DynamoStream,
    StepFunction,
    LogGroup,
    EcsService,
    Ec2Instance,
    LoadBalancer,
    Unknown,
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ---------------------------------------------------------------------------
// EdgeKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub enum EdgeKind {
    Triggers,
    Invokes,
    Consumes,
    Publishes,
    ReadsFrom,
    WritesTo,
}

impl std::fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<IndexMap<String, String>>,
    #[serde(default = "default_props")]
    pub props: serde_json::Value,
}

fn default_props() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

// ---------------------------------------------------------------------------
// Edge
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
    #[serde(default = "default_props")]
    pub props: serde_json::Value,
}

// ---------------------------------------------------------------------------
// GraphStats
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub nodes_by_kind: IndexMap<String, usize>,
    pub edges_by_kind: IndexMap<String, usize>,
    pub top_fan_in: Vec<(String, usize)>,
    pub top_fan_out: Vec<(String, usize)>,
}

// ---------------------------------------------------------------------------
// Graph
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub generated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    pub regions: Vec<String>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub stats: GraphStats,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            generated_at: chrono::Utc::now().to_rfc3339(),
            profile: None,
            regions: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            warnings: Vec::new(),
            stats: GraphStats::default(),
        }
    }

    /// Remove duplicate nodes (by id) and edges (by from+to+kind), then sort
    /// deterministically.
    pub fn dedupe_and_sort(&mut self) {
        // Dedupe nodes by id, keep first occurrence
        let mut seen = std::collections::HashSet::new();
        self.nodes.retain(|n| seen.insert(n.id.clone()));

        // Dedupe edges by (from, to, kind)
        let mut seen_edges = std::collections::HashSet::new();
        self.edges.retain(|e| {
            seen_edges.insert((e.from.clone(), e.to.clone(), format!("{:?}", e.kind)))
        });

        // Sort nodes by kind, then name, then id
        self.nodes.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.id.cmp(&b.id))
        });

        // Sort edges by kind, then from, then to
        self.edges.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then_with(|| a.from.cmp(&b.from))
                .then_with(|| a.to.cmp(&b.to))
        });
    }

    /// Compute graph statistics.
    pub fn compute_stats(&mut self) {
        let mut nodes_by_kind: IndexMap<String, usize> = IndexMap::new();
        for node in &self.nodes {
            *nodes_by_kind.entry(node.kind.to_string()).or_default() += 1;
        }

        let mut edges_by_kind: IndexMap<String, usize> = IndexMap::new();
        for edge in &self.edges {
            *edges_by_kind.entry(edge.kind.to_string()).or_default() += 1;
        }

        // Fan-in: count incoming edges per node
        let mut fan_in: HashMap<String, usize> = HashMap::new();
        for edge in &self.edges {
            *fan_in.entry(edge.to.clone()).or_default() += 1;
        }
        let mut top_fan_in: Vec<(String, usize)> = fan_in.into_iter().collect();
        top_fan_in.sort_by(|a, b| b.1.cmp(&a.1));
        top_fan_in.truncate(10);

        // Fan-out: count outgoing edges per node
        let mut fan_out: HashMap<String, usize> = HashMap::new();
        for edge in &self.edges {
            *fan_out.entry(edge.from.clone()).or_default() += 1;
        }
        let mut top_fan_out: Vec<(String, usize)> = fan_out.into_iter().collect();
        top_fan_out.sort_by(|a, b| b.1.cmp(&a.1));
        top_fan_out.truncate(10);

        self.stats = GraphStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            nodes_by_kind,
            edges_by_kind,
            top_fan_in,
            top_fan_out,
        };
    }

    /// Merge another graph's output into this graph.
    pub fn merge(&mut self, output: DiscoveryOutput) {
        self.nodes.extend(output.nodes);
        self.edges.extend(output.edges);
        self.warnings.extend(output.warnings);
    }
}

// ---------------------------------------------------------------------------
// DiscoveryOutput — returned by each discovery module
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct DiscoveryOutput {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// Diff support
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDiff {
    pub added_nodes: Vec<String>,
    pub removed_nodes: Vec<String>,
    pub added_edges: Vec<String>,
    pub removed_edges: Vec<String>,
}

impl GraphDiff {
    pub fn compute(old: &Graph, new: &Graph) -> Self {
        let old_node_ids: std::collections::HashSet<&str> =
            old.nodes.iter().map(|n| n.id.as_str()).collect();
        let new_node_ids: std::collections::HashSet<&str> =
            new.nodes.iter().map(|n| n.id.as_str()).collect();

        let added_nodes: Vec<String> = new_node_ids
            .difference(&old_node_ids)
            .map(|s| s.to_string())
            .collect();
        let removed_nodes: Vec<String> = old_node_ids
            .difference(&new_node_ids)
            .map(|s| s.to_string())
            .collect();

        let edge_key = |e: &Edge| format!("{} --{:?}--> {}", e.from, e.kind, e.to);
        let old_edge_keys: std::collections::HashSet<String> =
            old.edges.iter().map(edge_key).collect();
        let new_edge_keys: std::collections::HashSet<String> =
            new.edges.iter().map(edge_key).collect();

        let added_edges: Vec<String> = new_edge_keys
            .difference(&old_edge_keys)
            .cloned()
            .collect();
        let removed_edges: Vec<String> = old_edge_keys
            .difference(&new_edge_keys)
            .cloned()
            .collect();

        GraphDiff {
            added_nodes,
            removed_nodes,
            added_edges,
            removed_edges,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedupe_and_sort() {
        let mut g = Graph::new();
        g.nodes.push(Node {
            id: "arn:aws:lambda:us-east-1:123:function:alpha".into(),
            kind: NodeKind::Lambda,
            name: "alpha".into(),
            arn: Some("arn:aws:lambda:us-east-1:123:function:alpha".into()),
            region: Some("us-east-1".into()),
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });
        // Duplicate
        g.nodes.push(Node {
            id: "arn:aws:lambda:us-east-1:123:function:alpha".into(),
            kind: NodeKind::Lambda,
            name: "alpha".into(),
            arn: Some("arn:aws:lambda:us-east-1:123:function:alpha".into()),
            region: Some("us-east-1".into()),
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });
        g.nodes.push(Node {
            id: "arn:aws:lambda:us-east-1:123:function:beta".into(),
            kind: NodeKind::Lambda,
            name: "beta".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });

        g.edges.push(Edge {
            from: "a".into(),
            to: "b".into(),
            kind: EdgeKind::Triggers,
            props: serde_json::json!({}),
        });
        // Duplicate edge
        g.edges.push(Edge {
            from: "a".into(),
            to: "b".into(),
            kind: EdgeKind::Triggers,
            props: serde_json::json!({}),
        });

        g.dedupe_and_sort();
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.edges.len(), 1);
        // alpha before beta
        assert_eq!(g.nodes[0].name, "alpha");
        assert_eq!(g.nodes[1].name, "beta");
    }

    #[test]
    fn test_compute_stats() {
        let mut g = Graph::new();
        g.nodes.push(Node {
            id: "l1".into(),
            kind: NodeKind::Lambda,
            name: "fn1".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });
        g.nodes.push(Node {
            id: "l2".into(),
            kind: NodeKind::Lambda,
            name: "fn2".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });
        g.nodes.push(Node {
            id: "q1".into(),
            kind: NodeKind::SqsQueue,
            name: "queue1".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });
        g.edges.push(Edge {
            from: "q1".into(),
            to: "l1".into(),
            kind: EdgeKind::Triggers,
            props: serde_json::json!({}),
        });
        g.edges.push(Edge {
            from: "q1".into(),
            to: "l2".into(),
            kind: EdgeKind::Triggers,
            props: serde_json::json!({}),
        });
        g.compute_stats();
        assert_eq!(g.stats.node_count, 3);
        assert_eq!(g.stats.edge_count, 2);
        assert_eq!(g.stats.nodes_by_kind["Lambda"], 2);
        assert_eq!(g.stats.top_fan_out[0], ("q1".to_string(), 2));
    }

    #[test]
    fn test_diff() {
        let mut old = Graph::new();
        old.nodes.push(Node {
            id: "a".into(),
            kind: NodeKind::Lambda,
            name: "a".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });

        let mut new = Graph::new();
        new.nodes.push(Node {
            id: "b".into(),
            kind: NodeKind::Lambda,
            name: "b".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });

        let diff = GraphDiff::compute(&old, &new);
        assert_eq!(diff.added_nodes, vec!["b".to_string()]);
        assert_eq!(diff.removed_nodes, vec!["a".to_string()]);
    }

    #[test]
    fn test_graph_serialization_roundtrip() {
        let mut g = Graph::new();
        g.nodes.push(Node {
            id: "test".into(),
            kind: NodeKind::Lambda,
            name: "test-fn".into(),
            arn: Some("arn:aws:lambda:us-east-1:123:function:test-fn".into()),
            region: Some("us-east-1".into()),
            account_id: Some("123".into()),
            tags: None,
            props: serde_json::json!({"runtime": "nodejs18.x"}),
        });
        g.compute_stats();

        let json = serde_json::to_string_pretty(&g).unwrap();
        let parsed: Graph = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.nodes.len(), 1);
        assert_eq!(parsed.nodes[0].name, "test-fn");
    }
}
