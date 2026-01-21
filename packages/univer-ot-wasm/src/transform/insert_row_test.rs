#[cfg(test)]
mod tests {
    use crate::mutations::types::{
        InsertColMutationParams, InsertRowMutationParams, RemoveColMutationParams,
        RemoveRowsMutationParams, SetRangeValuesMutationParams,
    };
    use crate::transform::insert_row::InsertRowTransform;
    use crate::transform::mutation_transform::MutationTransform;
    use crate::transform::test_utils::test_utils::create_mutation_info;
    use crate::transform::TransformService;
    use crate::types::{ObjectMatrixPrimitiveType, Range, SubUnitParams};
    use serde_json;

    #[test]
    fn test_insert_row_transform_trait() {
        let _transform = InsertRowTransform::default();
        assert_eq!(
            InsertRowTransform::mutation_id(),
            "sheet.mutation.insert-row"
        );
    }

    // InsertRow × SetRangeValues
    #[test]
    fn test_transform_insert_row_set_range_values() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::to_value(InsertRowMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 2,
                    start_column: 0,
                    end_row: 2,
                    end_column: 0,
                },
                row_info: None,
            })
            .unwrap(),
        );

        let mut cell_data = serde_json::Map::new();
        let mut row_data = serde_json::Map::new();
        row_data.insert("2".to_string(), serde_json::json!({"v": "test"}));
        cell_data.insert("2".to_string(), serde_json::Value::Object(row_data));

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
        assert!(cell_value.data.contains_key("3"));
        assert!(!cell_value.data.contains_key("2"));
    }

    // InsertRow × InsertRow
    #[test]
    fn test_transform_insert_row_insert_row() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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

        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::to_value(InsertRowMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 2,
                    start_column: 0,
                    end_row: 2,
                    end_column: 0,
                },
                row_info: None,
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: InsertRowMutationParams =
            serde_json::from_value(result.m2_prime.unwrap().params.clone()).unwrap();
        assert_eq!(transformed_params.range.start_row, 3);
        assert_eq!(transformed_params.range.end_row, 3);
    }

    // InsertRow × InsertRow (m2 before m1)
    #[test]
    fn test_transform_insert_row_insert_row_shift_m1() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::to_value(InsertRowMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 5,
                    start_column: 0,
                    end_row: 5,
                    end_column: 0,
                },
                row_info: None,
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
                    start_row: 2,
                    start_column: 0,
                    end_row: 2,
                    end_column: 0,
                },
                row_info: None,
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: InsertRowMutationParams =
            serde_json::from_value(result.m1_prime.unwrap().params.clone()).unwrap();
        assert_eq!(transformed_params.range.start_row, 6);
        assert_eq!(transformed_params.range.end_row, 6);
    }

    // InsertRow × InsertRow (same position: m1 shifts)
    #[test]
    fn test_transform_insert_row_insert_row_same_position() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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

        let transformed_params: InsertRowMutationParams =
            serde_json::from_value(result.m1_prime.unwrap().params.clone()).unwrap();
        assert_eq!(transformed_params.range.start_row, 4);
        assert_eq!(transformed_params.range.end_row, 4);
    }

    // InsertRow × InsertCol
    #[test]
    fn test_transform_insert_row_insert_col() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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
        // Insert row and insert col are independent
        assert_eq!(result.m1_prime.unwrap().id, m1.id);
        assert_eq!(result.m2_prime.unwrap().id, m2.id);
    }

    // InsertRow × RemoveRows
    #[test]
    fn test_transform_insert_row_remove_rows() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::to_value(InsertRowMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 5,
                    start_column: 0,
                    end_row: 5,
                    end_column: 0,
                },
                row_info: None,
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
                    end_row: 3,
                    end_column: 0,
                },
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: InsertRowMutationParams =
            serde_json::from_value(result.m1_prime.unwrap().params.clone()).unwrap();
        // Row 5 should become row 2 (5 - 3 = 2)
        assert_eq!(transformed_params.range.start_row, 2);
        assert_eq!(transformed_params.range.end_row, 2);
    }

    // InsertRow × RemoveRows (delete wins when overlapped)
    #[test]
    fn test_transform_insert_row_remove_rows_conflict() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::to_value(InsertRowMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 2,
                    start_column: 0,
                    end_row: 2,
                    end_column: 0,
                },
                row_info: None,
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
                    end_row: 3,
                    end_column: 0,
                },
            })
            .unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        assert!(result.m1_prime.is_none());

        let transformed_params: RemoveRowsMutationParams =
            serde_json::from_value(result.m2_prime.unwrap().params.clone()).unwrap();
        assert_eq!(transformed_params.range.start_row, 1);
        assert_eq!(transformed_params.range.end_row, 4);
    }

    // InsertRow × RemoveCol
    #[test]
    fn test_transform_insert_row_remove_col() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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
        // Insert row and remove col are independent
        assert_eq!(result.m1_prime.unwrap().id, m1.id);
        assert_eq!(result.m2_prime.unwrap().id, m2.id);
    }

    #[test]
    fn test_compose_insert_row_contiguous() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::to_value(InsertRowMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 2,
                    start_column: 0,
                    end_row: 3,
                    end_column: 0,
                },
                row_info: None,
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
                    start_row: 4,
                    start_column: 0,
                    end_row: 4,
                    end_column: 0,
                },
                row_info: None,
            })
            .unwrap(),
        );

        let result = service.compose_internal(&m1, &m2);
        assert_eq!(result.len(), 1);
        let composed_params: InsertRowMutationParams =
            serde_json::from_value(result[0].params.clone()).unwrap();
        assert_eq!(composed_params.range.start_row, 2);
        assert_eq!(composed_params.range.end_row, 4);
    }

    #[test]
    fn test_compose_insert_row_non_contiguous() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::to_value(InsertRowMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 2,
                    start_column: 0,
                    end_row: 2,
                    end_column: 0,
                },
                row_info: None,
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
                    start_row: 5,
                    start_column: 0,
                    end_row: 5,
                    end_column: 0,
                },
                row_info: None,
            })
            .unwrap(),
        );

        let result = service.compose_internal(&m1, &m2);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_compose_insert_row_different_sheet() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::to_value(InsertRowMutationParams {
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
                row_info: None,
            })
            .unwrap(),
        );

        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::to_value(InsertRowMutationParams {
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
                row_info: None,
            })
            .unwrap(),
        );

        let result = service.compose_internal(&m1, &m2);
        assert_eq!(result.len(), 2);
    }
}
