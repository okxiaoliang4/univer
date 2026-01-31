# Thread Comment Mutations - Batch Analysis

This document covers thread comment mutations (beyond AddCommentMutation which has its own doc).

## Covered Mutations

### 1. AddCommentMutation
**Status**: ✅ Full document exists ([add_comment.md](add_comment.md))

### 2. UpdateCommentMutation
**ID**: `thread-comment.mutation.update-comment`
**Purpose**: Updates comment content by ID
**Strategy**: ID-based (mostly identity with position ops)
**Parameters**: `comment_id, payload, silent`

### 3. UpdateCommentRefMutation
**ID**: `thread-comment.mutation.update-comment-ref`
**Purpose**: Updates comment position reference
**Strategy**: Position-based + Shifting
**Parameters**: `comment_id, payload` (with row, col)

### 4. ResolveCommentMutation
**ID**: `thread-comment.mutation.resolve-comment`
**Purpose**: Marks comment as resolved/unresolved
**Strategy**: ID-based + LWW
**Parameters**: `comment_id, resolved`

### 5. DeleteCommentMutation
**ID**: `thread-comment.mutation.delete-comment`
**Purpose**: Removes comment thread by ID
**Strategy**: ID-based (mostly identity)
**Parameters**: `comment_id`

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col)
**Positions affected**:
- Add: comment.ref (row, col)
- UpdateRef: payload (row, col)
- Update/Resolve/Delete: No position in params

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: `comment.id` or `comment_id`
- Different IDs → Independent
- Same ID → LWW or conflict

## Transform Coverage Matrix

### Position-Based (Add, UpdateRef)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | row field shifts (see add_comment.md) |
| InsertColMutation | ✅ | Shifting | col field shifts |
| RemoveRowsMutation | ✅ | Shifting + Removal | Comment removed if in range |
| RemoveColMutation | ✅ | Shifting + Removal | Comment removed if in range |
| MoveRowsMutation | ⏳ | Complex | Comment position may move |
| MoveColumnsMutation | ⏳ | Complex | Comment position may move |
| MoveRangeMutation | ⏳ | Complex | Comment position may move |

### ID-Based (Update, Resolve, Delete)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| All structural mutations | ❌ | Identity | No position in params |

### Self-Transforms

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddCommentMutation (self) | ✅ | Identity (diff ID) / LWW (same ID) | See main doc |
| UpdateCommentMutation (self) | ✅ | LWW | Same ID: m2 wins |
| UpdateCommentRefMutation (self) | ✅ | LWW | Same ID: m2 wins |
| ResolveCommentMutation (self) | ✅ | LWW | Same ID: m2 wins |
| DeleteCommentMutation (self) | ✅ | Idempotent | Same ID: both delete |

### Cross-Comment Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddCommentMutation vs UpdateCommentMutation | ❌ | Identity | Different operations on same ID |
| AddCommentMutation vs UpdateCommentRefMutation | ⏳ | Complex | Add vs position change |
| AddCommentMutation vs ResolveCommentMutation | ❌ | Identity | Different operations |
| AddCommentMutation vs DeleteCommentMutation | ⏳ | Conflict | Add vs Delete same ID |
| UpdateCommentMutation vs DeleteCommentMutation | ⏳ | Conflict | Update vs Delete: Delete wins |
| UpdateCommentRefMutation vs DeleteCommentMutation | ⏳ | Conflict | UpdateRef vs Delete: Delete wins |
| ResolveCommentMutation vs DeleteCommentMutation | ⏳ | Conflict | Resolve vs Delete: Delete wins |

## Implementation Pattern

### Position-Based (UpdateRef)
```rust
// Similar to AddCommentMutation
fn create_insert_row_vs_update_comment_ref() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: UpdateCommentRefMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift comment ref row
        if let Some(ref mut row) = m2_params.payload.row {
            if *row >= insert_start {
                *row += insert_count;
            }
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}
```

### ID-Based (Update, Resolve, Delete)
```rust
// Mostly identity with structural mutations
fn create_insert_row_vs_update_comment() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        identity(m1, m2)  // No position in Update params
    })
}

// Self-transform: LWW
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params = parse(m1)?;
        let m2_params = parse(m2)?;

        // Different comment IDs: independent
        if m1_params.comment_id != m2_params.comment_id {
            return identity(m1, m2);
        }

        // Same ID: m2 wins (LWW) or idempotent (Delete)
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

## Key Insights

1. **ID-based system**: Comments identified by unique `comment.id`
2. **Position for Add/UpdateRef**: Most mutations don't have position
3. **Rich operations**: Update content, update position, resolve, delete
4. **LWW for updates**: Same ID = last write wins
5. **Delete wins**: Delete always wins over other operations

## Implementation Checklist

- [x] AddCommentMutation fully documented
- [ ] UpdateCommentMutation transforms
- [ ] UpdateCommentRefMutation transforms
- [ ] ResolveCommentMutation transforms
- [ ] DeleteCommentMutation transforms
- [ ] Cross-comment conflict resolution
- [ ] Tests written
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for thread comment mutations
