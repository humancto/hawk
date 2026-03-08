use hawk_core::{Graph, NodeKind};

// ---------------------------------------------------------------------------
// Mermaid renderer
// ---------------------------------------------------------------------------

/// Options for Mermaid rendering.
pub struct MermaidOptions {
    /// If true, include all node types. If false, only Lambda + direct triggers.
    pub full: bool,
    /// Max label length before truncation.
    pub max_label_len: usize,
}

impl Default for MermaidOptions {
    fn default() -> Self {
        Self {
            full: false,
            max_label_len: 40,
        }
    }
}

/// Render a Graph as a Mermaid flowchart string.
pub fn render_mermaid(graph: &Graph, opts: &MermaidOptions) -> String {
    let mut out = String::from("flowchart LR\n");

    let nodes = filter_nodes(graph, opts.full);
    let node_ids: std::collections::HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();

    // Group nodes by kind into subgraphs
    let groups = group_nodes(&nodes);

    for (group_name, group_nodes) in &groups {
        out.push_str(&format!("    subgraph {group_name}\n"));
        for node in group_nodes {
            let safe_id = sanitize_id(&node.id);
            let label = truncate_label(&node.name, opts.max_label_len);
            let label = escape_mermaid(&label);
            out.push_str(&format!("        {safe_id}[\"{label}\"]\n"));
        }
        out.push_str("    end\n");
    }

    // Render edges
    for edge in &graph.edges {
        if !node_ids.contains(edge.from.as_str()) || !node_ids.contains(edge.to.as_str()) {
            continue;
        }
        let from = sanitize_id(&edge.from);
        let to = sanitize_id(&edge.to);
        let label = format!("{:?}", edge.kind);
        out.push_str(&format!("    {from} -->|{label}| {to}\n"));
    }

    out
}

// ---------------------------------------------------------------------------
// DOT / Graphviz renderer
// ---------------------------------------------------------------------------

/// Options for DOT rendering.
pub struct DotOptions {
    /// If true, include all node types. If false, only Lambda + direct triggers.
    pub full: bool,
    /// Max label length before truncation.
    pub max_label_len: usize,
}

impl Default for DotOptions {
    fn default() -> Self {
        Self {
            full: false,
            max_label_len: 40,
        }
    }
}

/// Render a Graph as a Graphviz DOT string.
pub fn render_dot(graph: &Graph, opts: &DotOptions) -> String {
    let mut out =
        String::from("digraph hawk {\n    rankdir=LR;\n    node [shape=box, style=filled];\n\n");

    let nodes = filter_nodes(graph, opts.full);
    let node_ids: std::collections::HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();

    // Group into subgraphs
    let groups = group_nodes(&nodes);
    for (i, (group_name, group_nodes)) in groups.iter().enumerate() {
        out.push_str(&format!("    subgraph cluster_{i} {{\n"));
        out.push_str(&format!("        label=\"{group_name}\";\n"));
        out.push_str("        style=dashed;\n");
        for node in group_nodes {
            let safe_id = sanitize_id(&node.id);
            let label = truncate_label(&node.name, opts.max_label_len);
            let label = escape_dot(&label);
            let color = dot_color(&node.kind);
            out.push_str(&format!(
                "        {safe_id} [label=\"{label}\", fillcolor=\"{color}\"];\n"
            ));
        }
        out.push_str("    }\n\n");
    }

    // Render edges
    for edge in &graph.edges {
        if !node_ids.contains(edge.from.as_str()) || !node_ids.contains(edge.to.as_str()) {
            continue;
        }
        let from = sanitize_id(&edge.from);
        let to = sanitize_id(&edge.to);
        let label = format!("{:?}", edge.kind);
        out.push_str(&format!("    {from} -> {to} [label=\"{label}\"];\n"));
    }

    out.push_str("}\n");
    out
}

fn dot_color(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Lambda => "#FF9900",
        NodeKind::ApiGateway | NodeKind::ApiRoute => "#4A90D9",
        NodeKind::EventRule => "#DD4444",
        NodeKind::SqsQueue => "#44BB44",
        NodeKind::SnsTopic => "#9B59B6",
        NodeKind::S3Bucket => "#27AE60",
        NodeKind::DynamoStream => "#2C3E50",
        NodeKind::StepFunction => "#3498DB",
        NodeKind::LogGroup => "#95A5A6",
        _ => "#BDC3C7",
    }
}

fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn filter_nodes(graph: &Graph, full: bool) -> Vec<&hawk_core::Node> {
    if full {
        graph.nodes.iter().collect()
    } else {
        let lambda_ids: std::collections::HashSet<&str> = graph
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Lambda)
            .map(|n| n.id.as_str())
            .collect();
        let connected_ids: std::collections::HashSet<&str> = graph
            .edges
            .iter()
            .flat_map(|e| {
                let mut ids = Vec::new();
                if lambda_ids.contains(e.from.as_str()) || lambda_ids.contains(e.to.as_str()) {
                    ids.push(e.from.as_str());
                    ids.push(e.to.as_str());
                }
                ids
            })
            .collect();
        graph
            .nodes
            .iter()
            .filter(|n| lambda_ids.contains(n.id.as_str()) || connected_ids.contains(n.id.as_str()))
            .collect()
    }
}

fn group_nodes<'a>(nodes: &[&'a hawk_core::Node]) -> Vec<(String, Vec<&'a hawk_core::Node>)> {
    use std::collections::BTreeMap;

    let mut map: BTreeMap<String, Vec<&hawk_core::Node>> = BTreeMap::new();

    for node in nodes {
        let group = match node.kind {
            NodeKind::Lambda | NodeKind::EcsService | NodeKind::Ec2Instance => "Compute",
            NodeKind::EventRule
            | NodeKind::ApiGateway
            | NodeKind::ApiRoute
            | NodeKind::SnsTopic
            | NodeKind::SqsQueue
            | NodeKind::LogGroup => "Events",
            NodeKind::S3Bucket | NodeKind::DynamoStream => "Storage",
            NodeKind::StepFunction => "Orchestration",
            _ => "Other",
        };
        map.entry(group.to_string()).or_default().push(node);
    }

    map.into_iter().collect()
}

/// Replace characters that break Mermaid/DOT IDs.
fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => c,
            _ => '_',
        })
        .collect()
}

fn escape_mermaid(s: &str) -> String {
    s.replace('"', "'")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn truncate_label(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hawk_core::*;

    fn sample_graph() -> Graph {
        let mut g = Graph::new();
        g.nodes.push(Node {
            id: "arn:aws:lambda:us-east-1:123:function:fn1".into(),
            kind: NodeKind::Lambda,
            name: "fn1".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });
        g.nodes.push(Node {
            id: "arn:aws:sqs:us-east-1:123:queue1".into(),
            kind: NodeKind::SqsQueue,
            name: "queue1".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });
        g.edges.push(Edge {
            from: "arn:aws:sqs:us-east-1:123:queue1".into(),
            to: "arn:aws:lambda:us-east-1:123:function:fn1".into(),
            kind: EdgeKind::Triggers,
            props: serde_json::json!({}),
        });
        g
    }

    #[test]
    fn test_render_mermaid_basic() {
        let g = sample_graph();
        let result = render_mermaid(
            &g,
            &MermaidOptions {
                full: true,
                ..Default::default()
            },
        );
        assert!(result.contains("flowchart LR"));
        assert!(result.contains("fn1"));
        assert!(result.contains("queue1"));
        assert!(result.contains("Triggers"));
    }

    #[test]
    fn test_render_mermaid_deterministic() {
        let mut g = Graph::new();
        g.nodes.push(Node {
            id: "l1".into(),
            kind: NodeKind::Lambda,
            name: "alpha".into(),
            arn: None,
            region: None,
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });
        let r1 = render_mermaid(&g, &MermaidOptions::default());
        let r2 = render_mermaid(&g, &MermaidOptions::default());
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_render_dot_basic() {
        let g = sample_graph();
        let result = render_dot(
            &g,
            &DotOptions {
                full: true,
                ..Default::default()
            },
        );
        assert!(result.contains("digraph hawk"));
        assert!(result.contains("fn1"));
        assert!(result.contains("queue1"));
        assert!(result.contains("Triggers"));
        assert!(result.contains("rankdir=LR"));
    }

    #[test]
    fn test_render_dot_deterministic() {
        let g = sample_graph();
        let r1 = render_dot(
            &g,
            &DotOptions {
                full: true,
                ..Default::default()
            },
        );
        let r2 = render_dot(
            &g,
            &DotOptions {
                full: true,
                ..Default::default()
            },
        );
        assert_eq!(r1, r2);
    }
}
