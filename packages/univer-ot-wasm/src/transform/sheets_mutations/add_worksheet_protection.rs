use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct AddWorksheetProtectionTransform;

impl_identity_transform!(
    AddWorksheetProtectionTransform,
    "sheet.mutation.add-worksheet-protection"
);
