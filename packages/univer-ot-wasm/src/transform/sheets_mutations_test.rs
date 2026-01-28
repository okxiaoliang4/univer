#[cfg(test)]
mod tests {
    use crate::transform::mutation_transform::MutationTransform;
    use crate::transform::sheets_mutations::{
        AddDataValidationTransform, AddRangeProtectionTransform, AddRangeThemeTransform,
        AddWorksheetMergeTransform, AddWorksheetProtectionTransform, CopyWorksheetEndTransform,
        DeleteRangeProtectionTransform, DeleteWorksheetProtectionTransform,
        DeleteWorksheetRangeThemeStyleTransform, EmptyTransform, InsertSheetTransform,
        MoveColumnsTransform, MoveRangeTransform, MoveRowsTransform,
        RegisterWorksheetRangeThemeStyleTransform, RemoveDataValidationTransform,
        RemoveNumfmtTransform, RemoveRangeThemeTransform, RemoveSheetTransform,
        RemoveWorksheetMergeTransform, ReorderRangeTransform, SetColHiddenTransform,
        SetColVisibleTransform, SetFrozenTransform, SetGridlinesColorTransform, SetNumfmtTransform,
        SetRangeProtectionTransform, SetRangeThemeTransform, SetRowHiddenTransform,
        SetRowVisibleTransform, SetTabColorTransform, SetWorkbookNameTransform,
        SetWorksheetColWidthTransform, SetWorksheetColumnCountTransform,
        SetWorksheetDefaultStyleTransform, SetWorksheetHiddenTransform, SetWorksheetNameTransform,
        SetWorksheetOrderTransform, SetWorksheetPermissionPointsTransform,
        SetWorksheetProtectionTransform, SetWorksheetRangeThemeStyleTransform,
        SetWorksheetRightToLeftTransform, SetWorksheetRowAutoHeightTransform,
        SetWorksheetRowCountTransform, SetWorksheetRowHeightTransform,
        SetWorksheetRowIsAutoHeightTransform, ToggleGridlinesTransform,
        UnregisterWorksheetRangeThemeStyleTransform, UpdateDataValidationTransform,
    };
    use crate::transform::test_utils::test_utils::create_mutation_info;
    use serde_json;

    #[test]
    fn test_add_range_protection_shifts_on_insert_row() {
        let transform = AddRangeProtectionTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.add-range-protection".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "rules": [{
                    "id": "rule-1",
                    "ranges": [{"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}]
                }]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let rules = params.get("rules").unwrap().as_array().unwrap();
        let ranges = rules[0].get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_set_frozen_shifts_on_remove_col() {
        let transform = SetFrozenTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-frozen".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "startRow": 0,
                "startColumn": 3,
                "ySplit": 1,
                "xSplit": 1
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        assert_eq!(params.get("startColumn").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_move_range_shifts_on_insert_row() {
        let transform = MoveRangeTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.move-range".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "fromRange": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0},
                "toRange": {"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 0},
                "from": {"subUnitId": "sheet", "value": {"1": {"0": {"v": 1}}}},
                "to": {"subUnitId": "sheet", "value": {"2": {"0": {"v": 2}}}}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let from_range = params.get("fromRange").unwrap();
        assert_eq!(from_range.get("startRow").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_set_worksheet_row_height_shifts_on_insert_row() {
        let transform = SetWorksheetRowHeightTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-row-height".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 1, "startColumn": 0, "endRow": 2, "endColumn": 1}],
                "rowHeight": {"1": 24, "2": 30}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 2);
        let row_height = params.get("rowHeight").unwrap().as_object().unwrap();
        assert!(row_height.contains_key("2"));
        assert!(row_height.contains_key("3"));
    }

    #[test]
    fn test_set_worksheet_row_auto_height_shifts_on_remove_rows() {
        let transform = SetWorksheetRowAutoHeightTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-row-auto-height".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "rowsAutoHeightInfo": [
                    {"row": 1, "autoHeight": 20},
                    {"row": 3, "autoHeight": 22}
                ]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let rows = params
            .get("rowsAutoHeightInfo")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].get("row").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_set_worksheet_col_width_shifts_on_insert_col() {
        let transform = SetWorksheetColWidthTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-col-width".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 2}],
                "colWidth": {"1": 80, "2": 90}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_insert_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 2);
        let col_width = params.get("colWidth").unwrap().as_object().unwrap();
        assert!(col_width.contains_key("2"));
        assert!(col_width.contains_key("3"));
    }

    #[test]
    fn test_move_rows_shifts_on_insert_row() {
        let transform = MoveRowsTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.move-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "sourceRange": {"startRow": 2, "startColumn": 0, "endRow": 3, "endColumn": 0},
                "targetRange": {"startRow": 5, "startColumn": 0, "endRow": 6, "endColumn": 0}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let source_range = params.get("sourceRange").unwrap();
        assert_eq!(source_range.get("startRow").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_move_columns_shifts_on_remove_col() {
        let transform = MoveColumnsTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.move-columns".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "sourceRange": {"startRow": 0, "startColumn": 3, "endRow": 0, "endColumn": 4},
                "targetRange": {"startRow": 0, "startColumn": 6, "endRow": 0, "endColumn": 7}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let source_range = params.get("sourceRange").unwrap();
        assert_eq!(
            source_range.get("startColumn").unwrap().as_u64().unwrap(),
            2
        );
    }

    #[test]
    fn test_reorder_range_shifts_on_insert_row() {
        let transform = ReorderRangeTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.reorder-range".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 3, "endColumn": 1},
                "order": {"1": 2, "2": 1}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let order = params.get("order").unwrap().as_object().unwrap();
        assert!(order.contains_key("2"));
        assert!(order.contains_key("3"));
    }

    #[test]
    fn test_set_range_protection_removed_by_row_delete() {
        let transform = SetRangeProtectionTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-range-protection".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ruleId": "rule-1",
                "rule": {
                    "id": "rule-1",
                    "ranges": [{"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 1}]
                }
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_set_range_protection_shifts_on_insert_col() {
        let transform = SetRangeProtectionTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-range-protection".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ruleId": "rule-1",
                "rule": {
                    "id": "rule-1",
                    "ranges": [{"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2}]
                }
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_insert_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let rule = params.get("rule").unwrap();
        let ranges = rule.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_set_range_protection_shifts_on_insert_row() {
        let transform = SetRangeProtectionTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-range-protection".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ruleId": "rule-1",
                "rule": {
                    "id": "rule-1",
                    "ranges": [{"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 0}]
                }
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let rule = params.get("rule").unwrap();
        let ranges = rule.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_set_row_hidden_removed_by_row_delete() {
        let transform = SetRowHiddenTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-row-hidden".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_set_col_visible_shifts_on_insert_col() {
        let transform = SetColVisibleTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-col-visible".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2}]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_set_numfmt_shifts_on_remove_rows() {
        let transform = SetNumfmtTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set.numfmt".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "values": {"a": {"ranges": [{"startRow": 3, "startColumn": 0, "endRow": 3, "endColumn": 0}] }},
                "refMap": {"a": {"pattern": "0"}}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let values = params.get("values").unwrap().get("a").unwrap();
        let ranges = values.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_remove_numfmt_shifts_on_insert_col() {
        let transform = RemoveNumfmtTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.remove.numfmt".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_insert_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_set_worksheet_range_theme_style_shifts_on_insert_row() {
        let transform = SetWorksheetRangeThemeStyleTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-range-theme-style".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "themeName": "theme",
                "range": {"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 1}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let range = params.get("range").unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_delete_worksheet_range_theme_style_shifts_on_remove_col() {
        let transform = DeleteWorksheetRangeThemeStyleTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.remove-worksheet-range-theme-style".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "themeName": "theme",
                "range": {"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let range = params.get("range").unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_add_worksheet_merge_shifts_on_remove_col() {
        let transform = AddWorksheetMergeTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.add-worksheet-merge".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 3}]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_remove_worksheet_merge_shifts_on_insert_row() {
        let transform = RemoveWorksheetMergeTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.remove-worksheet-merge".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 1}]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_set_row_visible_shifts_on_insert_row() {
        let transform = SetRowVisibleTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-row-visible".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 0}]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_set_row_visible_removed_by_row_delete() {
        let transform = SetRowVisibleTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-row-visible".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_set_col_hidden_removed_by_col_delete() {
        let transform = SetColHiddenTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-col-hidden".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2}]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_set_col_hidden_shifts_on_insert_col() {
        let transform = SetColHiddenTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-col-hidden".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2}]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 1, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_insert_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_add_range_protection_removed_by_col_delete() {
        let transform = AddRangeProtectionTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.add-range-protection".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "rules": [{
                    "id": "rule-1",
                    "ranges": [{"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}]
                }]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_set_frozen_shifts_on_insert_row() {
        let transform = SetFrozenTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-frozen".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "startRow": 2,
                "startColumn": 0,
                "ySplit": 1,
                "xSplit": 0
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        assert_eq!(params.get("startRow").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_set_frozen_shifts_on_remove_rows() {
        let transform = SetFrozenTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-frozen".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "startRow": 3,
                "startColumn": 0,
                "ySplit": 2,
                "xSplit": 0
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 2, "startColumn": 0, "endRow": 3, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        assert_eq!(params.get("startRow").unwrap().as_u64().unwrap(), 2);
        assert_eq!(params.get("ySplit").unwrap().as_u64().unwrap(), 0);
    }

    #[test]
    fn test_move_range_removed_by_row_delete() {
        let transform = MoveRangeTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.move-range".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "fromRange": {"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 0},
                "toRange": {"startRow": 3, "startColumn": 0, "endRow": 3, "endColumn": 0},
                "from": {"subUnitId": "sheet", "value": {"2": {"0": {"v": 1}}}},
                "to": {"subUnitId": "sheet", "value": {"3": {"0": {"v": 2}}}}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 2, "startColumn": 0, "endRow": 3, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_move_range_removed_by_col_delete() {
        let transform = MoveRangeTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.move-range".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "fromRange": {"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2},
                "toRange": {"startRow": 0, "startColumn": 3, "endRow": 0, "endColumn": 3},
                "from": {"subUnitId": "sheet", "value": {"0": {"2": {"v": 1}}}},
                "to": {"subUnitId": "sheet", "value": {"0": {"3": {"v": 2}}}}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 3}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_reorder_range_shifts_on_remove_col() {
        let transform = ReorderRangeTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.reorder-range".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 2, "endRow": 1, "endColumn": 2},
                "order": {"0": 1}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let range = params.get("range").unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_set_worksheet_row_is_auto_height_shifts_on_remove_col() {
        let transform = SetWorksheetRowIsAutoHeightTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-row-is-auto-height".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 1, "startColumn": 2, "endRow": 1, "endColumn": 2}],
                "autoHeightInfo": {"1": 1}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_set_worksheet_row_is_auto_height_shifts_on_insert_row() {
        let transform = SetWorksheetRowIsAutoHeightTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-row-is-auto-height".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 1}],
                "autoHeightInfo": {"2": 1}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 3);
        let auto_height = params.get("autoHeightInfo").unwrap().as_object().unwrap();
        assert!(auto_height.contains_key("3"));
    }

    #[test]
    fn test_set_worksheet_row_is_auto_height_removed_by_row_delete() {
        let transform = SetWorksheetRowIsAutoHeightTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-row-is-auto-height".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 1}],
                "autoHeightInfo": {"2": 1}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_set_worksheet_row_height_shifts_on_remove_col() {
        let transform = SetWorksheetRowHeightTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-row-height".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 1, "startColumn": 2, "endRow": 1, "endColumn": 2}],
                "rowHeight": {"1": 24}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_set_worksheet_col_width_shifts_on_remove_row() {
        let transform = SetWorksheetColWidthTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-col-width".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 1}],
                "colWidth": {"0": 80}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_set_row_hidden_shifts_on_insert_row() {
        let transform = SetRowHiddenTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-row-hidden".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 0}]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_set_col_visible_removed_by_col_delete() {
        let transform = SetColVisibleTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-col-visible".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2}]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_add_range_protection_removed_by_row_delete() {
        let transform = AddRangeProtectionTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.add-range-protection".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "rules": [{
                    "id": "rule-1",
                    "ranges": [{"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}]
                }]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_set_range_protection_removed_by_col_delete() {
        let transform = SetRangeProtectionTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-range-protection".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ruleId": "rule-1",
                "rule": {
                    "id": "rule-1",
                    "ranges": [{"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}]
                }
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_set_numfmt_shifts_on_insert_col() {
        let transform = SetNumfmtTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set.numfmt".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "values": {"a": {"ranges": [{"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2}] }},
                "refMap": {"a": {"pattern": "0"}}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_insert_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let values = params.get("values").unwrap().get("a").unwrap();
        let ranges = values.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_remove_numfmt_shifts_on_remove_rows() {
        let transform = RemoveNumfmtTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.remove.numfmt".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 0}]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_remove_numfmt_discards_set_numfmt_on_overlap() {
        let remove_numfmt = create_mutation_info(
            "sheet.mutation.remove.numfmt".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 1}]
            }),
        );
        let set_numfmt = create_mutation_info(
            "sheet.mutation.set.numfmt".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "values": {"a": {"ranges": [{"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}] }},
                "refMap": {"a": {"pattern": "0"}}
            }),
        );
        let result =
            RemoveNumfmtTransform::default().transform_with_set_numfmt(&remove_numfmt, &set_numfmt);
        assert!(result.m2_prime.is_none());
    }

    #[test]
    fn test_move_range_shifts_on_insert_col() {
        let transform = MoveRangeTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.move-range".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "fromRange": {"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2},
                "toRange": {"startRow": 0, "startColumn": 3, "endRow": 0, "endColumn": 3},
                "from": {"subUnitId": "sheet", "value": {"0": {"2": {"v": 1}}}},
                "to": {"subUnitId": "sheet", "value": {"0": {"3": {"v": 2}}}}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_insert_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let from_range = params.get("fromRange").unwrap();
        assert_eq!(from_range.get("startColumn").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_move_rows_shifts_on_remove_rows() {
        let transform = MoveRowsTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.move-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "sourceRange": {"startRow": 3, "startColumn": 0, "endRow": 4, "endColumn": 0},
                "targetRange": {"startRow": 6, "startColumn": 0, "endRow": 7, "endColumn": 0}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let source_range = params.get("sourceRange").unwrap();
        assert_eq!(source_range.get("startRow").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_move_columns_shifts_on_insert_col() {
        let transform = MoveColumnsTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.move-columns".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "sourceRange": {"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 3},
                "targetRange": {"startRow": 0, "startColumn": 5, "endRow": 0, "endColumn": 6}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_insert_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let source_range = params.get("sourceRange").unwrap();
        assert_eq!(
            source_range.get("startColumn").unwrap().as_u64().unwrap(),
            3
        );
    }

    #[test]
    fn test_reorder_range_shifts_on_remove_rows() {
        let transform = ReorderRangeTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.reorder-range".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 2, "startColumn": 0, "endRow": 3, "endColumn": 1},
                "order": {"2": 3}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let order = params.get("order").unwrap().as_object().unwrap();
        assert!(order.contains_key("1"));
    }

    #[test]
    fn test_set_worksheet_row_height_shifts_on_insert_col() {
        let transform = SetWorksheetRowHeightTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-row-height".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2}],
                "rowHeight": {"0": 24}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_insert_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_set_worksheet_row_is_auto_height_shifts_on_insert_col() {
        let transform = SetWorksheetRowIsAutoHeightTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-row-is-auto-height".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2}],
                "autoHeightInfo": {"0": 1}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_insert_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_set_worksheet_col_width_shifts_on_insert_row() {
        let transform = SetWorksheetColWidthTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-col-width".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 1}],
                "colWidth": {"0": 80}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_set_worksheet_range_theme_style_shifts_on_remove_rows() {
        let transform = SetWorksheetRangeThemeStyleTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-range-theme-style".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "themeName": "theme",
                "range": {"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 1}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let range = params.get("range").unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_delete_worksheet_range_theme_style_removed_by_row_delete() {
        let transform = DeleteWorksheetRangeThemeStyleTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.remove-worksheet-range-theme-style".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "themeName": "theme",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 1}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_set_frozen_shifts_on_insert_col() {
        let transform = SetFrozenTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-frozen".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "startRow": 0,
                "startColumn": 2,
                "ySplit": 0,
                "xSplit": 1
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_insert_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        assert_eq!(params.get("startColumn").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_set_worksheet_col_width_shifts_on_remove_col() {
        let transform = SetWorksheetColWidthTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-col-width".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 0, "startColumn": 2, "endRow": 0, "endColumn": 2}],
                "colWidth": {"2": 80}
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}
            }),
        );
        let result = transform.transform_with_remove_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let ranges = params.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 1);
        let col_width = params.get("colWidth").unwrap().as_object().unwrap();
        assert!(col_width.contains_key("1"));
    }

    #[test]
    fn test_set_worksheet_row_auto_height_shifts_on_insert_row() {
        let transform = SetWorksheetRowAutoHeightTransform::default();
        let m1 = create_mutation_info(
            "sheet.mutation.set-worksheet-row-auto-height".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "rowsAutoHeightInfo": [
                    {"row": 1, "autoHeight": 20},
                    {"row": 2, "autoHeight": 22}
                ]
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let rows = params
            .get("rowsAutoHeightInfo")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(rows[0].get("row").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_identity_range_theme_transforms() {
        let add_transform = AddRangeThemeTransform::default();
        let set_transform = SetRangeThemeTransform::default();
        let register_transform = RegisterWorksheetRangeThemeStyleTransform::default();
        let unregister_transform = UnregisterWorksheetRangeThemeStyleTransform::default();
        let delete_transform = DeleteRangeProtectionTransform::default();
        let insert_row = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let add_range_theme = create_mutation_info(
            "sheet.mutation.add-range-theme".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "styleJSON": {"name": "theme"}
            }),
        );
        let set_range_theme = create_mutation_info(
            "sheet.mutation.set-range-theme".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "styleName": "theme",
                "style": {"headerRowStyle": {"bg": "#fff"}}
            }),
        );
        let register_theme_style = create_mutation_info(
            "sheet.mutation.register-worksheet-range-theme-style".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "themeName": "theme",
                "rangeThemeStyleJson": {"name": "theme"}
            }),
        );
        let unregister_theme_style = create_mutation_info(
            "sheet.mutation.unregister-worksheet-range-theme-style".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "themeName": "theme"
            }),
        );
        let delete_params = create_mutation_info(
            "sheet.mutation.delete-range-protection".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ruleIds": ["rule-1"]
            }),
        );
        assert_eq!(
            add_transform
                .transform_with_insert_row(&add_range_theme, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            add_range_theme.params
        );
        assert_eq!(
            set_transform
                .transform_with_insert_row(&set_range_theme, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_range_theme.params
        );
        assert_eq!(
            register_transform
                .transform_with_insert_row(&register_theme_style, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            register_theme_style.params
        );
        assert_eq!(
            unregister_transform
                .transform_with_insert_row(&unregister_theme_style, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            unregister_theme_style.params
        );
        assert_eq!(
            delete_transform
                .transform_with_insert_row(&delete_params, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            delete_params.params
        );
    }

    #[test]
    fn test_remove_range_theme_discards_matching_theme_mutations() {
        let remove_theme = create_mutation_info(
            "sheet.mutation.remove-range-theme".to_string(),
            serde_json::json!({"unitId": "unit", "styleName": "theme"}),
        );
        let add_theme = create_mutation_info(
            "sheet.mutation.add-range-theme".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "styleJSON": {"name": "theme"}
            }),
        );
        let set_theme = create_mutation_info(
            "sheet.mutation.set-range-theme".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "styleName": "theme",
                "style": {"headerRowStyle": {"bg": "#fff"}}
            }),
        );
        let result_add = RemoveRangeThemeTransform::default()
            .transform_with_add_range_theme(&remove_theme, &add_theme);
        assert!(result_add.m2_prime.is_none());
        let result_set = RemoveRangeThemeTransform::default()
            .transform_with_set_range_theme(&remove_theme, &set_theme);
        assert!(result_set.m2_prime.is_none());
    }

    #[test]
    fn test_identity_misc_transforms() {
        let add_ws_protection = create_mutation_info(
            "sheet.mutation.add-worksheet-protection".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "permissions": []}),
        );
        let copy_sheet = create_mutation_info(
            "sheet.mutation.copy-worksheet-end".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet"}),
        );
        let delete_ws_protection = create_mutation_info(
            "sheet.mutation.delete-worksheet-protection".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet"}),
        );
        let empty = create_mutation_info("sheet.mutation.empty".to_string(), serde_json::json!({}));
        let insert_sheet = create_mutation_info(
            "sheet.mutation.insert-sheet".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "sheet": {"id": "sheet"}}),
        );
        let remove_range_theme = create_mutation_info(
            "sheet.mutation.remove-range-theme".to_string(),
            serde_json::json!({"unitId": "unit", "styleName": "theme"}),
        );
        let remove_sheet = create_mutation_info(
            "sheet.mutation.remove-sheet".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet"}),
        );
        let set_gridlines_color = create_mutation_info(
            "sheet.mutation.set-gridlines-color".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "color": "#000"}),
        );
        let set_tab_color = create_mutation_info(
            "sheet.mutation.set-tab-color".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "color": "#fff"}),
        );
        let set_workbook_name = create_mutation_info(
            "sheet.mutation.set-workbook-name".to_string(),
            serde_json::json!({"unitId": "unit", "name": "book"}),
        );
        let set_column_count = create_mutation_info(
            "sheet.mutation.set-worksheet-column-count".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "columnCount": 10}),
        );
        let set_default_style = create_mutation_info(
            "sheet.mutation.set-worksheet-default-style".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "style": {"bg": "#fff"}}),
        );
        let set_hidden = create_mutation_info(
            "sheet.mutation.set-worksheet-hidden".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "hidden": true}),
        );
        let set_name = create_mutation_info(
            "sheet.mutation.set-worksheet-name".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "name": "Sheet1"}),
        );
        let set_order = create_mutation_info(
            "sheet.mutation.set-worksheet-order".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "order": 1}),
        );
        let set_permission = create_mutation_info(
            "sheet.mutation.set-worksheet-permission-points".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "permissionPoints": []}),
        );
        let set_protection = create_mutation_info(
            "sheet.mutation.set-worksheet-protection".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "permissions": []}),
        );
        let set_rtl = create_mutation_info(
            "sheet.mutation.set-worksheet-right-to-left".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "rightToLeft": true}),
        );
        let set_row_count = create_mutation_info(
            "sheet.mutation.set-worksheet-row-count".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "rowCount": 100}),
        );
        let toggle_gridlines = create_mutation_info(
            "sheet.mutation.toggle-gridlines".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "show": true}),
        );
        let insert_row = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}}),
        );
        assert_eq!(
            AddWorksheetProtectionTransform::default()
                .transform_with_insert_row(&add_ws_protection, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            add_ws_protection.params
        );
        assert_eq!(
            CopyWorksheetEndTransform::default()
                .transform_with_insert_row(&copy_sheet, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            copy_sheet.params
        );
        assert_eq!(
            DeleteWorksheetProtectionTransform::default()
                .transform_with_insert_row(&delete_ws_protection, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            delete_ws_protection.params
        );
        assert_eq!(
            EmptyTransform::default()
                .transform_with_insert_row(&empty, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            empty.params
        );
        assert_eq!(
            InsertSheetTransform::default()
                .transform_with_insert_row(&insert_sheet, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            insert_sheet.params
        );
        assert_eq!(
            RemoveRangeThemeTransform::default()
                .transform_with_insert_row(&remove_range_theme, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            remove_range_theme.params
        );
        assert_eq!(
            RemoveSheetTransform::default()
                .transform_with_insert_row(&remove_sheet, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            remove_sheet.params
        );
        assert_eq!(
            SetGridlinesColorTransform::default()
                .transform_with_insert_row(&set_gridlines_color, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_gridlines_color.params
        );
        assert_eq!(
            SetTabColorTransform::default()
                .transform_with_insert_row(&set_tab_color, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_tab_color.params
        );
        assert_eq!(
            SetWorkbookNameTransform::default()
                .transform_with_insert_row(&set_workbook_name, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_workbook_name.params
        );
        assert_eq!(
            SetWorksheetColumnCountTransform::default()
                .transform_with_insert_row(&set_column_count, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_column_count.params
        );
        assert_eq!(
            SetWorksheetDefaultStyleTransform::default()
                .transform_with_insert_row(&set_default_style, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_default_style.params
        );
        assert_eq!(
            SetWorksheetHiddenTransform::default()
                .transform_with_insert_row(&set_hidden, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_hidden.params
        );
        assert_eq!(
            SetWorksheetNameTransform::default()
                .transform_with_insert_row(&set_name, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_name.params
        );
        assert_eq!(
            SetWorksheetOrderTransform::default()
                .transform_with_insert_row(&set_order, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_order.params
        );
        assert_eq!(
            SetWorksheetPermissionPointsTransform::default()
                .transform_with_insert_row(&set_permission, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_permission.params
        );
        assert_eq!(
            SetWorksheetProtectionTransform::default()
                .transform_with_insert_row(&set_protection, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_protection.params
        );
        assert_eq!(
            SetWorksheetRightToLeftTransform::default()
                .transform_with_insert_row(&set_rtl, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_rtl.params
        );
        assert_eq!(
            SetWorksheetRowCountTransform::default()
                .transform_with_insert_row(&set_row_count, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            set_row_count.params
        );
        assert_eq!(
            ToggleGridlinesTransform::default()
                .transform_with_insert_row(&toggle_gridlines, &insert_row)
                .m1_prime
                .unwrap()
                .params,
            toggle_gridlines.params
        );
    }

    #[test]
    fn test_remove_sheet_discards_mutations_on_same_sheet() {
        let remove_sheet = create_mutation_info(
            "sheet.mutation.remove-sheet".to_string(),
            serde_json::json!({
                "unitId": "workbook1",
                "subUnitId": "sheet1"
            }),
        );

        let set_range_values = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::json!({
                "unitId": "workbook1",
                "subUnitId": "sheet1",
                "cellValue": {}
            }),
        );

        let insert_row = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "workbook1",
                "subUnitId": "sheet1",
                "range": {
                    "startRow": 5,
                    "startColumn": 0,
                    "endRow": 7,
                    "endColumn": 10
                }
            }),
        );

        let add_protection = create_mutation_info(
            "sheet.mutation.add-range-protection".to_string(),
            serde_json::json!({
                "unitId": "workbook1",
                "subUnitId": "sheet1",
                "range": {
                    "startRow": 0,
                    "startColumn": 0,
                    "endRow": 10,
                    "endColumn": 10
                }
            }),
        );

        let result1 = RemoveSheetTransform::default()
            .transform_with_set_range_values(&remove_sheet, &set_range_values);
        assert!(result1.m1_prime.is_some());
        assert!(result1.m2_prime.is_none());

        let result2 =
            RemoveSheetTransform::default().transform_with_insert_row(&remove_sheet, &insert_row);
        assert!(result2.m1_prime.is_some());
        assert!(result2.m2_prime.is_none());

        let result3 = RemoveSheetTransform::default()
            .transform_with_add_range_protection(&remove_sheet, &add_protection);
        assert!(result3.m1_prime.is_some());
        assert!(result3.m2_prime.is_none());
    }

    #[test]
    fn test_remove_sheet_preserves_mutations_on_different_sheet() {
        let remove_sheet = create_mutation_info(
            "sheet.mutation.remove-sheet".to_string(),
            serde_json::json!({
                "unitId": "workbook1",
                "subUnitId": "sheet1"
            }),
        );

        let set_range_values_other_sheet = create_mutation_info(
            "sheet.mutation.set-range-values".to_string(),
            serde_json::json!({
                "unitId": "workbook1",
                "subUnitId": "sheet2",
                "cellValue": {}
            }),
        );

        let insert_row_other_workbook = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "workbook2",
                "subUnitId": "sheet1",
                "range": {
                    "startRow": 5,
                    "startColumn": 0,
                    "endRow": 7,
                    "endColumn": 10
                }
            }),
        );

        let result1 = RemoveSheetTransform::default()
            .transform_with_set_range_values(&remove_sheet, &set_range_values_other_sheet);
        assert!(result1.m1_prime.is_some());
        assert!(result1.m2_prime.is_some());

        let result2 = RemoveSheetTransform::default()
            .transform_with_insert_row(&remove_sheet, &insert_row_other_workbook);
        assert!(result2.m1_prime.is_some());
        assert!(result2.m2_prime.is_some());
    }

    #[test]
    fn test_remove_worksheet_merge_discards_add_on_overlap() {
        let remove_merge = create_mutation_info(
            "sheet.mutation.remove-worksheet-merge".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 0, "startColumn": 0, "endRow": 1, "endColumn": 1}]
            }),
        );
        let add_merge = create_mutation_info(
            "sheet.mutation.add-worksheet-merge".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ranges": [{"startRow": 1, "startColumn": 1, "endRow": 2, "endColumn": 2}]
            }),
        );
        let result = RemoveWorksheetMergeTransform::default()
            .transform_with_add_worksheet_merge(&remove_merge, &add_merge);
        assert!(result.m2_prime.is_none());
    }

    #[test]
    fn test_delete_worksheet_range_theme_style_discards_set_on_overlap() {
        let delete_theme_style = create_mutation_info(
            "sheet.mutation.remove-worksheet-range-theme-style".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 1, "endColumn": 1}
            }),
        );
        let set_theme_style = create_mutation_info(
            "sheet.mutation.set-worksheet-range-theme-style".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 1, "startColumn": 1, "endRow": 2, "endColumn": 2},
                "themeName": "theme"
            }),
        );
        let result = DeleteWorksheetRangeThemeStyleTransform::default()
            .transform_with_set_worksheet_range_theme_style(&delete_theme_style, &set_theme_style);
        assert!(result.m2_prime.is_none());
    }

    #[test]
    fn test_delete_range_protection_discards_matching_rule() {
        let delete_protection = create_mutation_info(
            "sheet.mutation.delete-range-protection".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ruleIds": ["rule-1"]
            }),
        );
        let add_protection = create_mutation_info(
            "sheet.mutation.add-range-protection".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "rules": [{"id": "rule-1", "ranges": [{"startRow": 0, "startColumn": 0, "endRow": 1, "endColumn": 1}]}]
            }),
        );
        let result = DeleteRangeProtectionTransform::default()
            .transform_with_add_range_protection(&delete_protection, &add_protection);
        assert!(result.m2_prime.is_none());
    }

    #[test]
    fn test_delete_worksheet_protection_discards_set() {
        let delete_protection = create_mutation_info(
            "sheet.mutation.delete-worksheet-protection".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet"}),
        );
        let set_protection = create_mutation_info(
            "sheet.mutation.set-worksheet-protection".to_string(),
            serde_json::json!({"unitId": "unit", "subUnitId": "sheet", "permissions": []}),
        );
        let result = DeleteWorksheetProtectionTransform::default()
            .transform_with_set_worksheet_protection(&delete_protection, &set_protection);
        assert!(result.m2_prime.is_none());
    }

    #[test]
    fn test_add_data_validation_shifts_on_insert_row() {
        let transform = AddDataValidationTransform::default();
        let m1 = create_mutation_info(
            "data-validation.mutation.addRule".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "rule": {
                    "uid": "rule-1",
                    "ranges": [{"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}]
                }
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let rule = params.get("rule").unwrap().as_object().unwrap();
        let ranges = rule.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_add_data_validation_shifts_on_insert_col() {
        let transform = AddDataValidationTransform::default();
        let m1 = create_mutation_info(
            "data-validation.mutation.addRule".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "rule": {
                    "uid": "rule-1",
                    "ranges": [{"startRow": 0, "startColumn": 1, "endRow": 0, "endColumn": 1}]
                }
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-col".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_col(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let rule = params.get("rule").unwrap().as_object().unwrap();
        let ranges = rule.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startColumn").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_add_data_validation_shifts_on_remove_rows() {
        let transform = AddDataValidationTransform::default();
        let m1 = create_mutation_info(
            "data-validation.mutation.addRule".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "rule": {
                    "uid": "rule-1",
                    "ranges": [{"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 0}]
                }
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let rule = params.get("rule").unwrap().as_object().unwrap();
        let ranges = rule.get("ranges").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_add_data_validation_removed_on_overlap() {
        let transform = AddDataValidationTransform::default();
        let m1 = create_mutation_info(
            "data-validation.mutation.addRule".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "rule": {
                    "uid": "rule-1",
                    "ranges": [{"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}]
                }
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_update_data_validation_shifts_on_insert_row() {
        let transform = UpdateDataValidationTransform::default();
        let m1 = create_mutation_info(
            "data-validation.mutation.updateRule".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ruleId": "rule-1",
                "payload": {
                    "type": "RANGE",
                    "payload": [{"startRow": 1, "startColumn": 0, "endRow": 1, "endColumn": 0}]
                }
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let payload = params.get("payload").unwrap().as_object().unwrap();
        let ranges = payload.get("payload").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 2);
    }

    #[test]
    fn test_remove_data_validation_identity() {
        let transform = RemoveDataValidationTransform::default();
        let m1 = create_mutation_info(
            "data-validation.mutation.removeRule".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ruleId": "rule-1"
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.insert-row".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_insert_row(&m1, &m2);
        // RemoveDataValidation should be identity - no changes needed
        assert!(result.m1_prime.is_some());
        assert!(result.m2_prime.is_some());
    }

    #[test]
    fn test_update_data_validation_shifts_on_remove_rows() {
        let transform = UpdateDataValidationTransform::default();
        let m1 = create_mutation_info(
            "data-validation.mutation.updateRule".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ruleId": "rule-1",
                "payload": {
                    "type": "RANGE",
                    "payload": [{"startRow": 2, "startColumn": 0, "endRow": 2, "endColumn": 0}]
                }
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        let params = result.m1_prime.unwrap().params;
        let payload = params.get("payload").unwrap().as_object().unwrap();
        let ranges = payload.get("payload").unwrap().as_array().unwrap();
        let range = ranges[0].as_object().unwrap();
        assert_eq!(range.get("startRow").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_update_data_validation_removed_on_overlap() {
        let transform = UpdateDataValidationTransform::default();
        let m1 = create_mutation_info(
            "data-validation.mutation.updateRule".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ruleId": "rule-1",
                "payload": {
                    "type": "RANGE",
                    "payload": [{"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}]
                }
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        // Range is fully removed, mutation should be cancelled
        assert!(result.m1_prime.is_none());
    }

    #[test]
    fn test_update_data_validation_non_range_preserved() {
        let transform = UpdateDataValidationTransform::default();
        let m1 = create_mutation_info(
            "data-validation.mutation.updateRule".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "ruleId": "rule-1",
                "payload": {
                    "type": "SETTING",
                    "payload": {"prompt": "Enter value"}
                }
            }),
        );
        let m2 = create_mutation_info(
            "sheet.mutation.remove-rows".to_string(),
            serde_json::json!({
                "unitId": "unit",
                "subUnitId": "sheet",
                "range": {"startRow": 0, "startColumn": 0, "endRow": 0, "endColumn": 0}
            }),
        );
        let result = transform.transform_with_remove_rows(&m1, &m2);
        // Non-RANGE type should be preserved
        assert!(result.m1_prime.is_some());
        assert!(result.m2_prime.is_some());
    }
}
