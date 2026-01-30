use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Cell value - string, number, or boolean
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

/// Cell value type enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CellValueType {
    #[serde(rename = "1")]
    String = 1,
    #[serde(rename = "2")]
    Number = 2,
    #[serde(rename = "3")]
    Boolean = 3,
    #[serde(rename = "4")]
    ForceString = 4,
}

/// BooleanNumber enum (0 or 1)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BooleanNumber {
    #[serde(rename = "0")]
    False = 0,
    #[serde(rename = "1")]
    True = 1,
}

/// Range data structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IRange {
    pub start_row: i32,
    pub start_column: i32,
    pub end_row: i32,
    pub end_column: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_type: Option<i32>,
}

/// Document data (Univer Docs content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDocumentData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<IDocumentBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_style: Option<serde_json::Value>,
}

/// Document body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDocumentBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_stream: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_runs: Option<Vec<serde_json::Value>>,
}

/// Custom data - user stored fields
pub type CustomData = HashMap<String, serde_json::Value>;

/// Cell data with all properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ICellData {
    /// Origin value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v: Option<CellValue>,

    /// Cell value type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<CellValueType>,

    /// Formula string (e.g., "=SUM(A1:B4)")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f: Option<String>,

    /// Style ID or style data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<serde_json::Value>, // Can be string (style ID) or IStyleData

    /// Univer docs content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p: Option<IDocumentData>,

    /// Formula ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub si: Option<String>,

    /// Formula array reference
    #[serde(skip_serializing_if = "Option::is_none", rename = "ref")]
    pub ref_field: Option<String>,

    /// Excel formula prefix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xf: Option<String>,

    /// Custom data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<CustomData>,
}

/// Text decoration properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ITextDecoration {
    /// show (1 = true, 0 = false)
    pub s: BooleanNumber,
    /// color follows font color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c: Option<BooleanNumber>,
    /// color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl: Option<IColorStyle>,
    /// line type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<i32>,
}

/// Color style - RGB or theme color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IColorStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rgb: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub th: Option<i32>,
}

/// Border style data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBorderStyleData {
    pub s: i32, // BorderStyleTypes
    pub cl: IColorStyle,
}

/// Border data for all sides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBorderData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<IBorderStyleData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<IBorderStyleData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b: Option<IBorderStyleData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l: Option<IBorderStyleData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tl_br: Option<IBorderStyleData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tl_bc: Option<IBorderStyleData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tl_mr: Option<IBorderStyleData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bl_tr: Option<IBorderStyleData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ml_tr: Option<IBorderStyleData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bc_tr: Option<IBorderStyleData>,
}

/// Text rotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ITextRotation {
    /// angle
    pub a: f64,
    /// vertical (1 = true, 0 = false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v: Option<BooleanNumber>,
}

/// Padding data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPaddingData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l: Option<f64>,
}

/// Cell style data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IStyleData {
    /// font family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ff: Option<String>,
    /// font size (pt)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs: Option<f64>,
    /// italic (0 = false, 1 = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub it: Option<BooleanNumber>,
    /// bold (0 = false, 1 = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bl: Option<BooleanNumber>,
    /// underline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ul: Option<ITextDecoration>,
    /// strikethrough
    #[serde(skip_serializing_if = "Option::is_none")]
    pub st: Option<ITextDecoration>,
    /// overline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ol: Option<ITextDecoration>,
    /// background
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<IColorStyle>,
    /// border
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bd: Option<IBorderData>,
    /// foreground (font color)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl: Option<IColorStyle>,
    /// vertical alignment (Subscript/Superscript)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub va: Option<i32>,
    /// number format pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<INumberFormat>,
    /// text rotation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tr: Option<ITextRotation>,
    /// text direction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub td: Option<i32>,
    /// horizontal alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ht: Option<i32>,
    /// vertical alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vt: Option<i32>,
    /// wrap strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tb: Option<i32>,
    /// padding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pd: Option<IPaddingData>,
}

/// Number format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct INumberFormat {
    pub pattern: String,
}

/// Row data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRowData {
    /// height in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h: Option<f64>,
    /// is auto height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ia: Option<BooleanNumber>,
    /// auto height value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ah: Option<f64>,
    /// hidden (0 = false, 1 = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hd: Option<BooleanNumber>,
    /// style ID or style data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<serde_json::Value>,
    /// custom data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<CustomData>,
}

/// Column data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IColumnData {
    /// width in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w: Option<f64>,
    /// hidden (0 = false, 1 = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hd: Option<BooleanNumber>,
    /// style ID or style data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<serde_json::Value>,
    /// custom data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<CustomData>,
}

/// Freeze configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IFreeze {
    /// count of fixed columns
    pub x_split: i32,
    /// count of fixed rows
    pub y_split: i32,
    /// scrollable start row
    pub start_row: i32,
    /// scrollable start column
    pub start_column: i32,
}

/// Worksheet data snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IWorksheetData {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_color: Option<String>,
    pub hidden: BooleanNumber,
    pub freeze: IFreeze,
    pub row_count: i32,
    pub column_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zoom_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_top: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_left: Option<f64>,
    pub default_column_width: f64,
    pub default_row_height: f64,
    pub merge_data: Vec<IRange>,
    pub cell_data: IObjectMatrixPrimitiveType,
    pub row_data: IObjectArrayPrimitiveType<IRowData>,
    pub column_data: IObjectArrayPrimitiveType<IColumnData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_style: Option<serde_json::Value>,
    pub row_header: IHeaderConfig,
    pub column_header: IHeaderConfig,
    pub show_gridlines: BooleanNumber,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gridlines_color: Option<String>,
    pub right_to_left: BooleanNumber,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<CustomData>,
}

/// Header configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IHeaderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<BooleanNumber>,
}

/// Object matrix primitive type
pub type IObjectMatrixPrimitiveType = HashMap<String, HashMap<String, ICellData>>;

/// Object array primitive type
pub type IObjectArrayPrimitiveType<T> = HashMap<String, T>;

/// Copy options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ICopyToOptionsData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents_only: Option<bool>,
}

// ============================================================================
// Permission & Protection Types
// ============================================================================

/// View state enum for protection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViewStateEnum {
    #[serde(rename = "othersCanView")]
    OthersCanView,
    #[serde(rename = "noOneElseCanView")]
    NoOneElseCanView,
}

/// Edit state enum for protection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EditStateEnum {
    #[serde(rename = "designedUserCanEdit")]
    DesignedUserCanEdit,
    #[serde(rename = "onlyMe")]
    OnlyMe,
}

/// Unit object type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum UnitObject {
    #[serde(rename = "0")]
    Unknown = 0,
    #[serde(rename = "1")]
    Univer = 1,
    #[serde(rename = "2")]
    Workbook = 2,
    #[serde(rename = "3")]
    Worksheet = 3,
    #[serde(rename = "4")]
    Document = 4,
    #[serde(rename = "5")]
    Slide = 5,
}

/// Range protection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IRangeProtectionRule {
    pub ranges: Vec<IRange>,
    pub permission_id: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub unit_type: UnitObject,
    pub unit_id: String,
    pub sub_unit_id: String,
    pub view_state: ViewStateEnum,
    pub edit_state: EditStateEnum,
}

/// Worksheet protection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IWorksheetProtectionRule {
    pub permission_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub unit_type: UnitObject,
    pub unit_id: String,
    pub sub_unit_id: String,
    pub view_state: ViewStateEnum,
    pub edit_state: EditStateEnum,
}

/// Worksheet protection point rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IWorksheetProtectionPointRule {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub permission_id: String,
}

// ============================================================================
// Theme & Style Types
// ============================================================================

/// Range theme style
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IRangeThemeStyle {
    pub theme_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<serde_json::Value>,
}

// ============================================================================
// Dimension Types
// ============================================================================

/// Dimension enum (Row or Column)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Dimension {
    #[serde(rename = "0")]
    Rows = 0,
    #[serde(rename = "1")]
    Columns = 1,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformError {
    pub message: String,
    pub code: u32,
}

// ============================================================================
// Backward Compatibility Type Aliases
// ============================================================================

// Type aliases for backward compatibility with old code
pub type Range = IRange;
pub type ObjectMatrixPrimitiveType = IObjectMatrixPrimitiveType;

// SubUnitParams struct for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubUnitParams {
    pub unit_id: String,
    pub sub_unit_id: String,
}
