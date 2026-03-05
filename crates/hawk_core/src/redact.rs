use serde_json::Value;

/// Keys whose values must be completely redacted.
const REDACTED_KEYS: &[&str] = &[
    "secret",
    "password",
    "token",
    "key",
    "credential",
    "authorization",
    "auth",
    "private",
];

/// Recursively redact sensitive values from a JSON value in place.
/// - Removes values for keys that look like secrets
/// - Preserves structure so consumers know what fields existed
pub fn redact_props(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                let lower = k.to_lowercase();
                if REDACTED_KEYS.iter().any(|s| lower.contains(s)) {
                    *v = Value::String("[REDACTED]".into());
                } else {
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
    fn test_redact_props() {
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
}
