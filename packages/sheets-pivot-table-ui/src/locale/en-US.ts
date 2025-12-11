/**
 * Copyright 2023-present DreamNum Co., Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

const locale = {
    pivotTable: {
        menu: {
            insert: 'Pivot Table',
        },
        dialog: {
            createTitle: 'Create Pivot Table',
            sourceRangeLabel: 'Select source data range',
            targetRangeLabel: 'Choose where to place pivot table',
            targetRangeTypeNew: 'New sheet',
            targetRangeTypeExisting: 'Existing sheet',
            sourceRangeSingleCellError: 'Source range must contain more than one cell',
            sourceRangeSingleRowError: 'Source range must contain at least two rows (header and data)',
            sourceRangeWithMergeError: 'Source range cannot overlap with merged cells',
            targetRangeSingleCellError: 'Target location must be a single cell',
            invalidWorksheet: 'Invalid worksheet',
            cancel: 'Cancel',
            confirm: 'OK',
        },
        editor: {
            title: 'PivotTable Fields',
            availableFields: 'Available Fields',
            filters: 'Filters',
            columns: 'Columns',
            rows: 'Rows',
            values: 'Values',
            dragFieldsHere: 'Drag fields here',
            remove: 'Remove',
            add: 'Add',
            aggregationMethodLabel: 'Aggregation Method',
            aggregationType: {
                sum: 'Sum',
                count: 'Count',
                average: 'Average',
                min: 'Min',
                max: 'Max',
            },
            containerLabel: 'Column {0}',
            containerLabels: {
                sourceFields: 'Available Fields',
                filterFields: 'Filters',
                columnFields: 'Columns',
                rowFields: 'Rows',
                valueFields: 'Values',
            },
        },
        panel: {
            sourceRangeLabel: 'Source Range:',
            fieldConfigurationLabel: 'Field configuration:',
        },
    },
};

export default locale;
