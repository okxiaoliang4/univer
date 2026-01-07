# ui-mobile-components Specification

## Purpose
TBD - created by archiving change refactor-mobile-components-to-drawer. Update Purpose after archive.
## Requirements
### Requirement: Mobile Sidebar Drawer Implementation

The system SHALL render mobile sidebar components using the standardized `Drawer` component from `@univerjs/design`.

#### Scenario: Sidebar uses Drawer component
- **GIVEN** a mobile sidebar is being rendered
- **WHEN** the sidebar component mounts
- **THEN** it uses the `Drawer` component from `@univerjs/design`
- **AND** the Drawer has `direction="right"` to slide from the right side
- **AND** the Drawer uses `DrawerContent`, `DrawerHeader`, `DrawerTitle`, `DrawerFooter`, and `DrawerClose` sub-components

#### Scenario: Sidebar property mapping
- **GIVEN** sidebar options are provided via `ISidebarMethodOptions`
- **WHEN** the sidebar renders
- **THEN** the `visible` property is mapped to Drawer's `open` prop
- **AND** the `width` property is applied to `DrawerContent` via style prop
- **AND** the `header` property is rendered inside `DrawerHeader` with `DrawerTitle`
- **AND** the `children` property is rendered inside `DrawerContent`
- **AND** the `footer` property is rendered inside `DrawerFooter` (if provided)
- **AND** the `bodyStyle` property is applied to `DrawerContent` via style prop
- **AND** the `onClose` callback is mapped to Drawer's `onOpenChange` callback
- **AND** the `onOpen` callback is triggered when drawer opens (via `onOpenChange(true)`)

#### Scenario: Sidebar scroll container management
- **GIVEN** a mobile sidebar is rendered using Drawer
- **WHEN** the sidebar mounts
- **THEN** the scroll container reference is set via `sidebarService.setContainer()`
- **AND** the reference points to the scrollable element inside `DrawerContent`
- **AND** `sidebarService.getContainer()` returns the correct element
- **AND** when the sidebar unmounts, the container reference is cleared

#### Scenario: Sidebar scroll events
- **GIVEN** a mobile sidebar is rendered using Drawer
- **WHEN** the user scrolls within the sidebar content
- **THEN** scroll events are emitted via `sidebarService.scrollEvent$`
- **AND** the scroll event listener is attached to the scrollable element inside `DrawerContent`

#### Scenario: Sidebar close button
- **GIVEN** a mobile sidebar is rendered using Drawer
- **WHEN** the sidebar header is displayed
- **THEN** a `DrawerClose` button is rendered in the header
- **AND** clicking the close button triggers the `onClose` callback
- **AND** the sidebar's `visible` state is set to `false`

### Requirement: Mobile Sidebar Rendering

The mobile sidebar component SHALL use the `Drawer` component for rendering instead of custom `<section>` elements, while maintaining all existing functionality and API compatibility.

#### Scenario: Sidebar rendering structure
- **GIVEN** a mobile sidebar is being rendered
- **WHEN** the sidebar component renders
- **THEN** it uses `Drawer` as the root component instead of custom `<section>` elements
- **AND** all existing properties (`visible`, `width`, `header`, `children`, `footer`, `bodyStyle`, `onClose`, `onOpen`) continue to work as before
- **AND** the visual appearance and behavior match or improve upon the previous implementation

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

