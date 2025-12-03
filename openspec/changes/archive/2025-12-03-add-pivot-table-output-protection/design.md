# Design: Pivot Table Output Protection

## Context

Pivot tables in Univer dynamically generate output data based on source data and field configurations. The output is rendered to a target cell range. Currently, users can manually edit cells within pivot table output ranges, which creates data inconsistency issues.

The Univer permission system provides two approaches:
1. **Persisted protection rules**: Saved to snapshot, managed via `RangeProtectionRuleModel`
2. **Dynamic interceptors**: Runtime-only protection using `SheetInterceptorService`

For pivot tables, we need dynamic protection that:
- Does not persist to snapshot (pivot output is transient/calculated)
- Automatically adjusts when pivot table ranges change
- Integrates seamlessly with existing permission UI feedback

## Goals / Non-Goals

**Goals:**
- Prevent manual editing of pivot table output cells
- Provide clear error feedback when users attempt to edit
- Automatically protect/unprotect when pivot tables are created/deleted/moved
- Use lightweight runtime protection (no snapshot pollution)

**Non-Goals:**
- Persist protection rules to workbook snapshot
- Allow configurable "allow edit" exceptions for pivot output
- Protect pivot table source data range (separate concern)
- Add UI elements for managing pivot protection (automatic only)

## Decisions

### Decision 1: Use Cell Content Interceptor Pattern
**What:** Implement protection using `SheetInterceptorService.intercept(INTERCEPTOR_POINT.CELL_CONTENT)` to inject read-only metadata, similar to `SheetPermissionViewModelController`.

**Why:**
- Consistent with existing Univer permission architecture
- No snapshot persistence (pivot output is calculated, not stored)
- Automatically applies to all pivot output cells without per-cell tracking
- Efficient - uses caching patterns from `RangeProtectionCache`

**Alternatives considered:**
- Range protection rules (`RangeProtectionRuleModel`): Rejected because it persists to snapshot and requires explicit rule management
- Cell-level metadata storage: Rejected because it's redundant with calculated data and harder to maintain

### Decision 2: Block Commands via Permission Check Controller
**What:** Create `PivotTablePermissionController` that intercepts commands before execution using `ICommandService.beforeCommandExecuted()`.

**Why:**
- Follows pattern from `SheetPermissionCheckController` and `SheetPermissionCheckUIController`
- Prevents mutations at command level, not just UI level
- Can show user-friendly error dialogs
- Blocks both UI and API-driven edits

### Decision 3: Cache Pivot Table Output Ranges
**What:** Maintain an in-memory Map of `{unitId: {subUnitId: {row-col: pivotTableId}}}` for quick lookups.

**Why:**
- Interceptors are called frequently during rendering
- O(1) lookup performance critical for smooth UX
- Similar to `RangeProtectionCache` pattern
- Automatically updated via existing `pivotTableAdded$` and range change events

## Architecture

```
User attempts to edit cell in pivot output
          ↓
Command Layer: SetRangeValuesCommand triggered
          ↓
PivotTablePermissionController.beforeCommandExecuted()
  ├─ Check if target cell is in any pivot output range
  ├─ Use cached lookup: _pivotOutputCache.get(unitId, subUnitId, row, col)
  └─ If yes: throw CustomCommandExecutionError
          ↓
If not blocked: Continue to ViewModel
          ↓
Cell Content Interceptor: CELL_CONTENT intercept point
  ├─ Check if cell is in pivot output (cached lookup)
  ├─ If yes: inject { isPivotOutput: true, readOnly: true }
  └─ Return modified cell data
          ↓
Render Layer: Display cell with read-only visual indicators
```

### Key Components

1. **PivotTablePermissionController**
   - Location: `packages/sheets-pivot-table/src/controllers/pivot-table-permission.controller.ts`
   - Responsibilities:
     - Register cell content interceptor
     - Register command interceptor
     - Maintain pivot output range cache
     - Listen to pivot table lifecycle events
     - Show permission error dialogs

2. **SheetsPivotTableService Extensions**
   - Add method: `isPivotOutputCell(unitId, subUnitId, row, col): boolean`
   - Add method: `getPivotTableByOutputCell(unitId, subUnitId, row, col): PivotTable | undefined`

3. **Cache Structure**
   ```typescript
   private _pivotOutputCache = new Map<
     string, // unitId
     Map<
       string, // subUnitId
       Map<string, string> // "row-col" -> pivotTableId
     >
   >();
   ```

## Implementation Sequence

1. **Phase 1: Service Layer**
   - Add helper methods to `SheetsPivotTableService`
   - Add cache rebuild logic triggered by pivot table events

2. **Phase 2: Controller Layer**
   - Create `PivotTablePermissionController`
   - Register interceptors
   - Implement cache management

3. **Phase 3: Integration**
   - Register controller in plugin
   - Wire up event subscriptions
   - Add error handling

4. **Phase 4: Testing & Polish**
   - Unit tests for all components
   - E2E tests for user scenarios
   - Performance validation

## Risks / Trade-offs

**Risk: Performance impact on large spreadsheets**
- Mitigation: Use efficient caching with O(1) lookups
- Mitigation: Only check ranges on actual edit commands, not every render
- Mitigation: Lazy cache building (on-demand)

**Risk: Conflicts with other protection systems**
- Mitigation: Set interceptor priority lower than worksheet/range protection
- Mitigation: Document interaction with permission system in code comments

**Risk: Memory overhead from caching**
- Mitigation: Cache is cleared when worksheets unload
- Mitigation: Cache size proportional to number of pivot tables (typically small)

## Migration Plan

- No migration needed - this is a new feature
- Existing pivot tables automatically protected once plugin is updated
- No snapshot format changes

## Open Questions

None - design is straightforward and follows established patterns.

