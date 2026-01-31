# AddCommentMutation Conflict Analysis

**Mutation ID**: `thread-comment.mutation.add-comment`
**File**: `mutations/thread_comment/comment_mutation.rs`
**Transform**: `transforms/thread_comment/comment.rs`
**Tests**: `tests/thread_comment_tests.rs`

## Mutation Overview

**Purpose**: Adds a new thread comment to a worksheet at a specific cell position.

**Affects**:
- [ ] Rows
- [ ] Columns
- [x] Cells/Ranges (comment attached to specific cell)
- [ ] Worksheet properties
- [ ] Workbook properties
- [x] Feature-specific (thread comments)

**Parameters**:
```rust
pub struct AddCommentMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub comment: IThreadComment,   // Comment data with position
    pub sync: Option<bool>,        // Sync flag
}

pub struct IThreadComment {
    pub id: String,                // Comment thread ID
    pub ref: ICommentRef,          // Reference (position)
    pub text: CommentContent,      // Comment content
    pub dt: String,                // Timestamp
    pub person_id: Option<String>, // Author
    // ... other fields
}

pub struct ICommentRef {
    pub row: i32,                  // Row position
    pub col: i32,                  // Column position
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
- Single cell: comment.ref (row, col)

**Conflict scenarios**:
1. **Row insert above**: Comment row position shifts down
2. **Row remove**: Comment may be deleted if in removed rows
3. **Column insert left**: Comment column position shifts right
4. **Column remove**: Comment may be deleted if in removed columns

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: `comment.id` (thread ID)

**Conflicts when**: Same `id` → LWW (rare, usually different IDs)

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | comment.ref.row shifts if >= insert position |
| InsertColMutation | ✅ | Shifting | comment.ref.col shifts if >= insert position |
| RemoveRowsMutation | ✅ | Shifting + Removal | Comment removed if row in range |
| RemoveColMutation | ✅ | Shifting + Removal | Comment removed if col in range |
| MoveRowsMutation | ⏳ | Complex | Comment position may move |
| MoveColumnsMutation | ⏳ | Complex | Comment position may move |
| MoveRangeMutation | ⏳ | Complex | Comment position may move |

#### Data/Merge/Other Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ❌ | Identity | Cell values independent of comments |
| AddWorksheetMergeMutation | ❌ | Identity | Merge independent of comments |
| (Most other mutations) | ❌ | Identity | Independent operations |

### Thread Comment Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddCommentMutation (self) | ✅ | Identity (diff ID) / LWW (same ID) | Both comments added |
| UpdateCommentMutation | ⏳ | Conflict | Add vs Update same id |
| UpdateCommentRefMutation | ⏳ | Complex | Position change |
| ResolveCommentMutation | ❌ | Identity | Status update, not content |
| DeleteCommentMutation | ⏳ | Conflict | Add vs Delete same id |

## Detailed Conflict Resolution

### InsertRowMutation vs AddCommentMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Comment row position needs shifting

**Resolution Strategy**: Shifting (single field in nested struct)

**Implementation**:
```rust
fn create_insert_row_vs_comment() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: AddCommentMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift comment row if at or below insert position
        if m2_params.comment.ref.row >= insert_start {
            m2_params.comment.ref.row += insert_count;
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

### RemoveRowsMutation vs AddCommentMutation

**Conflict Analysis**:
- Comment in removed row should be removed

**Resolution Strategy**: Shifting + Removal

**Implementation**:
```rust
fn create_remove_row_vs_comment() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: RemoveRowsMutationParams = parse(m1)?;
        let mut m2_params: AddCommentMutationParams = parse(m2)?;

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;

        // Check if comment is in removed range
        if m2_params.comment.ref.row >= remove_start
           && m2_params.comment.ref.row <= remove_end {
            return TransformResultRef {
                m1_prime: MutationOutcome::Unchanged(m1),
                m2_prime: MutationOutcome::Removed,
                error: None,
            };
        }

        // Shift if below removed range
        if m2_params.comment.ref.row > remove_end {
            let remove_count = remove_end - remove_start + 1;
            m2_params.comment.ref.row -= remove_count;
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

### AddCommentMutation vs AddCommentMutation (Self-Transform)

**Conflict Analysis**:
- **Dimension 4 (Feature ID)**: Different comment.id = identity; Same id = LWW

**Resolution Strategy**: Identity (different IDs) / LWW (same ID)

**Implementation**:
```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: AddCommentMutationParams = parse(m1)?;
        let m2_params: AddCommentMutationParams = parse(m2)?;

        // Different comment IDs: both add (identity)
        if m1_params.comment.id != m2_params.comment.id {
            return identity(m1, m2);
        }

        // Same ID: m2 wins (LWW)
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

- [x] Self-transform (Identity/LWW) implemented
- [x] Transform with InsertRowMutation implemented
- [x] Transform with InsertColMutation implemented
- [x] Transform with RemoveRowsMutation implemented
- [x] Transform with RemoveColMutation implemented
- [ ] Transform with Move mutations implemented
- [ ] Transform with other comment mutations
- [x] Tests written for implemented scenarios
- [x] This document updated

## Notes

- Comments use nested `comment.ref` structure for position
- Comments have unique `id` for thread identification
- Multiple comments can exist at the same cell position (same cell, different threads)
- Position-based removal when cell is deleted

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
