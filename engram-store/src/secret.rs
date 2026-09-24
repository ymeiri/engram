//! Shared storage-boundary secret detection and redaction.

use crate::{StoreError, StoreResult};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

pub(crate) fn reject_serialized_secret_material<T: Serialize>(
    record_label: &str,
    record: &T,
) -> StoreResult<()> {
    let value = serde_json::to_value(record)?;
    if let Some(secret_kind) = secret_kind_in_json(&value) {
        return Err(StoreError::Policy(format!(
            "{record_label} contains a likely {secret_kind}; secret material was not persisted"
        )));
    }
    Ok(())
}

pub(crate) fn redact_serialized_secret_material<T>(
    record_label: &str,
    record: &T,
) -> StoreResult<(T, usize)>
where
    T: Serialize + DeserializeOwned,
{
    let mut value = serde_json::to_value(record)?;
    let redacted_fields = redact_json_strings(&mut value);
    if redacted_fields > 0 {
        tracing::warn!(
            record_label,
            redacted_fields,
            "Redacted likely secret material before durable persistence"
        );
    }
    Ok((serde_json::from_value(value)?, redacted_fields))
}

pub(crate) fn likely_secret_kind(value: &str) -> Option<&'static str> {
    let upper = value.to_ascii_uppercase();
    if upper.contains("-----BEGIN PRIVATE KEY-----")
        || upper.contains("-----BEGIN RSA PRIVATE KEY-----")
        || upper.contains("-----BEGIN OPENSSH PRIVATE KEY-----")
    {
        return Some("private key");
    }
    if upper.contains("AUTHORIZATION: BEARER ") {
        return Some("bearer token");
    }
    if contains_credential_url(value) {
        return Some("URL credential");
    }
    if contains_secret_assignment(value) {
        return Some("secret assignment");
    }

    for token in value.split(|character: char| {
        character.is_whitespace()
            || matches!(
                character,
                '\'' | '"' | '`' | ',' | ';' | '(' | ')' | '[' | ']'
            )
    }) {
        let token = token.trim_matches(|character: char| matches!(character, ':' | '='));
        if token.starts_with("AKIA")
            && token.len() == 20
            && token
                .chars()
                .all(|character| character.is_ascii_alphanumeric())
        {
            return Some("AWS access key");
        }
        if token.starts_with("github_pat_") && token.len() >= 30 {
            return Some("GitHub token");
        }
        if ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"]
            .iter()
            .any(|prefix| token.starts_with(prefix) && token.len() >= 20)
        {
            return Some("GitHub token");
        }
        if ["xoxb-", "xoxp-", "xoxa-", "xoxr-", "xoxs-"]
            .iter()
            .any(|prefix| token.starts_with(prefix) && token.len() >= 20)
        {
            return Some("Slack token");
        }
        if token.starts_with("sk-") && token.len() >= 24 {
            return Some("API key");
        }
        if looks_like_jwt(token) {
            return Some("JSON web token");
        }
    }
    None
}

fn secret_kind_in_json(value: &Value) -> Option<&'static str> {
    match value {
        Value::String(value) => likely_secret_kind(value),
        Value::Array(values) => values.iter().find_map(secret_kind_in_json),
        Value::Object(values) => values.iter().find_map(|(key, value)| {
            secret_field_kind(key)
                .filter(|_| is_secret_field_value(value))
                .or_else(|| likely_secret_kind(key))
                .or_else(|| secret_kind_in_json(value))
        }),
        Value::Null | Value::Bool(_) | Value::Number(_) => None,
    }
}

fn redact_json_strings(value: &mut Value) -> usize {
    match value {
        Value::String(value) => match likely_secret_kind(value) {
            Some(secret_kind) => {
                *value = format!("[redacted: likely {secret_kind}]");
                1
            }
            None => 0,
        },
        Value::Array(values) => values.iter_mut().map(redact_json_strings).sum(),
        Value::Object(values) => {
            let original = std::mem::take(values);
            let mut redacted_fields = 0;
            for (key, mut nested) in original {
                if let Some(secret_kind) =
                    secret_field_kind(&key).filter(|_| is_secret_field_value(&nested))
                {
                    redacted_fields += redact_secret_field_value(&mut nested, secret_kind);
                } else {
                    redacted_fields += redact_json_strings(&mut nested);
                }

                let key = if let Some(secret_kind) = likely_secret_kind(&key) {
                    redacted_fields += 1;
                    format!("[redacted key {redacted_fields}: likely {secret_kind}]")
                } else {
                    key
                };
                values.insert(key, nested);
            }
            redacted_fields
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => 0,
    }
}

fn secret_field_kind(name: &str) -> Option<&'static str> {
    let normalized = name.trim().replace(['-', ' '], "_").to_ascii_uppercase();
    match normalized.as_str() {
        "API_KEY" | "APIKEY" | "API_TOKEN" | "ACCESS_TOKEN" | "AUTH_TOKEN" => {
            Some("API credential")
        }
        "PASSWORD" | "PASSWD" => Some("password"),
        "CLIENT_SECRET" | "PRIVATE_KEY" => Some("secret field"),
        "AUTHORIZATION" => Some("authorization credential"),
        _ => None,
    }
}

fn is_secret_field_value(value: &Value) -> bool {
    match value {
        Value::String(value) => is_secret_field_string(value),
        Value::Array(values) => values.iter().any(is_secret_field_value),
        Value::Object(values) => values.values().any(is_secret_field_value),
        Value::Null | Value::Bool(_) | Value::Number(_) => false,
    }
}

fn is_secret_field_string(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    value.trim().len() >= 8
        && !matches!(
            normalized.as_str(),
            "[redacted]"
                | "changeme"
                | "example"
                | "not-set"
                | "placeholder"
                | "redacted"
                | "your_token_here"
        )
}

fn redact_secret_field_value(value: &mut Value, secret_kind: &str) -> usize {
    match value {
        Value::String(value) if is_secret_field_string(value) => {
            *value = format!("[redacted: likely {secret_kind}]");
            1
        }
        Value::Array(values) => values
            .iter_mut()
            .map(|value| redact_secret_field_value(value, secret_kind))
            .sum(),
        Value::Object(values) => values
            .values_mut()
            .map(|value| redact_secret_field_value(value, secret_kind))
            .sum(),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => 0,
    }
}

fn contains_credential_url(value: &str) -> bool {
    value.split_whitespace().any(|token| {
        let Some((_, authority_and_path)) = token.split_once("://") else {
            return false;
        };
        let authority = authority_and_path.split('/').next().unwrap_or_default();
        authority
            .split_once('@')
            .is_some_and(|(user_info, _)| user_info.contains(':'))
    })
}

fn contains_secret_assignment(value: &str) -> bool {
    const SECRET_NAMES: &[&str] = &[
        "API_KEY",
        "API_TOKEN",
        "ACCESS_TOKEN",
        "AUTH_TOKEN",
        "PASSWORD",
        "PASSWD",
        "CLIENT_SECRET",
        "PRIVATE_KEY",
    ];

    value.lines().any(|line| {
        line.split_whitespace().any(|token| {
            let token = token.trim_matches(|character: char| {
                matches!(
                    character,
                    '\'' | '"' | '`' | ',' | ';' | '(' | ')' | '[' | ']'
                )
            });
            let token = token.strip_prefix("export").unwrap_or(token).trim();
            let Some((name, assigned)) = token.split_once('=') else {
                return false;
            };
            let name = name.trim().to_ascii_uppercase();
            let assigned = assigned.trim().trim_matches(['\'', '"', '`']);
            SECRET_NAMES.contains(&name.as_str()) && assigned.len() >= 8
        })
    })
}

fn looks_like_jwt(token: &str) -> bool {
    let segments = token.split('.').collect::<Vec<_>>();
    segments.len() == 3
        && segments[0].starts_with("eyJ")
        && segments
            .iter()
            .all(|segment| segment.len() >= 8 && segment.chars().all(is_base64_url_character))
}

fn is_base64_url_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '=')
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serialized_rejection_scans_nested_object_keys_and_values() {
        let key_canary = json!({"API_TOKEN=synthetic-key-secret": "safe"});
        let value_canary = json!({"nested": ["run API_TOKEN=synthetic-value-secret cargo test"]});
        let named_field_canary = json!({"password": "synthetic-json-secret"});

        assert!(reject_serialized_secret_material("record", &key_canary).is_err());
        assert!(reject_serialized_secret_material("record", &value_canary).is_err());
        assert!(reject_serialized_secret_material("record", &named_field_canary).is_err());
    }

    #[test]
    fn serialized_redaction_removes_secret_values_without_failing() {
        let record = json!({
            "query": "Authorization: Bearer synthetic-redaction-secret",
            "password": "synthetic-password-secret",
            "API_TOKEN=synthetic-key-secret": "safe value",
            "safe": "retain this"
        });

        let (redacted, count) = redact_serialized_secret_material("test record", &record).unwrap();

        assert_eq!(count, 3);
        assert_eq!(redacted["safe"], "retain this");
        assert!(redacted["query"].as_str().unwrap().contains("redacted"));
        let persisted = redacted.to_string();
        assert!(!persisted.contains("synthetic-redaction-secret"));
        assert!(!persisted.contains("synthetic-password-secret"));
        assert!(!persisted.contains("synthetic-key-secret"));
    }
}
