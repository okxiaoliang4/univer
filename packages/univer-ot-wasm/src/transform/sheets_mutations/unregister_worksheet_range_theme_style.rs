use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_identity_transform;

#[derive(Default)]
pub struct UnregisterWorksheetRangeThemeStyleTransform;

impl_identity_transform!(
    UnregisterWorksheetRangeThemeStyleTransform,
    "sheet.mutation.unregister-worksheet-range-theme-style"
);
