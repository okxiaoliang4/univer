use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct AddRangeThemeTransform;

impl_identity_transform!(AddRangeThemeTransform, "sheet.mutation.add-range-theme");
