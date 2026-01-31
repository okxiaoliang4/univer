# SetRowDataMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.set-row-data`
**File**: `mutations/sheets/set_row_data_mutation.rs`
**Transform**: `transforms/sheets/row_col_data.rs`
**Tests**: `tests/set_row_data_tests.rs`

## Mutation Overview

**Purpose**: Sets row-level metadata (height, visibility, custom data) for one or more rows.

**Affects**:
- [x] Rows (row metadata)
- [ ] Columns
- [ ] Cells/Ranges
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct SetRowDataMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub row_data: IObjectArrayPrimitiveType<IRowData>,  // HashMap<String, IRowData>
}

pub struct IRowData {
    pub h: Option<f64>,            // Height
    pub ah: Option<f64>,           // Auto height
    pub ia: Option<bool>,          // Is auto height
    pub hd: Option<u8>,            // Hidden (BooleanNumber)
    pub custom: Option<Value>,     // Custom data
}
```

**Data Structure**:
```json
{
  "rowData": {
    "0": { "h": 25.0, "hd": 0 },
    "5": { "h": 40.0, "ia": true },
    "10": { "hd": 1 }
  }
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
- Row indices: Keys in the row_data HashMap

**Conflict scenarios**:
1. **Row insert above**: Row keys shift down
2. **Row remove**: Row keys may be deleted or shifted up
3. **Same row, different data**: LWW at row level

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: N/A

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | Row keys shift (like cell_value) |
| InsertColMutation | ❌ | Identity | Column insert doesn't affect row data |
| RemoveRowsMutation | ✅ | Shifting + Removal | Row keys removed/shifted |
| RemoveColMutation | ❌ | Identity | Column removal doesn't affect row data |
| MoveRowsMutation | ⏳ | Complex | Row data may move |
| MoveColumnsMutation | ❌ | Identity | Column move doesn't affect row data |
| MoveRangeMutation | ⏳ | Complex | May affect rows |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ❌ | Identity | Cell values independent of row metadata |
| SetRowDataMutation (self) | ✅ | LWW (row-level) | Same rows: m2 wins |
| SetColDataMutation | ❌ | Identity | Row vs column metadata |
| ReorderRangeMutation | ⏳ | Complex | May affect rows |

#### Merge/Dimension/Other Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddWorksheetMergeMutation | ❌ | Identity | Merge independent of row metadata |
| SetWorksheetRowHeightMutation | ⏳ | Conflict | Both set row height |
| (Most other mutations) | ❌ | Identity | Independent operations |

## Detailed Conflict Resolution

### InsertRowMutation vs SetRowDataMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Row keys need shifting

**Resolution Strategy**: Shifting (HashMap keys)

**Implementation**:
```rust
fn create_insert_row_vs_row_data() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: SetRowDataMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift row keys (similar to cell_value shifting)
        shift_row_keys_for_insert_generic(&mut m2_params.row_data, insert_start, insert_count);

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}
```

**Status**: ✅ Implemented

---

### SetRowDataMutation vs SetRowDataMutation (Self-Transform)

**Conflict Analysis**:
- **Dimension 3 (Position)**: Different rows = no conflict; Same rows = LWW

**Resolution Strategy**: LWW at row level

**Implementation**:
```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: SetRowDataMutationParams = parse(m1)?;
        let m2_params: SetRowDataMutationParams = parse(m2)?;

        let mut m1_prime_row_data = m1_params.row_data.clone();

        // Remove rows from m1 that m2 also sets (LWW)
        for row_key in m2_params.row_data.keys() {
            m1_prime_row_data.remove(row_key);
        }

        if m1_prime_row_data.is_empty() {
            return TransformResultRef {
                m1_prime: MutationOutcome::Removed,
                m2_prime: MutationOutcome::Unchanged(m2),
                error: None,
            };
        }

        let mut m1_prime_params = m1_params.clone();
        m1_prime_params.row_data = m1_prime_row_data;

        TransformResultRef {
            m1_prime: MutationOutcome::Modified(serialize(m1_prime_params)),
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

**Status**: ✅ Implemented

## Implementation Checklist

- [x] Self-transform (LWW at row level) implemented
- [x] Transform with InsertRowMutation implemented
- [x] Transform with RemoveRowsMutation implemented
- [ ] Transform with MoveRowsMutation implemented
- [x] Tests written for implemented scenarios
- [x] This document updated

## Notes

- SetRowDataMutation uses HashMap<String, IRowData> (row index as string key)
- Similar shifting logic to SetRangeValuesMutation but simpler (only row keys)
- Row-level LWW: conflicting rows go to m2, non-conflicting preserved in m1

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
