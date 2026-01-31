---
name: univer-ot-dev
description: Develop Operational Transformation (OT) mutations and transforms for Univer collaborative editing. Use when implementing new mutations in packages/univer-ot-wasm/crates/ot-core/src/mutations/, creating transforms in packages/univer-ot-wasm/crates/ot-core/src/transforms/, analyzing conflict dimensions (unitId, subUnitId, row/col/range, feature plugin IDs), implementing conflict resolution strategies (identity, shifting, LWW, conflict scope, removal), ensuring 100% transform coverage, maintaining zero-copy optimization patterns, or following TDD workflow for OT development.
---

# Univer OT Development

Develop mutations and transforms for Univer's Operational Transformation system with comprehensive conflict resolution.

## Quick Reference

**Codebase locations:**
- Mutations: `packages/univer-ot-wasm/crates/ot-core/src/mutations/`
- Transforms: `packages/univer-ot-wasm/crates/ot-core/src/transforms/`
- Utilities: `packages/univer-ot-wasm/crates/ot-core/src/utils/`
- Tests: `packages/univer-ot-wasm/crates/ot-core/tests/`

**Key files:**
- `transforms/constants.rs` - All mutation ID constants
- `transforms/mod.rs` - Main transform registration
- `utils/shift.rs` - Zero-copy shift utilities
- `utils/params.rs` - Quick parameter extraction utilities

**Skill reference docs:**
- `.claude/skills/univer-ot-dev/references/mutations/` - Per-mutation conflict analysis documents
- `.claude/skills/univer-ot-dev/references/mutations/TEMPLATE.md` - Document template

## Directory Structure

**CRITICAL**: The `mutations/` and `transforms/` directories must be perfectly synchronized. Every module in mutations must have a corresponding module in transforms.

**Current module structure (14 modules)**:
```
mutations/                    transforms/
├── data_validation/          ├── data_validation/
├── docs/                     ├── docs/
├── docs_hyper_link/          ├── docs_hyper_link/
├── engine_formula/           ├── engine_formula/
├── sheets/                   ├── sheets/
├── sheets_conditional_       ├── sheets_conditional_
│   formatting/               │   formatting/
├── sheets_drawing/           ├── sheets_drawing/
├── sheets_filter/            ├── sheets_filter/
├── sheets_hyper_link/        ├── sheets_hyper_link/
├── sheets_note/              ├── sheets_note/
├── sheets_numfmt/            ├── sheets_numfmt/
├── sheets_pivot_table/       ├── sheets_pivot_table/
├── sheets_table/             ├── sheets_table/
└── thread_comment/           └── thread_comment/
```

**Module naming rules:**
- Use snake_case for all directory names
- Feature plugins use prefix: `sheets_`, `docs_`, etc.
- Core plugins have no prefix: `data_validation`, `thread_comment`
- Each module has a `mod.rs` with `register_transforms()` function

**When adding new mutations:**
1. Create mutation in `mutations/[module]/`
2. Create corresponding transform module in `transforms/[module]/`
3. Add module declaration to both `mutations/mod.rs` and `transforms/mod.rs`
4. Register transform in `transforms/mod.rs::register_all()`

## Core Principles

### 1. Mutation-Transform Correspondence
**CRITICAL**: Mutation IDs and params must exactly match between mutations/ and transforms/. Never modify mutation IDs arbitrarily.

### 2. Conflict Dimension Granularity
Resolve conflicts at the finest granularity possible:
- **Cell-level**: Only conflicting cells use LWW, preserve non-conflicting cells
- **Row/Column-level**: Operations on different rows/columns are independent
- **Range-level**: Partial overlaps adjust boundaries, preserve non-overlapping portions

### 3. Zero-Copy Optimization
Prioritize performance:
- Early worksheet checks before parsing (zero clones)
- Use `std::mem::take()` for HashMap shifts
- Return identity results when possible (references, not clones)
- Conditional cloning only when modification needed

### 4. Comprehensive Coverage
Every mutation must have transforms with ALL other mutations. Use TDD to ensure 100% coverage.

## TDD Workflow for OT Development

**CRITICAL**: Each mutation MUST have a comprehensive conflict analysis document in `references/mutations/[mutation_name].md`. This document serves as:
- Complete transform coverage checklist
- Conflict resolution reference
- Implementation roadmap
- Team knowledge base

### Step 1: Create/Update Mutation

**When to do**: Adding new feature or modifying existing mutation

**File naming convention**: Use snake_case with `_mutation` suffix
- Example: `insert_row_col_mutation.rs`, `set_range_values_mutation.rs`

**Process:**
1. Define mutation in `mutations/[module]/[operation]_mutation.rs`
2. Follow standard pattern (see [mutation-patterns.md](references/mutation-patterns.md))
3. Add module declaration in module's `mod.rs`
4. Add `pub use` export in module's `mod.rs`
5. Add to `transforms/constants.rs` if core mutation

**Validation:**
- [ ] File name uses snake_case with `_mutation.rs` suffix
- [ ] Mutation ID follows `[type].mutation.[kebab-case]` format
- [ ] `pub const ID: &'static str` defined in mutation struct impl
- [ ] Params struct has all required fields (unitId, subUnitId, etc.)
- [ ] Optional fields use `Option<T>` with `skip_serializing_if`
- [ ] Module declared in `mod.rs`: `pub mod [name]_mutation;`
- [ ] Exported in `mod.rs`: `pub use [name]_mutation::*;`
- [ ] Listed in constants if core mutation
- [ ] **Conflict analysis document created** in `.claude/skills/univer-ot-dev/references/mutations/[name].md`

### Step 2: Create/Update Conflict Analysis Document

**MANDATORY**: Create or update the conflict analysis document in `.claude/skills/univer-ot-dev/references/mutations/[mutation_name].md`

Use the template at `references/mutations/TEMPLATE.md` and complete:

1. **Mutation Overview**: Purpose, affected dimensions, parameters
2. **Conflict Dimensions Analysis**: Analyze all 4 dimensions
3. **Transform Coverage Matrix**: List ALL mutations with status and strategy
4. **Detailed Conflict Resolution**: For each non-identity transform, document:
   - Conflict analysis
   - Resolution strategy
   - Implementation approach
   - Test cases

**Why this document is critical**:
- Ensures no mutations are forgotten
- Provides implementation roadmap
- Serves as team reference
- Tracks progress towards 100% coverage

**See**:
- [conflict-dimensions.md](references/conflict-dimensions.md) for detailed analysis framework
- [mutations/TEMPLATE.md](references/mutations/TEMPLATE.md) for document template

### Step 3: Analyze Conflict Dimensions

**For each mutation in the coverage matrix**, analyze conflicts across 4 dimensions:

**Dimension 1**: unitId (workbook)
- Different workbooks → Identity

**Dimension 2**: subUnitId (worksheet)
- Different worksheets → Identity

**Dimension 3**: Row/Col/Range position
- No overlap → Identity
- Structural mutation (insert/remove) → Shifting
- Same position → LWW or conflict scope

**Dimension 4**: Feature plugin ID
- Different feature types → Typically identity
- Same feature, same ID → LWW
- Same feature, different IDs → Check position

**Document findings** in the mutation's conflict analysis document.

### Step 3: Plan Transform Strategy

For each mutation pair, determine resolution strategy:

1. **Identity** (~40%) - Different worksheets or no interaction
2. **Shifting** - Insert/remove affects positions
3. **LWW** - Same cell/property conflict
4. **Conflict Scope** - Independent rows/columns
5. **Removal** - One negates the other

**See [resolution-strategies.md](references/resolution-strategies.md) for implementation patterns**

**Update the conflict analysis document** with chosen strategies for each mutation pair.

### Step 5: Write Tests FIRST (TDD)

**Before implementing transform**, write comprehensive tests in `tests/`:

**File naming**: Use snake_case matching mutation name (e.g., `insert_row_tests.rs`, `set_range_values_tests.rs`)

**CRITICAL**: Always use `Mutation::ID` constants, never hardcode string literals.

```rust
use ot_core::{MutationInfo, TransformService};
use ot_core::mutations::sheets::{InsertRowMutation, SetRangeValuesMutation};
use serde_json::json;

#[test]
fn test_insert_row_vs_set_range_values_shift() {
    let service = TransformService::new();

    // CORRECT: Use Mutation::ID constant
    let m1 = MutationInfo {
        id: InsertRowMutation::ID.to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 5,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: SetRangeValuesMutation::ID.to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "10": {
                    "0": { "v": "test" }
                }
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.error.is_none());
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());

    // Verify row 10 was shifted to row 11
    let m2_prime = result.m2_prime.unwrap();
    assert!(m2_prime.params["cellValue"]["11"].is_object());
}
```

**WRONG - Never do this:**
```rust
let m1 = MutationInfo {
    id: "sheet.mutation.insert-row".to_string(),  // ❌ Hardcoded string
    // ...
};
```

**Test scenarios for each pair:**
- [ ] Different worksheets (identity)
- [ ] Same worksheet, no conflict (identity or independent)
- [ ] Same worksheet, conflict (resolution strategy)
- [ ] Edge cases (boundaries, complete overlap, partial overlap)
- [ ] Multiple rows/columns/cells

**Update the conflict analysis document** with test case status.

### Step 6: Implement Transform

**File naming**: Use snake_case matching mutation name (e.g., `insert_row.rs`, `set_range_values.rs`)
- Transform for `insert_row_col_mutation.rs` → `transforms/sheets/insert_row.rs`
- Transform for `set_range_values_mutation.rs` → `transforms/sheets/set_range_values.rs`

Create transform in `transforms/[module]/[operation].rs`:

**Standard structure:**
```rust
use crate::mutations::sheets::{
    InsertRowMutation, InsertRowMutationParams,
    SetRangeValuesMutation, SetRangeValuesMutationParams,
};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::params::same_worksheet;
use crate::utils::shift::shift_row_keys_for_insert;
use std::sync::Arc;

pub const MUTATION_ID: MutationId = InsertRowMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform
    registry.register_symmetric_ref(MUTATION_ID, create_self_transform());

    // Bidirectional transforms with other mutations
    registry.register_bidirectional_ref(
        SetRangeValuesMutation::ID,
        MUTATION_ID,
        create_insert_row_vs_set_range_values(),
    );

    // Identity transforms (batch)
    registry.register_identity(MUTATION_ID, AddWorksheetMergeMutation::ID);
}

fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Early worksheet check
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        // Parse params
        let m1_params: InsertRowMutationParams =
            serde_json::from_value(m1.params.clone())
                .map_err(|_| parse_error(m1, m2, "Failed to parse m1 params"))?;

        let mut m2_params: InsertRowMutationParams =
            serde_json::from_value(m2.params.clone())
                .map_err(|_| parse_error(m1, m2, "Failed to parse m2 params"))?;

        // Implement conflict resolution
        let m1_start = m1_params.range.start_row;
        let m1_count = m1_params.range.end_row - m1_start + 1;

        if m2_params.range.start_row >= m1_start {
            m2_params.range.start_row += m1_count;
            m2_params.range.end_row += m1_count;
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}
```

**Optimization checklist:**
- [ ] Early worksheet check before parsing
- [ ] Use shift utilities from `utils/shift.rs`
- [ ] Conditional cloning only when needed
- [ ] Return identity when possible
- [ ] Handle all edge cases (overlap detection)

**Update the conflict analysis document** with implementation details and code snippets.

### Step 7: Run Tests and Iterate

```bash
cd packages/univer-ot-wasm/crates/ot-core
cargo test [test_name]
```

**Iterate until:**
- [ ] All tests pass
- [ ] Coverage includes all scenarios
- [ ] Zero-copy patterns applied
- [ ] No unnecessary cloning

**Update the conflict analysis document** status (✅ Implemented) for completed transforms.

### Step 8: Register in Module

**File naming**: Module file name should match the transform operation name in snake_case

**In `transforms/[module]/mod.rs`:**
```rust
// Add module declaration (alphabetical order preferred)
pub mod insert_row;
pub mod set_range_values;
// ... other modules

// Register all transforms in this module
pub fn register_transforms(registry: &mut TransformRegistry) {
    // ... existing registrations
    insert_row::register_transforms(registry);
    set_range_values::register_transforms(registry);
}
```

**In `transforms/mod.rs` (if new feature module):**
```rust
// Module declarations (must match mutations/ directory exactly)
pub mod constants;

// Core sheets transforms
pub mod sheets;

// Feature plugin transforms (alphabetical order, matching mutations/mod.rs)
pub mod data_validation;
pub mod docs;
pub mod docs_hyper_link;
pub mod engine_formula;
pub mod sheets_conditional_formatting;
pub mod sheets_drawing;
pub mod sheets_filter;
pub mod sheets_hyper_link;
pub mod sheets_note;
pub mod sheets_numfmt;
pub mod sheets_pivot_table;
pub mod sheets_table;
pub mod thread_comment;

// Register all transforms
pub fn register_all(registry: &mut TransformRegistry) {
    // Core sheets
    sheets::register_transforms(registry);

    // Feature plugins (alphabetical order)
    data_validation::register_transforms(registry);
    docs::register_transforms(registry);
    docs_hyper_link::register_transforms(registry);
    engine_formula::register_transforms(registry);
    sheets_conditional_formatting::register_transforms(registry);
    sheets_drawing::register_transforms(registry);
    sheets_filter::register_transforms(registry);
    sheets_hyper_link::register_transforms(registry);
    sheets_note::register_transforms(registry);
    sheets_numfmt::register_transforms(registry);
    sheets_pivot_table::register_transforms(registry);
    sheets_table::register_transforms(registry);
    thread_comment::register_transforms(registry);
}
```

**File structure alignment with mutations**:
```
mutations/sheets/insert_row_col_mutation.rs
  → transforms/sheets/insert_row.rs
  → tests/insert_row_tests.rs

mutations/sheets/set_range_values_mutation.rs
  → transforms/sheets/set_range_values.rs
  → tests/set_range_values_tests.rs
```

### Step 9: Coverage Validation

Ensure transform coverage for new mutation:

**Manual checklist** (or use coverage script):
- [ ] Transforms with all core sheets mutations (53 total)
- [ ] Transforms with relevant feature plugins
- [ ] Self-transform if needed
- [ ] All registered in module's `register_transforms()`
- [ ] **Conflict analysis document updated** with all implementation statuses

**Run full test suite:**
```bash
cargo test --all
```

**Verify documentation completeness**:
- [ ] All mutations in coverage matrix have status (✅/🚧/⏳/❌)
- [ ] Detailed conflict resolution documented for non-identity transforms
- [ ] Implementation checklist complete
- [ ] Last Updated section filled in

## Common Implementation Patterns

### Pattern 1: Simple Identity (Different Worksheets)

```rust
fn create_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        // Same worksheet, but operations don't interfere
        identity(m1, m2)
    })
}
```

### Pattern 2: Insert Row Shifts Target

```rust
use crate::mutations::sheets::{InsertRowMutation, InsertRowMutationParams, SetRangeValuesMutationParams};
use crate::utils::shift::shift_row_keys_for_insert;

fn create_insert_row_vs_set_range_values() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams =
            serde_json::from_value(m1.params.clone())
                .map_err(|_| parse_error(m1, m2, "Failed to parse m1"))?;

        let m2_params: SetRangeValuesMutationParams =
            serde_json::from_value(m2.params.clone())
                .map_err(|_| parse_error(m1, m2, "Failed to parse m2"))?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        let mut m2_prime = m2_params.clone();

        // Shift cell value HashMap keys
        if let Some(cell_value) = &mut m2_prime.cell_value {
            shift_row_keys_for_insert(cell_value, insert_start, insert_count);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_prime).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}
```

### Pattern 3: Cell-Level LWW

```rust
use crate::mutations::sheets::{SetRangeValuesMutation, SetRangeValuesMutationParams};

fn create_set_values_vs_set_values() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: SetRangeValuesMutationParams =
            serde_json::from_value(m1.params.clone())
                .map_err(|_| parse_error(m1, m2, "Failed to parse m1"))?;

        let m2_params: SetRangeValuesMutationParams =
            serde_json::from_value(m2.params.clone())
                .map_err(|_| parse_error(m1, m2, "Failed to parse m2"))?;

        let mut m1_prime = m1_params.clone();

        if let (Some(m1_cells), Some(m2_cells)) = (&m1_params.cell_value, &m2_params.cell_value) {
            let mut m1_prime_cells = m1_cells.clone();

            // Remove conflicting cells from m1 (cell-level granularity)
            for (row_key, m1_row) in m1_cells {
                if let Some(m2_row) = m2_cells.get(row_key) {
                    let mut m1_prime_row = m1_row.clone();

                    for col_key in m1_row.keys() {
                        if m2_row.contains_key(col_key) {
                            m1_prime_row.remove(col_key);  // LWW: m2 wins on this cell
                        }
                    }

                    if m1_prime_row.is_empty() {
                        m1_prime_cells.remove(row_key);
                    } else {
                        m1_prime_cells.insert(row_key.clone(), m1_prime_row);
                    }
                }
            }

            m1_prime.cell_value = if m1_prime_cells.is_empty() {
                None
            } else {
                Some(m1_prime_cells)
            };
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Modified(MutationInfo {
                id: m1.id.clone(),
                params: serde_json::to_value(m1_prime).unwrap_or_else(|_| m1.params.clone()),
            }),
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

## Anti-Patterns to Avoid

### ❌ Don't: Arbitrary Mutation ID Changes
```rust
// WRONG - breaks existing transforms
pub const ID: &'static str = "sheet.action.set-values-new";
```

### ❌ Don't: Parse Before Worksheet Check
```rust
// WRONG - wastes performance
let m1_params = parse(m1)?;
let m2_params = parse(m2)?;

if !same_worksheet(&m1_params, &m2_params) {
    return identity(m1, m2);
}
```

### ❌ Don't: Always Clone
```rust
// WRONG - unnecessary clone
let m2_prime = m2_params.clone();
// ... no modifications
TransformResultRef {
    m2_prime: MutationOutcome::Modified(serialize(m2_prime)),
    // ...
}
```

### ❌ Don't: Coarse-Grained Conflict Resolution
```rust
// WRONG - removes entire range when only one cell conflicts
if ranges_overlap(&m1_params.range, &m2_params.range) {
    return TransformResultRef {
        m1_prime: MutationOutcome::Removed,  // Too coarse!
        // ...
    };
}
```

## Reference Documentation

**Detailed guides in `references/`:**

- **[conflict-dimensions.md](references/conflict-dimensions.md)** - Complete guide to analyzing conflicts across 4 dimensions
- **[resolution-strategies.md](references/resolution-strategies.md)** - The 5 conflict resolution strategies with implementation patterns
- **[mutation-patterns.md](references/mutation-patterns.md)** - Standard mutation definition patterns and examples

**Per-mutation conflict analysis** in `references/mutations/`:**

- **[TEMPLATE.md](references/mutations/TEMPLATE.md)** - Template for creating mutation conflict analysis documents
- Each mutation should have its own `[mutation_name].md` documenting:
  - Complete conflict analysis with all other mutations
  - Transform coverage matrix with implementation status
  - Detailed resolution strategies
  - Test case checklists

**Purpose**: These documents ensure 100% transform coverage and serve as implementation roadmaps.

## Quick Commands

```bash
# Run specific test
cargo test test_insert_row_vs_set_range_values

# Run all tests for a module
cargo test insert_row

# Run with output
cargo test test_name -- --nocapture

# Check format
cargo fmt --check

# Run clippy
cargo clippy
```

## Coverage Goals

For each new mutation, ensure:

- [ ] Self-transform if needed (mutation vs itself)
- [ ] Transforms with all 53 core sheets mutations
- [ ] Transforms with relevant feature plugins
- [ ] Identity transforms for unrelated mutations
- [ ] Bidirectional registration (forward + reverse)
- [ ] Comprehensive test coverage (all scenarios)
- [ ] Zero-copy optimizations applied
- [ ] All tests passing

## Transform Registration Checklist

- [ ] Create transform file in appropriate module
- [ ] Implement all required transform functions
- [ ] Export in module's `mod.rs`
- [ ] Register in `register_transforms()` function
- [ ] Add to parent module if new module
- [ ] Update `constants.rs` if core mutation
- [ ] Write comprehensive tests
- [ ] Validate coverage against all existing mutations
