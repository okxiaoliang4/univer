# Hyperlink Mutations - Batch Analysis

This document covers sheet hyperlink mutations (cell-attached links).

## Covered Mutations

### 1. AddHyperLinkMutation
**ID**: `sheets.mutation.add-hyper-link`
**Purpose**: Adds hyperlink at cell position
**Strategy**: Position-based + ID-based + Shifting
**Parameters**: `link: IHyperLink` with `row, col, id, url`

### 2. RemoveHyperLinkMutation
**ID**: `sheets.mutation.remove-hyper-link`
**Purpose**: Removes hyperlink by ID
**Strategy**: ID-based (mostly identity)
**Parameters**: `id: String`

### 3. UpdateHyperLinkMutation
**ID**: `sheets.mutation.update-hyper-link`
**Purpose**: Updates hyperlink content by ID
**Strategy**: ID-based (mostly identity)
**Parameters**: `id, payload`

### 4. UpdateHyperLinkRefMutation
**ID**: `sheets.mutation.update-hyper-link-ref`
**Purpose**: Updates hyperlink position
**Strategy**: Position-based + Shifting
**Parameters**: `id, row, column, silent`

### 5. UpdateRichHyperLinkMutation
**ID**: `sheets.mutation.update-rich-hyper-link`
**Purpose**: Updates rich hyperlink in cell
**Strategy**: Position-based + Shifting
**Parameters**: `row, col, id, url`

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col)
**Positions affected**:
- Add/Update mutations: `row, col` fields
- Position needs shifting with structural mutations

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: `id` (hyperlink ID)
- Different IDs → Independent
- Same ID → LWW or conflict

## Transform Coverage Matrix

### Position-Based Mutations (Add, UpdateRef, UpdateRich)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | row field shifts if >= insert position |
| InsertColMutation | ✅ | Shifting | col field shifts if >= insert position |
| RemoveRowsMutation | ✅ | Shifting + Removal | Link removed if row in range |
| RemoveColMutation | ✅ | Shifting + Removal | Link removed if col in range |
| MoveRowsMutation | ⏳ | Complex | Link position may move |
| MoveColumnsMutation | ⏳ | Complex | Link position may move |
| MoveRangeMutation | ⏳ | Complex | Link position may move |

### ID-Based Mutations (Remove, Update)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| All structural mutations | ❌ | Identity | No position in params |
| AddHyperLinkMutation | ⏳ | Conflict | Add vs Remove/Update same ID |

### Self-Transforms

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddHyperLinkMutation (self) | ✅ | Identity (diff ID) / LWW (same ID) | By link.id |
| RemoveHyperLinkMutation (self) | ✅ | Identity or Idempotent | By id |
| UpdateHyperLinkMutation (self) | ✅ | LWW | Same ID: m2 wins |
| UpdateHyperLinkRefMutation (self) | ✅ | LWW | Same ID: m2 wins |

## Implementation Pattern

### Position-Based (Add, UpdateRef, UpdateRich)
```rust
// Similar to UpdateNoteMutation
fn create_insert_row_vs_hyperlink() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: AddHyperLinkMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift hyperlink row
        if m2_params.link.row >= insert_start {
            m2_params.link.row += insert_count;
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}
```

### ID-Based (Remove, Update)
```rust
// Mostly identity with structural mutations
fn create_insert_row_vs_remove_hyperlink() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        identity(m1, m2)  // No position in RemoveHyperLink params
    })
}
```

## Implementation Checklist

- [x] Position-based transforms with structural mutations
- [x] ID-based self-transforms (LWW/Identity)
- [ ] Add vs Remove/Update conflict resolution
- [ ] Tests written
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for hyperlink mutations
