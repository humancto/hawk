use crate::arn::extract_lambda_arns;
use crate::ctx::AwsCtx;
use hawk_core::{DiscoveryOutput, Edge, EdgeKind, Node, NodeKind};
use tracing::{info, warn};

pub async fn discover(ctx: &AwsCtx) -> anyhow::Result<DiscoveryOutput> {
    let mut output = DiscoveryOutput::default();
    let region = ctx.region_str();

    // List state machines (paginated)
    let mut paginator = ctx.sfn.list_state_machines().into_paginator().send();

    let mut machines = Vec::new();
    while let Some(page) = paginator.next().await {
        match page {
            Ok(resp) => {
                machines.extend(resp.state_machines);
            }
            Err(e) => {
                output
                    .warnings
                    .push(format!("SFN ListStateMachines error: {e}"));
                return Ok(output);
            }
        }
    }

    info!("Discovered {} Step Functions in {}", machines.len(), region);

    for machine in &machines {
        let sm_arn = machine.state_machine_arn().to_string();
        let sm_name = machine.name().to_string();

        output.nodes.push(Node {
            id: sm_arn.clone(),
            kind: NodeKind::StepFunction,
            name: sm_name,
            arn: Some(sm_arn.clone()),
            region: Some(region.clone()),
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });

        // Describe to get definition
        let definition = match ctx
            .sfn
            .describe_state_machine()
            .state_machine_arn(&sm_arn)
            .send()
            .await
        {
            Ok(resp) => resp.definition,
            Err(e) => {
                warn!("DescribeStateMachine({sm_arn}) error: {e}");
                output
                    .warnings
                    .push(format!("DescribeStateMachine({sm_arn}) error: {e}"));
                continue;
            }
        };

        // Parse definition JSON and extract Lambda ARNs
        let lambda_arns = extract_lambda_arns(&definition);
        let mut seen = std::collections::HashSet::new();
        for lambda_arn in lambda_arns {
            if !seen.insert(lambda_arn.clone()) {
                continue;
            }
            output.edges.push(Edge {
                from: sm_arn.clone(),
                to: lambda_arn,
                kind: EdgeKind::Invokes,
                props: serde_json::json!({}),
            });
        }
    }

    Ok(output)
}
