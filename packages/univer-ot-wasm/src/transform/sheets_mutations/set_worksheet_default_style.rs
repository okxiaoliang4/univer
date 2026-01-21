use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct SetWorksheetDefaultStyleTransform;

impl_identity_transform!(
    SetWorksheetDefaultStyleTransform,
    "sheet.mutation.set-worksheet-default-style"
);
