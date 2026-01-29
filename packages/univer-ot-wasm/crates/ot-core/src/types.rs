use serde::{Deserialize, Serialize};

// ============================================================================
// Core OT Types (Pure Rust, no Internal suffix)
// ============================================================================

/// Mutation information - the core type for all OT operations
/// Note: No "Internal" suffix - this is the authoritative Rust type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationInfo {
    pub id: String,
    pub params: serde_json::Value,
}

/// Result of transforming two concurrent mutations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    pub m1_prime: Option<MutationInfo>,
    pub m2_prime: Option<MutationInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
// Zero-Copy Optimization Types
// ============================================================================

/// Represents the outcome of a mutation after transformation.
///
/// This enum enables zero-copy returns for identity transforms by allowing
/// us to return a reference when the mutation is unchanged, and only clone
/// when the mutation was actually modified.
///
/// # Memory Optimization
/// - `Unchanged` - returns a reference, no allocation
/// - `Modified` - owns the modified data, requires allocation
/// - `Removed` - mutation should be removed, no data
#[derive(Debug)]
pub enum MutationOutcome<'a> {
    /// Mutation is unchanged - returns reference to original (zero-copy)
    Unchanged(&'a MutationInfo),
    /// Mutation was modified - returns owned modified version
    Modified(MutationInfo),
    /// Mutation should be removed/nullified
    Removed,
}

impl<'a> MutationOutcome<'a> {
    /// Convert to Option<MutationInfo>, cloning only for Unchanged variant
    #[inline]
    pub fn into_owned(self) -> Option<MutationInfo> {
        match self {
            MutationOutcome::Unchanged(m) => Some(m.clone()),
            MutationOutcome::Modified(m) => Some(m),
            MutationOutcome::Removed => None,
        }
    }

    /// Check if this outcome represents an unchanged mutation
    #[inline]
    pub fn is_unchanged(&self) -> bool {
        matches!(self, MutationOutcome::Unchanged(_))
    }

    /// Check if this outcome represents a modified mutation
    #[inline]
    pub fn is_modified(&self) -> bool {
        matches!(self, MutationOutcome::Modified(_))
    }

    /// Check if this outcome represents a removed mutation
    #[inline]
    pub fn is_removed(&self) -> bool {
        matches!(self, MutationOutcome::Removed)
    }

    /// Check if this outcome has a mutation (not removed)
    /// This is the equivalent of Option::is_some()
    #[inline]
    pub fn is_some(&self) -> bool {
        !matches!(self, MutationOutcome::Removed)
    }

    /// Check if this outcome is removed (no mutation)
    /// This is the equivalent of Option::is_none()
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, MutationOutcome::Removed)
    }

    /// Unwrap the mutation, panicking if removed
    /// This is the equivalent of Option::unwrap()
    #[inline]
    pub fn unwrap(self) -> MutationInfo {
        match self {
            MutationOutcome::Unchanged(m) => m.clone(),
            MutationOutcome::Modified(m) => m,
            MutationOutcome::Removed => panic!("called `MutationOutcome::unwrap()` on a `Removed` value"),
        }
    }

    /// Get a reference to the mutation if present
    #[inline]
    pub fn as_ref(&self) -> Option<&MutationInfo> {
        match self {
            MutationOutcome::Unchanged(m) => Some(m),
            MutationOutcome::Modified(m) => Some(m),
            MutationOutcome::Removed => None,
        }
    }
}

/// Optimized TransformResult using references where possible
///
/// This allows identity transforms to return references instead of clones,
/// significantly reducing memory allocations in common cases.
#[derive(Debug)]
pub struct TransformResultRef<'a> {
    pub m1_prime: MutationOutcome<'a>,
    pub m2_prime: MutationOutcome<'a>,
    pub error: Option<String>,
}

impl<'a> TransformResultRef<'a> {
    /// Create an identity transform result (zero-copy for both mutations)
    #[inline]
    pub fn identity(m1: &'a MutationInfo, m2: &'a MutationInfo) -> Self {
        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    }

    /// Create an error result (returns both mutations unchanged with error)
    #[inline]
    pub fn parse_error(m1: &'a MutationInfo, m2: &'a MutationInfo, msg: &str) -> Self {
        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Unchanged(m2),
            error: Some(msg.to_string()),
        }
    }

    /// Convert to owned TransformResult (clones Unchanged variants)
    #[inline]
    pub fn into_owned(self) -> TransformResult {
        TransformResult {
            m1_prime: self.m1_prime.into_owned(),
            m2_prime: self.m2_prime.into_owned(),
            error: self.error,
        }
    }
}

/// Mutation with operation ID for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationInfoWithOpId {
    pub id: String,
    pub params: serde_json::Value,
    #[serde(rename = "opId")]
    pub op_id: String,
}

// ============================================================================
// Spreadsheet Data Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v: Option<CellValue>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<CellValueType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub f: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<StyleData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub p: Option<DocumentData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub si: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<CustomData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellValueType {
    String = 1,
    Number = 2,
    Boolean = 3,
    ForceString = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ff: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub b: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub i: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomData {
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    #[serde(rename = "startRow", alias = "start_row")]
    pub start_row: u32,
    #[serde(rename = "startColumn", alias = "start_column")]
    pub start_column: u32,
    #[serde(rename = "endRow", alias = "end_row")]
    pub end_row: u32,
    #[serde(rename = "endColumn", alias = "end_column")]
    pub end_column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitParams {
    #[serde(rename = "unitId", alias = "unit_id")]
    pub unit_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubUnitParams {
    #[serde(rename = "unitId", alias = "unit_id")]
    pub unit_id: String,
    #[serde(rename = "subUnitId", alias = "sub_unit_id")]
    pub sub_unit_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMatrixPrimitiveType {
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformError {
    pub message: String,
    pub code: u32,
}
