## MODIFIED Requirements

### Requirement: Mobile Ribbon Layout

The mobile ribbon component SHALL render toolbar items in a vertical list layout optimized for mobile devices, displaying all items uniformly without collapsing logic.

#### Scenario: Vertical list layout
- **GIVEN** a mobile ribbon is being rendered
- **WHEN** the ribbon component mounts
- **THEN** all toolbar items are displayed in a vertical list layout
- **AND** items are arranged from top to bottom
- **AND** no items are hidden or collapsed based on available width
- **AND** each toolbar item uses the MobileToolbarItem component

#### Scenario: Group separators
- **GIVEN** a mobile ribbon with multiple toolbar groups
- **WHEN** the ribbon renders
- **THEN** horizontal divider lines are displayed between groups
- **AND** dividers provide clear visual separation between groups

#### Scenario: MobileToolbarItem button rendering
- **GIVEN** a toolbar item with type BUTTON
- **WHEN** MobileToolbarItem renders the item
- **THEN** it displays as a list item with icon on the left and title in the center
- **AND** no arrow indicator is displayed
- **AND** clicking the item executes the command following the same behavior as ToolbarItem.tsx

#### Scenario: MobileToolbarItem selector rendering
- **GIVEN** a toolbar item with type SELECTOR, BUTTON_SELECTOR, or SUBITEMS
- **WHEN** MobileToolbarItem renders the item
- **THEN** it displays as a list item with icon on the left, title in the center, and a right arrow indicator on the right
- **AND** the right arrow icon is imported from @univerjs/icons
- **AND** clicking the item opens a Drawer component
- **AND** the Drawer displays selector options following existing rendering rules

#### Scenario: Selector drawer behavior
- **GIVEN** a selector toolbar item is clicked
- **WHEN** the drawer opens
- **THEN** it uses the Drawer component from @univerjs/design
- **AND** the drawer slides from the right side
- **AND** selector options are rendered inside the drawer following the same rules as desktop ToolbarItem
- **AND** selecting an option executes the appropriate command and closes the drawer

## REMOVED Requirements

### Requirement: Classic Style Support in Mobile Ribbon

**Reason**: Classic style adds unnecessary complexity for mobile devices and is not needed for mobile ribbon implementation.

**Migration**: Mobile ribbon will no longer support `ribbonType === 'classic'`. Desktop ribbon continues to support classic style separately.

### Requirement: Collapsed Toolbar Items in Mobile Ribbon

**Reason**: Mobile ribbon uses vertical layout which doesn't require collapsing items based on available width. All items are displayed uniformly.

**Migration**: All toolbar items are now always visible in the vertical list. The ResizeObserver and collapsedIds logic is removed.
