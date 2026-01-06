# find-replace-mobile-ui Specification

## Purpose
TBD - created by archiving change add-mobile-find-replace-ui. Update Purpose after archive.
## Requirements
### Requirement: Mobile Find-Replace Sidebar Implementation

The system SHALL render mobile find-replace components using the standardized `Sidebar` component from `@univerjs/ui` instead of Dialog components.

#### Scenario: Sidebar opens when find-replace is triggered
- **GIVEN** the mobile UI plugin is loaded
- **WHEN** a user triggers find-replace (via menu, shortcut, or command)
- **THEN** a sidebar opens from the right side of the screen
- **AND** the sidebar displays the find-replace interface
- **AND** the sidebar uses `ISidebarService.open()` with appropriate options

#### Scenario: Sidebar closes when find-replace session ends
- **GIVEN** a mobile find-replace sidebar is open
- **WHEN** the user closes the sidebar or the find-replace session terminates
- **THEN** the sidebar closes via `ISidebarService.close()`
- **AND** the find-replace service state is properly cleaned up

#### Scenario: Sidebar property mapping
- **GIVEN** find-replace state is available via `IFindReplaceService`
- **WHEN** the sidebar renders
- **THEN** the sidebar header displays the appropriate title (find or replace mode)
- **AND** the sidebar children render the `MobileFindReplace` component
- **AND** the sidebar width is set appropriately for mobile screens
- **AND** the sidebar `onClose` callback terminates the find-replace session

### Requirement: Mobile Find-Replace Component

The system SHALL provide a `MobileFindReplace` component that renders the find-replace UI optimized for mobile devices.

#### Scenario: Component renders find mode
- **GIVEN** find-replace is in find-only mode (`replaceRevealed` is false)
- **WHEN** the `MobileFindReplace` component renders
- **THEN** it displays the find input field
- **AND** it displays match count and position information
- **AND** it provides navigation controls (next/previous match)
- **AND** it provides an option to reveal replace mode

#### Scenario: Component renders replace mode
- **GIVEN** find-replace is in replace mode (`replaceRevealed` is true)
- **WHEN** the `MobileFindReplace` component renders
- **THEN** it displays both find and replace input fields
- **AND** it displays all find options (direction, scope, by, case sensitive, whole cell)
- **AND** it provides find, replace, and replace-all action buttons
- **AND** buttons are appropriately disabled based on state (empty find string, no matches, etc.)

#### Scenario: Component uses shared business logic hooks
- **GIVEN** shared hooks exist (`useFindReplaceLogic`, `useFindReplaceOptions`)
- **WHEN** the `MobileFindReplace` component is implemented
- **THEN** it uses `useFindReplaceLogic` hook for state management and event handlers
- **AND** it uses `useFindReplaceOptions` hook for option generation
- **AND** it does not duplicate business logic code from desktop implementation

### Requirement: Business Logic Code Reuse

The system SHALL ensure that business logic is shared between desktop and mobile implementations through reusable hooks, preventing code duplication. This requirement MUST be enforced to prevent scenarios where modifying one platform's business logic requires remembering to modify the other platform's code.

#### Scenario: Shared hooks extract business logic
- **GIVEN** find-replace business logic exists in desktop components
- **WHEN** mobile implementation is created
- **THEN** business logic is extracted into `useFindReplaceLogic` hook
- **AND** option generation logic is extracted into `useFindReplaceOptions` hook
- **AND** both desktop and mobile components use these hooks
- **AND** no duplicate business logic code exists
- **AND** all business logic changes are made in the hooks, not in individual components

#### Scenario: Desktop component uses shared hooks
- **GIVEN** shared hooks are created
- **WHEN** `FindReplaceDialog.tsx` is refactored
- **THEN** it uses `useFindReplaceLogic` hook instead of inline logic
- **AND** it uses `useFindReplaceOptions` hook for options
- **AND** desktop behavior remains unchanged
- **AND** code duplication is eliminated
- **AND** business logic is not duplicated in the component file

#### Scenario: Mobile component uses shared hooks
- **GIVEN** shared hooks exist
- **WHEN** `MobileFindReplace.tsx` is implemented
- **THEN** it uses `useFindReplaceLogic` hook for all business logic
- **AND** it uses `useFindReplaceOptions` hook for options
- **AND** it only contains UI-specific code (layout, styling)
- **AND** business logic changes automatically apply to both desktop and mobile
- **AND** no business logic is implemented directly in the mobile component

#### Scenario: Code reuse enforcement
- **GIVEN** business logic hooks exist for find-replace
- **WHEN** a developer needs to modify find-replace business logic
- **THEN** the modification MUST be made in the shared hook
- **AND** the modification automatically applies to both desktop and mobile
- **AND** developers MUST NOT add business logic directly to desktop or mobile components
- **AND** code review MUST verify that business logic is not duplicated between platforms

### Requirement: Mobile Controller Registration

The system SHALL register mobile-specific controllers only when the mobile UI plugin is available. The mobile controller SHALL be similar to the desktop controller but use Sidebar instead of Dialog, and register commands but not shortcuts.

#### Scenario: Mobile controller registers conditionally
- **GIVEN** a mobile plugin exists for find-replace
- **WHEN** the plugin initializes
- **THEN** it checks if mobile UI plugin is available
- **AND** if available, registers `MobileFindReplaceController`
- **AND** if not available, skips mobile controller registration
- **AND** desktop functionality is not affected

#### Scenario: Mobile controller registers commands but not shortcuts
- **GIVEN** `MobileFindReplaceController` is registered
- **WHEN** the controller initializes
- **THEN** it registers the same commands as desktop controller:
  - `OpenFindDialogOperation`
  - `OpenReplaceDialogOperation`
  - `GoToNextMatchOperation`
  - `GoToPreviousMatchOperation`
  - `ReplaceAllMatchesCommand`
  - `ReplaceCurrentMatchCommand`
  - `FocusSelectionOperation`
- **AND** it does NOT register shortcuts (mobile devices don't have keyboards)
- **AND** it registers menu schema (same as desktop)

#### Scenario: Mobile controller uses sidebar instead of dialog
- **GIVEN** `MobileFindReplaceController` is registered
- **WHEN** `IFindReplaceService.stateUpdates$` emits `revealed: true`
- **THEN** the controller opens the sidebar via `ISidebarService` instead of dialog
- **AND** when `revealed` becomes false or session terminates, the controller closes the sidebar
- **AND** the controller registers `MobileFindReplace` component with ComponentManager
- **AND** the controller handles sidebar lifecycle similar to how desktop controller handles dialog lifecycle

### Requirement: Mobile Component Registration

The system SHALL register the mobile find-replace component with ComponentManager for use in sidebar.

#### Scenario: Component registered with unique key
- **GIVEN** `MobileFindReplace` component exists
- **WHEN** the mobile plugin initializes
- **THEN** the component is registered with ComponentManager
- **AND** the component key is unique (e.g., `'univer.find-replace.mobile'`)
- **AND** the component can be referenced in sidebar options via `children.label`

### Requirement: Sheets Find-Replace Mobile Plugin

The system SHALL provide a mobile plugin for sheets find-replace that ensures sheets find-replace provider is available when mobile find-replace UI is used, and uses mobile-specific controller instead of desktop controller.

#### Scenario: Sheets mobile plugin depends on find-replace mobile plugin
- **GIVEN** `UniverSheetsFindReplaceMobilePlugin` exists
- **WHEN** the plugin is initialized
- **THEN** it depends on `UniverSheetsFindReplacePlugin` (sheets find-replace provider)
- **AND** it depends on `UniverFindReplaceMobilePlugin` (mobile find-replace UI)
- **AND** it depends on `UniverSheetsPlugin` (sheets core)
- **AND** it ensures sheets find-replace provider is registered before mobile UI tries to use it

#### Scenario: Sheets mobile controller uses sidebar instead of dialog
- **GIVEN** `SheetsFindReplaceMobileController` exists
- **WHEN** the mobile plugin initializes
- **THEN** it registers `SheetsFindReplaceMobileController` instead of desktop `SheetsFindReplaceController`
- **AND** the mobile controller does NOT depend on `FindReplaceController` (desktop dialog controller)
- **AND** the mobile controller uses `IFindReplaceService.terminate()` to close sidebar (via mobile sidebar controller)
- **AND** when editor is activated, the mobile controller terminates find-replace session (which closes sidebar)

#### Scenario: Sheets mobile plugin registration
- **GIVEN** mobile sheets find-replace plugin exists
- **WHEN** mobile find-replace UI is used with sheets
- **THEN** the sheets find-replace provider is available
- **AND** find-replace operations work correctly in sheets on mobile
- **AND** the mobile controller handles sidebar closing via `IFindReplaceService.terminate()`
- **AND** the plugin follows the same dependency pattern as other mobile plugins

