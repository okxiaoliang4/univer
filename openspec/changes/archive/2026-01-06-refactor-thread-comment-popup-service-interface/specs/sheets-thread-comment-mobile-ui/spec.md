## MODIFIED Requirements

### Requirement: Mobile Thread Comment Sidebar

The system SHALL display thread comment interface in a sidebar on mobile platforms instead of using canvas popups.

#### Scenario: Sidebar opens when viewing thread comment on mobile
- **GIVEN** the user is on a mobile device
- **WHEN** a thread comment popup action is triggered (e.g., clicking a cell with comment)
- **THEN** the `ISidebarService.open()` method is called with `SheetsThreadCommentCell` component
- **AND** the sidebar slides in from the right side of the screen
- **AND** the sidebar displays the thread comment tree component
- **AND** the mobile service (`SheetsThreadCommentMobilePopupService`) handles sidebar display via `ISheetsThreadCommentPopupService` interface

#### Scenario: Mobile plugin registration
- **GIVEN** the mobile UI plugin is loaded
- **WHEN** `UniverSheetsThreadCommentMobileUIPlugin` is initialized
- **THEN** `ISheetsThreadCommentPopupService` token is bound to `SheetsThreadCommentMobilePopupService`
- **AND** `SheetsThreadCommentMobileHoverController` is registered (uses `currentClickedCell$` for touch interactions)
- **AND** `SheetsThreadCommentPopupController` is registered (shared with desktop, injects interface)
- **AND** the mobile hover controller subscribes to click events and triggers sidebar display

## ADDED Requirements

### Requirement: Mobile-specific hover controller

The system SHALL use a mobile-specific hover controller optimized for touch interactions.

#### Scenario: Mobile hover controller uses click events
- **GIVEN** the mobile plugin is loaded
- **WHEN** examining `SheetsThreadCommentMobileHoverController`
- **THEN** it subscribes to `currentClickedCell$` observable instead of `currentCell$`
- **AND** it injects `ISheetsThreadCommentPopupService` interface
- **AND** it triggers sidebar display on cell click/tap

#### Scenario: Mobile plugin registers mobile hover controller
- **GIVEN** the mobile plugin is loaded
- **WHEN** examining mobile plugin registration
- **THEN** `SheetsThreadCommentMobileHoverController` is registered
- **AND** desktop `SheetsThreadCommentHoverController` is NOT registered in mobile plugin
- **AND** mobile hover controller provides better touch interaction experience

