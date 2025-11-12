# PivotEngineV2 Design Decisions

## Context

The current `PivotEngine` outputs data in a sparse matrix format (`IObjectMatrixPrimitiveType<Nullable<ICellData>>`), which requires UI components to parse and reconstruct the pivot table structure. This makes it difficult to:
- Render Cross-Tabulation tables with proper subtotals and totals
- Support collapse/expand functionality for hierarchical groups
- Handle multi-value fields efficiently
- Detect empty data and provide dimension information

## Goals

- Provide structured Cross-Tabulation output format optimized for UI rendering
- Support multi-level row/column hierarchies with grouping information
- Enable collapse/expand functionality through grouping metadata
- Separate input configuration from output results for better caching
- Support field-level subtotal configuration
- Maintain compatibility with existing aggregation logic

## Non-Goals

- Replace existing `PivotEngine` (both will coexist)
- Add sorting support (Phase 2)
- Add formatting support (Phase 2)
- Add custom label templates (Phase 2)
- Add performance monitoring metadata (Phase 2)

## Decisions

### Decision: Separate Input and Output Interfaces

**What**: Create `IPivotTableCrossTabConfig` for input and `IPivotTableCrossTabData` for output.

**Why**:
- Clear separation of concerns
- Better caching (can cache output without storing input)
- Easier serialization (output can be serialized independently)
- Follows single responsibility principle

**Alternatives considered**:
- Single interface with both config and results: Rejected - violates separation of concerns
- Include config reference in output: Rejected - adds unnecessary data

### Decision: Field-Level Subtotal Configuration

**What**: Add `showSubTotals?: boolean` to `IPivotField` instead of global configuration.

**Why**:
- More flexible - each field can independently control subtotals
- First field's subtotal naturally becomes grand total
- Matches Excel behavior
- Easier to understand and configure

**Alternatives considered**:
- Global subtotal configuration: Rejected - less flexible
- Separate subtotal configuration object: Rejected - adds complexity

### Decision: Use Record Instead of Map for Level Maps

**What**: Use `Record<number, string[]>` instead of `Map<number, string[]>` for `rowLevelMap` and `columnLevelMap`.

**Why**:
- JSON serializable without custom serialization
- Easier to work with in TypeScript
- Sufficient for the use case (numeric keys)

**Alternatives considered**:
- Keep Map: Rejected - serialization issues
- Use array of objects: Rejected - O(n) lookup vs O(1)

### Decision: Remove grandtotal from rowTypes/columnTypes

**What**: Only use `'data' | 'subtotal'` in `rowTypes` and `columnTypes`, remove `'grandtotal'`.

**Why**:
- Grand total is just the first field's subtotal (level=0, fieldIndex=0)
- Reduces redundancy
- Can be detected via `subtotalRows/subtotalColumns` metadata

**Alternatives considered**:
- Keep grandtotal type: Rejected - redundant information

### Decision: Individual Setter Methods

**What**: Provide individual setter methods (`setRowFields`, `setColumnFields`, etc.) instead of single `setConfig`.

**Why**:
- Matches existing `PivotEngine` pattern
- More granular control
- Better for incremental updates
- Each setter can mark dirty independently

**Alternatives considered**:
- Single `setConfig` method: Rejected - doesn't match existing pattern
- Both individual and batch setters: Rejected - adds complexity

### Decision: Include Value Field in Subtotal Metadata

**What**: Add `value: string` field to `subtotalRows` and `subtotalColumns` arrays.

**Why**:
- Provides grouping value directly (e.g., "华北") without parsing rowHeaders
- Makes it easier to generate labels and tooltips
- Reduces need to look up rowHeaders

**Alternatives considered**:
- Parse from rowHeaders: Rejected - less efficient, more error-prone

### Decision: Remove Duplicate Values in Header Display

**What**: In `getCalculatedCellMatrix()`, duplicate values in column headers and row headers are only displayed once within their parent context, with subsequent occurrences shown as empty strings.

**Why**:
- Reduces visual clutter in the rendered table
- Matches common pivot table UI patterns (e.g., Excel)
- Makes it easier to visually group related columns/rows
- Parent-level values appear only once across the entire row/column
- Child-level values appear once per parent group (e.g., "线下门店" appears once under Q1, once under Q2)

**Implementation**:
- For column headers:
  - Level 0 (top level): Track last value, only show on first occurrence or when value changes
  - Level > 0: Track parent key (all previous levels) and value, only show when parent changes or value changes within same parent
  - Example: Q1 -> 线下门店, 线上商店; Q2 -> 线下门店, 线上商店 (each child value appears once per parent)
- For row headers: Track last value at each level, only show value when it changes from previous row
- Empty strings are used for duplicate values to maintain cell alignment

**Alternatives considered**:
- Show all values: Rejected - creates visual clutter
- Global deduplication (remove all duplicates): Rejected - loses parent-child relationship context
- Merge cells: Rejected - matrix format doesn't support cell merging, would require separate rendering logic

## Risks / Trade-offs

### Risk: Performance with Large Datasets
**Mitigation**:
- Use caching mechanism (`_isDirty` flag)
- Only recalculate when configuration changes
- Optimize grouping algorithms (single-pass where possible)

### Risk: Memory Usage with 3D Array
**Mitigation**:
- Only allocate what's needed
- Consider sparse representation for very large datasets (future optimization)

### Trade-off: Structured Output vs Flexibility
**Decision**: Structured output is more valuable for UI rendering than flexibility of sparse matrix format.

## Migration Plan

- No migration needed - new class alongside existing `PivotEngine`
- UI components can gradually migrate to use `PivotEngineV2`
- Existing `PivotEngine` remains available for backward compatibility

## Open Questions

- Should we add support for custom subtotal labels in Phase 2?
- Should we add performance metrics (calculation time) in Phase 2?
- Should we support calculated fields in future phases?

