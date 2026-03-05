use crate::ctx::AwsCtx;
use hawk_core::Graph;
use tracing::info;

/// Which discovery modules to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    LambdaOnly,
    All,
}

/// Run discovery against AWS and return a merged Graph.
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
    let lambda_output = crate::lambda::discover(ctx).await;
    graph.merge(lambda_output);

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

        graph.merge(eb);
        graph.merge(s3);
        graph.merge(sns);
        graph.merge(logs);
        graph.merge(sfn);
        graph.merge(apigw);
    }

    graph.dedupe_and_sort();
    graph.compute_stats();

    info!(
        "Discovery complete: {} nodes, {} edges",
        graph.stats.node_count, graph.stats.edge_count
    );

    Ok(graph)
}
