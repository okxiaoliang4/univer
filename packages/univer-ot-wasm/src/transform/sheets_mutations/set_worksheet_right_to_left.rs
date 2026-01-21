use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct SetWorksheetRightToLeftTransform;

impl_identity_transform!(
    SetWorksheetRightToLeftTransform,
    "sheet.mutation.set-worksheet-right-to-left"
);
