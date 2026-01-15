#[cfg(test)]
mod tests {
    use crate::types::*;
    use crate::transform::remove_col::RemoveColTransform;
    use crate::transform::mutation_transform::MutationTransform;
    use serde_json;
    use crate::transform::test_utils::test_utils::create_mutation_info;
    use crate::transform::TransformService;

    #[test]
    fn test_remove_col_transform_trait() {
        let transform = RemoveColTransform::default();
        assert_eq!(RemoveColTransform::mutation_id(), "sheet.mutation.remove-col");
    }

    // RemoveCol × SetRangeValues
    #[test]
    fn test_transform_remove_col_set_range_values() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::to_value(RemoveColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 2,
                    end_row: 0,
                    end_column: 4,
                },
            }).unwrap(),
        );

        let mut cell_data = serde_json::Map::new();
        let mut row_data = serde_json::Map::new();
        row_data.insert("1".to_string(), serde_json::json!({"v": "test"}));
        cell_data.insert("0".to_string(), serde_json::Value::Object(row_data));

        let m2 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data }),
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());
        // Remove col doesn't need to change when set_range_values happens
        assert_eq!(result.m1_prime.id, m1.id);
    }

    // RemoveCol × InsertRow
    #[test]
    fn test_transform_remove_col_insert_row() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::to_value(RemoveColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 2,
                    end_row: 0,
                    end_column: 4,
                },
            }).unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::to_value(InsertRowMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 1,
                    start_column: 0,
                    end_row: 1,
                    end_column: 0,
                },
                row_info: None,
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());
        // Remove col and insert row are independent
        assert_eq!(result.m1_prime.id, m1.id);
        assert_eq!(result.m2_prime.id, m2.id);
    }

    // RemoveCol × InsertCol
    #[test]
    fn test_transform_remove_col_insert_col() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::to_value(RemoveColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 3,
                    end_row: 0,
                    end_column: 5,
                },
            }).unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::to_value(InsertColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 1,
                    end_row: 0,
                    end_column: 1,
                },
                col_info: None,
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());
        // Currently returns identity transform (column transform not yet implemented)
        assert_eq!(result.m1_prime.id, m1.id);
        assert_eq!(result.m2_prime.id, m2.id);
    }

    // RemoveCol × RemoveRows
    #[test]
    fn test_transform_remove_col_remove_rows() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::to_value(RemoveColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 2,
                    end_row: 0,
                    end_column: 4,
                },
            }).unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
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
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());
        // Remove col and remove rows are independent
        assert_eq!(result.m1_prime.id, m1.id);
        assert_eq!(result.m2_prime.id, m2.id);
    }

    // RemoveCol × RemoveCol
    #[test]
    fn test_transform_remove_col_remove_col() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::to_value(RemoveColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 1,
                    end_row: 0,
                    end_column: 2,
                },
            }).unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::to_value(RemoveColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 4,
                    end_row: 0,
                    end_column: 5,
                },
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());
        // Currently returns identity transform (column transform not yet implemented)
        assert_eq!(result.m1_prime.id, m1.id);
        assert_eq!(result.m2_prime.id, m2.id);
    }
}
