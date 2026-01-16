#[cfg(test)]
mod tests {
    use crate::types::*;
    use crate::transform::insert_col::InsertColTransform;
    use crate::transform::mutation_transform::MutationTransform;
    use crate::transform::test_utils::test_utils::create_mutation_info;
    use serde_json;
    use crate::transform::TransformService;

    #[test]
    fn test_insert_col_transform_trait() {
        let transform = InsertColTransform::default();
        assert_eq!(InsertColTransform::mutation_id(), "sheet.mutation.insert-col");
    }

    // InsertCol × SetRangeValues
    #[test]
    fn test_transform_insert_col_set_range_values() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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
            }).unwrap(),
        );

        let mut cell_data = serde_json::Map::new();
        let mut row_data = serde_json::Map::new();
        row_data.insert("2".to_string(), serde_json::json!({"v": "test"}));
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

        let transformed_params: SetRangeValuesMutationParams = serde_json::from_value(result.m2_prime.params.clone()).unwrap();
        assert!(transformed_params.cell_value.is_some());
        let cell_value = transformed_params.cell_value.unwrap();
        let row = cell_value.data.get("0").unwrap().as_object().unwrap();
        assert!(row.contains_key("3"));
        assert!(!row.contains_key("2"));
    }

    // InsertCol × InsertRow
    #[test]
    fn test_transform_insert_col_insert_row() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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
        // Insert col and insert row are independent
        assert_eq!(result.m1_prime.id, m1.id);
        assert_eq!(result.m2_prime.id, m2.id);
    }

    // InsertCol × InsertCol
    #[test]
    fn test_transform_insert_col_insert_col() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: InsertColMutationParams = serde_json::from_value(result.m2_prime.params.clone()).unwrap();
        assert_eq!(transformed_params.range.start_column, 3);
        assert_eq!(transformed_params.range.end_column, 3);
    }

    // InsertCol × InsertCol (m2 before m1)
    #[test]
    fn test_transform_insert_col_insert_col_shift_m1() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::to_value(InsertColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "test-unit".to_string(),
                    sub_unit_id: "test-sheet".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 5,
                    end_row: 0,
                    end_column: 5,
                },
                col_info: None,
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
                    start_column: 2,
                    end_row: 0,
                    end_column: 2,
                },
                col_info: None,
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: InsertColMutationParams = serde_json::from_value(result.m1_prime.params.clone()).unwrap();
        assert_eq!(transformed_params.range.start_column, 6);
        assert_eq!(transformed_params.range.end_column, 6);
    }

    // InsertCol × RemoveRows
    #[test]
    fn test_transform_insert_col_remove_rows() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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
        // Insert col and remove rows are independent
        assert_eq!(result.m1_prime.id, m1.id);
        assert_eq!(result.m2_prime.id, m2.id);
    }

    // InsertCol × RemoveCol
    #[test]
    fn test_transform_insert_col_remove_col() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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

        let m2 = create_mutation_info(
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

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_none());

        let transformed_params: RemoveColMutationParams = serde_json::from_value(result.m2_prime.params.clone()).unwrap();
        assert_eq!(transformed_params.range.start_column, 4);
        assert_eq!(transformed_params.range.end_column, 6);
    }

    // InsertCol × RemoveCol (conflict)
    #[test]
    fn test_transform_insert_col_remove_col_conflict() {
        let service = TransformService::new();

        let m1 = create_mutation_info(
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
                    start_column: 1,
                    end_row: 0,
                    end_column: 3,
                },
            }).unwrap(),
        );

        let result = service.transform_internal(&m1, &m2);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("conflicts"));
    }
}
