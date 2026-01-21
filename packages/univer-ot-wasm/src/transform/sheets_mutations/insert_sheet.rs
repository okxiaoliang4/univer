use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct InsertSheetTransform;

impl_identity_transform!(InsertSheetTransform, "sheet.mutation.insert-sheet");
