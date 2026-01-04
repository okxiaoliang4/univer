## Context

The mobile sidebar component (`MobileSidebar`) currently uses custom implementations for drawer-like behavior. The project has standardized on the `Drawer` component from `@univerjs/design`, which is built on top of the `vaul` library. This refactoring aligns the mobile component with the design system.

**Current State:**
- `MobileSidebar`: Uses custom `<section>` elements with manual CSS transitions and positioning

**Target State:**
- Component fully uses `Drawer` from `@univerjs/design`
- All existing properties remain compatible
- Behavior matches or improves upon current implementation

## Goals / Non-Goals

### Goals
- Refactor sidebar component to use `Drawer` component
- Maintain 100% API compatibility (no breaking changes)
- Preserve all existing functionality (scroll management, events, callbacks)
- Improve code maintainability and consistency

### Non-Goals
- Changing the public API (`ISidebarMethodOptions`)
- Modifying the service layer APIs
- Adding new features beyond what Drawer provides
- Supporting desktop sidebar (out of scope)
- Refactoring mobile dialog components (out of scope)

## Decisions

### Decision: Use Drawer Component from @univerjs/design

**Rationale:**
- Already in dependencies
- Provides gesture support, animations, and accessibility features
- Consistent with other mobile UI components (FunctionBrowser already uses it)
- Reduces maintenance burden

**Alternatives Considered:**
- Keep custom implementation: More maintenance, less consistent
- Use vaul directly: Bypasses design system abstraction

### Decision: Map `visible` to `open` for Sidebar

**Rationale:**
- Drawer uses `open` prop (standard React pattern)
- Sidebar uses `visible` prop (existing API)
- Map internally to maintain API compatibility

**Implementation:**
```typescript
<Drawer open={options?.visible} onOpenChange={handleOpenChange}>
```

### Decision: Use `direction="right"` for Sidebar

**Rationale:**
- Sidebar slides in from the right (matches current behavior)
- Drawer supports `direction` prop: "top" | "bottom" | "left" | "right"

### Decision: Preserve Scroll Container Management

**Rationale:**
- Sidebar service provides `setContainer()` and `getContainer()` methods
- Other code may depend on this API
- Need to attach ref to the scrollable element inside DrawerContent

**Implementation:**
- Attach scroll ref to the scrollable container inside `DrawerContent`
- May need to wrap content in a scrollable div if DrawerContent doesn't provide one

## Risks / Trade-offs

### Risk: Scroll Container Reference May Break

**Mitigation:**
- Test thoroughly that scroll container ref attaches correctly
- May need to wrap DrawerContent children in a scrollable div
- Verify `sidebarService.getContainer()` still works after refactor

### Risk: Scroll Events May Not Fire Correctly

**Mitigation:**
- Ensure scroll event listener attaches to correct element
- Test scroll event emission in sidebar service
- May need to attach listener to inner scrollable element

### Risk: Property Mapping Edge Cases

**Mitigation:**
- Document all mappings clearly
- Test each property individually
- Ask user for clarification on incompatible properties before implementation

### Trade-off: Less Control Over Animations

**Acceptance:**
- vaul provides standard animations
- Custom animations would require more work
- Standard animations are acceptable for consistency

## Migration Plan

### Phase 1: Analysis
1. Document all properties and their mappings
2. Identify incompatible properties
3. Get user confirmation on handling incompatible properties

### Phase 2: Implementation
1. Refactor `MobileSidebar`
2. Test sidebar thoroughly

### Phase 3: Validation
1. Run all existing tests
2. Manual testing on mobile devices
3. Visual regression testing (if applicable)
4. Performance testing

### Rollback Plan
- Keep old implementation in git history
- Can revert if critical issues found
- No database or data migration needed

## Open Questions

1. **Scroll Container**: Should the scroll container be the `DrawerContent` itself or an inner element? Need to verify DrawerContent structure.

2. **Sidebar Width**: Current sidebar uses `w-96` (384px) default. Should this be preserved or use Drawer's default width?

3. **onOpen Callback**: Sidebar has `onOpen` callback. Should this fire on `onOpenChange(true)` or separately? Current behavior needs verification.
