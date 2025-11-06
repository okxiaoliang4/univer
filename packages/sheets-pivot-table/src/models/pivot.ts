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

import type { ICellData, IObjectMatrixPrimitiveType, Nullable } from '@univerjs/core';
import type { IPivotField, IPivotFilterCriteria } from '../types/type';
import { Disposable } from '@univerjs/core';
import { createAggregator } from '../model/aggregation/functions';
import { AggregationType } from '../types/enum';

interface IPivotConfig {
    valueFields: IPivotField[];
    rowFields: IPivotField[];
    columnFields: IPivotField[];
    filterFields: IPivotField[];
    sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>;
}

interface IPivot {
    valueFields: IPivotField[];
    rowFields: IPivotField[];
    columnFields: IPivotField[];
    filterFields: IPivotField[];
    sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>;
}

export class Pivot extends Disposable implements IPivot {
    valueFields: IPivotField[];
    rowFields: IPivotField[];
    columnFields: IPivotField[];
    filterFields: IPivotField[];
    sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>;
    private _calculatedData: IObjectMatrixPrimitiveType<Nullable<ICellData>> | null;

    // Dirty flag and cache management
    private _isDirty: boolean = true;

    constructor(config: IPivotConfig) {
        super();

        this.valueFields = config.valueFields;
        this.rowFields = config.rowFields;
        this.columnFields = config.columnFields;
        this.filterFields = config.filterFields;
        this.sourceData = config.sourceData;
        this._calculatedData = null;
    }

    /**
     * Get the calculated data, if the pivot is dirty, it will recalculate the data
     * @returns The calculated data
     */
    getCalculatedData(): IObjectMatrixPrimitiveType<Nullable<ICellData>> | null {
        if (this._isDirty) {
            this._calculatedData = this._calculate();
            this._isDirty = false;
        }
        return this._calculatedData;
    }

    getValueFields(): IPivotField[] {
        return this.valueFields;
    }

    getRowFields(): IPivotField[] {
        return this.rowFields;
    }

    getColumnFields(): IPivotField[] {
        return this.columnFields;
    }

    getFilterFields(): IPivotField[] {
        return this.filterFields;
    }

    setValueFields(valueFields: IPivotField[]): void {
        this.valueFields = valueFields;
        this._markDirty();
    }

    setRowFields(rowFields: IPivotField[]): void {
        this.rowFields = rowFields;
        this._markDirty();
    }

    setColumnFields(columnFields: IPivotField[]): void {
        this.columnFields = columnFields;
        this._markDirty();
    }

    setFilterFields(filterFields: IPivotField[]): void {
        this.filterFields = filterFields;
        this._markDirty();
    }

    setSourceData(sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>): void {
        this.sourceData = sourceData;
        this._markDirty();
    }

    /**
     * Mark the pivot as dirty and schedule a debounced recalculation
     */
    private _markDirty(): void {
        this._isDirty = true;
    }

    /**
     * Check if the pivot is currently dirty
     */
    isDirty(): boolean {
        return this._isDirty;
    }

    /**
     * Calculate the pivot data
     * @returns The calculated data
     */
    private _calculate(): IObjectMatrixPrimitiveType<Nullable<ICellData>> | null {
        const sourceData = this.sourceData;
        const rowFields = this.rowFields;
        const columnFields = this.columnFields;
        const valueFields = this.valueFields;
        const filterFields = this.filterFields;

        // Extract header row and data rows
        let { dataRows } = this._extractData(sourceData);

        if (dataRows.length === 0 || valueFields.length === 0) {
            return null;
        }

        // Apply filters if any
        if (filterFields && filterFields.length > 0) {
            dataRows = this._applyFilters(dataRows, filterFields);
            if (dataRows.length === 0) {
                return null; // No data after filtering
            }
        }

        // Get field indices from header
        const getFieldIndex = (field: IPivotField): number => {
            return field.sourceColumnIndex;
        };

        const rowFieldIndices = rowFields.map(getFieldIndex).filter((i) => i >= 0);
        const columnFieldIndices = columnFields.map(getFieldIndex).filter((i) => i >= 0);
        const valueFieldIndices = valueFields.map(getFieldIndex).filter((i) => i >= 0);

        // Build result based on configuration
        if (columnFields.length === 0) {
            // Simple row-based pivot (no column fields)
            return this._calculateRowOnly(dataRows, rowFieldIndices, valueFieldIndices, rowFields, valueFields);
        } else {
            // 2D pivot (with column fields)
            return this._calculate2D(dataRows, rowFieldIndices, columnFieldIndices, valueFieldIndices, valueFields);
        }
    }

    private _extractData(sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>): {
        headerRow: ICellData[];
        dataRows: ICellData[][];
    } {
        const rows = Object.keys(sourceData).map(Number).sort((a, b) => a - b);
        if (rows.length === 0) {
            return { headerRow: [], dataRows: [] };
        }

        // Find the maximum column index across all rows
        let maxCol = 0;
        for (const rowIndex of rows) {
            const rowData = sourceData[rowIndex];
            if (rowData) {
                const cols = Object.keys(rowData).map(Number);
                for (const col of cols) {
                    if (col > maxCol) {
                        maxCol = col;
                    }
                }
            }
        }

        // First row is header
        const headerRowIndex = rows[0];
        const headerRow: ICellData[] = [];
        const headerRowData = sourceData[headerRowIndex];
        for (let col = 0; col <= maxCol; col++) {
            headerRow.push(headerRowData?.[col] || {});
        }

        // Rest are data rows - ensure all rows have the same number of columns
        const dataRows: ICellData[][] = [];
        for (let i = 1; i < rows.length; i++) {
            const rowIndex = rows[i];
            const rowData = sourceData[rowIndex];
            const row: ICellData[] = [];
            for (let col = 0; col <= maxCol; col++) {
                row.push(rowData?.[col] || {});
            }
            dataRows.push(row);
        }

        return { headerRow, dataRows };
    }

    private _calculateRowOnly(
        dataRows: ICellData[][],
        rowFieldIndices: number[],
        valueFieldIndices: number[],
        rowFields: IPivotField[],
        valueFields: IPivotField[]
    ): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        const result: IObjectMatrixPrimitiveType<Nullable<ICellData>> = {};

        if (rowFieldIndices.length === 0) {
            let currentRow = 0;

              // Add header row if there are multiple value fields
            if (valueFields.length > 1) {
                result[currentRow] = {};
                for (let i = 0; i < valueFields.length; i++) {
                    const aggregation = valueFields[i].aggregation || AggregationType.SUM;
                    result[currentRow][i] = { v: this._getAggregationLabel(aggregation, valueFields[i].name) };
                }
                currentRow++;
            }

              // Calculate aggregated values
            result[currentRow] = {};
            for (let i = 0; i < valueFieldIndices.length; i++) {
                const valueFieldIndex = valueFieldIndices[i];
                const aggregation = valueFields[i].aggregation || AggregationType.SUM;
                const aggregator = createAggregator(aggregation);
                aggregator.init();

                for (const row of dataRows) {
                    const valueCell = row[valueFieldIndex];
                    if (valueCell) {
                        aggregator.addValue(valueCell);
                    }
                }

                result[currentRow][i] = { v: aggregator.getResult() };
            }

            return result;
        }

        const groups = this._groupByFields(dataRows, rowFieldIndices);

        // Build result matrix
        let currentRow = 0;

        // Add header row for multiple value fields
        if (valueFields.length > 1) {
            result[currentRow] = {};
            // Row field headers
            for (let i = 0; i < rowFields.length; i++) {
                result[currentRow][i] = { v: rowFields[i].name };
            }
            // Value field headers
            for (let i = 0; i < valueFields.length; i++) {
                const aggregation = valueFields[i].aggregation || AggregationType.SUM;
                result[currentRow][rowFields.length + i] = { v: this._getAggregationLabel(aggregation, valueFields[i].name) };
            }
            currentRow++;
        }

        // Sort groups by key
        const sortedGroupKeys = Array.from(groups.keys()).sort();

        // Add data rows
        for (const groupKey of sortedGroupKeys) {
            const groupRows = groups.get(groupKey);
            if (!groupRows) continue;

            result[currentRow] = {};

            // Add row field values
            const fieldValues = groupKey.split('|');
            for (let i = 0; i < fieldValues.length; i++) {
                result[currentRow][i] = { v: fieldValues[i] === '(blank)' ? '' : fieldValues[i] };
            }

            // Calculate aggregated values for each value field
            for (let i = 0; i < valueFieldIndices.length; i++) {
                const valueFieldIndex = valueFieldIndices[i];
                const aggregation = valueFields[i].aggregation || AggregationType.SUM;
                const aggregator = createAggregator(aggregation);
                aggregator.init();

                for (const row of groupRows) {
                    const valueCell = row[valueFieldIndex];
                    if (valueCell) {
                        aggregator.addValue(valueCell);
                    }
                }

                result[currentRow][rowFieldIndices.length + i] = { v: aggregator.getResult() };
            }

            currentRow++;
        }

        return result;
    }

    private _calculate2D(
        dataRows: ICellData[][],
        rowFieldIndices: number[],
        columnFieldIndices: number[],
        valueFieldIndices: number[],
        valueFields: IPivotField[]
    ): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        const result: IObjectMatrixPrimitiveType<Nullable<ICellData>> = {};

        // Get all unique column combinations
        const columnCombos = this._getColumnCombinations(dataRows, columnFieldIndices);

        // Group data by row fields
        const rowGroups = rowFieldIndices.length > 0
            ? this._groupByFields(dataRows, rowFieldIndices)
            : new Map([['', dataRows]]);

        const sortedRowKeys = Array.from(rowGroups.keys()).sort();

        // Build header row(s)
        let currentRow = 0;

        // For simplicity, create a single header row with column values
        result[currentRow] = {};
        result[currentRow][0] = { v: '' }; // Top-left corner

        for (let i = 0; i < columnCombos.length; i++) {
            const combo = columnCombos[i];
            const label = combo.map((v) => v === '(blank)' ? '' : v).join(' - ');
            result[currentRow][i + 1] = { v: label || combo[0] };
        }
        currentRow++;

        // Build data rows
        for (const rowKey of sortedRowKeys) {
            const groupRows = rowGroups.get(rowKey);
            if (!groupRows) continue;

            result[currentRow] = {};

            // Add row field values
            const fieldValues = rowKey ? rowKey.split('|') : [''];
            result[currentRow][0] = { v: fieldValues[0] === '(blank)' ? '' : fieldValues[0] };

            // Calculate values for each column combination
            for (let colIdx = 0; colIdx < columnCombos.length; colIdx++) {
                const columnCombo = columnCombos[colIdx];

              // Filter rows that match this column combination
                const filteredRows = this._filterByColumnCombo(groupRows, columnFieldIndices, columnCombo);

              // For now, use first value field
                const valueFieldIndex = valueFieldIndices[0];
                const aggregation = valueFields[0].aggregation || AggregationType.SUM;
                const aggregator = createAggregator(aggregation);
                aggregator.init();

                for (const row of filteredRows) {
                    const valueCell = row[valueFieldIndex];
                    if (valueCell) {
                        aggregator.addValue(valueCell);
                    }
                }

                result[currentRow][colIdx + 1] = { v: aggregator.getResult() };
            }

            currentRow++;
        }

        return result;
    }

    private _groupByFields(dataRows: ICellData[][], fieldIndices: number[]): Map<string, ICellData[][]> {
        const groups = new Map<string, ICellData[][]>();

        for (const row of dataRows) {
            const keyParts: string[] = [];
            for (const fieldIndex of fieldIndices) {
                const cell = row[fieldIndex];
                const value = cell?.v?.toString() || '(blank)';
                keyParts.push(value);
            }
            const key = keyParts.join('|');

            if (!groups.has(key)) {
                groups.set(key, []);
            }
            const group = groups.get(key);
            if (group) {
                group.push(row);
            }
        }

        return groups;
    }

    private _getColumnCombinations(dataRows: ICellData[][], fieldIndices: number[]): string[][] {
        const combinations = new Set<string>();

        for (const row of dataRows) {
            const combo: string[] = [];
            for (const fieldIndex of fieldIndices) {
                const cell = row[fieldIndex];
                const value = cell?.v?.toString() || '(blank)';
                combo.push(value);
            }
            combinations.add(combo.join('|'));
        }

        // Convert to array and sort
        const result: string[][] = [];
        const sorted = Array.from(combinations).sort();
        for (const comboKey of sorted) {
            result.push(comboKey.split('|'));
        }

        return result;
    }

    private _filterByColumnCombo(dataRows: ICellData[][], fieldIndices: number[], columnCombo: string[]): ICellData[][] {
        return dataRows.filter((row) => {
            for (let i = 0; i < fieldIndices.length; i++) {
                const fieldIndex = fieldIndices[i];
                const expectedValue = columnCombo[i];
                const actualCell = row[fieldIndex];
                const actualValue = actualCell?.v?.toString() || '(blank)';
                if (actualValue !== expectedValue) {
                    return false;
                }
            }
            return true;
        });
    }

    /**
     * Apply filters to data rows
     * @param dataRows The data rows to filter
     * @param filterFields The filter fields configuration
     * @returns Filtered data rows
     */
    private _applyFilters(dataRows: ICellData[][], filterFields: IPivotField[]): ICellData[][] {
        return dataRows.filter((row) => {
            // Apply all filters with AND logic
            for (const filterField of filterFields) {
                if (!filterField.filter) continue;

                const fieldIndex = filterField.sourceColumnIndex;
                const cell = row[fieldIndex];
                const cellValue = cell?.v;

                if (!this._matchesFilter(cellValue, filterField.filter)) {
                    return false; // Row doesn't match this filter, exclude it
                }
            }
            return true; // Row matches all filters
        });
    }

    /**
     * Check if a value matches the filter criteria
     * @param value The value to check
     * @param filter The filter criteria
     * @returns True if value matches the filter
     */
    private _matchesFilter(value: any, filter: IPivotFilterCriteria): boolean {
        if (filter.type === 'value') {
            // Value filter: check if value is in the allowed values list
            return this._matchesValueFilter(value, filter);
        } else if (filter.type === 'condition') {
            // Condition filter: check against operator and condition value
            return this._matchesConditionFilter(value, filter);
        }
        return true;
    }

    /**
     * Check value filter (include specific values)
     * @param value The value to check
     * @param filter The filter criteria
     * @returns True if value is in the allowed list
     */
    private _matchesValueFilter(value: any, filter: IPivotFilterCriteria): boolean {
        if (!filter.values || filter.values.length === 0) {
            return true; // No values specified, allow all
        }

        const valueStr = value?.toString() || '';
        return filter.values.includes(valueStr);
    }

    /**
     * Check condition filter (comparison/text matching)
     * @param value The value to check
     * @param filter The filter criteria
     * @returns True if value matches the condition
     */
    private _matchesConditionFilter(value: any, filter: IPivotFilterCriteria): boolean {
        if (!filter.operator || filter.conditionValue === undefined) {
            return true;
        }

        const { operator, conditionValue } = filter;

        switch (operator) {
            case 'equals':
                return value?.toString() === conditionValue?.toString();

            case 'notEquals':
                return value?.toString() !== conditionValue?.toString();

            case 'greaterThan': {
                const numValue = typeof value === 'number' ? value : Number(value);
                const numCondition = typeof conditionValue === 'number' ? conditionValue : Number(conditionValue);
                return !Number.isNaN(numValue) && !Number.isNaN(numCondition) && numValue > numCondition;
            }

            case 'lessThan': {
                const numValue = typeof value === 'number' ? value : Number(value);
                const numCondition = typeof conditionValue === 'number' ? conditionValue : Number(conditionValue);
                return !Number.isNaN(numValue) && !Number.isNaN(numCondition) && numValue < numCondition;
            }

            case 'contains': {
                const valueStr = value?.toString() || '';
                const conditionStr = conditionValue?.toString() || '';
                return valueStr.includes(conditionStr);
            }

            default:
                return true;
        }
    }

    /**
     * Get the label for aggregation type
     * @param aggregation The aggregation type
     * @param fieldName The field name
     * @returns The formatted label
     */
    private _getAggregationLabel(aggregation: AggregationType, fieldName: string): string {
        const labels: Record<AggregationType, string> = {
            [AggregationType.SUM]: 'Sum',
            [AggregationType.COUNT]: 'Count',
            [AggregationType.AVERAGE]: 'Average',
            [AggregationType.MIN]: 'Min',
            [AggregationType.MAX]: 'Max',
        };
        return `${labels[aggregation]} of ${fieldName}`;
    }
}
