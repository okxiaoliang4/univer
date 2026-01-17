use crate::types::{RemoveRowsMutationParams, TransformError};

pub struct RemoveRowsMutation;

impl RemoveRowsMutation {
    pub fn apply(params: &RemoveRowsMutationParams) -> Result<bool, TransformError> {
        // Validate range
        if params.range.start_row > params.range.end_row {
            return Err(TransformError {
                message: "Invalid range: start_row > end_row".to_string(),
                code: 4001,
            });
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Range, RemoveRowsMutationParams, SubUnitParams};

    #[test]
    fn test_remove_rows_mutation_valid_params() {
        let params = RemoveRowsMutationParams {
            sub_unit_params: SubUnitParams {
                unit_id: "test-unit".to_string(),
                sub_unit_id: "test-sheet".to_string(),
            },
            range: Range {
                start_row: 1,
                start_column: 0,
                end_row: 3,
                end_column: 0,
            },
        };

        let result = RemoveRowsMutation::apply(&params);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
