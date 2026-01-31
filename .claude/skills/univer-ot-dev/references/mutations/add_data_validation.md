# AddDataValidationMutation Conflict Analysis

**Mutation ID**: `data-validation.mutation.addRule`
**File**: `mutations/data_validation/data_validation_mutation.rs`
**Transform**: `transforms/sheets_data_validation/data_validation.rs`
**Tests**: `tests/data_validation_tests.rs`

## Mutation Overview

**Purpose**: Adds a data validation rule to a worksheet, defining constraints for cell input.

**Affects**:
- [ ] Rows
- [ ] Columns
- [x] Cells/Ranges (rule applies to ranges)
- [ ] Worksheet properties
- [ ] Workbook properties
- [x] Feature-specific (data validation)

**Parameters**:
```rust
pub struct AddDataValidationMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub rule: IDataValidationRule, // The validation rule
    pub index: Option<i32>,        // Insert position in rule list
    pub source: Option<String>,    // Source identifier
}

pub struct IDataValidationRule {
    pub uid: String,               // Unique rule ID
    pub ranges: Vec<IRange>,       // Ranges this rule applies to
    pub r#type: ValidationType,    // Validation type
    pub error_style: Option<ErrorStyle>,
    pub operator: Option<DataValidationOperator>,
    pub formula1: Option<String>,
    pub formula2: Option<String>,
    // ... other properties
}
```

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected by this mutation**:
- Ranges: Rule.ranges define where validation applies

**Conflict scenarios**:
1. **Row insert/remove**: Rule ranges need adjustment
2. **Column insert/remove**: Rule ranges need adjustment

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: `uid` (data validation rule ID)

**Conflicts when**: Same `uid` → LWW (rare, usually different IDs)

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | Rule ranges shift rows |
| InsertColMutation | ✅ | Shifting | Rule ranges shift columns |
| RemoveRowsMutation | ✅ | Shifting + Removal | Rule ranges shrink/removed |
| RemoveColMutation | ✅ | Shifting + Removal | Rule ranges shrink/removed |
| MoveRowsMutation | ⏳ | Complex | Rule ranges may be affected |
| MoveColumnsMutation | ⏳ | Complex | Rule ranges may be affected |
| MoveRangeMutation | ⏳ | Complex | Rule ranges may be affected |

#### Data/Merge/Other Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ❌ | Identity | Cell values independent of DV rules |
| AddWorksheetMergeMutation | ❌ | Identity | Merge independent of DV rules |
| (Most other mutations) | ❌ | Identity | Independent operations |

### Data Validation Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddDataValidationMutation (self) | ✅ | Identity (diff ID) / LWW (same ID) | Both rules added |
| RemoveDataValidationMutation | ⏳ | Conflict | Add vs Remove same uid |
| UpdateDataValidationMutation | ⏳ | Conflict | Add vs Update same uid |

## Detailed Conflict Resolution

### InsertRowMutation vs AddDataValidationMutation

**Resolution Strategy**: Shifting

**Implementation**: Similar to conditional formatting - shift rule.ranges

**Status**: ✅ Implemented

---

### AddDataValidationMutation vs AddDataValidationMutation (Self-Transform)

**Conflict Analysis**:
- **Dimension 4 (Feature ID)**: Different uid = identity; Same uid = LWW

**Resolution Strategy**: Identity (different IDs) / LWW (same ID)

**Status**: ✅ Implemented

## Implementation Checklist

- [x] Self-transform implemented
- [x] Transforms with structural mutations implemented
- [x] Tests written for implemented scenarios
- [x] This document updated

## Notes

- Data validation rules have unique `uid` identifiers
- Multiple rules can exist per worksheet
- Rules with different UIDs are independent

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
