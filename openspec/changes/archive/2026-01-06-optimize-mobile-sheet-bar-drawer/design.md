## Context

The mobile sheet bar (`MobileSheetBar.tsx`) currently only provides basic sheet switching functionality. Users cannot access sheet management operations (rename, delete, copy, change color, protection settings) that are available on desktop via right-click context menu. This proposal adds a Drawer-based menu interface for mobile users.

## Goals / Non-Goals

### Goals
- Provide mobile users access to sheet context menu operations
- Use existing `Drawer` component for consistent mobile UI patterns
- Reuse existing `UIMenu` with `ContextMenuPosition.FOOTER_TABS` for consistency with desktop
- Maintain existing sheet switching behavior for inactive sheets

### Non-Goals
- Changing desktop sheet bar behavior
- Adding new menu options (using existing `FOOTER_TABS` menu schema)
- Creating a new Drawer component (using existing one from `@univerjs/design`)

## Decisions

### Decision: Use Drawer instead of DropdownMenu
- **Rationale**: Drawer provides better mobile UX with bottom-up animation, larger touch targets, and better visibility. The existing `SheetBarMenu` uses `DropdownMenu` but that's for a different use case (showing all sheets). For context menu operations, Drawer is more appropriate for mobile.
- **Alternatives considered**:
  - DropdownMenu: Too small for mobile, harder to interact with
  - Modal: Too intrusive, Drawer is more mobile-friendly

### Decision: Show dropdown icon only on active sheet
- **Rationale**: Clear visual indicator that the active sheet has additional options. Reduces visual clutter and follows common mobile UI patterns.
- **Alternatives considered**:
  - Show icon on all sheets: Too cluttered
  - No icon, always open drawer on click: Less discoverable

### Decision: Click active sheet to open drawer, click inactive to switch
- **Rationale**: Intuitive behavior - clicking active sheet opens options, clicking inactive switches sheets. Follows common mobile app patterns.
- **Alternatives considered**:
  - Separate button for drawer: Takes up space, less discoverable
  - Long-press to open drawer: Less discoverable, requires explanation

### Decision: Reuse existing UIMenu with FOOTER_TABS context
- **Rationale**: Consistency with desktop behavior, no need to maintain separate menu schemas, all existing commands work automatically.
- **Alternatives considered**:
  - Create new mobile-specific menu: More maintenance, potential inconsistency

### Decision: Drawer direction bottom
- **Rationale**: Standard mobile pattern for action menus, easy to dismiss with swipe down, doesn't obstruct sheet bar.
- **Alternatives considered**:
  - Top: Less common, harder to reach
  - Right/Left: Not appropriate for menu content

## Risks / Trade-offs

### Risk: Drawer might overlap with sheet content
- **Mitigation**: Drawer uses `max-h-[80vh]` and slides from bottom, leaving top portion visible. User can dismiss by tapping backdrop.

### Risk: Menu options might be too small for mobile
- **Mitigation**: UIMenu component should handle touch targets appropriately. If issues arise, can adjust menu item styling in follow-up.

### Risk: Performance impact of rendering menu
- **Mitigation**: Menu is only rendered when drawer is open, minimal performance impact. Menu schema is already loaded.

## Migration Plan

No migration needed - this is a new feature addition. Existing mobile sheet bar behavior remains unchanged for inactive sheets.

## Open Questions

- Should the drawer remember scroll position if reopened? (Probably not needed for menu)
- Should we add haptic feedback on mobile when opening drawer? (Nice to have, not required)
