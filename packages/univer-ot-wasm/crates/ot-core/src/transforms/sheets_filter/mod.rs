mod filter;

use crate::registry::TransformRegistry;

pub fn register_transforms(registry: &mut TransformRegistry) {
    filter::register_transforms(registry);
}
