use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct SetTabColorTransform;

impl_identity_transform!(SetTabColorTransform, "sheet.mutation.set-tab-color");
