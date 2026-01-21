use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::impl_move_rows_cols_transform;

#[derive(Default)]
pub struct MoveColumnsTransform;

impl_move_rows_cols_transform!(MoveColumnsTransform, "sheet.mutation.move-columns", false);
