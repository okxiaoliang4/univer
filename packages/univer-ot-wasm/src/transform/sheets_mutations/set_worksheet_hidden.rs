use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct SetWorksheetHiddenTransform;

impl_identity_transform!(
    SetWorksheetHiddenTransform,
    "sheet.mutation.set-worksheet-hidden"
);
