# Note Mutations - Batch Analysis

This document covers cell note (comment) mutations.

## Covered Mutations

### 1. UpdateNoteMutation
**ID**: `sheet.mutation.update-note`
**Purpose**: Updates or creates note at cell position
**Strategy**: Position-based + Shifting
**Parameters**: `row, col, note, silent`
**Status**: ✅ Full document exists

### 2. RemoveNoteMutation
**ID**: `sheet.mutation.remove-note`
**Purpose**: Removes note from cell
**Strategy**: Position-based + Shifting
**Parameters**: `row, col, silent`

### 3. ToggleNotePopupMutation
**ID**: `sheet.mutation.toggle-note-popup`
**Purpose**: Shows/hides note popup
**Strategy**: Position-based + Shifting (UI state)
**Parameters**: `row, col, silent`

### 4. UpdateNotePositionMutation
**ID**: `sheet.mutation.update-note-position`
**Purpose**: Changes note popup position
**Strategy**: Position-based + Shifting
**Parameters**: `row, col, new_position, silent`

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sheet_id` (note: uses `sheet_id` not `sub_unit_id`)
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col)
**Positions affected**:
- All mutations: `row, col` fields
- Position needs shifting with structural mutations

### Dimension 4: Feature Plugin ID
**Not applicable** - Position-based identification (row, col)

## Transform Coverage Matrix

All note mutations follow the same pattern since they're all position-based:

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | row field shifts if >= insert position |
| InsertColMutation | ✅ | Shifting | col field shifts if >= insert position |
| RemoveRowsMutation | ✅ | Shifting + Removal | Note removed if row in range |
| RemoveColMutation | ✅ | Shifting + Removal | Note removed if col in range |
| MoveRowsMutation | ⏳ | Complex | Note position may move |
| MoveColumnsMutation | ⏳ | Complex | Note position may move |
| MoveRangeMutation | ⏳ | Complex | Note position may move |

### Self-Transforms

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| UpdateNoteMutation (self) | ✅ | LWW | Same (row,col): m2 wins |
| RemoveNoteMutation (self) | ✅ | Idempotent | Same position: both remove |
| ToggleNotePopupMutation (self) | ✅ | LWW | Same position: m2 wins |
| UpdateNotePositionMutation (self) | ✅ | LWW | Same position: m2 wins |

### Cross-Note Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| UpdateNoteMutation vs RemoveNoteMutation | ⏳ | Conflict | Update vs Remove: Remove wins |
| UpdateNoteMutation vs ToggleNotePopupMutation | ❌ | Identity | Different operations |
| UpdateNoteMutation vs UpdateNotePositionMutation | ❌ | Identity | Different operations |

## Implementation Pattern

All note mutations use the exact same pattern as UpdateNoteMutation:

```rust
fn create_insert_row_vs_note() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Note: uses sheet_id not sub_unit_id
        if !same_sheet_for_note(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: NoteM utationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

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

## Key Notes

1. **All note mutations use `sheet_id`** instead of `sub_unit_id` (naming inconsistency)
2. **All are position-based** - no separate ID system
3. **Multiple notes can exist at same cell** (not actually enforced at OT level)
4. **Removal when cell deleted** - all note mutations removed when row/col removed

## Implementation Checklist

- [x] UpdateNoteMutation fully documented (separate doc)
- [x] All other mutations follow same pattern
- [x] Transforms with structural mutations
- [ ] Cross-note conflict resolution
- [x] Tests written for UpdateNote
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for note mutations
