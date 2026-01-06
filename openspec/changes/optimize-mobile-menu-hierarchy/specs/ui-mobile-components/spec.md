## ADDED Requirements

### Requirement: Mobile Menu Hierarchical Navigation

The mobile menu component SHALL support hierarchical menu structures using Drawer components for navigation, preserving menu organization and providing native mobile navigation patterns.

#### Scenario: Menu items with children open Drawer
- **GIVEN** a mobile menu is rendered with menu items that have children (via `IMenuSchema.children`)
- **WHEN** a user clicks on a menu item that has children
- **THEN** a Drawer opens from the right side
- **AND** the Drawer header displays the parent menu item's title or label
- **AND** the Drawer content displays the children menu items
- **AND** the children menu items are rendered using the same MobileMenu component recursively

#### Scenario: SUBITEMS type menu items open Drawer
- **GIVEN** a mobile menu contains a menu item with `type === MenuItemType.SUBITEMS`
- **WHEN** a user clicks on the SUBITEMS menu item
- **THEN** a Drawer opens from the right side
- **AND** the submenu items are fetched via `menuManagerService.getMenuByPositionKey(menuItem.id)`
- **AND** the fetched submenu items are rendered in the Drawer content
- **AND** if the submenu is empty, the Drawer still opens but shows no items

#### Scenario: SELECTOR type menu items display selections in Drawer
- **GIVEN** a mobile menu contains a menu item with `type === MenuItemType.SELECTOR` and `selections` property
- **WHEN** a user clicks on the SELECTOR menu item
- **THEN** a Drawer opens from the right side
- **AND** the Drawer content displays the selections as menu options
- **AND** clicking a selection triggers the `onOptionSelect` callback with the selection value
- **AND** observable selections are supported (similar to desktop implementation)

#### Scenario: Flat menus render without Drawer
- **GIVEN** a mobile menu contains only menu items without children
- **WHEN** the mobile menu renders
- **THEN** all menu items are displayed directly in the menu grid
- **AND** no Drawer component is rendered
- **AND** clicking menu items triggers their commands directly

#### Scenario: Nested Drawers for multi-level hierarchy
- **GIVEN** a mobile menu has multiple levels of hierarchy (children with children)
- **WHEN** a user navigates through menu levels
- **THEN** clicking a child menu item that also has children opens a new Drawer
- **AND** the new Drawer displays the nested children
- **AND** multiple Drawers can be open simultaneously (stacked)
- **AND** closing a Drawer returns to the previous level

#### Scenario: Drawer header and navigation
- **GIVEN** a Drawer is open displaying menu children
- **WHEN** the Drawer renders
- **THEN** the Drawer header displays the parent menu item's title or label
- **AND** a close button is available in the header
- **AND** clicking the close button closes the Drawer
- **AND** swiping the Drawer closed (if supported) also closes it

#### Scenario: Menu item states in Drawer
- **GIVEN** menu items with children are displayed in a Drawer
- **WHEN** the Drawer renders
- **THEN** disabled menu items are displayed but not clickable
- **AND** hidden menu items are not rendered
- **AND** activated menu items show their activated state
- **AND** menu item icons, labels, and titles are displayed correctly

#### Scenario: Menu item click handling with Drawer
- **GIVEN** a menu item without children is clicked
- **WHEN** the click occurs
- **THEN** the `onOptionSelect` callback is triggered with the menu item's information
- **AND** if the menu item is inside a Drawer, the Drawer remains open (unless the action closes it)
- **AND** if the menu item has children, a Drawer opens instead of triggering the callback
