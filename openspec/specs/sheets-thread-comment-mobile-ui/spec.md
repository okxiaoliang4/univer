# sheets-thread-comment-mobile-ui Specification

## Purpose
TBD - created by archiving change add-mobile-thread-comment-ui. Update Purpose after archive.
## Requirements
### Requirement: Mobile Thread Comment Sidebar

The system SHALL display thread comment interface in a sidebar on mobile platforms instead of using canvas popups.

#### Scenario: Sidebar opens when viewing thread comment on mobile
- **GIVEN** the user is on a mobile device
- **WHEN** a thread comment popup action is triggered (e.g., hovering over a cell with comment or clicking comment indicator)
- **THEN** the `ISidebarService.open()` method is called with `SheetsThreadCommentCell` component
- **AND** the sidebar slides in from the right side of the screen
- **AND** the sidebar displays the thread comment tree component

#### Scenario: Sidebar displays comment component
- **GIVEN** the sidebar is open for thread comment viewing
- **WHEN** the sidebar renders
- **THEN** the `SheetsThreadCommentCell` component is rendered inside the sidebar content area
- **AND** all comment interactions (view, add, reply, edit, delete) are accessible and functional
- **AND** the component styling adapts to sidebar width constraints

#### Scenario: Sidebar closes when comment interaction completes
- **GIVEN** the sidebar is open for thread comment viewing
- **WHEN** the user clicks the close button or completes an interaction
- **THEN** the sidebar is closed via `ISidebarService.close()`
- **AND** the popup state in `SheetsThreadCommentPopupService` is cleared

#### Scenario: Sidebar closes when clicking outside
- **GIVEN** the sidebar is open for thread comment viewing
- **WHEN** the user clicks the close button or outside the sidebar
- **THEN** the sidebar is closed
- **AND** any temporary popup state is cleared

#### Scenario: Popup persistence in sidebar context
- **GIVEN** the sidebar is open with a temporary popup (triggered by hover)
- **WHEN** the user interacts with the comment (e.g., clicks on it)
- **THEN** the `persistPopup()` method is called
- **AND** the popup state transitions from temporary to persistent
- **AND** the sidebar remains open

#### Scenario: Mobile plugin registration
- **GIVEN** the mobile UI plugin is loaded
- **WHEN** `UniverSheetsThreadCommentMobileUIPlugin` is initialized
- **THEN** the mobile popup controller is registered in the dependency injection container
- **AND** the controller subscribes to `SheetsThreadCommentPopupService.activePopup$` observable
- **AND** the controller responds to popup state changes by opening/closing the sidebar

#### Scenario: Desktop behavior unchanged
- **GIVEN** the user is on a desktop device
- **WHEN** a thread comment popup action is triggered
- **THEN** the canvas popup service continues to be used
- **AND** the sidebar service is not invoked
- **AND** the existing desktop popup behavior remains unchanged

