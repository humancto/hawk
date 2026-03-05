use crate::ctx::AwsCtx;
use hawk_core::{DiscoveryOutput, Edge, EdgeKind, Node, NodeKind};
use tracing::{info, warn};

pub async fn discover(ctx: &AwsCtx) -> DiscoveryOutput {
    let mut output = DiscoveryOutput::default();
    let region = ctx.region_str();

    // List all rules (paginated manually since list_rules may not have into_paginator)
    let mut rules = Vec::new();
    let mut next_token: Option<String> = None;
    loop {
        let mut req = ctx.eventbridge.list_rules();
        if let Some(ref token) = next_token {
            req = req.next_token(token);
        }
        match req.send().await {
            Ok(resp) => {
                if let Some(r) = resp.rules {
                    rules.extend(r);
                }
                next_token = resp.next_token;
                if next_token.is_none() {
                    break;
                }
            }
            Err(e) => {
                output.warnings.push(format!("EventBridge ListRules error: {e}"));
                return output;
            }
        }
    }

    info!("Discovered {} EventBridge rules in {}", rules.len(), region);

    for rule in &rules {
        let name = rule.name().unwrap_or_default().to_string();
        let rule_arn = match rule.arn() {
            Some(a) => a.to_string(),
            None => format!("eventrule:{name}:{region}"),
        };

        let props = serde_json::json!({
            "schedule_expression": rule.schedule_expression(),
            "event_pattern": rule.event_pattern(),
            "state": rule.state().map(|s| s.as_str()),
        });

        output.nodes.push(Node {
            id: rule_arn.clone(),
            kind: NodeKind::EventRule,
            name: name.clone(),
            arn: Some(rule_arn.clone()),
            region: Some(region.clone()),
            account_id: None,
            tags: None,
            props,
        });

        // List targets for this rule
        match ctx
            .eventbridge
            .list_targets_by_rule()
            .rule(&name)
            .send()
            .await
        {
            Ok(resp) => {
                if let Some(targets) = resp.targets {
                    for target in targets {
                        let target_arn = target.arn().to_string();
                        if target_arn.contains(":function:") {
                            let edge_props = serde_json::json!({
                                "target_id": target.id(),
                            });
                            output.edges.push(Edge {
                                from: rule_arn.clone(),
                                to: target_arn,
                                kind: EdgeKind::Triggers,
                                props: edge_props,
                            });
                        }
                    }
                }
            }
            Err(e) => {
                warn!("ListTargetsByRule({name}) error: {e}");
                output
                    .warnings
                    .push(format!("ListTargetsByRule({name}) error: {e}"));
            }
        }
    }

    output
}
