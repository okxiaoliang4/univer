mod pivot_table;

use crate::registry::TransformRegistry;

pub fn register_transforms(registry: &mut TransformRegistry) {
    pivot_table::register_transforms(registry);
}
