use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct SetRangeThemeTransform;

impl_identity_transform!(SetRangeThemeTransform, "sheet.mutation.set-range-theme");
