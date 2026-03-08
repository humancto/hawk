use crate::ctx::AwsCtx;
use hawk_core::{DiscoveryOutput, Graph};
use tracing::{info, warn};

/// Which discovery modules to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    LambdaOnly,
    All,
}

/// Merge a discovery result into the graph, converting errors into warnings
/// so that one service failure doesn't kill the entire scan.
fn merge_result(graph: &mut Graph, service: &str, result: anyhow::Result<DiscoveryOutput>) {
    match result {
        Ok(output) => graph.merge(output),
        Err(e) => {
            let msg = format!("{service} discovery failed: {e}");
            warn!("{msg}");
            graph.warnings.push(msg);
        }
    }
}

/// Run discovery against AWS and return a merged Graph.
/// Individual service failures are captured as warnings rather than
/// aborting the entire scan (graceful degradation).
pub async fn discover_all(
    ctx: &AwsCtx,
    scope: Scope,
    profile: Option<&str>,
) -> anyhow::Result<Graph> {
    let mut graph = Graph::new();
    graph.profile = profile.map(|s| s.to_string());
    graph.regions = vec![ctx.region_str()];

    // Lambda is always included
    info!("Starting Lambda discovery...");
    let lambda_result = crate::lambda::discover(ctx).await;
    merge_result(&mut graph, "Lambda", lambda_result);

    if scope == Scope::All {
        // Run remaining discoveries concurrently
        let (eb, s3, sns, logs, sfn, apigw) = tokio::join!(
            crate::eventbridge::discover(ctx),
            crate::s3::discover(ctx),
            crate::sns::discover(ctx),
            crate::logs::discover(ctx),
            crate::sfn::discover(ctx),
            crate::apigw::discover(ctx),
        );

        merge_result(&mut graph, "EventBridge", eb);
        merge_result(&mut graph, "S3", s3);
        merge_result(&mut graph, "SNS", sns);
        merge_result(&mut graph, "CloudWatch Logs", logs);
        merge_result(&mut graph, "Step Functions", sfn);
        merge_result(&mut graph, "API Gateway v2", apigw);
    }

    graph.dedupe_and_sort();
    graph.compute_stats();

    info!(
        "Discovery complete: {} nodes, {} edges, {} warnings",
        graph.stats.node_count,
        graph.stats.edge_count,
        graph.warnings.len()
    );

    Ok(graph)
}
