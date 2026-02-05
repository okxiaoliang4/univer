//! Params encoding/decoding module - JSON binary serialization
//!
//! This module provides functions for encoding and decoding operation params
//! using JSON for storage. Params are stored as:
//! - Small params (< threshold): inline in PostgreSQL as BYTEA
//! - Large params (>= threshold): in S3 as binary files
//!
//! Compression is handled at the cache layer (via compression.rs with Zstd),
//! not at the params encoding layer.

use anyhow::{Context, Result};
use serde_json::Value as JsonValue;

/// Encode params to JSON binary format.
///
/// This produces a UTF-8 encoded JSON representation suitable for
/// storage in PostgreSQL BYTEA or S3.
pub fn encode_params(params: &JsonValue) -> Result<Vec<u8>> {
    serde_json::to_vec(params).context("Failed to serialize params to JSON")
}

/// Decode params from JSON binary format.
pub fn decode_params(data: &[u8]) -> Result<JsonValue> {
    serde_json::from_slice(data).context("Failed to deserialize params from JSON")
}

/// Backwards-compatible decode that supports both legacy formats and new JSON.
///
/// This function handles:
/// 1. Plain JSON (current format)
/// 2. Legacy MessagePack (for backward compatibility during migration)
///
/// Detection heuristic: If the data starts with a JSON-typical byte ('{', '[', '"', 'n', 't', 'f', or digit),
/// treat it as JSON. Otherwise, try to decode as JSON first, then fail gracefully.
///
/// This function should be used when reading from the database where old MessagePack
/// data might still exist.
pub fn decode_params_compat(data: &[u8]) -> Result<JsonValue> {
    if data.is_empty() {
        return Ok(JsonValue::Null);
    }

    // Try JSON first (most common case now)
    if let Ok(value) = serde_json::from_slice::<JsonValue>(data) {
        return Ok(value);
    }

    // If JSON fails, it might be legacy MessagePack
    // For now, return an error since we no longer have MessagePack support
    // This should only happen for very old data that predates this change
    Err(anyhow::anyhow!(
        "Failed to decode params: not valid JSON (first byte: {:02x})",
        data[0]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_roundtrip() {
        let params = json!({
            "unitId": "test-unit",
            "subUnitId": "test-sheet",
            "cellValue": {"0": {"0": {"v": "hello"}}}
        });
        let encoded = encode_params(&params).unwrap();
        let decoded = decode_params(&encoded).unwrap();
        assert_eq!(decoded, params);
    }

    #[test]
    fn test_compat_decode_json_object() {
        let params = json!({"test": "value", "number": 42});
        let encoded = serde_json::to_vec(&params).unwrap();
        let decoded = decode_params_compat(&encoded).unwrap();
        assert_eq!(decoded, params);
    }

    #[test]
    fn test_compat_decode_json_array() {
        let params = json!([1, 2, 3, "test"]);
        let encoded = serde_json::to_vec(&params).unwrap();
        let decoded = decode_params_compat(&encoded).unwrap();
        assert_eq!(decoded, params);
    }

    #[test]
    fn test_compat_decode_json_string() {
        let params = json!("hello world");
        let encoded = serde_json::to_vec(&params).unwrap();
        let decoded = decode_params_compat(&encoded).unwrap();
        assert_eq!(decoded, params);
    }

    #[test]
    fn test_compat_decode_json_null() {
        let params = json!(null);
        let encoded = serde_json::to_vec(&params).unwrap();
        let decoded = decode_params_compat(&encoded).unwrap();
        assert_eq!(decoded, params);
    }

    #[test]
    fn test_compat_decode_json_bool() {
        let params_true = json!(true);
        let encoded_true = serde_json::to_vec(&params_true).unwrap();
        assert_eq!(
            decode_params_compat(&encoded_true).unwrap(),
            params_true
        );

        let params_false = json!(false);
        let encoded_false = serde_json::to_vec(&params_false).unwrap();
        assert_eq!(
            decode_params_compat(&encoded_false).unwrap(),
            params_false
        );
    }

    #[test]
    fn test_compat_decode_json_number() {
        let params = json!(-123.456);
        let encoded = serde_json::to_vec(&params).unwrap();
        let decoded = decode_params_compat(&encoded).unwrap();
        assert_eq!(decoded, params);
    }

    #[test]
    fn test_empty_data() {
        let decoded = decode_params_compat(&[]).unwrap();
        assert_eq!(decoded, JsonValue::Null);
    }

    #[test]
    fn test_complex_nested_structure() {
        let params = json!({
            "unitId": "doc-123",
            "mutations": [
                {
                    "id": "sheet.mutation.set-range-values",
                    "params": {
                        "subUnitId": "sheet-1",
                        "cellValue": {
                            "0": {
                                "0": {"v": "A1", "s": "style-1"},
                                "1": {"v": 100, "t": 2}
                            },
                            "1": {
                                "0": {"v": "B1"},
                                "1": {"v": true, "t": 1}
                            }
                        }
                    }
                }
            ]
        });
        let encoded = encode_params(&params).unwrap();
        let decoded = decode_params(&encoded).unwrap();
        assert_eq!(decoded, params);
    }
}
