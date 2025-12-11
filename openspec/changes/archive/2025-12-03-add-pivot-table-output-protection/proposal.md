# Add Pivot Table Output Protection

## Why

Pivot table output ranges are dynamically calculated and rendered by the system. Users should not be able to manually edit cells within the pivot table output area, as this would:
- Create data inconsistency between the calculated pivot data and displayed values
- Cause confusion when pivot table refreshes overwrite manual edits
- Break the integrity of the pivot table calculation model

## What Changes

- Add cell content interceptor to mark pivot table output cells as read-only dynamically
- Add command interceptor to block edit operations on pivot table output ranges
- Integrate with existing permission system patterns (using interceptors, not persisted rules)
- Provide clear visual feedback when users attempt to edit protected pivot table cells

## Impact

- Affected specs: `sheets-pivot-table`
- Affected code:
  - `packages/sheets-pivot-table/src/controllers/` - New permission controller
  - `packages/sheets-pivot-table/src/services/pivot-table.service.ts` - Helper methods to identify pivot output ranges
  - Integration with `@univerjs/sheets` permission and interceptor systems
- No breaking changes - this is an additive feature that enhances data integrity

