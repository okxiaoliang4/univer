## 1. Extract Business Logic to Hooks

- [x] 1.1 Create `packages/find-replace/src/views/dialog/hooks/use-find-replace-logic.ts` hook that extracts:
  - Find/replace state management (from `FindDialog` and `ReplaceDialog`)
  - Event handlers (onFindStringChange, onClickFindButton, onClickReplaceButton, etc.)
  - Focus management logic
  - Returns all necessary state and handlers for both desktop and mobile

- [x] 1.2 Create `packages/find-replace/src/views/dialog/hooks/use-find-replace-options.ts` hook that extracts:
  - Option generation logic (findScopeOptions, findDirectionOptions, findByOptions)
  - Returns options arrays for both desktop and mobile

- [x] 1.3 Refactor `FindReplaceDialog.tsx` to use the new hooks
  - Replace inline logic with hook calls
  - Ensure desktop behavior remains unchanged
  - Verify all existing functionality still works

## 2. Create Mobile Component

- [x] 2.1 Create `packages/find-replace/src/views/mobile/MobileFindReplace.tsx` component
  - Use shared hooks from step 1
  - Render find/replace UI optimized for mobile (sidebar layout)
  - Support both find-only and replace modes
  - Register component with ComponentManager using unique key

- [x] 2.2 Create mobile-specific styling if needed
  - Ensure touch-friendly button sizes
  - Optimize layout for narrow screens
  - Test on various mobile screen sizes

## 3. Create Mobile Controller

- [x] 3.1 Create `packages/find-replace/src/controllers/mobile/mobile-find-replace.controller.ts`
  - Similar structure to `FindReplaceController` but uses Sidebar instead of Dialog
  - Register same commands as desktop controller (OpenFindDialogOperation, OpenReplaceDialogOperation, etc.)
  - Do NOT register shortcuts (mobile devices don't have keyboards)
  - Register menu schema (same as desktop)
  - Register `MobileFindReplace` component with ComponentManager
  - Listen to `IFindReplaceService.stateUpdates$` for `revealed` state changes
  - Use `ISidebarService` to open/close sidebar when find-replace is triggered
  - Handle sidebar lifecycle similar to desktop dialog lifecycle

- [x] 3.2 Create `packages/find-replace/src/mobile-plugin.ts`
  - Register `MobileFindReplaceController` only when mobile UI plugin is available
  - Register `IFindReplaceService` with `FindReplaceService` implementation
  - Follow pattern from other mobile plugins

## 4. Update Desktop Controller

- [x] 4.1 Update `FindReplaceController.ts` to detect mobile environment
  - Check if mobile UI plugin is loaded before registering mobile controller
  - Ensure desktop dialog behavior is not affected
  - Maintain backward compatibility

## 5. Testing and Validation

- [x] 5.1 Test mobile find-replace functionality
  - Verify sidebar opens when find/replace is triggered
  - Verify all find/replace operations work correctly
  - Verify sidebar closes properly
  - Test both find-only and replace modes

- [x] 5.2 Test desktop functionality remains unchanged
  - Verify dialog still works as before
  - Verify no regressions in existing behavior
  - Test all find/replace features

- [x] 5.3 Verify code reuse
  - Ensure business logic is shared between desktop and mobile
  - Verify no duplicate business logic code exists
  - Check that hooks are properly used in both components

- [x] 5.4 Run linter and type checker
  - Fix any linting errors
  - Fix any TypeScript errors
  - Ensure code follows project conventions

## 6. Documentation

- [x] 6.1 Update component documentation if needed
- [x] 6.2 Add comments explaining hook usage and code reuse pattern

## 7. Sheets Find-Replace Mobile Plugin

- [x] 7.1 Create `packages/sheets-find-replace/src/controllers/mobile/sheet-find-replace.controller.ts`
  - Create mobile-specific controller that does NOT depend on `FindReplaceController` (desktop)
  - Use `IFindReplaceService.terminate()` to close sidebar instead of `closePanel()`
  - Register `SheetsFindReplaceProvider` for sheets find-replace functionality
  - Handle editor activation by terminating find-replace session

- [x] 7.2 Export `SheetsFindReplaceProvider` from `sheet-find-replace.controller.ts`
  - Change `class` to `export class` so it can be used in mobile controller

- [x] 7.3 Create `packages/sheets-find-replace/src/mobile-plugin.ts`
  - Add `@DependentOn` decorator with `UniverSheetsFindReplacePlugin`, `UniverFindReplaceMobilePlugin`, and `UniverSheetsPlugin`
  - Register `SheetsFindReplaceMobileController` instead of desktop controller
  - Follow the same pattern as other mobile plugins

- [x] 7.4 Export mobile plugin from `packages/sheets-find-replace/src/index.ts`
  - Add export for `UniverSheetsFindReplaceMobilePlugin`

- [x] 7.5 Update spec to include sheets mobile plugin requirements
  - Add requirement for sheets find-replace mobile plugin
  - Document that mobile controller uses sidebar instead of dialog
  - Document dependency relationships
