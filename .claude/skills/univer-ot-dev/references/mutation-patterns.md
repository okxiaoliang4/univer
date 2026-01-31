# Mutation Definition Patterns

Standard patterns for defining mutations in `/packages/univer-ot-wasm/crates/ot-core/src/mutations/`.

## File Naming Convention

**CRITICAL**: Use snake_case with `_mutation.rs` suffix
- `insert_row_col_mutation.rs` (contains InsertRowMutation and InsertColMutation)
- `set_range_values_mutation.rs` (contains SetRangeValuesMutation)
- `add_worksheet_merge_mutation.rs` (contains AddWorksheetMergeMutation)

**Naming pattern**: `[operation]_mutation.rs` where operation describes what the mutation does in snake_case.

## Standard Mutation Structure

Every mutation follows this exact pattern:

```rust
// 1. PARAMS STRUCT (Serde-serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct [MutationName]Params {
    pub unit_id: String,              // Required: workbook ID
    pub sub_unit_id: String,          // Required (usually): worksheet ID
    pub [field_name]: [Type],         // Feature-specific fields

    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_field: Option<[Type]>,  // Optional fields
}

// 2. MUTATION STRUCT (never instantiated)
pub struct [MutationName]Mutation;

// 3. IMPL BLOCK with ID and handler
impl [MutationName]Mutation {
    pub const ID: &'static str = "[prefix].mutation.[kebab-case-name]";

    pub fn handler(_params: [MutationName]Params) -> Result<bool, String> {
        Ok(true)
    }
}
```

## Mutation ID Format

**Convention**: `"[document-type].mutation.[operation-name]"`

Examples:
- `"sheet.mutation.set-range-values"`
- `"sheet.mutation.insert-row"`
- `"sheet.mutation.add-worksheet-merge"`
- `"sheet.mutation.move-range"`

## Common Parameter Patterns

### Pattern 1: Range-based Mutation
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRangeValuesMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub cell_value: Option<IObjectMatrixPrimitiveType>,  // HashMap<String, HashMap<String, ICellData>>
}
```

### Pattern 2: Row/Column Structural Mutation
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertRowMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub range: IRange,  // Contains start_row, end_row, start_column, end_column

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_value: Option<IObjectMatrixPrimitiveType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_info: Option<IObjectArrayPrimitiveType<IRowData>>,
}
```

### Pattern 3: Property Mutation
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetNameMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub name: String,
}
```

### Pattern 4: Feature Plugin Mutation
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddConditionalFormattingMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub cf_id: String,          // Feature-specific ID
    pub rule: IConditionFormattingRule,
}
```

### Pattern 5: Multi-Range Mutation
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWorksheetMergeMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
}
```

## Type Conventions

**Core types** (from `mutations/sheets/types.rs`):

```rust
pub type IObjectMatrixPrimitiveType = HashMap<String, HashMap<String, ICellData>>;
pub type IObjectArrayPrimitiveType<T> = HashMap<String, T>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IRange {
    pub start_row: i32,
    pub start_column: i32,
    pub end_row: i32,
    pub end_column: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_type: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ICellData {
    // Cell content, style, etc.
}
```

## Module Organization

```
mutations/
├── mod.rs                    # Re-exports all modules
├── sheets/
│   ├── mod.rs               # Re-exports all sheets mutations
│   ├── types.rs             # Shared types for sheets
│   ├── set_range_values_mutation.rs
│   ├── insert_row_col_mutation.rs
│   └── ...
├── sheets_filter/
│   ├── mod.rs
│   ├── types.rs
│   └── set_filter_range_mutation.rs
└── ...
```

## Required Steps for New Mutation

1. **Create mutation file** in appropriate module directory with `_mutation.rs` suffix
2. **Define Params struct** with all required and optional fields
3. **Define Mutation struct** (unit struct)
4. **Implement ID constant** following naming convention (`pub const ID: &'static str`)
5. **Implement handler function** (stub returning Ok(true))
6. **Add module declaration** in module's `mod.rs`: `pub mod [name]_mutation;`
7. **Add pub use export** in module's `mod.rs`: `pub use [name]_mutation::*;`
8. **Add to constants.rs** if part of core sheets mutations
9. **Create corresponding test file** in `tests/` with snake_case name (e.g., `[operation]_tests.rs`)
10. **Update transform planning** (see transform-patterns.md)

## Examples

### Example 1: Simple Property Mutation

```rust
// mutations/sheets/set_tab_color_mutation.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTabColorMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub color: String,
}

pub struct SetTabColorMutation;

impl SetTabColorMutation {
    pub const ID: &'static str = "sheet.mutation.set-tab-color";

    pub fn handler(_params: SetTabColorMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
```

### Example 2: Complex Structural Mutation

```rust
// mutations/sheets/move_range_mutation.rs

use serde::{Deserialize, Serialize};
use super::types::{IRange, IObjectMatrixPrimitiveType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveRangeFromTo {
    pub value: IObjectMatrixPrimitiveType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_properties: Option<HashMap<String, IRowData>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_properties: Option<HashMap<String, IColumnData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveRangeMutationParams {
    pub unit_id: String,
    pub from_range: IRange,
    pub to_range: IRange,
    pub from: MoveRangeFromTo,
    pub to: MoveRangeFromTo,
}

pub struct MoveRangeMutation;

impl MoveRangeMutation {
    pub const ID: &'static str = "sheet.mutation.move-range";

    pub fn handler(_params: MoveRangeMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
```

### Example 3: Feature Plugin Mutation

```rust
// mutations/sheets_filter/set_filter_range_mutation.rs

use serde::{Deserialize, Serialize};
use super::types::IFilterColumn;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetFilterRangeMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub range: IRange,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_columns: Option<Vec<IFilterColumn>>,
}

pub struct SetFilterRangeMutation;

impl SetFilterRangeMutation {
    pub const ID: &'static str = "sheet.mutation.set-filter-range";

    pub fn handler(_params: SetFilterRangeMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
```

## Validation Checklist

Before committing a new mutation:

- [ ] File name uses snake_case with `_mutation.rs` suffix
- [ ] Params struct uses `#[derive(Debug, Clone, Serialize, Deserialize)]`
- [ ] Params uses `#[serde(rename_all = "camelCase")]`
- [ ] Optional fields use `#[serde(skip_serializing_if = "Option::is_none")]`
- [ ] Mutation ID defined as `pub const ID: &'static str`
- [ ] Mutation ID follows `[type].mutation.[kebab-case]` format
- [ ] Mutation ID is unique (check existing mutations)
- [ ] Handler returns `Result<bool, String>`
- [ ] Module declared in `mod.rs`: `pub mod [name]_mutation;`
- [ ] Exported in `mod.rs`: `pub use [name]_mutation::*;`
- [ ] Added to `constants.rs` if core mutation
- [ ] Test file created in `tests/` with snake_case name
- [ ] Tests use `Mutation::ID` constant, not string literals
- [ ] Transform coverage planned (all existing mutations)
