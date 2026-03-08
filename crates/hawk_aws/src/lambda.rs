use crate::arn::{account_from_arn, name_from_arn, region_from_arn, resource_kind_from_arn};
use crate::ctx::AwsCtx;
use hawk_core::{DiscoveryOutput, Edge, EdgeKind, Node, NodeKind};
use tracing::{info, warn};

pub async fn discover(ctx: &AwsCtx) -> anyhow::Result<DiscoveryOutput> {
    let mut output = DiscoveryOutput::default();
    let region = ctx.region_str();

    // --- List all Lambda functions (paginated) ---
    let mut paginator = ctx.lambda.list_functions().into_paginator().send();

    let mut functions = Vec::new();
    while let Some(page) = paginator.next().await {
        match page {
            Ok(resp) => {
                if let Some(fns) = resp.functions {
                    functions.extend(fns);
                }
            }
            Err(e) => {
                output
                    .warnings
                    .push(format!("Lambda ListFunctions error: {e}"));
                return Ok(output);
            }
        }
    }

    info!(
        "Discovered {} Lambda functions in {}",
        functions.len(),
        region
    );

    for func in &functions {
        let arn = func.function_arn().unwrap_or_default().to_string();
        let name = func.function_name().unwrap_or_default().to_string();

        // Collect env var keys only (no values)
        let env_keys: Vec<String> = func
            .environment()
            .and_then(|e| e.variables())
            .map(|vars| vars.keys().cloned().collect())
            .unwrap_or_default();

        let layers: Vec<String> = func
            .layers()
            .iter()
            .filter_map(|l| l.arn().map(|a| a.to_string()))
            .collect();

        let props = serde_json::json!({
            "runtime": func.runtime().map(|r| r.as_str()),
            "memory_size": func.memory_size(),
            "timeout": func.timeout(),
            "handler": func.handler(),
            "last_modified": func.last_modified(),
            "architectures": func.architectures().iter().map(|a| a.as_str()).collect::<Vec<_>>(),
            "layers": layers,
            "env_keys": env_keys,
        });

        output.nodes.push(Node {
            id: arn.clone(),
            kind: NodeKind::Lambda,
            name,
            arn: Some(arn.clone()),
            region: Some(region.clone()),
            account_id: account_from_arn(&arn),
            tags: None,
            props,
        });
    }

    // --- Event source mappings (paginated) ---
    let mut esm_paginator = ctx
        .lambda
        .list_event_source_mappings()
        .into_paginator()
        .send();

    while let Some(page) = esm_paginator.next().await {
        match page {
            Ok(resp) => {
                if let Some(mappings) = resp.event_source_mappings {
                    for mapping in mappings {
                        let source_arn = match mapping.event_source_arn() {
                            Some(a) => a.to_string(),
                            None => continue,
                        };
                        let fn_arn = match mapping.function_arn() {
                            Some(a) => a.to_string(),
                            None => continue,
                        };

                        // Create source node if needed
                        let source_kind = resource_kind_from_arn(&source_arn);
                        let source_name = name_from_arn(&source_arn);
                        output.nodes.push(Node {
                            id: source_arn.clone(),
                            kind: source_kind,
                            name: source_name,
                            arn: Some(source_arn.clone()),
                            region: region_from_arn(&source_arn),
                            account_id: account_from_arn(&source_arn),
                            tags: None,
                            props: serde_json::json!({}),
                        });

                        let state_str = mapping.state().map(|s| format!("{s:?}"));
                        let start_pos_str = mapping.starting_position().map(|s| format!("{s:?}"));
                        let props = serde_json::json!({
                            "batch_size": mapping.batch_size(),
                            "enabled": state_str,
                            "starting_position": start_pos_str,
                            "maximum_batching_window_in_seconds": mapping.maximum_batching_window_in_seconds(),
                        });

                        output.edges.push(Edge {
                            from: source_arn,
                            to: fn_arn,
                            kind: EdgeKind::Triggers,
                            props,
                        });
                    }
                }
            }
            Err(e) => {
                warn!("ListEventSourceMappings error: {e}");
                output
                    .warnings
                    .push(format!("ListEventSourceMappings error: {e}"));
            }
        }
    }

    Ok(output)
}
