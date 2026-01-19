## ADDED Requirements
### Requirement: Compose sheet mutations
The OT wasm layer SHALL provide a compose API that combines two sequential sheet mutations when they target the same sheet and share the same mutation type.

#### Scenario: Compose compatible operations
- **WHEN** two mutations share the same id and target the same unit/sub-unit
- **THEN** the compose API returns a list containing the composed mutation

#### Scenario: Compose incompatible operations
- **WHEN** two mutations have different ids or different unit/sub-unit targets
- **THEN** the compose API returns the original two mutations unchanged

### Requirement: Compose list of mutations
The OT wasm layer SHALL provide a compose_list API that applies compose sequentially over a list of mutations, returning the minimized list.

#### Scenario: Compose list with adjacent compatible operations
- **WHEN** a list contains adjacent mutations with the same id and target
- **THEN** the compose_list API returns a shortened list with composed mutations

#### Scenario: Compose list without compatible operations
- **WHEN** a list contains no adjacent compatible mutations
- **THEN** the compose_list API returns the original list
