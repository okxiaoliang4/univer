# Worksheet Property Mutations - Batch Analysis

This document covers simple worksheet-level property mutations that follow the same pattern: **Last-Write-Wins (LWW)** at worksheet scope.

## Covered Mutations

All mutations in this document are worksheet-level singletons - only one value per worksheet. Same worksheet = LWW (m2 wins).

### 1. SetGridlinesColorMutation
**ID**: `sheet.mutation.set-gridlines-color`
**Purpose**: Sets gridlines color for worksheet
**Strategy**: LWW at worksheet level

### 2. ToggleGridlinesMutation
**ID**: `sheet.mutation.toggle-gridlines`
**Purpose**: Shows/hides gridlines
**Strategy**: LWW at worksheet level

### 3. SetTabColorMutation
**ID**: `sheet.mutation.set-tab-color`
**Purpose**: Sets worksheet tab color
**Strategy**: LWW at worksheet level

### 4. SetWorksheetHideMutation
**ID**: `sheet.mutation.set-worksheet-hidden`
**Purpose**: Hides/shows worksheet
**Strategy**: LWW at worksheet level

### 5. SetWorksheetNameMutation
**ID**: `sheet.mutation.set-worksheet-name`
**Purpose**: Renames worksheet
**Strategy**: LWW at worksheet level

### 6. SetWorksheetOrderMutation
**ID**: `sheet.mutation.set-worksheet-order`
**Purpose**: Changes worksheet tab order
**Strategy**: LWW at worksheet level

### 7. SetWorksheetRightToLeftMutation
**ID**: `sheet.mutation.set-worksheet-right-to-left`
**Purpose**: Sets RTL direction
**Strategy**: LWW at worksheet level

### 8. SetWorksheetDefaultStyleMutation
**ID**: `sheet.mutation.set-worksheet-default-style`
**Purpose**: Sets default cell style
**Strategy**: LWW at worksheet level

### 9. SetWorksheetRowCountMutation
**ID**: `sheet.mutation.set-worksheet-row-count`
**Purpose**: Sets total row count
**Strategy**: LWW at worksheet level

### 10. SetWorksheetColumnCountMutation
**ID**: `sheet.mutation.set-worksheet-column-count`
**Purpose**: Sets total column count
**Strategy**: LWW at worksheet level

## Unified Conflict Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position
**Not applicable** - These are worksheet-level properties with no position

### Dimension 4: Feature Plugin ID
**Not applicable** - Singleton per worksheet

## Transform Coverage Matrix

All these mutations have the same pattern:

### With ALL Other Mutations

| Mutation Type | Status | Strategy | Notes |
|---------------|--------|----------|-------|
| All structural mutations | ❌ | Identity | Position-independent properties |
| All data mutations | ❌ | Identity | Data-independent properties |
| All feature plugins | ❌ | Identity | Independent properties |
| Self (same mutation type) | ✅ | LWW | m2 wins on same worksheet |
| Other worksheet properties | ❌ | Identity | Different properties |

## Standard Implementation

All mutations follow this exact pattern:

```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        // Same worksheet: m2 wins, m1 is no-op
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

## Implementation Checklist

- [x] All self-transforms use LWW pattern
- [x] All cross-mutation transforms are identity
- [x] Tests follow standard pattern
- [x] Zero-copy optimizations applied
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for all simple worksheet properties
