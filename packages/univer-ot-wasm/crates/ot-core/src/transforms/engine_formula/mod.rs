pub mod formula_mutation;
pub mod register_function_mutation;
pub mod set_array_formula_data_mutation;
pub mod set_defined_name_mutation;
pub mod set_feature_calculation_mutation;
pub mod set_formula_calculation_mutation;
pub mod set_formula_data_mutation;
pub mod set_image_formula_data_mutation;
pub mod set_other_formula_mutation;
pub mod set_super_table_mutation;

use crate::registry::TransformRegistry;

/// Register all formula engine transforms
///
/// This module handles transforms for formula-related mutations.
///
/// Note: Formula OT requires special handling due to complex dependencies
/// and calculation chains. Currently out of scope for initial implementation.
pub fn register_transforms(registry: &mut TransformRegistry) {
    formula_mutation::register_transforms(registry);
    register_function_mutation::register_transforms(registry);
    set_array_formula_data_mutation::register_transforms(registry);
    set_defined_name_mutation::register_transforms(registry);
    set_feature_calculation_mutation::register_transforms(registry);
    set_formula_calculation_mutation::register_transforms(registry);
    set_formula_data_mutation::register_transforms(registry);
    set_image_formula_data_mutation::register_transforms(registry);
    set_other_formula_mutation::register_transforms(registry);
    set_super_table_mutation::register_transforms(registry);
}
