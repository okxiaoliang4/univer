//! Compression utilities for Redis cache storage
//!
//! Provides JSON + Zstd one-shot serialization to reduce memory usage
//! and improve Redis storage efficiency.
//!
//! ## Design Principles
//!
//! 1. **Single Serialization**: struct → JSON → Zstd binary (no intermediate buffers)
//! 2. **Smart Compression**: Small data (< 256 bytes) stored as-is, large data compressed
//! 3. **Streaming Support**: Batch operations use Zstd streams for memory efficiency
//!
//! ## Data Format
//!
//! All compressed data uses a 1-byte prefix to indicate compression type:
//! - `0x00`: Raw JSON (no compression)
//! - `0x01`: Zstd compressed JSON

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};
use zstd::stream::{Decoder, Encoder};

/// Compression type markers
const COMPRESSION_NONE: u8 = 0;
const COMPRESSION_ZSTD: u8 = 1;

/// Threshold for compression (bytes)
/// Data smaller than this is stored as raw JSON
const COMPRESSION_THRESHOLD: usize = 256;

/// Zstd compression level (1-22, 3 is a good balance of speed/ratio)
const ZSTD_LEVEL: i32 = 3;

/// Serialize a value to Zstd-compressed JSON (always compressed)
///
/// Use this when you know the data is large enough to benefit from compression.
/// For general use, prefer `serialize_smart` which applies a size threshold.
///
/// # Example
/// ```ignore
/// let data = serialize_zstd(&my_struct)?;
/// let restored: MyStruct = deserialize_zstd(&data)?;
/// ```
pub fn serialize_zstd<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut encoder =
        Encoder::new(Vec::new(), ZSTD_LEVEL).context("Failed to create Zstd encoder")?;

    // serde_json writes directly to the Zstd stream (zero intermediate allocation)
    serde_json::to_writer(&mut encoder, value).context("Failed to serialize to JSON")?;

    encoder.finish().context("Failed to finish Zstd compression")
}

/// Deserialize a value from Zstd-compressed JSON
///
/// # Example
/// ```ignore
/// let restored: MyStruct = deserialize_zstd(&compressed_data)?;
/// ```
pub fn deserialize_zstd<T: DeserializeOwned>(data: &[u8]) -> Result<T> {
    let decoder = Decoder::new(data).context("Failed to create Zstd decoder")?;
    serde_json::from_reader(decoder).context("Failed to deserialize from JSON")
}

/// Smart serialize: small data as raw JSON, large data with Zstd compression
///
/// This is the recommended function for general use. It applies a threshold
/// to avoid compression overhead for small data.
///
/// # Data Format
/// ```text
/// [1 byte: compression flag][payload]
/// - flag 0x00: payload is raw JSON
/// - flag 0x01: payload is Zstd-compressed JSON
/// ```
///
/// # Example
/// ```ignore
/// let data = serialize_smart(&my_struct)?;
/// let restored: MyStruct = deserialize_smart(&data)?;
/// ```
pub fn serialize_smart<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    // First serialize to JSON to check size
    let json = serde_json::to_vec(value).context("Failed to serialize to JSON")?;

    if json.len() < COMPRESSION_THRESHOLD {
        // Small data: store as raw JSON with flag
        let mut result = Vec::with_capacity(1 + json.len());
        result.push(COMPRESSION_NONE);
        result.extend_from_slice(&json);
        Ok(result)
    } else {
        // Large data: compress with Zstd
        let compressed =
            zstd::encode_all(json.as_slice(), ZSTD_LEVEL).context("Failed to compress with Zstd")?;

        let mut result = Vec::with_capacity(1 + compressed.len());
        result.push(COMPRESSION_ZSTD);
        result.extend_from_slice(&compressed);
        Ok(result)
    }
}

/// Smart deserialize: handles both raw JSON and Zstd-compressed data
///
/// Automatically detects the compression type from the first byte and
/// deserializes accordingly.
///
/// # Example
/// ```ignore
/// let restored: MyStruct = deserialize_smart(&data)?;
/// ```
pub fn deserialize_smart<T: DeserializeOwned>(data: &[u8]) -> Result<T> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("Empty data"));
    }

    let (flag, payload) = data.split_first().unwrap();

    match *flag {
        COMPRESSION_NONE => {
            // Raw JSON
            serde_json::from_slice(payload).context("Failed to deserialize raw JSON")
        }
        COMPRESSION_ZSTD => {
            // Zstd compressed
            let decompressed =
                zstd::decode_all(payload).context("Failed to decompress Zstd data")?;
            serde_json::from_slice(&decompressed).context("Failed to deserialize decompressed JSON")
        }
        _ => Err(anyhow::anyhow!("Unknown compression flag: {}", flag)),
    }
}

/// Batch serialize: multiple items into a single Zstd stream
///
/// More efficient than serializing each item separately when dealing
/// with multiple items. Uses streaming compression.
///
/// # Data Format
/// ```text
/// [Zstd stream containing:]
///   [4 bytes: item count (little-endian u32)]
///   For each item:
///     [4 bytes: JSON length (little-endian u32)]
///     [JSON bytes]
/// ```
///
/// # Example
/// ```ignore
/// let data = serialize_batch_zstd(&items)?;
/// let restored: Vec<MyStruct> = deserialize_batch_zstd(&data)?;
/// ```
pub fn serialize_batch_zstd<T: Serialize>(items: &[T]) -> Result<Vec<u8>> {
    let mut encoder =
        Encoder::new(Vec::new(), ZSTD_LEVEL).context("Failed to create Zstd encoder")?;

    // Write item count
    let count = items.len() as u32;
    encoder
        .write_all(&count.to_le_bytes())
        .context("Failed to write item count")?;

    for item in items {
        // Serialize each item to JSON
        let json = serde_json::to_vec(item).context("Failed to serialize item to JSON")?;

        // Write length + JSON
        encoder
            .write_all(&(json.len() as u32).to_le_bytes())
            .context("Failed to write JSON length")?;
        encoder
            .write_all(&json)
            .context("Failed to write JSON data")?;
    }

    encoder.finish().context("Failed to finish batch compression")
}

/// Batch deserialize: single Zstd stream into multiple items
///
/// # Example
/// ```ignore
/// let restored: Vec<MyStruct> = deserialize_batch_zstd(&data)?;
/// ```
pub fn deserialize_batch_zstd<T: DeserializeOwned>(data: &[u8]) -> Result<Vec<T>> {
    let mut decoder = Decoder::new(data).context("Failed to create Zstd decoder")?;

    // Read item count
    let mut count_buf = [0u8; 4];
    decoder
        .read_exact(&mut count_buf)
        .context("Failed to read item count")?;
    let count = u32::from_le_bytes(count_buf) as usize;

    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        // Read length
        let mut len_buf = [0u8; 4];
        decoder
            .read_exact(&mut len_buf)
            .with_context(|| format!("Failed to read JSON length for item {}", i))?;
        let len = u32::from_le_bytes(len_buf) as usize;

        // Read JSON
        let mut json = vec![0u8; len];
        decoder
            .read_exact(&mut json)
            .with_context(|| format!("Failed to read JSON data for item {}", i))?;

        // Deserialize
        let item: T = serde_json::from_slice(&json)
            .with_context(|| format!("Failed to deserialize item {}", i))?;
        result.push(item);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        id: i64,
        name: String,
        data: Vec<u8>,
    }

    #[test]
    fn test_serialize_zstd() {
        let data = TestData {
            id: 42,
            name: "test".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };

        let compressed = serialize_zstd(&data).unwrap();
        let restored: TestData = deserialize_zstd(&compressed).unwrap();

        assert_eq!(data, restored);
    }

    #[test]
    fn test_serialize_smart_small_data() {
        let data = TestData {
            id: 1,
            name: "a".to_string(),
            data: vec![1],
        };

        let serialized = serialize_smart(&data).unwrap();

        // Small data should not be compressed
        assert_eq!(serialized[0], COMPRESSION_NONE);

        let restored: TestData = deserialize_smart(&serialized).unwrap();
        assert_eq!(data, restored);
    }

    #[test]
    fn test_serialize_smart_large_data() {
        let data = TestData {
            id: 42,
            name: "a".repeat(300), // Large enough to trigger compression
            data: vec![1; 100],
        };

        let serialized = serialize_smart(&data).unwrap();

        // Large data should be compressed
        assert_eq!(serialized[0], COMPRESSION_ZSTD);

        let restored: TestData = deserialize_smart(&serialized).unwrap();
        assert_eq!(data, restored);
    }

    #[test]
    fn test_serialize_batch_zstd() {
        let items: Vec<TestData> = (0..10)
            .map(|i| TestData {
                id: i,
                name: format!("item-{}", i),
                data: vec![i as u8; 10],
            })
            .collect();

        let compressed = serialize_batch_zstd(&items).unwrap();
        let restored: Vec<TestData> = deserialize_batch_zstd(&compressed).unwrap();

        assert_eq!(items, restored);
    }

    #[test]
    fn test_deserialize_smart_empty_data() {
        let result: Result<TestData> = deserialize_smart(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_smart_invalid_flag() {
        let result: Result<TestData> = deserialize_smart(&[0xFF, 0x01, 0x02]);
        assert!(result.is_err());
    }

    #[test]
    fn test_compression_ratio() {
        // Test that compression actually reduces size for large data
        let data = TestData {
            id: 42,
            name: "a".repeat(1000),
            data: vec![0; 1000], // Highly compressible
        };

        let json = serde_json::to_vec(&data).unwrap();
        let compressed = serialize_smart(&data).unwrap();

        // Compressed should be smaller than raw JSON (minus the flag byte overhead)
        assert!(
            compressed.len() < json.len(),
            "Compressed size {} should be smaller than JSON size {}",
            compressed.len(),
            json.len()
        );
    }
}
