mod hyper_link;

use crate::registry::TransformRegistry;

pub fn register_transforms(registry: &mut TransformRegistry) {
    hyper_link::register_transforms(registry);
}
