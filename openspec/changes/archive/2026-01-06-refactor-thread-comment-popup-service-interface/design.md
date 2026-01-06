# Design: Refactor Thread Comment Popup Service to Interface-Based Architecture

## Overview

This design refactors the `SheetsThreadCommentPopupService` from a concrete class to an interface-based architecture with platform-specific implementations. This allows mobile and desktop to share the same controllers while using different UI mechanisms (sidebar vs canvas popup).

## Architecture Decision

### Current Architecture (Problematic)

```
Desktop Plugin:
  - SheetsThreadCommentPopupService (concrete class, canvas popup)
  - SheetsThreadCommentPopupController
  - SheetsThreadCommentHoverController

Mobile Plugin:
  - SheetsThreadCommentPopupService (same concrete class, but needs sidebar)
  - SheetsThreadCommentMobilePopupController (workaround)
  - SheetsThreadCommentMobileHoverController (workaround)
```

**Problems:**
- Mobile controllers duplicate logic from desktop controllers
- Service is tightly coupled to desktop implementation
- Cannot cleanly override behavior for mobile

### Proposed Architecture (Solution)

```
Interface:
  - ISheetsThreadCommentPopupService (interface + token)

Desktop Plugin:
  - ISheetsThreadCommentPopupService -> SheetsThreadCommentDesktopPopupService
  - SheetsThreadCommentPopupController (injects interface)
  - SheetsThreadCommentHoverController (injects interface, uses `currentCell$` for hover)

Mobile Plugin:
  - ISheetsThreadCommentPopupService -> SheetsThreadCommentMobilePopupService
  - SheetsThreadCommentPopupController (same, injects interface)
  - SheetsThreadCommentMobileHoverController (injects interface, uses `currentClickedCell$` for touch/click)
```

**Benefits:**
- Controllers are platform-agnostic (inject interface)
- Service implementations are platform-specific
- No code duplication in controllers
- Clean separation of concerns

## Interface Design

### ISheetsThreadCommentPopupService

```typescript
export interface ISheetsThreadCommentPopupService {
    /** Observable stream of active popup state */
    activePopup$: Observable<Nullable<IThreadCommentPopup>>;
    
    /** Get current active popup state */
    get activePopup(): Nullable<IThreadCommentPopup>;
    
    /** Show popup/sidebar with comment interface */
    showPopup(location: IThreadCommentPopup, onHide?: () => void): void;
    
    /** Hide popup/sidebar */
    hidePopup(): void;
    
    /** Make temporary popup persistent */
    persistPopup(): void;
}
```

### Redi Token

```typescript
export const ISheetsThreadCommentPopupService = createIdentifier<ISheetsThreadCommentPopupService>(
    'sheets-thread-comment-ui.sheets-thread-comment-popup.service'
);
```

## Implementation Details

### Desktop Implementation

`SheetsThreadCommentDesktopPopupService`:
- Uses `CellPopupManagerService` to show canvas popups
- Maintains existing behavior exactly as before
- No changes to popup logic, only renamed and implements interface

### Mobile Implementation

`SheetsThreadCommentMobilePopupService`:
- Uses `ISidebarService` to show sidebar instead of canvas popup
- Implements same interface methods
- Maps `showPopup()` to `sidebarService.open()` with `SHEETS_THREAD_COMMENT_MODAL` component
- Maps `hidePopup()` to `sidebarService.close()`
- Maintains `activePopup$` observable for state synchronization
- Sidebar ID: `'sheets-thread-comment-popup'`
- Sidebar title: Generated from cell reference (e.g., "Comment A1")

## Service Registration

### Desktop Plugin (`plugin.ts`)

```typescript
override onStarting(): void {
    ([
        // ... other dependencies
        [ISheetsThreadCommentPopupService, { useClass: SheetsThreadCommentDesktopPopupService }],
    ] as Dependency[]).forEach((dep) => {
        this._injector.add(dep);
    });
}
```

### Mobile Plugin (`mobile-plugin.ts`)

```typescript
override onStarting(): void {
    ([
        // ... other dependencies
        [ISheetsThreadCommentPopupService, { useClass: SheetsThreadCommentMobilePopupService }],
    ] as Dependency[]).forEach((dep) => {
        this._injector.add(dep);
    });
}
```

## Controller Updates

All controllers change from:
```typescript
@Inject(SheetsThreadCommentPopupService) private readonly _popupService: SheetsThreadCommentPopupService
```

To:
```typescript
@Inject(ISheetsThreadCommentPopupService) private readonly _popupService: ISheetsThreadCommentPopupService
```

Controllers remain unchanged otherwise - they work with both implementations via the interface.

## Component Updates

React components using `useDependency` change from:
```typescript
const popupService = useDependency(SheetsThreadCommentPopupService);
```

To:
```typescript
const popupService = useDependency(ISheetsThreadCommentPopupService);
```

## Migration Strategy

1. Extract interface and create token (non-breaking)
2. Rename existing service to desktop implementation (non-breaking if exported)
3. Create mobile implementation
4. Update plugin registrations
5. Update controllers/components to inject interface
6. Remove mobile controllers
7. Update exports

Each step can be done incrementally with tests passing at each stage.

## Testing Strategy

- Unit tests verify interface contract
- Unit tests for each implementation
- Integration tests verify plugin registration
- E2E tests verify desktop and mobile behavior
- Manual testing on both platforms

## Risks and Mitigations

**Risk**: Breaking changes if interface doesn't match existing API
**Mitigation**: Interface extracted directly from existing public API, no changes

**Risk**: Mobile implementation bugs
**Mitigation**: Comprehensive unit tests, integration tests, manual testing

**Risk**: Performance impact
**Mitigation**: No performance impact - same code paths, just different injection

## Future Considerations

- Could extend interface for platform-specific features if needed
- Could add more platform implementations (tablet, etc.)
- Interface pattern can be reused for other platform-specific services
