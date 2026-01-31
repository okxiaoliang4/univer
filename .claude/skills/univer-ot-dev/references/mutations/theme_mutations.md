# Theme Mutations - Batch Analysis

This document covers range theme and worksheet theme mutations.

## Range Theme Mutations

### 1. AddRangeThemeMutation
**ID**: `sheet.mutation.add-range-theme`
**Purpose**: Adds theme to ranges
**Strategy**: Range-based + Shifting
**Parameters**: Ranges with theme styles

### 2. RemoveRangeThemeMutation
**ID**: `sheet.mutation.remove-range-theme`
**Purpose**: Removes theme from ranges
**Strategy**: Name-based removal

### 3. SetRangeThemeMutation
**ID**: `sheet.mutation.set-range-theme`
**Purpose**: Updates range theme
**Strategy**: Name-based + LWW

## Worksheet Range Theme Mutations

### 4. SetWorksheetRangeThemeStyleMutation
**ID**: `sheet.mutation.set-worksheet-range-theme-style`
**Purpose**: Sets theme style for range in worksheet
**Strategy**: Range-based + Shifting
**Parameters**: `range: IRange, theme_name: String`

### 5. DeleteWorksheetRangeThemeStyleMutation
**ID**: `sheet.mutation.remove-worksheet-range-theme-style`
**Purpose**: Removes theme style from range
**Strategy**: Range-based + Shifting

## Unit-Level Theme Mutations

### 6. RegisterWorksheetRangeThemeStyleMutation
**ID**: `sheet.mutation.register-worksheet-range-theme-style`
**Purpose**: Registers theme style at unit level
**Strategy**: LWW at unit level (no worksheet)

### 7. UnregisterWorksheetRangeThemeStyleMutation
**ID**: `sheet.mutation.unregister-worksheet-range-theme-style`
**Purpose**: Unregisters theme style
**Strategy**: LWW at unit level

## Conflict Dimensions Analysis

### Range-Based Themes
- **Dimension 1-2**: Standard worksheet matching
- **Dimension 3**: Ranges need shifting
- **Dimension 4**: `theme_name` for identity

### Unit-Level Themes
- **Dimension 1**: Unit ID only
- **Dimension 2-4**: Not applicable

## Transform Coverage Matrix

### Range Theme Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ⏳ | Shifting | Theme ranges shift rows |
| InsertColMutation | ⏳ | Shifting | Theme ranges shift columns |
| RemoveRowsMutation | ⏳ | Shifting + Removal | Themes may be removed |
| RemoveColMutation | ⏳ | Shifting + Removal | Themes may be removed |
| Self | ✅ | Identity (diff name) / LWW (same name) | By theme_name |

### Unit-Level Themes

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| All position-based mutations | ❌ | Identity | Unit-level, no position |
| Self | ✅ | LWW | m2 wins on same unit + name |

## Implementation Pattern

### Range-Based (with ranges)
```rust
// Similar to AddConditionalRuleMutation
fn create_insert_row_vs_theme() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Shift ranges in theme
        shift_range_rows_for_insert(&mut m2_params.range, insert_start, insert_count);
        // ...
    })
}
```

### Unit-Level (LWW)
```rust
// Check unit_id only (no sub_unit_id)
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if m1_params.unit_id != m2_params.unit_id {
            return identity(m1, m2);
        }

        if m1_params.theme_name != m2_params.theme_name {
            return identity(m1, m2);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

## Implementation Checklist

- [ ] Range theme transforms with structural mutations
- [ ] Unit-level theme self-transforms (LWW)
- [ ] Add vs Remove conflict resolution
- [ ] Tests written
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for theme mutations
