//! Common types used across sheets mutations
//!
//! This module provides both re-exports of core types and sheets-specific types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Re-exports from crate::types
// ============================================================================

/// Re-export commonly used core types for convenience
pub use crate::types::{
    // Basic types
    BooleanNumber,
    CellValue,
    CellValueType,
    CustomData,

    // Range types
    IRange,
    Range,
    SubUnitParams,

    // Cell and style types
    ICellData,
    IStyleData,
    IColorStyle,
    IBorderData,
    IBorderStyleData,
    ITextDecoration,
    ITextRotation,
    IPaddingData,
    INumberFormat,

    // Row and column types
    IRowData,
    IColumnData,

    // Worksheet types
    IWorksheetData,
    IFreeze,
    IHeaderConfig,

    // Document types
    IDocumentData,
    IDocumentBody,

    // Protection types
    IRangeProtectionRule,
    IWorksheetProtectionRule,
    IWorksheetProtectionPointRule,
    ViewStateEnum,
    EditStateEnum,
    UnitObject,

    // Theme types
    IRangeThemeStyle,

    // Dimension types
    Dimension,

    // Matrix and array types
    IObjectMatrixPrimitiveType,
    ObjectMatrixPrimitiveType,
    IObjectArrayPrimitiveType,

    // Copy options
    ICopyToOptionsData,
};

// ============================================================================
// Range Theme Style Types (Sheets-specific)
// ============================================================================

/// Range theme style item - subset of IStyleData for theme styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRangeThemeStyleItem {
    /// Background color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<serde_json::Value>, // IColorStyle

    /// Overline decoration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ol: Option<serde_json::Value>, // ITextDecoration

    /// Border data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bd: Option<serde_json::Value>, // IBorderData

    /// Font color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl: Option<serde_json::Value>, // IColorStyle

    /// Horizontal alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ht: Option<i32>,

    /// Vertical alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vt: Option<i32>,

    /// Bold (0 = false, 1 = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bl: Option<i32>,
}

/// Range theme style JSON structure with all style components
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IRangeThemeStyleJSON {
    /// Theme name
    pub name: String,

    /// Style for the entire range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whole_style: Option<IRangeThemeStyleItem>,

    /// Style for header row
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_row_style: Option<IRangeThemeStyleItem>,

    /// Style for header column
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_column_style: Option<IRangeThemeStyleItem>,

    /// Style for first row
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_row_style: Option<IRangeThemeStyleItem>,

    /// Style for second row (for alternating patterns)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_row_style: Option<IRangeThemeStyleItem>,

    /// Style for last row
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_row_style: Option<IRangeThemeStyleItem>,

    /// Style for first column
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_column_style: Option<IRangeThemeStyleItem>,

    /// Style for second column (for alternating patterns)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_column_style: Option<IRangeThemeStyleItem>,

    /// Style for last column
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_column_style: Option<IRangeThemeStyleItem>,
}

/// Partial range theme style (without name) for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialRangeThemeStyle {
    /// Style for the entire range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whole_style: Option<IRangeThemeStyleItem>,

    /// Style for header row
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_row_style: Option<IRangeThemeStyleItem>,

    /// Style for header column
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_column_style: Option<IRangeThemeStyleItem>,

    /// Style for first row
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_row_style: Option<IRangeThemeStyleItem>,

    /// Style for second row (for alternating patterns)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_row_style: Option<IRangeThemeStyleItem>,

    /// Style for last row
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_row_style: Option<IRangeThemeStyleItem>,

    /// Style for first column
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_column_style: Option<IRangeThemeStyleItem>,

    /// Style for second column (for alternating patterns)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_column_style: Option<IRangeThemeStyleItem>,

    /// Style for last column
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_column_style: Option<IRangeThemeStyleItem>,
}

// ============================================================================
// Number Format Types (Sheets-specific)
// ============================================================================

/// Number format ranges - associates ranges with a format pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumfmtRanges {
    /// List of ranges that share the same number format
    pub ranges: Vec<IRange>,
}

/// Number format reference item - contains the format pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumfmtRefItem {
    /// The number format pattern string
    pub pattern: String,
}

// ============================================================================
// Helper Type Aliases
// ============================================================================

/// HashMap for row/column data indexed by row/column number as string
pub type RowColDataMap<T> = HashMap<String, T>;

/// HashMap for cell data (row -> column -> cell)
pub type CellDataMap = HashMap<String, HashMap<String, ICellData>>;

/// HashMap for style data indexed by style ID
pub type StyleMap = HashMap<String, Option<IStyleData>>;

/// HashMap for number format values (format ID -> ranges)
pub type NumfmtValuesMap = HashMap<String, NumfmtRanges>;

/// HashMap for number format reference (format ID -> pattern)
pub type NumfmtRefMap = HashMap<String, NumfmtRefItem>;

/// HashMap for reordering (old index -> new index)
pub type ReorderMap = HashMap<u32, u32>;

// ============================================================================
// Transform Parameter Types (used in OT transforms)
// ============================================================================
// Note: These types are different from the mutation parameter types above.
// They use SubUnitParams with flattened serialization for OT transform operations.

/// Row data for insert operations (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hd: Option<u32>,
}

/// Column data for insert operations (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hd: Option<u32>,
}

/// Set range values mutation parameters (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRangeValuesMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    #[serde(rename = "cellValue", alias = "cell_value")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_value: Option<ObjectMatrixPrimitiveType>,
}

/// Insert row mutation parameters (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertRowMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    pub range: Range,

    #[serde(rename = "rowInfo", alias = "row_info")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_info: Option<Vec<RowData>>,
}

/// Insert column mutation parameters (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertColMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    pub range: Range,

    #[serde(rename = "colInfo", alias = "col_info")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_info: Option<Vec<ColumnData>>,
}

/// Remove rows mutation parameters (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveRowsMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    pub range: Range,
}

/// Remove column mutation parameters (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveColMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    pub range: Range,
}

/// Move rows mutation parameters (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRowsMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    #[serde(rename = "sourceRange", alias = "source_range")]
    pub source_range: Range,

    #[serde(rename = "targetRange", alias = "target_range")]
    pub target_range: Range,
}

/// Move columns mutation parameters (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveColsMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    #[serde(rename = "sourceRange", alias = "source_range")]
    pub source_range: Range,

    #[serde(rename = "targetRange", alias = "target_range")]
    pub target_range: Range,
}

/// Move range mutation parameters (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRangeMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    #[serde(rename = "fromRange", alias = "from_range")]
    pub from_range: Range,

    #[serde(rename = "toRange", alias = "to_range")]
    pub to_range: Range,
}

/// Set frozen mutation parameters (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFrozenMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    #[serde(rename = "startRow", alias = "start_row")]
    pub start_row: i32,

    #[serde(rename = "startColumn", alias = "start_column")]
    pub start_column: i32,

    #[serde(rename = "ySplit", alias = "y_split")]
    pub y_split: i32,

    #[serde(rename = "xSplit", alias = "x_split")]
    pub x_split: i32,
}

/// Insert sheet mutation parameters (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertSheetMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    pub index: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sheet: Option<serde_json::Value>,
}

/// Remove sheet mutation parameters (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveSheetMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
}
