use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct CopyWorksheetEndTransform;

impl_identity_transform!(
    CopyWorksheetEndTransform,
    "sheet.mutation.copy-worksheet-end"
);
