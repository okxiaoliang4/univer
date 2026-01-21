use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_set_frozen_transform;

#[derive(Default)]
pub struct SetFrozenTransform;

impl_set_frozen_transform!(SetFrozenTransform, "sheet.mutation.set-frozen");
