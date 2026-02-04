//! Generic parameter types for unified OT transformation
//!
//! These types provide type-safe parsing of mutation parameters while preserving
//! additional fields through `#[serde(flatten)]`.
//!
//! # Design
//!
//! Each type embeds `SubUnitParams` for location context and uses `#[serde(flatten)]`
//! to preserve any additional fields not explicitly defined.
//!
//! # Example
//!
//! ```ignore
//! let params: GenericRangesParams = serde_json::from_value(json!({
//!     "unitId": "unit1",
//!     "subUnitId": "sheet1",
//!     "ranges": [{"startRow": 0, "endRow": 5, "startColumn": 0, "endColumn": 5}],
//!     "mergeType": 1  // preserved in `other`
//! }))?;
//! ```

use serde::{Deserialize, Serialize};
use crate::types::{IRange, IObjectMatrixPrimitiveType, SubUnitParams};

// ============================================================================
// Generic Parameter Types
// ============================================================================

/// Generic parameters for mutations with a single `range` field
///
/// Used by: InsertRow, InsertCol, RemoveRow, RemoveCol, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericRangeParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    pub range: IRange,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

/// Generic parameters for mutations with a `ranges` field (Vec<IRange>)
///
/// Used by: AddWorksheetMerge, SetRangeProtection, SetRangeTheme, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericRangesParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    pub ranges: Vec<IRange>,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

/// Generic parameters for mutations with a `cellValue` field
///
/// Used by: SetRangeValues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericCellValueParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    #[serde(rename = "cellValue")]
    pub cell_value: Option<IObjectMatrixPrimitiveType>,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

/// Generic parameters for mutations with a `rowData` field
///
/// Used by: SetRowData, SetWorksheetRowHeight (batch mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericRowDataParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    #[serde(rename = "rowData")]
    pub row_data: std::collections::HashMap<String, serde_json::Value>,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

/// Generic parameters for mutations with a `columnData` field
///
/// Used by: SetColData, SetWorksheetColWidth (batch mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericColDataParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    #[serde(rename = "columnData")]
    pub column_data: std::collections::HashMap<String, serde_json::Value>,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

/// Generic parameters for mutations with `row` and `col` fields
///
/// Used by: UpdateNote, RemoveNote, ToggleNotePopup, UpdateHyperLinkRef, etc.
/// Supports both `subUnitId`/`sheetId` naming and `col`/`column` naming conventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericRowColParams {
    pub unit_id: String,
    #[serde(alias = "subUnitId", alias = "sheetId")]
    pub sub_unit_id: String,
    pub row: i32,
    #[serde(alias = "column")]
    pub col: i32,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

/// Generic parameters for mutations with only a `col` field (column index)
///
/// Used by: SetSheetsFilterCriteria, etc.
/// Only affected by column operations (InsertCols, RemoveCols, MoveCols).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericColParams {
    pub unit_id: String,
    #[serde(alias = "subUnitId", alias = "sheetId")]
    pub sub_unit_id: String,
    #[serde(alias = "column")]
    pub col: i32,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

/// Generic parameters for mutations with only a `row` field (row index)
///
/// Used by mutations that reference a single row position.
/// Only affected by row operations (InsertRows, RemoveRows, MoveRows).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericRowParams {
    pub unit_id: String,
    #[serde(alias = "subUnitId", alias = "sheetId")]
    pub sub_unit_id: String,
    pub row: i32,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

// ============================================================================
// HasLocation Trait - for types that have unit/subunit location
// ============================================================================

/// Trait for types that have location context (unit_id, sub_unit_id)
pub trait WorksheetParams {
    fn unit_id(&self) -> &str;
    fn sub_unit_id(&self) -> &str;

    /// Check if this location matches another
    fn same_sheet(&self, other: &impl WorksheetParams) -> bool {
        self.unit_id() == other.unit_id() && self.sub_unit_id() == other.sub_unit_id()
    }
}

impl WorksheetParams for SubUnitParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl WorksheetParams for GenericRangeParams {
    fn unit_id(&self) -> &str { &self.sub_unit_params.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_params.sub_unit_id }
}

impl WorksheetParams for GenericRangesParams {
    fn unit_id(&self) -> &str { &self.sub_unit_params.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_params.sub_unit_id }
}

impl WorksheetParams for GenericCellValueParams {
    fn unit_id(&self) -> &str { &self.sub_unit_params.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_params.sub_unit_id }
}

impl WorksheetParams for GenericRowDataParams {
    fn unit_id(&self) -> &str { &self.sub_unit_params.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_params.sub_unit_id }
}

impl WorksheetParams for GenericColDataParams {
    fn unit_id(&self) -> &str { &self.sub_unit_params.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_params.sub_unit_id }
}

impl WorksheetParams for GenericRowColParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl WorksheetParams for GenericColParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl WorksheetParams for GenericRowParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generic_ranges_params_parse() {
        let json_val = json!({
            "unitId": "unit1",
            "subUnitId": "sheet1",
            "ranges": [
                {"startRow": 0, "endRow": 5, "startColumn": 0, "endColumn": 5}
            ],
            "mergeType": 1
        });

        let params: GenericRangesParams = serde_json::from_value(json_val).unwrap();

        assert_eq!(params.sub_unit_params.unit_id, "unit1");
        assert_eq!(params.sub_unit_params.sub_unit_id, "sheet1");
        assert_eq!(params.ranges.len(), 1);
        assert_eq!(params.ranges[0].start_row, 0);
        assert!(params.other.contains_key("mergeType"));
    }

    #[test]
    fn test_generic_cell_value_params_parse() {
        let json_val = json!({
            "unitId": "unit1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": {
                    "0": {"v": "hello"}
                }
            }
        });

        let params: GenericCellValueParams = serde_json::from_value(json_val).unwrap();

        assert_eq!(params.sub_unit_params.unit_id, "unit1");
        assert!(params.cell_value.is_some());
    }

    #[test]
    fn test_generic_row_data_params_parse() {
        let json_val = json!({
            "unitId": "unit1",
            "subUnitId": "sheet1",
            "rowData": {
                "0": {"h": 25},
                "5": {"h": 30}
            }
        });

        let params: GenericRowDataParams = serde_json::from_value(json_val).unwrap();

        assert_eq!(params.sub_unit_params.unit_id, "unit1");
        assert!(params.row_data.contains_key("0"));
        assert!(params.row_data.contains_key("5"));
    }

    #[test]
    fn test_flatten_preserves_other_fields() {
        let json_val = json!({
            "unitId": "unit1",
            "subUnitId": "sheet1",
            "ranges": [],
            "customField": "value",
            "anotherField": 123
        });

        let params: GenericRangesParams = serde_json::from_value(json_val.clone()).unwrap();

        // Serialize back and check preserved fields
        let serialized = serde_json::to_value(&params).unwrap();
        assert_eq!(serialized["customField"], "value");
        assert_eq!(serialized["anotherField"], 123);
    }

    #[test]
    fn test_has_location_same_location() {
        let params1 = GenericRangesParams {
            sub_unit_params: SubUnitParams {
                unit_id: "unit1".to_string(),
                sub_unit_id: "sheet1".to_string(),
            },
            ranges: vec![],
            other: serde_json::Map::new(),
        };

        let params2 = GenericCellValueParams {
            sub_unit_params: SubUnitParams {
                unit_id: "unit1".to_string(),
                sub_unit_id: "sheet1".to_string(),
            },
            cell_value: None,
            other: serde_json::Map::new(),
        };

        let params3 = GenericRangesParams {
            sub_unit_params: SubUnitParams {
                unit_id: "unit1".to_string(),
                sub_unit_id: "sheet2".to_string(), // different sheet
            },
            ranges: vec![],
            other: serde_json::Map::new(),
        };

        assert!(params1.same_sheet(&params2));
        assert!(!params1.same_sheet(&params3));
    }
}
