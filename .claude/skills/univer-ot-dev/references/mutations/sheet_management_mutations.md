# Sheet Management Mutations - Batch Analysis

This document covers mutations that operate at the sheet level (different worksheets).

## Covered Mutations

### 1. InsertSheetMutation
**ID**: `sheet.mutation.insert-sheet`
**Purpose**: Inserts new worksheet into workbook
**Strategy**: Identity with other worksheet mutations (different sub_unit_id)
**Parameters**: `unit_id, sheet: IWorksheetData, index, styles`

### 2. RemoveSheetMutation
**ID**: `sheet.mutation.remove-sheet`
**Purpose**: Removes worksheet from workbook
**Strategy**: Identity with other worksheet mutations (different sub_unit_id)
**Parameters**: `unit_id, sub_unit_id, sub_unit_name`

### 3. CopyWorksheetEndMutation
**ID**: `sheet.mutation.copy-worksheet-end`
**Purpose**: Copies worksheet to end of workbook
**Strategy**: Identity with other worksheet mutations (different sub_unit_id)
**Parameters**: `unit_id, sub_unit_id`

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Key Point**: These mutations operate on DIFFERENT worksheets
- **Conflicts when**: Rarely - only when managing the same worksheet
- **No conflict when**: Almost always → Identity transform

### Dimension 3: Position
**Not applicable** - Sheet-level operations

### Dimension 4: Feature Plugin ID
**Not applicable**

## Transform Coverage Matrix

All sheet management mutations are **identity** with worksheet-level operations because they create/remove/copy entire worksheets (different sub_unit_id).

### With ALL Worksheet Operations

| Mutation Type | Status | Strategy | Notes |
|---------------|--------|----------|-------|
| All structural mutations (insert/remove rows/cols) | ❌ | Identity | Different sub_unit_id |
| All data mutations | ❌ | Identity | Different sub_unit_id |
| All feature plugins | ❌ | Identity | Different sub_unit_id |
| All worksheet properties | ❌ | Identity | Different sub_unit_id |
| Self | ⏳ | Complex | Same worksheet operations |

## Self-Transform Logic

### InsertSheetMutation vs InsertSheetMutation
- **Same index**: Order matters (first shifts second)
- **Different index**: Both apply with index adjustment
- **Strategy**: Index-based shifting

### RemoveSheetMutation vs RemoveSheetMutation
- **Same sub_unit_id**: Idempotent (both remove same sheet)
- **Different sub_unit_id**: Independent
- **Strategy**: Identity or idempotent

### InsertSheetMutation vs RemoveSheetMutation
- **Complex**: Insert may shift remove target
- **Strategy**: Index-based adjustment

## Implementation Pattern

```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Different workbooks: identity
        if m1_params.unit_id != m2_params.unit_id {
            return identity(m1, m2);
        }

        // Sheet management logic based on index/sub_unit_id
        // Most worksheet operations will see identity because
        // they have different sub_unit_id

        // ... specific sheet management logic
    })
}
```

## Key Insight

**99% of transforms are identity** because:
1. Sheet management creates/removes entire worksheets
2. Other mutations operate WITHIN a worksheet (specific sub_unit_id)
3. Different sub_unit_id → No conflict

Only sheet management vs sheet management has non-trivial transforms.

## Implementation Checklist

- [ ] Self-transforms (index-based logic)
- [ ] Cross-sheet-management transforms
- [x] Identity with all worksheet operations (by design)
- [ ] Tests written
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for sheet management mutations
