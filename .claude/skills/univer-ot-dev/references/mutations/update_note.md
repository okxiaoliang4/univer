# UpdateNoteMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.update-note`
**File**: `mutations/sheets_note/note_mutation.rs`
**Transform**: `transforms/sheets_note/note.rs`
**Tests**: `tests/note_tests.rs`

## Mutation Overview

**Purpose**: Updates or creates a note (comment) at a specific cell position.

**Affects**:
- [ ] Rows
- [ ] Columns
- [x] Cells/Ranges (note attached to specific cell)
- [ ] Worksheet properties
- [ ] Workbook properties
- [x] Feature-specific (notes)

**Parameters**:
```rust
pub struct UpdateNoteMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sheet_id: String,          // Worksheet ID (note: named sheet_id, not sub_unit_id)
    pub row: i32,                  // Row position
    pub col: i32,                  // Column position
    pub note: INoteData,           // Note content
    pub silent: Option<bool>,      // Silent update flag
}
```

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sheet_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected by this mutation**:
- Single cell: (row, col)

**Conflict scenarios**:
1. **Row insert above**: Note row position shifts down
2. **Row remove**: Note may be deleted if in removed rows
3. **Column insert left**: Note column position shifts right
4. **Column remove**: Note may be deleted if in removed columns

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: Position-based (row, col) - note at specific cell

**Conflicts when**: Same (row, col) position → LWW

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | row field shifts if >= insert position |
| InsertColMutation | ✅ | Shifting | col field shifts if >= insert position |
| RemoveRowsMutation | ✅ | Shifting + Removal | Note removed if row in range |
| RemoveColMutation | ✅ | Shifting + Removal | Note removed if col in range |
| MoveRowsMutation | ⏳ | Complex | Note position may move |
| MoveColumnsMutation | ⏳ | Complex | Note position may move |
| MoveRangeMutation | ⏳ | Complex | Note position may move |

#### Data/Merge/Other Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ❌ | Identity | Cell values independent of notes |
| AddWorksheetMergeMutation | ❌ | Identity | Merge independent of notes |
| (Most other mutations) | ❌ | Identity | Independent operations |

### Note Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| UpdateNoteMutation (self) | ✅ | LWW | Same cell: m2 wins |
| RemoveNoteMutation | ⏳ | Conflict | Update vs Remove - Update wins |
| ToggleNotePopupMutation | ❌ | Identity | Visibility, not content |
| UpdateNotePositionMutation | ⏳ | Complex | Position change |

## Detailed Conflict Resolution

### InsertRowMutation vs UpdateNoteMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Note row position needs shifting

**Resolution Strategy**: Shifting (single field)

**Implementation**:
```rust
fn create_insert_row_vs_note() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Early worksheet check (note uses sheet_id)
        if !same_sheet_for_note(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: UpdateNoteMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift note row if at or below insert position
        if m2_params.row >= insert_start {
            m2_params.row += insert_count;
        }

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

### RemoveRowsMutation vs UpdateNoteMutation

**Conflict Analysis**:
- Note in removed row should be removed

**Resolution Strategy**: Shifting + Removal

**Implementation**:
```rust
fn create_remove_row_vs_note() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if !same_sheet_for_note(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: RemoveRowsMutationParams = parse(m1)?;
        let mut m2_params: UpdateNoteMutationParams = parse(m2)?;

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;

        // Check if note is in removed range
        if m2_params.row >= remove_start && m2_params.row <= remove_end {
            return TransformResultRef {
                m1_prime: MutationOutcome::Unchanged(m1),
                m2_prime: MutationOutcome::Removed,
                error: None,
            };
        }

        // Shift if below removed range
        if m2_params.row > remove_end {
            let remove_count = remove_end - remove_start + 1;
            m2_params.row -= remove_count;
        }

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

### UpdateNoteMutation vs UpdateNoteMutation (Self-Transform)

**Conflict Analysis**:
- **Dimension 3 (Position)**: Same (row, col) = conflict

**Resolution Strategy**: LWW at position level

**Implementation**:
```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if !same_sheet_for_note(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: UpdateNoteMutationParams = parse(m1)?;
        let m2_params: UpdateNoteMutationParams = parse(m2)?;

        // Different cell positions: both apply
        if m1_params.row != m2_params.row || m1_params.col != m2_params.col {
            return identity(m1, m2);
        }

        // Same cell: m2 wins (LWW)
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

**Status**: ✅ Implemented

## Implementation Checklist

- [x] Self-transform (LWW at position) implemented
- [x] Transform with InsertRowMutation implemented
- [x] Transform with InsertColMutation implemented
- [x] Transform with RemoveRowsMutation implemented
- [x] Transform with RemoveColMutation implemented
- [ ] Transform with Move mutations implemented
- [x] Tests written for implemented scenarios
- [x] This document updated

## Notes

- Notes use `sheet_id` instead of `sub_unit_id` (different naming)
- Notes are position-based (row, col), not ID-based
- Single position = simpler than range-based mutations
- Need to check if position is in removed range

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
