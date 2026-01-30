mod table;

use crate::registry::TransformRegistry;

pub fn register_transforms(registry: &mut TransformRegistry) {
    table::register_transforms(registry);
}
