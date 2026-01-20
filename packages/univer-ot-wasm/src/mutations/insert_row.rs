use crate::mutations::types::InsertRowMutationParams;
use crate::types::TransformError;

pub struct InsertRowMutation;

impl InsertRowMutation {
    pub fn apply(params: &InsertRowMutationParams) -> Result<bool, TransformError> {
        // Validate range
        if params.range.start_row > params.range.end_row {
            return Err(TransformError {
                message: "Invalid range: start_row > end_row".to_string(),
                code: 2001,
            });
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutations::types::InsertRowMutationParams;
    use crate::types::{Range, SubUnitParams};

    #[test]
    fn test_insert_row_mutation_valid_params() {
        let params = InsertRowMutationParams {
            sub_unit_params: SubUnitParams {
                unit_id: "test-unit".to_string(),
                sub_unit_id: "test-sheet".to_string(),
            },
            range: Range {
                start_row: 1,
                start_column: 0,
                end_row: 2,
                end_column: 0,
            },
            row_info: None,
        };

        let result = InsertRowMutation::apply(&params);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_insert_row_mutation_invalid_range() {
        let params = InsertRowMutationParams {
            sub_unit_params: SubUnitParams {
                unit_id: "test-unit".to_string(),
                sub_unit_id: "test-sheet".to_string(),
            },
            range: Range {
                start_row: 5,
                start_column: 0,
                end_row: 2,
                end_column: 0,
            },
            row_info: None,
        };

        let result = InsertRowMutation::apply(&params);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, 2001);
    }
}
