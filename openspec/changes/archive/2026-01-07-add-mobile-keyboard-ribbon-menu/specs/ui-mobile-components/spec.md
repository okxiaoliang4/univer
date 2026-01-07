## ADDED Requirements

### Requirement: Mobile Ribbon Rendering

The mobile ribbon component SHALL render menu items from both the main ribbon position and a subMenu position, displaying subMenu items in a dedicated area next to the ribbon tabs.

#### Scenario: SubMenu items are rendered in MobileRibbon
- **GIVEN** menu items are registered at the RibbonPosition.SUBMENU position
- **WHEN** MobileRibbon component renders
- **THEN** subMenu items are fetched using `menuManagerService.getMenuByPositionKey(RibbonPosition.SUBMENU)`
- **AND** subMenu items are rendered in the subMenu area (between DefaultMenu and ribbon toolbar)
- **AND** each subMenu item is rendered using `ToolbarItem` component (not `MobileToolbarItem`)
- **AND** subMenu items respect their `hidden$` observable state
- **AND** subMenu items are displayed horizontally in the subMenu area

#### Scenario: SubMenu area layout
- **GIVEN** MobileRibbon is rendering with subMenu items
- **WHEN** subMenu items exist
- **THEN** the subMenu area is visible and positioned between ribbon tabs and toolbar content
- **AND** subMenu items are displayed in a flex container with appropriate spacing
- **AND** the subMenu area is hidden when no subMenu items are available

## ADDED Requirements

### Requirement: Mobile Keyboard Ribbon Menu Item

The mobile keyboard UI SHALL provide a menu item in the MobileRibbon subMenu area that allows users to toggle the mobile keyboard visibility.

#### Scenario: Keyboard toggle menu item registration
- **GIVEN** MobileKeyboardUIPlugin is initialized
- **WHEN** the plugin registers its menu schema
- **THEN** a menu item is registered at RibbonPosition.SUBMENU position
- **AND** the menu item uses `KeyboardToggleKeyboardOperation.id` as its command
- **AND** the menu item is configured with KeyboardIcon icon only (no title, tooltip, or custom label)

#### Scenario: Keyboard toggle menu item behavior
- **GIVEN** the keyboard toggle menu item is rendered in MobileRibbon subMenu
- **WHEN** user clicks the menu item
- **THEN** `KeyboardToggleKeyboardOperation` is executed
- **AND** the mobile keyboard visibility is toggled
- **AND** the menu item's activated state reflects keyboard visibility (`isKeyboardVisible$`)

#### Scenario: Keyboard toggle menu item state
- **GIVEN** the keyboard toggle menu item is rendered
- **WHEN** keyboard visibility changes
- **THEN** the menu item's activated state updates to reflect `isKeyboardVisible$` observable
- **AND** the menu item is disabled when `keyboardEnabled$` is false
- **AND** the menu item displays only the KeyboardIcon icon
