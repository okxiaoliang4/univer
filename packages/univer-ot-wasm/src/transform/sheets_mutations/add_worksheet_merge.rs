use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_ranges_transform;

#[derive(Default)]
pub struct AddWorksheetMergeTransform;

impl_ranges_transform!(
    AddWorksheetMergeTransform,
    "sheet.mutation.add-worksheet-merge"
);
