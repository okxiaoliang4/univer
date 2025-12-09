## 1. Implementation
- [ ] 1.1 Review existing pivot group toggle flow and graphics extension usage in render controllers.
- [ ] 1.2 Define and register a graphics renderer for pivot group expand/collapse buttons within pivot output cells.
- [ ] 1.3 Wire pointer/click handling to toggle row/column groups, trigger recalculation, and refresh rendering.
- [ ] 1.4 Add or update tests covering renderer drawing and group toggle interaction.
- [ ] 1.5 Document any renderer registration hooks or config flags needed for pivot UI.

## 2. Validation
- [ ] 2.1 Run `openspec validate add-pivot-ui-group-toggle-renderer --strict`
