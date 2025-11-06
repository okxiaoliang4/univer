# Testing Guide for Pivot Table Dialog

## Quick Test Steps

### 1. Build the Package
```bash
cd /Volumes/data/projects/univer
pnpm install
pnpm build:lib
```

### 2. Manual Testing

#### Test Case 1: Dialog Opens Successfully
1. Start the Univer application
2. Create a spreadsheet with some data (e.g., 3 columns x 5 rows)
3. Select any cell in the data range
4. Click Insert menu > Pivot Table
5. **Expected**: Dialog opens with title "Create Pivot Table"

#### Test Case 2: Source Range Validation
1. Open the dialog
2. Try to set source range to a single cell (e.g., "A1")
3. **Expected**: Error message "Source range must contain more than one cell"

#### Test Case 3: Source Range Single Row Error
1. Open the dialog
2. Set source range to single row (e.g., "A1:C1")
3. **Expected**: Error message "Source range must contain at least two rows (header and data)"

#### Test Case 4: Target Range Validation
1. Open the dialog
2. Set target range to multiple cells (e.g., "E1:F2")
3. **Expected**: Error message "Target location must be a single cell"

#### Test Case 5: Valid Input
1. Open the dialog
2. Set source range to multi-row data (e.g., "A1:C5")
3. Set target to single cell (e.g., "E1")
4. Click OK
5. **Expected**: Dialog closes (pivot table creation pending AddPivotTableCommand implementation)

#### Test Case 6: Cancel Button
1. Open the dialog
2. Click Cancel
3. **Expected**: Dialog closes without action

#### Test Case 7: Auto Range Detection
1. Select a single cell in a data range
2. Open the dialog
3. **Expected**: Source range automatically expands to include continuous data

## Component Verification

### Verify Locale Registration
Check that locales are available:
```typescript
import { enUS, zhCN } from '@univerjs/sheets-pivot-table-ui';
console.log(enUS); // Should show English translations
console.log(zhCN); // Should show Chinese translations
```

### Verify Dialog Component Registration
In browser console:
```javascript
// Should be registered in ComponentManager
// Check if dialog opens when menu item is clicked
```

### Verify Operation Registration
```typescript
// Operations should be registered:
// - sheet.operation.open-create-pivot-table-dialog
// - sheet.operation.show-pivot-table-panel
// - sheet.operation.hide-pivot-table-panel
```

## Known Limitations (Expected)
1. Clicking OK does not create a pivot table yet (AddPivotTableCommand not implemented)
2. Only source/target range selection is implemented
3. No field configuration yet (future enhancement)

## Debugging Tips

### If Dialog Doesn't Open
1. Check browser console for errors
2. Verify ComponentManager has registered CREATE_PIVOT_TABLE_DIALOG
3. Verify ICommandService has registered the operation
4. Check if menu item is visible and enabled

### If Validation Doesn't Work
1. Check console for error messages
2. Verify RangeSelector is working properly
3. Check LocaleService has translations loaded

### If Locales Are Missing
1. Verify locales are imported in application
2. Check LocaleService configuration
3. Ensure locale files are built correctly

## Success Criteria
✅ Dialog opens when menu item is clicked
✅ Source range selector works and validates
✅ Target range selector works and validates
✅ Error messages display correctly
✅ OK button is disabled when validation fails
✅ Cancel button closes dialog
✅ Locales work for both en-US and zh-CN
✅ No console errors
✅ No linting errors

## Next Implementation Steps
After verifying dialog works:
1. Implement AddPivotTableCommand in sheets-pivot-table package
2. Connect dialog OK button to command execution
3. Add field configuration UI
4. Add pivot table rendering logic

