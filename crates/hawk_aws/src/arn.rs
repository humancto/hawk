use regex::Regex;
use std::sync::LazyLock;

static LAMBDA_ARN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"arn:aws:lambda:[a-z0-9-]+:\d{12}:function:[a-zA-Z0-9_-]+").unwrap()
});

/// Extract all Lambda ARNs found in a string.
pub fn extract_lambda_arns(text: &str) -> Vec<String> {
    LAMBDA_ARN_RE
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Determine the kind of AWS resource from an ARN string.
pub fn resource_kind_from_arn(arn: &str) -> hawk_core::NodeKind {
    if arn.contains(":function:") {
        hawk_core::NodeKind::Lambda
    } else if arn.contains(":sqs:") || arn.contains(":queue:") {
        hawk_core::NodeKind::SqsQueue
    } else if arn.contains(":dynamodb:") && arn.contains("/stream/") {
        hawk_core::NodeKind::DynamoStream
    } else if arn.contains(":sns:") {
        hawk_core::NodeKind::SnsTopic
    } else if arn.contains(":s3:") || arn.starts_with("arn:aws:s3:::") {
        hawk_core::NodeKind::S3Bucket
    } else if arn.contains(":events:") {
        hawk_core::NodeKind::EventRule
    } else if arn.contains(":states:") {
        hawk_core::NodeKind::StepFunction
    } else if arn.contains(":logs:") {
        hawk_core::NodeKind::LogGroup
    } else {
        hawk_core::NodeKind::Unknown
    }
}

/// Extract a short name from an ARN (last segment after : or /).
pub fn name_from_arn(arn: &str) -> String {
    arn.rsplit([':', '/'])
        .next()
        .unwrap_or(arn)
        .to_string()
}

/// Extract account ID from an ARN (field index 4).
pub fn account_from_arn(arn: &str) -> Option<String> {
    let parts: Vec<&str> = arn.split(':').collect();
    if parts.len() >= 5 && !parts[4].is_empty() {
        Some(parts[4].to_string())
    } else {
        None
    }
}

/// Extract region from an ARN (field index 3).
pub fn region_from_arn(arn: &str) -> Option<String> {
    let parts: Vec<&str> = arn.split(':').collect();
    if parts.len() >= 4 && !parts[3].is_empty() {
        Some(parts[3].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_lambda_arns() {
        let text = r#"{"Resource": "arn:aws:lambda:us-east-1:123456789012:function:MyFunc"}"#;
        let arns = extract_lambda_arns(text);
        assert_eq!(arns, vec!["arn:aws:lambda:us-east-1:123456789012:function:MyFunc"]);
    }

    #[test]
    fn test_resource_kind_from_arn() {
        assert_eq!(
            resource_kind_from_arn("arn:aws:sqs:us-east-1:123456789012:my-queue"),
            hawk_core::NodeKind::SqsQueue
        );
        assert_eq!(
            resource_kind_from_arn("arn:aws:lambda:us-east-1:123:function:fn1"),
            hawk_core::NodeKind::Lambda
        );
    }

    #[test]
    fn test_name_from_arn() {
        assert_eq!(
            name_from_arn("arn:aws:lambda:us-east-1:123:function:MyFunc"),
            "MyFunc"
        );
    }

    #[test]
    fn test_account_and_region() {
        let arn = "arn:aws:lambda:us-west-2:999888777666:function:test";
        assert_eq!(account_from_arn(arn), Some("999888777666".into()));
        assert_eq!(region_from_arn(arn), Some("us-west-2".into()));
    }
}
