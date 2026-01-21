use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_ranges_transform;

#[derive(Default)]
pub struct SetColHiddenTransform;

impl_ranges_transform!(SetColHiddenTransform, "sheet.mutation.set-col-hidden");
