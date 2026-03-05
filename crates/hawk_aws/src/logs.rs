use crate::ctx::AwsCtx;
use hawk_core::{DiscoveryOutput, Edge, EdgeKind, Node, NodeKind};
use tracing::{info, warn};

pub async fn discover(ctx: &AwsCtx) -> DiscoveryOutput {
    let mut output = DiscoveryOutput::default();
    let region = ctx.region_str();

    // List all log groups (paginated)
    let mut paginator = ctx.logs.describe_log_groups().into_paginator().send();

    let mut log_groups = Vec::new();
    while let Some(page) = paginator.next().await {
        match page {
            Ok(resp) => {
                if let Some(groups) = resp.log_groups {
                    log_groups.extend(groups);
                }
            }
            Err(e) => {
                output
                    .warnings
                    .push(format!("DescribeLogGroups error: {e}"));
                return output;
            }
        }
    }

    info!("Discovered {} CloudWatch log groups in {}", log_groups.len(), region);

    for group in &log_groups {
        let group_name = match group.log_group_name() {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Check subscription filters
        let filters = match ctx
            .logs
            .describe_subscription_filters()
            .log_group_name(&group_name)
            .send()
            .await
        {
            Ok(resp) => resp.subscription_filters.unwrap_or_default(),
            Err(e) => {
                warn!("DescribeSubscriptionFilters({group_name}) error: {e}");
                output
                    .warnings
                    .push(format!("DescribeSubscriptionFilters({group_name}) error: {e}"));
                continue;
            }
        };

        let lambda_filters: Vec<_> = filters
            .iter()
            .filter(|f| {
                f.destination_arn()
                    .map(|a| a.contains(":function:"))
                    .unwrap_or(false)
            })
            .collect();

        if lambda_filters.is_empty() {
            continue;
        }

        let log_group_id = format!("loggroup:{group_name}");
        let props = serde_json::json!({
            "retention_in_days": group.retention_in_days(),
        });

        output.nodes.push(Node {
            id: log_group_id.clone(),
            kind: NodeKind::LogGroup,
            name: group_name.clone(),
            arn: group.arn().map(|a| a.to_string()),
            region: Some(region.clone()),
            account_id: None,
            tags: None,
            props,
        });

        for filter in lambda_filters {
            let dest_arn = filter.destination_arn().unwrap_or_default().to_string();
            let edge_props = serde_json::json!({
                "filter_pattern": filter.filter_pattern(),
                "distribution": filter.distribution().map(|d| d.as_str()),
            });

            output.edges.push(Edge {
                from: log_group_id.clone(),
                to: dest_arn,
                kind: EdgeKind::Triggers,
                props: edge_props,
            });
        }
    }

    output
}
