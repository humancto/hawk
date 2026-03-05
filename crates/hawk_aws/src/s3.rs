use crate::ctx::AwsCtx;
use hawk_core::{DiscoveryOutput, Edge, EdgeKind, Node, NodeKind};
use tracing::info;

pub async fn discover(ctx: &AwsCtx) -> DiscoveryOutput {
    let mut output = DiscoveryOutput::default();
    let region = ctx.region_str();

    let buckets = match ctx.s3.list_buckets().send().await {
        Ok(resp) => resp.buckets.unwrap_or_default(),
        Err(e) => {
            output.warnings.push(format!("S3 ListBuckets error: {e}"));
            return output;
        }
    };

    info!("Discovered {} S3 buckets", buckets.len());

    for bucket in &buckets {
        let name = match bucket.name() {
            Some(n) => n.to_string(),
            None => continue,
        };
        let bucket_id = format!("arn:aws:s3:::{name}");

        // Get notification configuration
        let notif = match ctx
            .s3
            .get_bucket_notification_configuration()
            .bucket(&name)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                output
                    .warnings
                    .push(format!("S3 GetBucketNotification({name}) error: {e}"));
                continue;
            }
        };

        let lambda_configs = notif.lambda_function_configurations();
        if lambda_configs.is_empty() {
            continue;
        }

        output.nodes.push(Node {
            id: bucket_id.clone(),
            kind: NodeKind::S3Bucket,
            name: name.clone(),
            arn: Some(bucket_id.clone()),
            region: Some(region.clone()),
            account_id: None,
            tags: None,
            props: serde_json::json!({}),
        });

        for config in lambda_configs {
            // lambda_function_arn() returns &str in the AWS SDK
            let lambda_arn = config.lambda_function_arn().to_string();
            if lambda_arn.is_empty() {
                continue;
            }

            let events: Vec<String> = config
                .events()
                .iter()
                .map(|e| e.as_str().to_string())
                .collect();

            let props = serde_json::json!({
                "events": events,
            });

            output.edges.push(Edge {
                from: bucket_id.clone(),
                to: lambda_arn,
                kind: EdgeKind::Triggers,
                props,
            });
        }
    }

    output
}
