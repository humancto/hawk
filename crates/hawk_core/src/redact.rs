use serde_json::Value;

/// Keys whose values must be completely redacted (substring match on lowercased key).
const REDACTED_KEY_PATTERNS: &[&str] = &[
    "secret",
    "password",
    "passwd",
    "token",
    "credential",
    "authorization",
    "private",
    "passphrase",
    "connection_string",
    "certificate",
    "signing",
];

/// Keys that are exact-suffix matches (catches "api_key", "secret_key" but not "route_key").
const REDACTED_KEY_SUFFIXES: &[&str] = &[
    "_key",
    "_secret",
    "_token",
    "_password",
    "_auth",
    "_credential",
    "_cert",
];

/// Keys that are exact matches (case-insensitive).
const REDACTED_EXACT_KEYS: &[&str] = &["auth", "apikey", "api_key"];

/// Patterns in string values that indicate embedded credentials.
const CREDENTIAL_VALUE_PREFIXES: &[&str] = &[
    "postgres://",
    "postgresql://",
    "mysql://",
    "mongodb://",
    "mongodb+srv://",
    "redis://",
    "rediss://",
    "amqp://",
    "amqps://",
];

/// Known structural keys that should NEVER be redacted, even if they
/// match a suffix pattern. These are hawk graph metadata fields.
const SAFE_KEY_ALLOWLIST: &[&str] = &[
    "route_key",
    "env_keys",
    "partition_key",
    "sort_key",
    "primary_key",
    "hash_key",
    "range_key",
    "resource_key",
    "tag_key",
];

/// Check if a key name looks like it holds sensitive data.
fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_lowercase();

    // Allowlist: never redact known structural keys
    if SAFE_KEY_ALLOWLIST.iter().any(|k| lower == *k) {
        return false;
    }

    // Check exact keys
    if REDACTED_EXACT_KEYS.iter().any(|k| lower == *k) {
        return true;
    }

    // Check substring patterns
    if REDACTED_KEY_PATTERNS.iter().any(|p| lower.contains(p)) {
        return true;
    }

    // Check suffix patterns
    if REDACTED_KEY_SUFFIXES.iter().any(|s| lower.ends_with(s)) {
        return true;
    }

    false
}

/// Check if a string value contains embedded credentials.
fn contains_embedded_credentials(value: &str) -> bool {
    let lower = value.to_lowercase();
    CREDENTIAL_VALUE_PREFIXES
        .iter()
        .any(|p| lower.starts_with(p))
}

/// Recursively redact sensitive values from a JSON value in place.
/// - Removes values for keys that look like secrets
/// - Detects embedded credentials in connection string URLs
/// - Preserves structure so consumers know what fields existed
pub fn redact_props(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if is_sensitive_key(k) {
                    *v = Value::String("[REDACTED]".into());
                } else {
                    // Check if the value itself contains embedded credentials
                    if let Value::String(s) = v {
                        if contains_embedded_credentials(s) {
                            *v = Value::String("[REDACTED_URL]".into());
                            continue;
                        }
                    }
                    redact_props(v);
                }
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                redact_props(v);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_redact_basic_keys() {
        let mut val = json!({
            "runtime": "nodejs18.x",
            "secret_key": "super-secret-value",
            "nested": {
                "password": "hunter2",
                "safe_field": "visible"
            },
            "auth_token": "abc123"
        });
        redact_props(&mut val);
        assert_eq!(val["runtime"], "nodejs18.x");
        assert_eq!(val["secret_key"], "[REDACTED]");
        assert_eq!(val["nested"]["password"], "[REDACTED]");
        assert_eq!(val["nested"]["safe_field"], "visible");
        assert_eq!(val["auth_token"], "[REDACTED]");
    }

    #[test]
    fn test_redact_expanded_patterns() {
        let mut val = json!({
            "connection_string": "Server=mydb;User=admin;Password=secret",
            "tls_certificate": "-----BEGIN CERTIFICATE-----",
            "ssh_passphrase": "my-passphrase",
            "api_key": "sk-abc123",
            "signing_key": "hmac-sha256-key",
        });
        redact_props(&mut val);
        assert_eq!(val["connection_string"], "[REDACTED]");
        assert_eq!(val["tls_certificate"], "[REDACTED]");
        assert_eq!(val["ssh_passphrase"], "[REDACTED]");
        assert_eq!(val["api_key"], "[REDACTED]");
        assert_eq!(val["signing_key"], "[REDACTED]");
    }

    #[test]
    fn test_redact_embedded_credentials() {
        let mut val = json!({
            "database_url": "postgres://user:pass@host:5432/db",
            "cache_url": "redis://default:mypassword@redis.example.com:6379",
            "normal_url": "https://api.example.com/v1",
        });
        redact_props(&mut val);
        assert_eq!(val["database_url"], "[REDACTED_URL]");
        assert_eq!(val["cache_url"], "[REDACTED_URL]");
        assert_eq!(val["normal_url"], "https://api.example.com/v1");
    }

    #[test]
    fn test_safe_keys_not_redacted() {
        let mut val = json!({
            "route_key": "GET /api/users",
            "env_keys": ["DATABASE_URL", "API_KEY"],
            "batch_size": 10,
            "handler": "index.handler",
        });
        redact_props(&mut val);
        // These structural keys should NOT be redacted
        assert_eq!(val["route_key"], "GET /api/users");
        assert_eq!(val["batch_size"], 10);
        assert_eq!(val["handler"], "index.handler");
        // env_keys is a list of key names, not values — should be preserved
        assert!(val["env_keys"].is_array());
    }
}
