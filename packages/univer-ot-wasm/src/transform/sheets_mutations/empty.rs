use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct EmptyTransform;

impl_identity_transform!(EmptyTransform, "sheet.mutation.empty");
