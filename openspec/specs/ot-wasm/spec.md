# ot-wasm Specification

## Purpose
TBD - created by archiving change refactor-ot-wasm-ot-pipeline. Update Purpose after archive.
## Requirements
### Requirement: Optional Transform Results
The OT WASM transform API SHALL represent no-op results as absent mutations rather than sentinel mutation IDs.

#### Scenario: No-op transform
- **WHEN** a transform operation yields no effective mutation
- **THEN** the corresponding `m1_prime` or `m2_prime` is absent in the result

#### Scenario: Non-noop transform
- **WHEN** a transform operation yields an effective mutation
- **THEN** the result includes a concrete mutation object with `id` and `params`

### Requirement: Optional Results in List Transforms
The OT WASM list transform SHALL omit absent mutations from its output lists.

#### Scenario: Drop no-ops from list
- **WHEN** a list transform produces absent mutations
- **THEN** the output lists exclude those entries and preserve relative order of remaining mutations

### Requirement: Compose APIs
The OT WASM layer SHALL expose compose and compose_list APIs that merge compatible mutations without altering semantic ordering.

#### Scenario: Compose compatible operations
- **WHEN** two consecutive mutations are composable
- **THEN** the compose API returns a single merged mutation

#### Scenario: Compose incompatible operations
- **WHEN** two consecutive mutations are not composable
- **THEN** the compose API returns both mutations in order

### Requirement: Mutation Parameter Ownership
Mutation parameter types SHALL live in per-mutation modules, while shared primitives remain in `types.rs`.

#### Scenario: Per-mutation params
- **WHEN** a mutation-specific parameter type is required
- **THEN** it is defined under `packages/univer-ot-wasm/src/mutations/<mutation>/`

### Requirement: OT Performance Constraints
The OT pipeline SHALL avoid extra allocations across the WASM boundary by minimizing serialization round-trips and cloning.

#### Scenario: Transform list conversion
- **WHEN** transforming mutation lists at the WASM boundary
- **THEN** the implementation avoids JSON stringify/parse fallbacks for valid inputs

