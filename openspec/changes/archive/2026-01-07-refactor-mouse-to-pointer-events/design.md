## Context

The Univer codebase currently uses `onMouse*` events in React components, which work well on desktop but cause issues on mobile devices. Touch events don't trigger mouse events reliably, leading to interaction problems like the thread comment editor focus bug. The Pointer Events API provides a unified interface that handles mouse, touch, and pen inputs consistently.

**Current State:**
- React components use `onMouseUp`, `onMouseDown`, `onMouseMove`, etc.
- Touch interactions on mobile don't reliably trigger mouse events
- Some components already use pointer events in the rendering engine (`engine-render`)

**Target State:**
- All React components use `onPointer*` events instead of `onMouse*`
- Consistent behavior across desktop (mouse), mobile (touch), and pen inputs
- No regressions in existing desktop functionality

## Goals / Non-Goals

### Goals
- Replace all `onMouse*` event handlers with `onPointer*` equivalents
- Ensure mobile touch interactions work correctly
- Maintain backward compatibility with desktop mouse interactions
- Fix the thread comment editor focus issue on mobile
- Establish a standard for future event handling

### Non-Goals
- Changing event handling in the rendering engine (already uses pointer events)
- Modifying event handling in non-React code (unless necessary)
- Adding new pointer event features beyond basic mouse/touch compatibility
- Changing event propagation or bubbling behavior

## Decisions

### Decision: Use Pointer Events API

**Rationale:**
- Pointer Events API is the standard for unified input handling
- Supported in all modern browsers (including mobile)
- React provides full support for pointer events
- Already partially used in the rendering engine

**Alternatives Considered:**
- Keep mouse events and add touch event handlers: More code, harder to maintain, doesn't solve the problem
- Use only touch events: Breaks desktop functionality
- Custom event abstraction: Unnecessary complexity when Pointer Events API exists

### Decision: Direct Replacement Strategy

**Rationale:**
- `onPointerUp` is a direct replacement for `onMouseUp` with better compatibility
- Pointer events fire for both mouse and touch, so no conditional logic needed
- Simpler than maintaining separate mouse and touch handlers
- React's pointer events handle the differences automatically

**Implementation:**
- Direct 1:1 replacement: `onMouseUp` → `onPointerUp`, `onMouseDown` → `onPointerDown`, etc.
- No need to check `event.pointerType` unless specific behavior differs between input types
- Event properties (`preventDefault`, `stopPropagation`) work the same way

### Decision: Preserve Event Type Compatibility

**Rationale:**
- Some code may check `event.button` or `event.which` for specific behaviors
- Need to ensure these checks still work or are updated appropriately
- Pointer events provide `pointerType` to distinguish input methods if needed

**Implementation:**
- Update `event.button` checks to use `event.pointerType === 'mouse'` if needed
- Most cases won't need changes as pointer events are compatible with mouse events
- Test thoroughly to ensure no behavior regressions

## Risks / Trade-offs

### Risk: Event Property Differences
- **Risk**: Some code may rely on mouse-specific properties that don't exist in pointer events
- **Mitigation**: Thorough testing and updating property access where needed
- **Trade-off**: May need to add `pointerType` checks in some cases

### Risk: Browser Compatibility
- **Risk**: Older browsers may not support Pointer Events API
- **Mitigation**: Check browser support requirements (Univer requires modern browsers)
- **Trade-off**: May need polyfills if supporting very old browsers (unlikely)

### Risk: Performance Impact
- **Risk**: Pointer events might have different performance characteristics
- **Mitigation**: Pointer events are generally well-optimized in modern browsers
- **Trade-off**: Minimal, as pointer events are designed for performance

### Risk: Third-party Library Compatibility
- **Risk**: Some libraries might expect mouse events
- **Mitigation**: Test with all dependencies, update if necessary
- **Trade-off**: May need to wrap or adapt some library integrations

## Migration Plan

### Phase 1: High-Priority Components
1. Update thread comment editor (known issue)
2. Update rich text editor (core functionality)
3. Update UI component library (foundation)

### Phase 2: Feature Components
1. Update sheets-related components
2. Update docs-related components
3. Update slides and drawing components

### Phase 3: Testing and Validation
1. Comprehensive testing on desktop and mobile
2. Fix any issues discovered
3. Update documentation

### Rollback Plan
- Git history allows easy rollback if issues arise
- Changes are isolated to event handler names
- Can revert individual files if needed

## Open Questions

- Are there any components that specifically need mouse-only behavior?
- Should we add pointer type detection for any special cases?
- Do we need to update any event listener registrations outside React components?
