# Pivot Table Range Change Event Implementation Summary

## Overview
Added range change event support to the pivot table data source model, following the same pattern as `TableManager` in `sheets-table`. This enables proper tracking and management of pivot table ranges through the `IExclusiveRangeService`.

## Changes Made

### 1. Added Range Change Event Interface
**File**: `src/types/type.ts`
- Added `IPivotTableRangeChangedEvent` interface with fields:
  - `unitId`: Workbook unit ID
  - `subUnitId`: Worksheet subunit ID  
  - `tableId`: Pivot table ID
  - `range`: Updated source or target range (IRange)

### 2. Updated Data Source Model
**File**: `src/models/sheets-pivot-data-source-model.ts`
- Added `_tableRangeChanged$` subject and `tableRangeChanged$` observable
- Updated `setPivotTableConfig()` method to:
  - Compare old and new source range configurations
  - Emit range change event when source range changes
  - Update pivot table instance with new source range info
  - Handle target cell changes separately (no event needed for display position)

**Key Logic:**
```typescript
// Check if source range changed
const sourceRangeChanged = oldConfig && (
    oldConfig.sourceRangeInfo.range.startRow !== config.sourceRangeInfo.range.startRow ||
    oldConfig.sourceRangeInfo.range.startColumn !== config.sourceRangeInfo.range.startColumn ||
    oldConfig.sourceRangeInfo.range.endRow !== config.sourceRangeInfo.range.endRow ||
    oldConfig.sourceRangeInfo.range.endColumn !== config.sourceRangeInfo.range.endColumn
);

if (sourceRangeChanged) {
    pivotTable.setSourceRangeInfo(config.sourceRangeInfo);
    this._tableRangeChanged$.next({
        unitId,
        subUnitId,
        tableId: pivotTableId,
        range: config.sourceRangeInfo.range,
    });
}
```

### 3. Services Review
**Files**: `src/services/pivot-table.service.ts`, `src/services/pivot-table-calculation.service.ts`
- No changes needed - services already use `setPivotTableConfig()` which now emits events
- Service properly exposes data source model via `getDataSourceModel()`

### 4. Controller Verification
**File**: `src/controllers/pivot-table-range.controller.ts`
- Already properly subscribes to `tableRangeChanged$` event
- Updates `IExclusiveRangeService` when range changes:
  - Clears old exclusive range
  - Adds new exclusive range with updated bounds

### 5. Added Comprehensive Tests
**File**: `src/services/__tests__/pivot-table.service.spec.ts`
- Added test: "should emit range change event when source range is updated"
  - Verifies event is emitted with correct data
  - Uses async/await pattern (not deprecated `done()` callback)
  - Validates all event fields (unitId, subUnitId, tableId, range)
- Added test: "should not emit range change event when only fields config is updated"
  - Ensures event is only emitted for actual range changes
  - Verifies no false positives

## Test Results
```
✓ src/services/__tests__/pivot-table.service.spec.ts (13 tests) 5ms
  Test Files  1 passed (1)
       Tests  13 passed (13)
```

All tests pass successfully with no warnings.

## Architecture Pattern
Follows the same pattern as `TableManager` from `sheets-table`:
- Subject/Observable pattern for event emissions
- Event payload includes all necessary context (unitId, subUnitId, tableId, range)
- Controller subscribes to events and updates exclusive ranges
- Services use data model methods which emit events automatically

## Event Flow
1. User/System calls `service.updatePivotTableConfig()` with new range
2. Service calls `dataSourceModel.setPivotTableConfig()`
3. Model compares old and new ranges
4. If range changed:
   - Model updates pivot table instance
   - Model emits `tableRangeChanged$` event
5. `SheetPviotTableRangeController` receives event
6. Controller updates `IExclusiveRangeService` to reflect new range

## Benefits
- ✅ Proper exclusive range management prevents overlapping data regions
- ✅ Controllers can reactively respond to range changes
- ✅ Consistent with existing Univer patterns
- ✅ Well-tested with comprehensive unit tests
- ✅ No breaking changes to existing APIs

## Files Modified
- ✅ `src/types/type.ts` - Added interface
- ✅ `src/models/sheets-pivot-data-source-model.ts` - Added event and logic
- ✅ `src/services/__tests__/pivot-table.service.spec.ts` - Added tests

## Files Verified (No Changes Needed)
- ✅ `src/services/pivot-table.service.ts` - Already compatible
- ✅ `src/services/pivot-table-calculation.service.ts` - Not affected
- ✅ `src/controllers/pivot-table-range.controller.ts` - Already subscribes correctly

## Next Steps
The implementation is complete and tested. The range change events are now properly integrated into the pivot table system.

