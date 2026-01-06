# sheets-hyper-link-mobile-ui Specification

## Purpose
TBD - created by archiving change add-mobile-hyper-link-ui. Update Purpose after archive.
## Requirements
### Requirement: Mobile Hyperlink Edit Sidebar

The system SHALL display hyperlink editing interface in a sidebar on mobile platforms instead of using canvas popups.

#### Scenario: Sidebar opens when editing hyperlink on mobile
- **GIVEN** the user is on a mobile device
- **WHEN** a hyperlink editing action is triggered (e.g., clicking edit button on a hyperlink)
- **THEN** the `ISidebarService.open()` method is called with `CellLinkEdit` component
- **AND** the sidebar slides in from the right side of the screen
- **AND** the sidebar displays the hyperlink edit form

#### Scenario: Sidebar displays edit component
- **GIVEN** the sidebar is open for hyperlink editing
- **WHEN** the sidebar renders
- **THEN** the `CellLinkEdit` component is rendered inside the sidebar content area
- **AND** all form fields (label, type, payload) are accessible and functional
- **AND** the component styling adapts to sidebar width constraints

#### Scenario: Sidebar closes when editing completes
- **GIVEN** the sidebar is open for hyperlink editing
- **WHEN** the user submits the form or clicks cancel
- **THEN** the sidebar is closed via `ISidebarService.close()`
- **AND** the editing state in `SheetsHyperLinkPopupService` is cleared

#### Scenario: Sidebar closes when clicking outside
- **GIVEN** the sidebar is open for hyperlink editing
- **WHEN** the user clicks the close button or outside the sidebar
- **THEN** the sidebar is closed
- **AND** any unsaved changes are discarded

#### Scenario: Mobile plugin registration
- **GIVEN** the mobile UI plugin is loaded
- **WHEN** `UniverSheetsHyperLinkMobileUIPlugin` is initialized
- **THEN** the mobile popup controller is registered in the dependency injection container
- **AND** the controller subscribes to `SheetsHyperLinkPopupService.currentEditing$` observable
- **AND** the controller responds to editing state changes by opening/closing the sidebar

#### Scenario: Desktop behavior unchanged
- **GIVEN** the user is on a desktop device
- **WHEN** a hyperlink editing action is triggered
- **THEN** the canvas popup service continues to be used
- **AND** the sidebar service is not invoked
- **AND** the existing desktop popup behavior remains unchanged

