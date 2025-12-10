## ADDED Requirements
### Requirement: Pivot total toggles in field areas

The pivot table editor SHALL expose grand-total toggles inside the rowFields and columnFields areas, mapping directly to the first row/column field `showSubTotals` flags (no separate total switches).

#### Scenario: Render row/column total toggles
- **WHEN** the pivot editor renders rowFields and columnFields panels
- **THEN** each panel displays a grand-total toggle (e.g., checkbox or switch) inside the panel header
- **AND** the toggle state mirrors the first row/column field `showSubTotals` value
- **AND** the toggle is disabled or hidden when no row/column field is configured.

#### Scenario: Toggle row grand total
- **WHEN** a user toggles the row grand-total control
- **THEN** the editor updates the first row field `showSubTotals`
- **AND** triggers recalculation/render so row grand totals appear or disappear accordingly
- **AND** subtotal flags on deeper row fields remain unchanged.

#### Scenario: Toggle column grand total
- **WHEN** a user toggles the column grand-total control
- **THEN** the editor updates the first column field `showSubTotals`
- **AND** triggers recalculation/render so column grand totals (per value field) appear or disappear accordingly
- **AND** subtotal flags on deeper column fields remain unchanged.

