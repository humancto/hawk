use crate::ctx::AwsCtx;
use hawk_core::{DiscoveryOutput, Edge, EdgeKind, Node, NodeKind};
use tracing::{info, warn};

pub async fn discover(ctx: &AwsCtx) -> anyhow::Result<DiscoveryOutput> {
    let mut output = DiscoveryOutput::default();
    let region = ctx.region_str();

    // List all topics (paginated)
    let mut paginator = ctx.sns.list_topics().into_paginator().send();

    let mut topic_arns = Vec::new();
    while let Some(page) = paginator.next().await {
        match page {
            Ok(resp) => {
                if let Some(topics) = resp.topics {
                    for t in topics {
                        if let Some(arn) = t.topic_arn() {
                            topic_arns.push(arn.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                output.warnings.push(format!("SNS ListTopics error: {e}"));
                return Ok(output);
            }
        }
    }

    info!("Discovered {} SNS topics in {}", topic_arns.len(), region);

    for topic_arn in &topic_arns {
        let topic_name = topic_arn
            .rsplit(':')
            .next()
            .unwrap_or(topic_arn)
            .to_string();

        let mut has_lambda_sub = false;

        // List subscriptions by topic (paginated)
        let mut sub_paginator = ctx
            .sns
            .list_subscriptions_by_topic()
            .topic_arn(topic_arn)
            .into_paginator()
            .send();

        while let Some(page) = sub_paginator.next().await {
            match page {
                Ok(resp) => {
                    if let Some(subs) = resp.subscriptions {
                        for sub in subs {
                            let protocol = sub.protocol().unwrap_or_default().to_string();
                            let endpoint = sub.endpoint().unwrap_or_default().to_string();
                            let sub_arn = sub.subscription_arn().unwrap_or_default().to_string();

                            if protocol == "lambda" && endpoint.contains(":function:") {
                                has_lambda_sub = true;
                                let props = serde_json::json!({
                                    "protocol": protocol,
                                    "subscription_arn": sub_arn,
                                });
                                output.edges.push(Edge {
                                    from: topic_arn.clone(),
                                    to: endpoint,
                                    kind: EdgeKind::Triggers,
                                    props,
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("ListSubscriptionsByTopic({topic_arn}) error: {e}");
                    output
                        .warnings
                        .push(format!("ListSubscriptionsByTopic error: {e}"));
                }
            }
        }

        // Only add topic node if it has Lambda subscriptions
        if has_lambda_sub {
            output.nodes.push(Node {
                id: topic_arn.clone(),
                kind: NodeKind::SnsTopic,
                name: topic_name,
                arn: Some(topic_arn.clone()),
                region: Some(region.clone()),
                account_id: None,
                tags: None,
                props: serde_json::json!({}),
            });
        }
    }

    Ok(output)
}
