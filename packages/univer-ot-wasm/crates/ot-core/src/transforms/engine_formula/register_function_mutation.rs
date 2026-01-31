use crate::registry::TransformRegistry;

/// Register transforms for RegisterFunctionMutation
///
/// This mutation registers custom functions in the formula engine.
///
/// Transform strategy:
/// - Likely identity with most mutations (function registration is independent)
/// - Self-transform: LWW or conflict by function name
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement RegisterFunctionMutation transforms:
    // - Self-transform: LWW by function name
    // - vs other mutations: Likely identity (independent)
}
