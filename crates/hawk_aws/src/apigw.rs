use crate::arn::extract_lambda_arns;
use crate::ctx::AwsCtx;
use hawk_core::{DiscoveryOutput, Edge, EdgeKind, Node, NodeKind};
use tracing::{info, warn};

pub async fn discover(ctx: &AwsCtx) -> DiscoveryOutput {
    let mut output = DiscoveryOutput::default();
    let region = ctx.region_str();

    // List HTTP APIs
    let apis = match ctx.apigwv2.get_apis().send().await {
        Ok(resp) => resp.items.unwrap_or_default(),
        Err(e) => {
            output
                .warnings
                .push(format!("APIGWv2 GetApis error: {e}"));
            return output;
        }
    };

    info!("Discovered {} API Gateway v2 APIs in {}", apis.len(), region);

    for api in &apis {
        let api_id = match api.api_id() {
            Some(id) => id.to_string(),
            None => continue,
        };
        let api_name = api.name().unwrap_or("unnamed").to_string();
        let api_node_id = format!("apigwv2:{api_id}:{region}");

        output.nodes.push(Node {
            id: api_node_id.clone(),
            kind: NodeKind::ApiGateway,
            name: api_name,
            arn: None,
            region: Some(region.clone()),
            account_id: None,
            tags: None,
            props: serde_json::json!({
                "protocol_type": api.protocol_type().map(|p| p.as_str()),
            }),
        });

        // Get integrations for this API
        let integrations = match ctx
            .apigwv2
            .get_integrations()
            .api_id(&api_id)
            .send()
            .await
        {
            Ok(resp) => resp.items.unwrap_or_default(),
            Err(e) => {
                warn!("GetIntegrations({api_id}) error: {e}");
                output
                    .warnings
                    .push(format!("GetIntegrations({api_id}) error: {e}"));
                continue;
            }
        };

        // Build integration_id -> lambda ARN map
        let mut integration_map: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for integ in &integrations {
            let integ_id = match integ.integration_id() {
                Some(id) => id.to_string(),
                None => continue,
            };
            let uri = integ.integration_uri().unwrap_or_default();
            let arns = extract_lambda_arns(uri);
            if let Some(arn) = arns.into_iter().next() {
                integration_map.insert(integ_id, arn);
            }
        }

        // Get routes
        let routes = match ctx.apigwv2.get_routes().api_id(&api_id).send().await {
            Ok(resp) => resp.items.unwrap_or_default(),
            Err(e) => {
                warn!("GetRoutes({api_id}) error: {e}");
                output
                    .warnings
                    .push(format!("GetRoutes({api_id}) error: {e}"));
                continue;
            }
        };

        for route in &routes {
            let route_id = match route.route_id() {
                Some(id) => id.to_string(),
                None => continue,
            };
            let route_key = route.route_key().unwrap_or("*").to_string();
            let route_node_id = format!("route:{api_id}:{route_id}");

            output.nodes.push(Node {
                id: route_node_id.clone(),
                kind: NodeKind::ApiRoute,
                name: route_key.clone(),
                arn: None,
                region: Some(region.clone()),
                account_id: None,
                tags: None,
                props: serde_json::json!({
                    "route_key": route_key,
                    "authorization_type": route.authorization_type().map(|a| a.as_str()),
                }),
            });

            // Link route -> API
            output.edges.push(Edge {
                from: api_node_id.clone(),
                to: route_node_id.clone(),
                kind: EdgeKind::Triggers,
                props: serde_json::json!({}),
            });

            // Link route -> Lambda via integration target
            if let Some(target) = route.target() {
                // target is like "integrations/{integrationId}"
                if let Some(integ_id) = target.strip_prefix("integrations/") {
                    if let Some(lambda_arn) = integration_map.get(integ_id) {
                        output.edges.push(Edge {
                            from: route_node_id,
                            to: lambda_arn.clone(),
                            kind: EdgeKind::Triggers,
                            props: serde_json::json!({
                                "route_key": route_key,
                                "integration_type": "AWS_PROXY",
                            }),
                        });
                    }
                }
            }
        }
    }

    output
}
