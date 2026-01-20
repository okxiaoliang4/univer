#[cfg(test)]
mod tests {
    use crate::transform::mutation_transform::MutationTransform;
    use crate::transform::remove_rows::RemoveRowsTransform;
    use crate::transform::test_utils::test_utils::create_mutation_info;
    use crate::transform::TransformService;
    use crate::mutations::types::{
        InsertColMutationParams, InsertRowMutationParams, RemoveColMutationParams,
        RemoveRowsMutationParams, SetRangeValuesMutationParams,
    };
    use crate::types::{ObjectMatrixPrimitiveType, Range, SubUnitParams};
    use serde_json;

    #[test]
    fn test_remove_rows_transform_trait() {
        let _transform = RemoveRowsTransform::default();
        assert_eq!(
            RemoveRowsTransform::mutation_id(),
            "sheet.mutation.remove-rows"
        );
    }

    // RemoveRows × SetRangeValues
    #[test]
    fn test_transform_remove_rows_set_range_values() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 2,
                    start_column: 0,
                    end_row: 4,
                    end_column: 0,
                },
            })
            .unwrap(),
        );

        let mut cell_data = serde_json::Map::new();
        let mut row_data = serde_json::Map::new();
        row_data.insert("5".to_string(), serde_json::json!({"v": "test"}));
        cell_data.insert("5".to_string(), serde_json::Value::Object(row_data));

        let m2 = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::to_value(SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                cell_value: Some(ObjectMatrixPrimitiveType { data: cell_data }),
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: SetRangeValuesMutationParams =
            serde_json::from_value(result.m2_prime.unwrap().params.clone()).unwrap();
        assert!(transformed_params.cell_value.is_some());
        let cell_value = transformed_params.cell_value.unwrap();
        assert!(cell_value.data.contains_key("2"));
        assert!(!cell_value.data.contains_key("5"));
    }

    // RemoveRows × InsertRow
    #[test]
    fn test_transform_remove_rows_insert_row() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 3,
                    start_column: 0,
                    end_row: 5,
                    end_column: 0,
                },
            })
            .unwrap(),
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
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: RemoveRowsMutationParams =
            serde_json::from_value(result.m1_prime.unwrap().params.clone()).unwrap();
        // Remove range should shift down by 1
        assert_eq!(transformed_params.range.start_row, 4);
        assert_eq!(transformed_params.range.end_row, 6);
    }

    // RemoveRows × InsertRow (delete wins when overlapped)
    #[test]
    fn test_transform_remove_rows_insert_row_conflict() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 2,
                    start_column: 0,
                    end_row: 4,
                    end_column: 0,
                },
            })
            .unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::to_value(InsertRowMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 3,
                    start_column: 0,
                    end_row: 3,
                    end_column: 0,
                },
                row_info: None,
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: RemoveRowsMutationParams =
            serde_json::from_value(result.m1_prime.unwrap().params.clone()).unwrap();
        assert_eq!(transformed_params.range.start_row, 2);
        assert_eq!(transformed_params.range.end_row, 5);

        assert!(result.m2_prime.is_none());
    }

    // RemoveRows × InsertCol
    #[test]
    fn test_transform_remove_rows_insert_col() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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
            })
            .unwrap(),
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
                    start_column: 2,
                    end_row: 0,
                    end_column: 2,
                },
                col_info: None,
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());
        // Remove rows and insert col are independent
        assert_eq!(result.m1_prime.unwrap().id, m1.id);
        assert_eq!(result.m2_prime.unwrap().id, m2.id);
    }

    // RemoveRows × RemoveRows (overlap shrinks to remaining)
    #[test]
    fn test_transform_remove_rows_conflict() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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
            })
            .unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 2,
                    start_column: 0,
                    end_row: 4,
                    end_column: 0,
                },
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: RemoveRowsMutationParams =
            serde_json::from_value(result.m1_prime.unwrap().params.clone()).unwrap();
        assert_eq!(transformed_params.range.start_row, 1);
        assert_eq!(transformed_params.range.end_row, 1);

        let transformed_params_m2: RemoveRowsMutationParams =
            serde_json::from_value(result.m2_prime.unwrap().params.clone()).unwrap();
        assert_eq!(transformed_params_m2.range.start_row, 1);
        assert_eq!(transformed_params_m2.range.end_row, 1);
    }

    // RemoveRows × RemoveRows (non-overlapping)
    #[test]
    fn test_transform_remove_rows_remove_rows_non_overlapping() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
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
            })
            .unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 5,
                    start_column: 0,
                    end_row: 6,
                    end_column: 0,
                },
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: RemoveRowsMutationParams =
            serde_json::from_value(result.m2_prime.unwrap().params.clone()).unwrap();
        assert_eq!(transformed_params.range.start_row, 3);
        assert_eq!(transformed_params.range.end_row, 4);
    }

    #[test]
    fn test_compose_remove_rows_contiguous() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
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
            })
            .unwrap(),
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
                    end_row: 1,
                    end_column: 0,
                },
            })
            .unwrap(),
        );

        let result = service.compose_internal(&m1, &m2);
        assert_eq!(result.len(), 1);
        let composed_params: RemoveRowsMutationParams =
            serde_json::from_value(result[0].params.clone()).unwrap();
        assert_eq!(composed_params.range.start_row, 1);
        assert_eq!(composed_params.range.end_row, 3);
    }

    #[test]
    fn test_compose_remove_rows_non_contiguous() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
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
            })
            .unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 4,
                    start_column: 0,
                    end_row: 4,
                    end_column: 0,
                },
            })
            .unwrap(),
        );

        let result = service.compose_internal(&m1, &m2);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_compose_remove_rows_different_sheet() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "sheet-1".to_string(),
                },
                range: Range {
                    start_row: 1,
                    start_column: 0,
                    end_row: 1,
                    end_column: 0,
                },
            })
            .unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::to_value(RemoveRowsMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "sheet-2".to_string(),
                },
                range: Range {
                    start_row: 1,
                    start_column: 0,
                    end_row: 1,
                    end_column: 0,
                },
            })
            .unwrap(),
        );

        let result = service.compose_internal(&m1, &m2);
        assert_eq!(result.len(), 2);
    }
    // RemoveRows × RemoveCol
    #[test]
    fn test_transform_remove_rows_remove_col() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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
            })
            .unwrap(),
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
                    start_column: 2,
                    end_row: 0,
                    end_column: 4,
                },
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());
        // Remove rows and remove col are independent
        assert_eq!(result.m1_prime.unwrap().id, m1.id);
        assert_eq!(result.m2_prime.unwrap().id, m2.id);
    }
}