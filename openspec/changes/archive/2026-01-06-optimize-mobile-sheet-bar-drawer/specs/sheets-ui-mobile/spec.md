## MODIFIED Requirements

### Requirement: Mobile Sheet Bar Display

The system SHALL display a mobile-optimized sheet bar component that allows users to switch between sheets and access sheet management operations.

#### Scenario: Sheet bar displays all visible sheets
- **GIVEN** a workbook with multiple sheets
- **WHEN** the mobile sheet bar renders
- **THEN** all non-hidden sheets are displayed as tabs
- **AND** each tab shows the sheet name
- **AND** the active sheet tab is visually highlighted

#### Scenario: Active sheet displays dropdown indicator
- **GIVEN** a sheet is currently active
- **WHEN** the mobile sheet bar renders
- **THEN** the active sheet tab displays a `MoreDownIcon` dropdown arrow icon on the right side of the label
- **AND** inactive sheet tabs do NOT display the dropdown icon
- **AND** the icon is properly sized and positioned relative to the sheet name

#### Scenario: Switch to inactive sheet
- **GIVEN** the mobile sheet bar is displayed with multiple sheets
- **WHEN** the user taps on an inactive sheet tab
- **THEN** the tapped sheet becomes active
- **AND** the previously active sheet becomes inactive
- **AND** the dropdown icon moves to the newly active sheet tab
- **AND** the sheet content switches to show the newly active sheet

#### Scenario: Open drawer from active sheet
- **GIVEN** a sheet is currently active and displays the dropdown icon
- **WHEN** the user taps on the active sheet tab
- **THEN** a `Drawer` component opens from the bottom of the screen
- **AND** the drawer displays the sheet name in the `DrawerTitle` within `DrawerHeader`
- **AND** the drawer contains a `DrawerClose` button in the header
- **AND** the drawer content displays the `UIMenu` with `menuType={ContextMenuPosition.FOOTER_TABS}`

#### Scenario: Drawer displays sheet context menu
- **GIVEN** the drawer is open from the active sheet
- **WHEN** the drawer content renders
- **THEN** the `UIMenu` displays all menu items configured for `ContextMenuPosition.FOOTER_TABS`
- **AND** menu items include: Delete Sheet, Copy Sheet, Rename Sheet, Change Color, Hide Sheet, Show Menu List, and Protection options (if applicable)
- **AND** menu items are properly styled for mobile touch interaction

#### Scenario: Execute menu command from drawer
- **GIVEN** the drawer is open and displaying menu options
- **WHEN** the user selects a menu option (e.g., Rename Sheet)
- **THEN** the corresponding command is executed via `commandService.executeCommand`
- **AND** the command receives the current active sheet ID as `subUnitId` parameter
- **AND** the drawer closes automatically after command execution

#### Scenario: Close drawer without action
- **GIVEN** the drawer is open
- **WHEN** the user taps the backdrop overlay
- **THEN** the drawer closes
- **AND** no command is executed
- **WHEN** the user taps the `DrawerClose` button
- **THEN** the drawer closes
- **AND** no command is executed
- **WHEN** the user swipes down on the drawer
- **THEN** the drawer closes
- **AND** no command is executed

#### Scenario: Sheet list updates after menu operations
- **GIVEN** the drawer is open and a menu operation is performed (e.g., rename, delete, hide)
- **WHEN** the command completes
- **THEN** the mobile sheet bar updates to reflect the changes
- **AND** if a sheet was deleted, it disappears from the sheet list
- **AND** if a sheet was renamed, the tab label updates
- **AND** if a sheet was hidden, it disappears from the sheet list
- **AND** the drawer closes

#### Scenario: Drawer state management
- **GIVEN** the mobile sheet bar component is mounted
- **WHEN** the user switches to a different sheet while drawer is open
- **THEN** the drawer closes automatically
- **AND** the new sheet becomes active
- **WHEN** the drawer is open and a sheet operation changes the active sheet
- **THEN** the drawer closes and the sheet bar updates accordingly
