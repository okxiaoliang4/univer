//! Params encoding/decoding module - MessagePack binary serialization
//!
//! This module provides functions for encoding and decoding operation params
//! using MessagePack (rmp-serde) for efficient binary storage. MessagePack is
//! chosen over bincode because it supports self-describing types like serde_json::Value.
//!
//! It also includes backwards-compatible decoding for legacy JSON-encoded params
//! during migration.

use anyhow::{Context, Result};
use serde_json::Value as JsonValue;

/// Encode params to MessagePack binary format.
///
/// This produces a compact binary representation that is typically
/// 20-40% smaller than JSON and faster to serialize/deserialize.
pub fn encode_params(params: &JsonValue) -> Result<Vec<u8>> {
    rmp_serde::to_vec(params).context("Failed to serialize params with MessagePack")
}

/// Decode params from MessagePack binary format.
pub fn decode_params(data: &[u8]) -> Result<JsonValue> {
    rmp_serde::from_slice(data).context("Failed to deserialize params from MessagePack")
}

/// Backwards-compatible decode that supports both legacy JSON and new MessagePack formats.
///
/// Detection heuristic: If the data starts with a JSON-typical byte ('{', '[', '"', 'n', 't', 'f', or digit),
/// treat it as legacy JSON text. Otherwise, treat it as MessagePack binary.
///
/// This function should be used during the migration period when the database may
/// contain both old JSON-encoded params and new MessagePack-encoded params.
pub fn decode_params_compat(data: &[u8]) -> Result<JsonValue> {
    if data.is_empty() {
        return Ok(JsonValue::Null);
    }

    // Detect legacy JSON format by checking first byte
    // JSON values start with: { [ " n(ull) t(rue) f(alse) or digits (0-9, -)
    if matches!(
        data[0],
        b'{' | b'[' | b'"' | b'n' | b't' | b'f' | b'-' | b'0'..=b'9'
    ) {
        let text =
            std::str::from_utf8(data).context("Invalid UTF-8 in legacy JSON params")?;
        serde_json::from_str(text).context("Failed to parse legacy JSON params")
    } else {
        decode_params(data)
    }
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
    fn test_msgpack_smaller_than_json() {
        let params = json!({
            "unitId": "550e8400-e29b-41d4-a716-446655440000",
            "subUnitId": "550e8400-e29b-41d4-a716-446655440001",
            "range": {"startRow": 0, "endRow": 10, "startColumn": 0, "endColumn": 5}
        });
        let msgpack_size = encode_params(&params).unwrap().len();
        let json_size = serde_json::to_string(&params).unwrap().len();
        assert!(
            msgpack_size < json_size,
            "MessagePack size ({}) should be smaller than JSON size ({})",
            msgpack_size,
            json_size
        );
    }

    #[test]
    fn test_compat_decode_legacy_json_object() {
        let params = json!({"test": "value", "number": 42});
        let legacy = serde_json::to_string(&params).unwrap();
        let decoded = decode_params_compat(legacy.as_bytes()).unwrap();
        assert_eq!(decoded, params);
    }

    #[test]
    fn test_compat_decode_legacy_json_array() {
        let params = json!([1, 2, 3, "test"]);
        let legacy = serde_json::to_string(&params).unwrap();
        let decoded = decode_params_compat(legacy.as_bytes()).unwrap();
        assert_eq!(decoded, params);
    }

    #[test]
    fn test_compat_decode_legacy_json_string() {
        let params = json!("hello world");
        let legacy = serde_json::to_string(&params).unwrap();
        let decoded = decode_params_compat(legacy.as_bytes()).unwrap();
        assert_eq!(decoded, params);
    }

    #[test]
    fn test_compat_decode_legacy_json_null() {
        let params = json!(null);
        let legacy = serde_json::to_string(&params).unwrap();
        let decoded = decode_params_compat(legacy.as_bytes()).unwrap();
        assert_eq!(decoded, params);
    }

    #[test]
    fn test_compat_decode_legacy_json_bool() {
        let params_true = json!(true);
        let legacy_true = serde_json::to_string(&params_true).unwrap();
        assert_eq!(
            decode_params_compat(legacy_true.as_bytes()).unwrap(),
            params_true
        );

        let params_false = json!(false);
        let legacy_false = serde_json::to_string(&params_false).unwrap();
        assert_eq!(
            decode_params_compat(legacy_false.as_bytes()).unwrap(),
            params_false
        );
    }

    #[test]
    fn test_compat_decode_legacy_json_number() {
        let params = json!(-123.456);
        let legacy = serde_json::to_string(&params).unwrap();
        let decoded = decode_params_compat(legacy.as_bytes()).unwrap();
        assert_eq!(decoded, params);
    }

    #[test]
    fn test_compat_decode_new_msgpack() {
        let params = json!({"test": "value"});
        let encoded = encode_params(&params).unwrap();
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

    #[test]
    fn test_size_comparison_typical_mutation() {
        // Typical SetRangeValuesMutation params
        let params = json!({
            "unitId": "workbook-550e8400-e29b-41d4-a716-446655440000",
            "subUnitId": "sheet-550e8400-e29b-41d4-a716-446655440001",
            "cellValue": {
                "0": {
                    "0": {"v": "Hello", "s": "default"},
                    "1": {"v": 123, "t": 2},
                    "2": {"v": true, "t": 1},
                    "3": {"v": "=A1+B1", "t": 2}
                },
                "1": {
                    "0": {"v": "World"},
                    "1": {"v": 456.789},
                    "2": {"v": false, "t": 1}
                }
            }
        });
        let msgpack_size = encode_params(&params).unwrap().len();
        let json_size = serde_json::to_string(&params).unwrap().len();

        // Print sizes for visibility
        println!("JSON size: {} bytes", json_size);
        println!("MessagePack size: {} bytes", msgpack_size);
        println!("Savings: {:.1}%", (1.0 - msgpack_size as f64 / json_size as f64) * 100.0);

        assert!(msgpack_size < json_size);
    }
}
