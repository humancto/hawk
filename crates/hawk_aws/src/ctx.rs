use aws_config::SdkConfig;

/// Holds the shared AWS SDK config and service clients.
pub struct AwsCtx {
    pub config: SdkConfig,
    pub lambda: aws_sdk_lambda::Client,
    pub eventbridge: aws_sdk_eventbridge::Client,
    pub s3: aws_sdk_s3::Client,
    pub sns: aws_sdk_sns::Client,
    pub logs: aws_sdk_cloudwatchlogs::Client,
    pub sfn: aws_sdk_sfn::Client,
    pub apigwv2: aws_sdk_apigatewayv2::Client,
    pub sqs: aws_sdk_sqs::Client,
}

impl AwsCtx {
    pub async fn new(profile: Option<&str>, region: Option<&str>) -> anyhow::Result<Self> {
        let mut loader = aws_config::from_env();

        if let Some(p) = profile {
            loader = loader.profile_name(p);
        }
        if let Some(r) = region {
            loader = loader.region(aws_config::Region::new(r.to_string()));
        }

        let config = loader.load().await;

        Ok(Self {
            lambda: aws_sdk_lambda::Client::new(&config),
            eventbridge: aws_sdk_eventbridge::Client::new(&config),
            s3: aws_sdk_s3::Client::new(&config),
            sns: aws_sdk_sns::Client::new(&config),
            logs: aws_sdk_cloudwatchlogs::Client::new(&config),
            sfn: aws_sdk_sfn::Client::new(&config),
            apigwv2: aws_sdk_apigatewayv2::Client::new(&config),
            sqs: aws_sdk_sqs::Client::new(&config),
            config,
        })
    }

    pub fn region_str(&self) -> String {
        self.config
            .region()
            .map(|r| r.to_string())
            .unwrap_or_else(|| "us-east-1".to_string())
    }
}
