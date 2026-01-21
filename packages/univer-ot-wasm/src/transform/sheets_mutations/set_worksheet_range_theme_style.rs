use crate::mutations::sheets::SheetMutationRangeParams;
use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_range_transform;

#[derive(Default)]
pub struct SetWorksheetRangeThemeStyleTransform;

impl_range_transform!(
    SetWorksheetRangeThemeStyleTransform,
    "sheet.mutation.set-worksheet-range-theme-style"
);
