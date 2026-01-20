## ADDED Requirements

### Requirement: Optional Mutation Handling
The collaboration OT pipeline SHALL handle optional transform results without relying on sentinel mutation IDs.

#### Scenario: Skip absent mutation
- **WHEN** the transform service returns an absent mutation
- **THEN** the client skips applying it without additional filtering passes

### Requirement: Pre-Sync Composition
The client OT pipeline SHALL compose pending mutations before sending changesets when possible.

#### Scenario: Compose pending mutations
- **WHEN** multiple pending mutations are compatible for composition
- **THEN** the client sends the composed mutations to reduce payload size
