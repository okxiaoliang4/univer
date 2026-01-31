# [MutationName] Conflict Analysis

**Mutation ID**: `[type].mutation.[kebab-case-name]`
**File**: `mutations/[module]/[name]_mutation.rs`
**Transform**: `transforms/[module]/[name].rs`
**Tests**: `tests/[name]_tests.rs`

## Mutation Overview

**Purpose**: [Brief description of what this mutation does]

**Affects**:
- [ ] Rows
- [ ] Columns
- [ ] Cells/Ranges
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific (specify: ____________)

**Parameters**:
```rust
pub struct [MutationName]Params {
    pub unit_id: String,
    pub sub_unit_id: String,
    // ... list all parameters with brief descriptions
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
- Rows: [specify range or all]
- Columns: [specify range or all]
- Cells: [specify which cells are modified]

**Conflict scenarios**:
1. **Overlapping ranges**: [describe what happens]
2. **Adjacent ranges**: [describe what happens]
3. **Separated ranges**: Identity (no interaction)

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: [e.g., ruleId, filterId, pivotTableId, or N/A]

**Conflicts when**: [specify conditions, or N/A if not applicable]

## Transform Coverage Matrix

**Status Legend**:
- ✅ Implemented and tested
- 🚧 Implementation in progress
- ⏳ Planned
- ❌ Not needed (identity)

### Core Sheets Mutations (53 total)

#### Structural Mutations (Insert/Remove)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅/⏳ | Shifting | [Brief explanation] |
| InsertColMutation | ✅/⏳ | Shifting | [Brief explanation] |
| RemoveRowsMutation | ✅/⏳ | Shifting + Removal | [Brief explanation] |
| RemoveColMutation | ✅/⏳ | Shifting + Removal | [Brief explanation] |
| MoveRowsMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| MoveColumnsMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| MoveRangeMutation | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ✅/⏳ | LWW (cell-level) | [Brief explanation] |
| SetRowDataMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| SetColDataMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| ReorderRangeMutation | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Merge Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddWorksheetMergeMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| RemoveWorksheetMergeMutation | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Dimension Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetRowHeightMutation | ✅/⏳ | Conflict Scope | [Brief explanation] |
| SetWorksheetColWidthMutation | ✅/⏳ | Conflict Scope | [Brief explanation] |
| MarkDirtyAutoHeightMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| CancelMarkDirtyAutoHeightMutation | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Visibility Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRowVisibleMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| SetColVisibleMutation | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Protection Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddRangeProtectionMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| DeleteRangeProtectionMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| SetRangeProtectionMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| AddWorksheetProtectionMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| DeleteWorksheetProtectionMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| SetWorksheetProtectionMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| SetWorksheetPermissionPointsMutation | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Theme/Style Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddRangeThemeMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| RemoveRangeThemeMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| SetRangeThemeMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| RegisterRangeThemeMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| UnregisterRangeThemeStyleMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| AddWorksheetRangeThemeMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| DeleteWorksheetRangeThemeMutation | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Worksheet Configuration

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetFrozenMutation | ✅/⏳ | LWW | [Brief explanation] |
| SetGridlinesColorMutation | ✅/⏳ | LWW | [Brief explanation] |
| ToggleGridlinesMutation | ✅/⏳ | LWW | [Brief explanation] |
| SetTabColorMutation | ✅/⏳ | LWW | [Brief explanation] |
| SetWorksheetHideMutation | ✅/⏳ | LWW | [Brief explanation] |
| SetWorksheetNameMutation | ✅/⏳ | LWW | [Brief explanation] |
| SetWorksheetOrderMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| SetWorksheetRightToLeftMutation | ✅/⏳ | LWW | [Brief explanation] |
| SetWorksheetDefaultStyleMutation | ✅/⏳ | LWW | [Brief explanation] |

#### Sheet Management

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertSheetMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| RemoveSheetMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| CopyWorksheetEndMutation | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Worksheet Size

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetColumnCountMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| SetWorksheetRowCountMutation | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Workbook

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorkbookNameMutation | ❌ | Identity | Different scope (workbook level) |

#### Number Format

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetNumfmtMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| RemoveNumfmtMutation | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Other

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| EmptyMutation | ❌ | Identity | No-op mutation |

### Feature Plugin Mutations

#### Conditional Formatting

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddConditionalFormattingRuleMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| RemoveConditionalFormattingRuleMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| SetConditionalFormattingRuleMutation | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Data Validation

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| [DataValidationMutation] | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Filter

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetFilterRangeMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| RemoveFilterRangeMutation | ✅/⏳ | [Strategy] | [Brief explanation] |
| SetSheetsFilterCriteriaMutation | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Hyperlink

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| [HyperlinkMutations] | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Note

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| [NoteMutations] | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Pivot Table

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| [PivotTableMutations] | ✅/⏳ | [Strategy] | [Brief explanation] |

#### Table

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| [TableMutations] | ✅/⏳ | [Strategy] | [Brief explanation] |

## Detailed Conflict Resolution

### [Mutation1] vs ThisMutation

**Conflict Analysis**:
- **Dimension 1 (unitId)**: [Analysis]
- **Dimension 2 (subUnitId)**: [Analysis]
- **Dimension 3 (Position)**: [Detailed analysis of how positions conflict]
- **Dimension 4 (Feature ID)**: [Analysis if applicable]

**Resolution Strategy**: [Identity/Shifting/LWW/Conflict Scope/Removal]

**Implementation**:
```rust
fn create_transform() -> TransformFnRef {
    Arc::new(|m1, m2| {
        // Brief pseudocode or actual implementation
    })
}
```

**Test Cases**:
- [ ] Different worksheets (identity)
- [ ] Same worksheet, no position conflict
- [ ] Same worksheet, overlapping positions
- [ ] Edge case: [describe]

**Status**: ✅ Implemented | 🚧 In Progress | ⏳ Planned

---

[Repeat for each mutation that requires non-identity transform]

## Implementation Checklist

- [ ] All transforms registered in `transforms/[module]/[name].rs`
- [ ] Self-transform implemented (if needed)
- [ ] All structural mutation transforms (insert/remove) implemented
- [ ] All data mutation transforms implemented
- [ ] All feature plugin transforms implemented
- [ ] Tests written for all scenarios
- [ ] Zero-copy optimizations applied
- [ ] This document updated with implementation status

## Notes

[Any additional notes, edge cases, or future considerations]

## Last Updated

**Date**: [YYYY-MM-DD]
**By**: [Developer name]
**Changes**: [Brief description of what was updated]
