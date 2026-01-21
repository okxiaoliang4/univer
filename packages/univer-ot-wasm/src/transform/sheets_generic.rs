use crate::mutations::sheets::{
    SheetMoveRangeParams, SheetMoveRowsColsParams, SheetMutationRangeParams,
    SheetMutationRangesParams, SheetRangeThemeStyleParams, SheetReorderRangeParams,
    SheetRowColCountsParams, SheetRowColumnDataParams, SheetSetFrozenParams,
    SheetWorksheetMergeParams, SheetWorksheetRangeThemeStyleParams,
};
use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_transform_utils::{
    parse_move_range_params, parse_range_params, shift_move_range_value_for_insert,
    shift_move_range_value_for_remove, shift_range_cols_for_insert, shift_range_cols_for_remove,
    shift_range_rows_for_insert, shift_range_rows_for_remove, shift_set_frozen_for_insert,
    shift_set_frozen_for_remove,
};
use crate::types::{
    InsertColMutationParams, InsertRowMutationParams, MutationInfoInternal, Range,
    RemoveColMutationParams, RemoveRowsMutationParams, SetRangeValuesMutationParams,
    SubUnitParams, TransformResultInternal,
};
use crate::wasm_log_warn;
use serde_json;

#[derive(Default)]
pub struct SheetsGenericTransform {
    pub id: &'static str,
    pub kind: SheetTransformKind,
}

#[derive(Default, Clone, Copy)]
pub enum SheetTransformKind {
    #[default]
    Identity,
    Range,
    Ranges,
    RowData,
    ColumnData,
    RowCount,
    ColumnCount,
    MoveRange,
    MoveRows,
    MoveCols,
    ReorderRange,
    RangeThemeStyle,
    WorksheetRangeThemeStyle,
    WorksheetMerge,
    SetFrozen,
}

impl SheetsGenericTransform {
    pub fn new(id: &'static str, kind: SheetTransformKind) -> Self {
        Self { id, kind }
    }
}

impl MutationTransform for SheetsGenericTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.generic"
    }

    fn transform_with_set_range_values(
        &self,
        m1: &MutationInfoInternal,
        _m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        match self.kind {
            SheetTransformKind::MoveRange => {
                if let Some(mut params) = parse_move_range_params(m1) {
                    let mut shift = false;
                    if let Some(_) = params.from.value.data.get("0") {
                        shift = true;
                    }
                    if shift {
                        // move-range already contains explicit values; no shift for set-range-values
                        return TransformResultInternal {
                            m1_prime: Some(m1.clone()),
                            m2_prime: Some(_m2.clone()),
                            error: None,
                        };
                    }
                }
                TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(_m2.clone()),
                    error: None,
                }
            }
            _ => TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(_m2.clone()),
                error: None,
            },
        }
    }

    fn transform_with_insert_row(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let m2_params: InsertRowMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| InsertRowMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 0,
                    end_row: 0,
                    end_column: 0,
                },
                row_info: None,
            });
        let insert_start = m2_params.range.start_row;
        let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;

        match self.kind {
            SheetTransformKind::Range => {
                let mut params: SheetMutationRangeParams = parse_range_params(&m1.id, &m1.params);
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                shift_range_rows_for_insert(&mut params.range, insert_start, insert_count);
                TransformResultInternal {
                    m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }
            SheetTransformKind::Ranges => {
                let mut params: SheetMutationRangesParams = serde_json::from_value(m1.params.clone())
                    .unwrap_or_else(|_| SheetMutationRangesParams {
                        sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                        ranges: vec![],
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                params.ranges.iter_mut().for_each(|range| shift_range_rows_for_insert(range, insert_start, insert_count));
                TransformResultInternal {
                    m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }
            SheetTransformKind::RowData => {
                let mut params: SheetRowColumnDataParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetRowColumnDataParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    row_data: None,
                    column_data: None,
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                if let Some(ref mut row_data) = params.row_data {
                    let mut new_data = serde_json::Map::new();
                    for (row_key, row_value) in row_data.iter() {
                        if let Ok(row_num) = row_key.parse::<u32>() {
                            if row_num >= insert_start {
                                new_data.insert((row_num + insert_count).to_string(), row_value.clone());
                            } else {
                                new_data.insert(row_key.clone(), row_value.clone());
                            }
                        } else {
                            new_data.insert(row_key.clone(), row_value.clone());
                        }
                    }
                    *row_data = new_data;
                }
                TransformResultInternal {
                    m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }
            SheetTransformKind::RowCount => {
                TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::MoveRange => {
                if let Some(mut params) = parse_move_range_params(m1) {
                    if params.unit_id != m2_params.sub_unit_params.unit_id {
                        return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                    }
                    shift_move_range_value_for_insert(&mut params, Some((insert_start, insert_count)), None);
                    return TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None };
                }
                TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::MoveRows => {
                let mut params: SheetMoveRowsColsParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetMoveRowsColsParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    source_range: Range { start_row: 0, start_column: 0, end_row: 0, end_column: 0 },
                    target_range: Range { start_row: 0, start_column: 0, end_row: 0, end_column: 0 },
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                shift_range_rows_for_insert(&mut params.source_range, insert_start, insert_count);
                shift_range_rows_for_insert(&mut params.target_range, insert_start, insert_count);
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::ReorderRange => {
                let mut params: SheetReorderRangeParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetReorderRangeParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    range: Range { start_row: 0, start_column: 0, end_row: 0, end_column: 0 },
                    order: serde_json::Map::new(),
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                shift_range_rows_for_insert(&mut params.range, insert_start, insert_count);
                let mut new_order = serde_json::Map::new();
                for (key, value) in params.order.iter() {
                    if let Ok(row_num) = key.parse::<u32>() {
                        let new_key = if row_num >= insert_start { row_num + insert_count } else { row_num };
                        new_order.insert(new_key.to_string(), value.clone());
                    } else {
                        new_order.insert(key.clone(), value.clone());
                    }
                }
                params.order = new_order;
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::WorksheetMerge => {
                let mut params: SheetWorksheetMergeParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetWorksheetMergeParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    ranges: vec![],
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                params.ranges.iter_mut().for_each(|range| shift_range_rows_for_insert(range, insert_start, insert_count));
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::SetFrozen => {
                let mut params: SheetSetFrozenParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetSetFrozenParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    start_row: 0,
                    start_column: 0,
                    y_split: 0,
                    x_split: 0,
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                shift_set_frozen_for_insert(&mut params, Some((insert_start, insert_count)), None);
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            _ => TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None },
        }
    }

    fn transform_with_insert_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let m2_params: InsertColMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| InsertColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 0,
                    end_row: 0,
                    end_column: 0,
                },
                col_info: None,
            });
        let insert_start = m2_params.range.start_column;
        let insert_count = m2_params.range.end_column - m2_params.range.start_column + 1;

        match self.kind {
            SheetTransformKind::Range => {
                let mut params: SheetMutationRangeParams = parse_range_params(&m1.id, &m1.params);
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                shift_range_cols_for_insert(&mut params.range, insert_start, insert_count);
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::Ranges => {
                let mut params: SheetMutationRangesParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetMutationRangesParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    ranges: vec![],
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                params.ranges.iter_mut().for_each(|range| shift_range_cols_for_insert(range, insert_start, insert_count));
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::ColumnData => {
                let mut params: SheetRowColumnDataParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetRowColumnDataParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    row_data: None,
                    column_data: None,
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                if let Some(ref mut column_data) = params.column_data {
                    let mut new_data = serde_json::Map::new();
                    for (col_key, col_value) in column_data.iter() {
                        if let Ok(col_num) = col_key.parse::<u32>() {
                            if col_num >= insert_start {
                                new_data.insert((col_num + insert_count).to_string(), col_value.clone());
                            } else {
                                new_data.insert(col_key.clone(), col_value.clone());
                            }
                        } else {
                            new_data.insert(col_key.clone(), col_value.clone());
                        }
                    }
                    *column_data = new_data;
                }
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::ColumnCount => {
                TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::MoveRange => {
                if let Some(mut params) = parse_move_range_params(m1) {
                    if params.unit_id != m2_params.sub_unit_params.unit_id {
                        return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                    }
                    shift_move_range_value_for_insert(&mut params, None, Some((insert_start, insert_count)));
                    return TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None };
                }
                TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::MoveCols => {
                let mut params: SheetMoveRowsColsParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetMoveRowsColsParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    source_range: Range { start_row: 0, start_column: 0, end_row: 0, end_column: 0 },
                    target_range: Range { start_row: 0, start_column: 0, end_row: 0, end_column: 0 },
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                shift_range_cols_for_insert(&mut params.source_range, insert_start, insert_count);
                shift_range_cols_for_insert(&mut params.target_range, insert_start, insert_count);
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::ReorderRange => {
                let mut params: SheetReorderRangeParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetReorderRangeParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    range: Range { start_row: 0, start_column: 0, end_row: 0, end_column: 0 },
                    order: serde_json::Map::new(),
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                shift_range_cols_for_insert(&mut params.range, insert_start, insert_count);
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::WorksheetMerge => {
                let mut params: SheetWorksheetMergeParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetWorksheetMergeParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    ranges: vec![],
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                params.ranges.iter_mut().for_each(|range| shift_range_cols_for_insert(range, insert_start, insert_count));
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::SetFrozen => {
                let mut params: SheetSetFrozenParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetSetFrozenParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    start_row: 0,
                    start_column: 0,
                    y_split: 0,
                    x_split: 0,
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                shift_set_frozen_for_insert(&mut params, None, Some((insert_start, insert_count)));
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            _ => TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None },
        }
    }

    fn transform_with_remove_rows(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let m2_params: RemoveRowsMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| RemoveRowsMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 0,
                    end_row: 0,
                    end_column: 0,
                },
            });
        let remove_start = m2_params.range.start_row;
        let remove_end = m2_params.range.end_row;

        match self.kind {
            SheetTransformKind::Range => {
                let mut params: SheetMutationRangeParams = parse_range_params(&m1.id, &m1.params);
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                if !shift_range_rows_for_remove(&mut params.range, remove_start, remove_end) {
                    return TransformResultInternal { m1_prime: None, m2_prime: Some(m2.clone()), error: None };
                }
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::Ranges => {
                let mut params: SheetMutationRangesParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetMutationRangesParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    ranges: vec![],
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                params.ranges.retain_mut(|range| shift_range_rows_for_remove(range, remove_start, remove_end));
                TransformResultInternal {
                    m1_prime: if params.ranges.is_empty() { None } else { Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }) },
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }
            SheetTransformKind::RowData => {
                let mut params: SheetRowColumnDataParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetRowColumnDataParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    row_data: None,
                    column_data: None,
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                if let Some(ref mut row_data) = params.row_data {
                    let mut new_data = serde_json::Map::new();
                    for (row_key, row_value) in row_data.iter() {
                        if let Ok(row_num) = row_key.parse::<u32>() {
                            if row_num > remove_end {
                                new_data.insert((row_num - (remove_end - remove_start + 1)).to_string(), row_value.clone());
                            } else if row_num < remove_start {
                                new_data.insert(row_key.clone(), row_value.clone());
                            }
                        } else {
                            new_data.insert(row_key.clone(), row_value.clone());
                        }
                    }
                    *row_data = new_data;
                }
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::RowCount => TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None },
            SheetTransformKind::MoveRange => {
                if let Some(mut params) = parse_move_range_params(m1) {
                    if params.unit_id != m2_params.sub_unit_params.unit_id {
                        return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                    }
                    let valid = shift_move_range_value_for_remove(&mut params, Some((remove_start, remove_end)), None);
                    return TransformResultInternal {
                        m1_prime: if valid { Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }) } else { None },
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::MoveRows => {
                let mut params: SheetMoveRowsColsParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetMoveRowsColsParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    source_range: Range { start_row: 0, start_column: 0, end_row: 0, end_column: 0 },
                    target_range: Range { start_row: 0, start_column: 0, end_row: 0, end_column: 0 },
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                if !shift_range_rows_for_remove(&mut params.source_range, remove_start, remove_end)
                    || !shift_range_rows_for_remove(&mut params.target_range, remove_start, remove_end)
                {
                    return TransformResultInternal { m1_prime: None, m2_prime: Some(m2.clone()), error: None };
                }
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::ReorderRange => {
                let mut params: SheetReorderRangeParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetReorderRangeParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    range: Range { start_row: 0, start_column: 0, end_row: 0, end_column: 0 },
                    order: serde_json::Map::new(),
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                if !shift_range_rows_for_remove(&mut params.range, remove_start, remove_end) {
                    return TransformResultInternal { m1_prime: None, m2_prime: Some(m2.clone()), error: None };
                }
                let mut new_order = serde_json::Map::new();
                for (key, value) in params.order.iter() {
                    if let Ok(row_num) = key.parse::<u32>() {
                        if row_num > remove_end {
                            new_order.insert((row_num - (remove_end - remove_start + 1)).to_string(), value.clone());
                        } else if row_num < remove_start {
                            new_order.insert(key.clone(), value.clone());
                        }
                    } else {
                        new_order.insert(key.clone(), value.clone());
                    }
                }
                params.order = new_order;
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::WorksheetMerge => {
                let mut params: SheetWorksheetMergeParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetWorksheetMergeParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    ranges: vec![],
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                params.ranges.retain_mut(|range| shift_range_rows_for_remove(range, remove_start, remove_end));
                TransformResultInternal { m1_prime: if params.ranges.is_empty() { None } else { Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }) }, m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::SetFrozen => {
                let mut params: SheetSetFrozenParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetSetFrozenParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    start_row: 0,
                    start_column: 0,
                    y_split: 0,
                    x_split: 0,
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                shift_set_frozen_for_remove(&mut params, Some((remove_start, remove_end)), None);
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            _ => TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None },
        }
    }

    fn transform_with_remove_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let m2_params: RemoveColMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| RemoveColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 0,
                    end_row: 0,
                    end_column: 0,
                },
            });
        let remove_start = m2_params.range.start_column;
        let remove_end = m2_params.range.end_column;

        match self.kind {
            SheetTransformKind::Range => {
                let mut params: SheetMutationRangeParams = parse_range_params(&m1.id, &m1.params);
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                if !shift_range_cols_for_remove(&mut params.range, remove_start, remove_end) {
                    return TransformResultInternal { m1_prime: None, m2_prime: Some(m2.clone()), error: None };
                }
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::Ranges => {
                let mut params: SheetMutationRangesParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetMutationRangesParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    ranges: vec![],
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                params.ranges.retain_mut(|range| shift_range_cols_for_remove(range, remove_start, remove_end));
                TransformResultInternal { m1_prime: if params.ranges.is_empty() { None } else { Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }) }, m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::ColumnData => {
                let mut params: SheetRowColumnDataParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetRowColumnDataParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    row_data: None,
                    column_data: None,
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                if let Some(ref mut column_data) = params.column_data {
                    let mut new_data = serde_json::Map::new();
                    for (col_key, col_value) in column_data.iter() {
                        if let Ok(col_num) = col_key.parse::<u32>() {
                            if col_num > remove_end {
                                new_data.insert((col_num - (remove_end - remove_start + 1)).to_string(), col_value.clone());
                            } else if col_num < remove_start {
                                new_data.insert(col_key.clone(), col_value.clone());
                            }
                        } else {
                            new_data.insert(col_key.clone(), col_value.clone());
                        }
                    }
                    *column_data = new_data;
                }
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::ColumnCount => TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None },
            SheetTransformKind::MoveRange => {
                if let Some(mut params) = parse_move_range_params(m1) {
                    if params.unit_id != m2_params.sub_unit_params.unit_id {
                        return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                    }
                    let valid = shift_move_range_value_for_remove(&mut params, None, Some((remove_start, remove_end)));
                    return TransformResultInternal {
                        m1_prime: if valid { Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }) } else { None },
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::MoveCols => {
                let mut params: SheetMoveRowsColsParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetMoveRowsColsParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    source_range: Range { start_row: 0, start_column: 0, end_row: 0, end_column: 0 },
                    target_range: Range { start_row: 0, start_column: 0, end_row: 0, end_column: 0 },
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                if !shift_range_cols_for_remove(&mut params.source_range, remove_start, remove_end)
                    || !shift_range_cols_for_remove(&mut params.target_range, remove_start, remove_end)
                {
                    return TransformResultInternal { m1_prime: None, m2_prime: Some(m2.clone()), error: None };
                }
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::ReorderRange => {
                let mut params: SheetReorderRangeParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetReorderRangeParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    range: Range { start_row: 0, start_column: 0, end_row: 0, end_column: 0 },
                    order: serde_json::Map::new(),
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                if !shift_range_cols_for_remove(&mut params.range, remove_start, remove_end) {
                    return TransformResultInternal { m1_prime: None, m2_prime: Some(m2.clone()), error: None };
                }
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::WorksheetMerge => {
                let mut params: SheetWorksheetMergeParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetWorksheetMergeParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    ranges: vec![],
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                params.ranges.retain_mut(|range| shift_range_cols_for_remove(range, remove_start, remove_end));
                TransformResultInternal { m1_prime: if params.ranges.is_empty() { None } else { Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }) }, m2_prime: Some(m2.clone()), error: None }
            }
            SheetTransformKind::SetFrozen => {
                let mut params: SheetSetFrozenParams = serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| SheetSetFrozenParams {
                    sub_unit_params: SubUnitParams { unit_id: "".to_string(), sub_unit_id: "".to_string() },
                    start_row: 0,
                    start_column: 0,
                    y_split: 0,
                    x_split: 0,
                });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None };
                }
                shift_set_frozen_for_remove(&mut params, None, Some((remove_start, remove_end)));
                TransformResultInternal { m1_prime: Some(MutationInfoInternal { id: m1.id.clone(), params: serde_json::to_value(params).unwrap() }), m2_prime: Some(m2.clone()), error: None }
            }
            _ => TransformResultInternal { m1_prime: Some(m1.clone()), m2_prime: Some(m2.clone()), error: None },
        }
    }

    fn compose(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> Vec<MutationInfoInternal> {
        if m1.id != m2.id {
            return vec![m1.clone(), m2.clone()];
        }
        vec![m2.clone()]
    }
}

pub fn build_generic_transform(id: &'static str, kind: SheetTransformKind) -> SheetsGenericTransform {
    SheetsGenericTransform::new(id, kind)
}
