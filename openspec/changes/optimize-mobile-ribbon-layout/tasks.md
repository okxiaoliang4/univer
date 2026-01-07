## 1. Remove Classic Style Support
- [x] 1.1 Remove classic style handling from MobileRibbon.tsx
- [x] 1.2 Remove or update ribbonType prop handling (ensure classic is not used for mobile)
- [x] 1.3 Remove fakeToolbar logic related to classic style

## 2. Refactor Toolbar Layout to Vertical List
- [x] 2.1 Remove visibleGroups/collapsedIds separation logic from MobileRibbon.tsx
- [x] 2.2 Remove ResizeObserver and collapsedIds state management
- [x] 2.3 Remove fakeToolbar rendering logic
- [x] 2.4 Remove toolbarItemRefs tracking logic
- [x] 2.5 Change toolbar container from horizontal grid to vertical flex layout
- [x] 2.6 Update activeGroup computation to return all groups uniformly
- [x] 2.7 Remove "More functions" dropdown logic

## 3. Create MobileToolbarItem Component
- [x] 3.1 Create MobileToolbarItem.tsx file
- [x] 3.2 Implement renderButtonType method (no arrow indicator)
- [x] 3.3 Implement renderSelectorType method (with right arrow indicator)
- [x] 3.4 Import right arrow icon from @univerjs/icons (ChevronRightIcon or similar)
- [x] 3.5 Implement list item layout: icon (left), title (center), arrow (right, conditional)
- [x] 3.6 Copy button click interaction logic from ToolbarItem.tsx
- [x] 3.7 Implement selector drawer opening logic

## 4. Add Group Separators
- [x] 4.1 Add horizontal divider lines between toolbar groups
- [x] 4.2 Style dividers appropriately for mobile UI

## 5. Integrate Drawer for Selectors
- [x] 5.1 Import Drawer components from @univerjs/design
- [x] 5.2 Implement drawer state management for selector items
- [x] 5.3 Render selector options inside Drawer following existing rendering rules
- [x] 5.4 Handle drawer open/close events
- [x] 5.5 Ensure drawer slides from appropriate direction (right side)

## 6. Testing and Validation
- [x] 6.1 Test button items maintain original click behavior
- [x] 6.2 Test selector items open drawer correctly
- [x] 6.3 Test drawer displays selector options correctly
- [x] 6.4 Test group separators render correctly
- [x] 6.5 Test vertical list layout on various mobile screen sizes
- [x] 6.6 Verify no classic style references remain
- [x] 6.7 Run linting and type checking
