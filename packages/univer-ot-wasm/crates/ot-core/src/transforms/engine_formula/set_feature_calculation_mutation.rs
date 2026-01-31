use crate::registry::TransformRegistry;

/// Register transforms for SetFeatureCalculationMutation
///
/// Controls feature-level calculation settings:
/// - Enable/disable automatic calculation
/// - Calculation mode (auto, manual, etc.)
/// - Feature-specific calculation behavior
///
/// Transform strategy:
/// - Likely LWW at feature scope
/// - Identity with position-based mutations
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement SetFeatureCalculationMutation transforms:
    // - Self-transform: LWW at feature level
    // - vs other mutations: Identity (independent setting)
}
