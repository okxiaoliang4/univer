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
import type { IFieldsConfig, IPivotField, IPivotFilterCriteria } from '../types/type';
import { Disposable } from '@univerjs/core';
import { createAggregator } from '../common/aggregation/functions';
import { AggregationType } from '../types/enum';

// Constants
const BLANK_VALUE_PLACEHOLDER = '(blank)';
const GROUP_KEY_SEPARATOR = '|';

export interface IPivotEngineConfig extends IFieldsConfig {
    sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>;
}

export enum PivotValuePosition {
    Row,
    Column,
}

/**
 * Pivot table calculation engine
 * Provides efficient pivot table data calculation with caching support
 */
export class PivotEngine extends Disposable implements IPivotEngineConfig {
    valueFields: IPivotField[];
    rowFields: IPivotField[];
    columnFields: IPivotField[];
    filterFields: IPivotField[];
    sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>;

    valuePosition: PivotValuePosition;

    /** Cached calculated result */
    private _calculatedData: IObjectMatrixPrimitiveType<Nullable<ICellData>> | null;

    /** Dirty flag for cache invalidation */
    private _isDirty: boolean = true;

    constructor(config: IPivotEngineConfig) {
        super();

        this.valueFields = config.valueFields;
        this.rowFields = config.rowFields;
        this.columnFields = config.columnFields;
        this.filterFields = config.filterFields;
        this.sourceData = config.sourceData;
        this.valuePosition = config.valuePosition ?? PivotValuePosition.Column;
        this._calculatedData = null;
    }

    // #region Public API Methods

    /**
     * Get the calculated pivot data
     * Uses caching to avoid redundant calculations
     * @returns The calculated pivot table data or null if no data
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

    setValuePosition(valuePosition: PivotValuePosition): void {
        if (this.valuePosition !== valuePosition) {
            this.valuePosition = valuePosition;
            this._markDirty();
        }
    }

    /**
     * Check if the pivot table needs recalculation
     */
    isDirty(): boolean {
        return this._isDirty;
    }

    // #endregion

    // #region Core Calculation

    /**
     * Mark the pivot as dirty to trigger recalculation on next access
     */
    private _markDirty(): void {
        this._isDirty = true;
    }

    /**
     * Main calculation entry point
     * Orchestrates the entire pivot table calculation process
     * @returns The calculated pivot table data
     */
    private _calculate(): IObjectMatrixPrimitiveType<Nullable<ICellData>> | null {
        // Early return if no value fields
        if (this.valueFields.length === 0) {
            return null;
        }

        // Extract and prepare data
        let { dataRows } = this._extractData(this.sourceData);
        if (dataRows.length === 0) {
            return null;
        }

        // Apply filters before aggregation (performance optimization)
        if (this.filterFields.length > 0) {
            dataRows = this._applyFilters(dataRows, this.filterFields);
            if (dataRows.length === 0) {
                return null;
            }
        }

        // Extract field indices (do this once to avoid repeated property access)
        const fieldIndices = this._extractFieldIndices();

        // Choose calculation strategy based on pivot type
        return this.columnFields.length === 0
            ? this._calculateRowOnlyPivot(dataRows, fieldIndices)
            : this._calculate2DPivot(dataRows, fieldIndices);
    }

    /**
     * Extract field indices from field configurations
     * Optimizes repeated access to sourceColumnIndex
     * @returns Object containing all field indices
     */
    private _extractFieldIndices() {
        return {
            rowIndices: this.rowFields.map((f) => f.sourceColumnIndex).filter((i) => i >= 0),
            columnIndices: this.columnFields.map((f) => f.sourceColumnIndex).filter((i) => i >= 0),
            valueIndices: this.valueFields.map((f) => f.sourceColumnIndex).filter((i) => i >= 0),
        };
    }

    // #endregion

    // #region Data Extraction

    /**
     * Extract header and data rows from source data
     * Optimized to find max column in a single pass
     * Time Complexity: O(rows * cols)
     * @param sourceData The source data matrix
     * @returns Header row and data rows
     */
    private _extractData(sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>): {
        headerRow: ICellData[];
        dataRows: ICellData[][];
    } {
        const rowIndices = Object.keys(sourceData).map(Number).sort((a, b) => a - b);

        if (rowIndices.length === 0) {
            return { headerRow: [], dataRows: [] };
        }

        // Find max column index in a single pass (performance optimization)
        let maxCol = 0;
        for (const rowIndex of rowIndices) {
            const rowData = sourceData[rowIndex];
            if (rowData) {
                const cols = Object.keys(rowData);
                for (const colStr of cols) {
                    const col = Number(colStr);
                    if (col > maxCol) {
                        maxCol = col;
                    }
                }
            }
        }

        // Build header row
        const headerRowIndex = rowIndices[0];
        const headerRow = this._buildRowFromSparse(sourceData[headerRowIndex], maxCol);

        // Build data rows (skip header row at index 0)
        const dataRows: ICellData[][] = [];
        for (let i = 1; i < rowIndices.length; i++) {
            const rowIndex = rowIndices[i];
            dataRows.push(this._buildRowFromSparse(sourceData[rowIndex], maxCol));
        }

        return { headerRow, dataRows };
    }

    /**
     * Build a dense array from sparse row data
     * Reusable utility to avoid code duplication
     * @param rowData Sparse row data
     * @param maxCol Maximum column index
     * @returns Dense array of cells
     */
    private _buildRowFromSparse(
        rowData: Record<number, Nullable<ICellData>> | undefined,
        maxCol: number
    ): ICellData[] {
        const row: ICellData[] = [];
        for (let col = 0; col <= maxCol; col++) {
            row.push(rowData?.[col] || {});
        }
        return row;
    }

    // #endregion

    // #region Row-Only Pivot Calculation

    /**
     * Calculate pivot table with only row fields (no column fields)
     * Handles both grouped and non-grouped scenarios
     * @param dataRows Filtered data rows
     * @param fieldIndices Pre-extracted field indices
     * @param fieldIndices.rowIndices Row field column indices
     * @param fieldIndices.valueIndices Value field column indices
     * @returns Calculated pivot table
     */
    private _calculateRowOnlyPivot(
        dataRows: ICellData[][],
        fieldIndices: { rowIndices: number[]; valueIndices: number[] }
    ): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        const { rowIndices, valueIndices } = fieldIndices;

        // Case 1: No row fields - just aggregate all data
        if (rowIndices.length === 0) {
            return this._aggregateAllData(dataRows, valueIndices);
        }

        // Case 2: Has row fields - group and aggregate
        const groups = this._groupByFields(dataRows, rowIndices);
        return this._buildRowOnlyResult(groups, rowIndices, valueIndices);
    }

    /**
     * Aggregate all data without grouping (no row fields)
     * @param dataRows Data rows to aggregate
     * @param valueIndices Value field indices
     * @returns Result with aggregated values
     */
    private _aggregateAllData(
        dataRows: ICellData[][],
        valueIndices: number[]
    ): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        const result: IObjectMatrixPrimitiveType<Nullable<ICellData>> = {};
        let currentRow = 0;

        // Add header row if multiple value fields
        if (this.valueFields.length > 1) {
            result[currentRow] = this._buildValueFieldHeaders();
            currentRow++;
        }

        // Aggregate all values
        result[currentRow] = this._aggregateValues(dataRows, valueIndices);

        return result;
    }

    /**
     * Build result for row-only pivot with grouping
     * @param groups Grouped data by row fields
     * @param rowIndices Row field indices
     * @param valueIndices Value field indices
     * @returns Result matrix
     */
    private _buildRowOnlyResult(
        groups: Map<string, ICellData[][]>,
        rowIndices: number[],
        valueIndices: number[]
    ): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        const result: IObjectMatrixPrimitiveType<Nullable<ICellData>> = {};
        let currentRow = 0;

        // Add header row for multiple value fields
        if (this.valueFields.length > 1) {
            result[currentRow] = this._buildGroupedHeaders(this.rowFields);
            currentRow++;
        }

        // Add data rows for each group (sorted by key)
        const sortedGroupKeys = Array.from(groups.keys()).sort();
        for (const groupKey of sortedGroupKeys) {
            const groupRows = groups.get(groupKey);
            if (!groupRows) continue;

            result[currentRow] = this._buildGroupDataRow(groupKey, groupRows, rowIndices.length, valueIndices);
            currentRow++;
        }

        return result;
    }

    /**
     * Build header row for value fields only
     * @returns Row object with value field headers
     */
    private _buildValueFieldHeaders(): Record<number, ICellData> {
        const row: Record<number, ICellData> = {};
        for (let i = 0; i < this.valueFields.length; i++) {
            const field = this.valueFields[i];
            const aggregation = field.aggregation || AggregationType.SUM;
            row[i] = { v: this._getAggregationLabel(aggregation, field.name) };
        }
        return row;
    }

    /**
     * Build header row for grouped pivot (row fields + value fields)
     * @param rowFields Row field configurations
     * @returns Row object with all headers
     */
    private _buildGroupedHeaders(rowFields: IPivotField[]): Record<number, ICellData> {
        const row: Record<number, ICellData> = {};

        // Add row field headers
        for (let i = 0; i < rowFields.length; i++) {
            row[i] = { v: rowFields[i].name };
        }

        // Add value field headers
        for (let i = 0; i < this.valueFields.length; i++) {
            const field = this.valueFields[i];
            const aggregation = field.aggregation || AggregationType.SUM;
            row[rowFields.length + i] = { v: this._getAggregationLabel(aggregation, field.name) };
        }

        return row;
    }

    /**
     * Build a single data row for a group
     * @param groupKey Group key (pipe-separated values)
     * @param groupRows Rows in this group
     * @param rowFieldCount Number of row fields
     * @param valueIndices Value field indices
     * @returns Data row object
     */
    private _buildGroupDataRow(
        groupKey: string,
        groupRows: ICellData[][],
        rowFieldCount: number,
        valueIndices: number[]
    ): Record<number, ICellData> {
        const row: Record<number, ICellData> = {};

        // Add row field values
        const fieldValues = this._parseGroupKey(groupKey);
        for (let i = 0; i < fieldValues.length; i++) {
            row[i] = { v: fieldValues[i] };
        }

        // Add aggregated values
        const aggregatedValues = this._aggregateValues(groupRows, valueIndices);
        for (let i = 0; i < valueIndices.length; i++) {
            row[rowFieldCount + i] = aggregatedValues[i];
        }

        return row;
    }

    // #endregion

    // #region 2D Pivot Calculation

    /**
     * Calculate 2D pivot table (with both row and column fields)
     * @param dataRows Filtered data rows
     * @param fieldIndices Pre-extracted field indices
     * @param fieldIndices.rowIndices Row field column indices
     * @param fieldIndices.columnIndices Column field column indices
     * @param fieldIndices.valueIndices Value field column indices
     * @returns Calculated pivot table
     */
    private _calculate2DPivot(
        dataRows: ICellData[][],
        fieldIndices: { rowIndices: number[]; columnIndices: number[]; valueIndices: number[] }
    ): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        const { rowIndices, columnIndices, valueIndices } = fieldIndices;

        // Get unique column combinations (sorted)
        const columnCombos = this._getColumnCombinations(dataRows, columnIndices);

        // Group data by row fields
        const rowGroups = rowIndices.length > 0
            ? this._groupByFields(dataRows, rowIndices)
            : new Map([['', dataRows]]);

        // Build result
        return this._build2DResult(rowGroups, columnCombos, columnIndices, valueIndices);
    }

    /**
     * Build 2D pivot result matrix
     * @param rowGroups Grouped data by row fields
     * @param columnCombos Unique column combinations
     * @param columnIndices Column field indices
     * @param valueIndices Value field indices
     * @returns Result matrix
     */
    private _build2DResult(
        rowGroups: Map<string, ICellData[][]>,
        columnCombos: string[][],
        columnIndices: number[],
        valueIndices: number[]
    ): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        if (this.valuePosition === PivotValuePosition.Row) {
            return this._build2DResultWithValueInRow(rowGroups, columnCombos, columnIndices, valueIndices);
        }

        const result: IObjectMatrixPrimitiveType<Nullable<ICellData>> = {};
        let currentRow = 0;

        // Build header row
        result[currentRow] = this._build2DHeaderRow(columnCombos);
        currentRow++;

        // Build data rows
        const sortedRowKeys = Array.from(rowGroups.keys()).sort();
        for (const rowKey of sortedRowKeys) {
            const groupRows = rowGroups.get(rowKey);
            if (!groupRows) continue;

            result[currentRow] = this._build2DDataRow(
                rowKey,
                groupRows,
                columnCombos,
                columnIndices,
                valueIndices
            );
            currentRow++;
        }

        return result;
    }

    /**
     * Build header row for 2D pivot
     * @param columnCombos Unique column combinations
     * @returns Header row object
     */
    private _build2DHeaderRow(columnCombos: string[][]): Record<number, ICellData> {
        const row: Record<number, ICellData> = {};

        if (this.valuePosition === PivotValuePosition.Row) {
            // When value is in row, header structure is:
            // [row field headers] [value field header] [column combo 1] [column combo 2] ...
            let colIndex = 0;

            // Add row field headers
            for (let i = 0; i < this.rowFields.length; i++) {
                row[colIndex] = { v: this.rowFields[i].name };
                colIndex++;
            }

            // Add value field header (empty if single value field, or label if multiple)
            if (this.valueFields.length > 1) {
                row[colIndex] = { v: '' }; // Value field header column
            } else {
                row[colIndex] = { v: '' }; // Empty for single value field
            }
            colIndex++;

            // Add column combination headers
            for (let i = 0; i < columnCombos.length; i++) {
                const combo = columnCombos[i];
                const label = this._formatColumnComboLabel(combo);
                row[colIndex] = { v: label };
                colIndex++;
            }
        } else {
            // Default: value in column
            row[0] = { v: '' }; // Top-left corner

            for (let i = 0; i < columnCombos.length; i++) {
                const combo = columnCombos[i];
                const label = this._formatColumnComboLabel(combo);
                row[i + 1] = { v: label };
            }
        }

        return row;
    }

    /**
     * Build 2D pivot result matrix with value fields in rows
     * @param rowGroups Grouped data by row fields
     * @param columnCombos Unique column combinations
     * @param columnIndices Column field indices
     * @param valueIndices Value field indices
     * @returns Result matrix
     */
    private _build2DResultWithValueInRow(
        rowGroups: Map<string, ICellData[][]>,
        columnCombos: string[][],
        columnIndices: number[],
        valueIndices: number[]
    ): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        const result: IObjectMatrixPrimitiveType<Nullable<ICellData>> = {};
        let currentRow = 0;

        // Build header row
        result[currentRow] = this._build2DHeaderRow(columnCombos);
        currentRow++;

        // Build data rows: for each row group + value field combination
        const sortedRowKeys = Array.from(rowGroups.keys()).sort();
        for (const rowKey of sortedRowKeys) {
            const groupRows = rowGroups.get(rowKey);
            if (!groupRows) continue;

            const rowFieldValues = this._parseGroupKey(rowKey);

            // For each value field, create a row
            for (let valueIdx = 0; valueIdx < valueIndices.length; valueIdx++) {
                const valueField = this.valueFields[valueIdx];
                const aggregation = valueField.aggregation || AggregationType.SUM;

                const row: Record<number, ICellData> = {};

                // Add row field values
                for (let i = 0; i < rowFieldValues.length; i++) {
                    row[i] = { v: rowFieldValues[i] };
                }

                // Add value field label
                const valueLabelCol = rowFieldValues.length;
                row[valueLabelCol] = { v: this._getAggregationLabel(aggregation, valueField.name) };

                // Calculate and add values for each column combination
                for (let colIdx = 0; colIdx < columnCombos.length; colIdx++) {
                    const columnCombo = columnCombos[colIdx];
                    const filteredRows = this._filterByColumnCombo(groupRows, columnIndices, columnCombo);

                    const aggregatedValue = this._aggregateSingleValue(
                        filteredRows,
                        valueIndices[valueIdx],
                        aggregation
                    );

                    row[valueLabelCol + 1 + colIdx] = aggregatedValue;
                }

                result[currentRow] = row;
                currentRow++;
            }
        }

        return result;
    }

    /**
     * Build a single data row for 2D pivot
     * @param rowKey Row group key
     * @param groupRows Rows in this group
     * @param columnCombos Column combinations
     * @param columnIndices Column field indices
     * @param valueIndices Value field indices
     * @returns Data row object
     */
    private _build2DDataRow(
        rowKey: string,
        groupRows: ICellData[][],
        columnCombos: string[][],
        columnIndices: number[],
        valueIndices: number[]
    ): Record<number, ICellData> {
        const row: Record<number, ICellData> = {};

        // Add row label
        const fieldValues = this._parseGroupKey(rowKey);
        row[0] = { v: fieldValues[0] };

        // Calculate and add values for each column combination
        for (let colIdx = 0; colIdx < columnCombos.length; colIdx++) {
            const columnCombo = columnCombos[colIdx];
            const filteredRows = this._filterByColumnCombo(groupRows, columnIndices, columnCombo);

            // Use first value field (can be extended to support multiple)
            const aggregatedValue = this._aggregateSingleValue(
                filteredRows,
                valueIndices[0],
                this.valueFields[0].aggregation || AggregationType.SUM
            );

            row[colIdx + 1] = aggregatedValue;
        }

        return row;
    }

    // #endregion

    // #region Grouping and Filtering

    /**
     * Group data rows by specified field indices
     * Optimized to avoid redundant map lookups
     * Time Complexity: O(rows * fields)
     * @param dataRows Data rows to group
     * @param fieldIndices Indices of fields to group by
     * @returns Map of group key to rows
     */
    private _groupByFields(dataRows: ICellData[][], fieldIndices: number[]): Map<string, ICellData[][]> {
        const groups = new Map<string, ICellData[][]>();

        for (const row of dataRows) {
            // Build group key
            const key = this._buildGroupKey(row, fieldIndices);

            // Add row to group (optimized to avoid double lookup)
            const group = groups.get(key);
            if (group) {
                group.push(row);
            } else {
                groups.set(key, [row]);
            }
        }

        return groups;
    }

    /**
     * Build group key from row data and field indices
     * @param row Data row
     * @param fieldIndices Field indices to include in key
     * @returns Group key string
     */
    private _buildGroupKey(row: ICellData[], fieldIndices: number[]): string {
        const keyParts: string[] = [];
        for (const fieldIndex of fieldIndices) {
            const cell = row[fieldIndex];
            const value = cell?.v?.toString() || BLANK_VALUE_PLACEHOLDER;
            keyParts.push(value);
        }
        return keyParts.join(GROUP_KEY_SEPARATOR);
    }

    /**
     * Parse group key back into individual field values
     * Handles blank value conversion
     * @param groupKey Group key string
     * @returns Array of field values
     */
    private _parseGroupKey(groupKey: string): string[] {
        if (!groupKey) return [''];

        return groupKey.split(GROUP_KEY_SEPARATOR).map((value) =>
            value === BLANK_VALUE_PLACEHOLDER ? '' : value
        );
    }

    /**
     * Get all unique column combinations from data rows
     * Uses Set for efficient deduplication
     * Time Complexity: O(rows * fields)
     * @param dataRows Data rows
     * @param fieldIndices Column field indices
     * @returns Sorted unique column combinations
     */
    private _getColumnCombinations(dataRows: ICellData[][], fieldIndices: number[]): string[][] {
        const combinationSet = new Set<string>();

        // Collect unique combinations
        for (const row of dataRows) {
            const key = this._buildGroupKey(row, fieldIndices);
            combinationSet.add(key);
        }

        // Convert to sorted array of combinations
        return Array.from(combinationSet)
            .sort()
            .map((key) => key.split(GROUP_KEY_SEPARATOR));
    }

    /**
     * Format column combination for display
     * @param combo Column combination values
     * @returns Formatted label
     */
    private _formatColumnComboLabel(combo: string[]): string {
        const label = combo
            .map((v) => (v === BLANK_VALUE_PLACEHOLDER ? '' : v))
            .join(' - ');
        return label || combo[0];
    }

    /**
     * Filter rows that match a specific column combination
     * Optimized for early exit on mismatch
     * @param dataRows Rows to filter
     * @param fieldIndices Column field indices
     * @param columnCombo Target column combination
     * @returns Filtered rows
     */
    private _filterByColumnCombo(
        dataRows: ICellData[][],
        fieldIndices: number[],
        columnCombo: string[]
    ): ICellData[][] {
        return dataRows.filter((row) => {
            // Early exit on first mismatch (performance optimization)
            for (let i = 0; i < fieldIndices.length; i++) {
                const fieldIndex = fieldIndices[i];
                const expectedValue = columnCombo[i];
                const actualValue = row[fieldIndex]?.v?.toString() || BLANK_VALUE_PLACEHOLDER;

                if (actualValue !== expectedValue) {
                    return false;
                }
            }
            return true;
        });
    }

    // #endregion

    // #region Aggregation

    /**
     * Aggregate values from multiple rows
     * Creates aggregators for each value field and processes rows
     * @param dataRows Rows to aggregate
     * @param valueIndices Value field indices
     * @returns Aggregated values as cell objects
     */
    private _aggregateValues(
        dataRows: ICellData[][],
        valueIndices: number[]
    ): Record<number, ICellData> {
        const result: Record<number, ICellData> = {};

        for (let i = 0; i < valueIndices.length; i++) {
            const valueIndex = valueIndices[i];
            const field = this.valueFields[i];
            const aggregation = field.aggregation || AggregationType.SUM;

            result[i] = this._aggregateSingleValue(dataRows, valueIndex, aggregation);
        }

        return result;
    }

    /**
     * Aggregate a single value field across rows
     * Reusable aggregation logic
     * @param dataRows Rows to aggregate
     * @param valueIndex Value field index
     * @param aggregation Aggregation type
     * @returns Aggregated cell
     */
    private _aggregateSingleValue(
        dataRows: ICellData[][],
        valueIndex: number,
        aggregation: AggregationType
    ): ICellData {
        const aggregator = createAggregator(aggregation);
        aggregator.init();

        for (const row of dataRows) {
            const valueCell = row[valueIndex];
            if (valueCell) {
                aggregator.addValue(valueCell);
            }
        }

        return { v: aggregator.getResult() };
    }

    /**
     * Get human-readable label for aggregation type
     * @param aggregation Aggregation type
     * @param fieldName Field name
     * @returns Formatted label (e.g., "Sum of Sales")
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

    // #endregion

    // #region Filter Logic

    /**
     * Apply all filters to data rows
     * Uses AND logic - row must match all filters
     * Time Complexity: O(rows * filters)
     * @param dataRows Rows to filter
     * @param filterFields Filter configurations
     * @returns Filtered rows
     */
    private _applyFilters(dataRows: ICellData[][], filterFields: IPivotField[]): ICellData[][] {
        return dataRows.filter((row) => {
            // Early exit on first filter mismatch (AND logic optimization)
            for (const filterField of filterFields) {
                if (!filterField.filter) continue;

                const fieldIndex = filterField.sourceColumnIndex;
                const cellValue = row[fieldIndex]?.v;

                if (!this._matchesFilter(cellValue, filterField.filter)) {
                    return false;
                }
            }
            return true;
        });
    }

    /**
     * Check if a value matches filter criteria
     * Dispatches to appropriate filter type handler
     * @param value Value to check
     * @param filter Filter criteria
     * @returns True if value matches filter
     */
    private _matchesFilter(value: unknown, filter: IPivotFilterCriteria): boolean {
        return filter.type === 'value'
            ? this._matchesValueFilter(value, filter)
            : filter.type === 'condition'
                ? this._matchesConditionFilter(value, filter)
                : true;
    }

    /**
     * Check value filter (whitelist of allowed values)
     * @param value Value to check
     * @param filter Filter criteria
     * @returns True if value is in allowed list
     */
    private _matchesValueFilter(value: unknown, filter: IPivotFilterCriteria): boolean {
        // Empty filter means allow all
        if (!filter.values || filter.values.length === 0) {
            return true;
        }

        const valueStr = value != null ? String(value) : '';
        return filter.values.includes(valueStr);
    }

    /**
     * Check condition filter (comparison/text matching)
     * Handles different operator types efficiently
     * @param value Value to check
     * @param filter Filter criteria
     * @returns True if value matches condition
     */
    private _matchesConditionFilter(value: unknown, filter: IPivotFilterCriteria): boolean {
        if (!filter.operator || filter.conditionValue === undefined) {
            return true;
        }

        const { operator, conditionValue } = filter;

        switch (operator) {
            case 'equals':
                return (value != null ? String(value) : '') === (conditionValue != null ? String(conditionValue) : '');

            case 'notEquals':
                return (value != null ? String(value) : '') !== (conditionValue != null ? String(conditionValue) : '');

            case 'greaterThan':
                return this._compareNumbers(value, conditionValue, (a, b) => a > b);

            case 'lessThan':
                return this._compareNumbers(value, conditionValue, (a, b) => a < b);

            case 'contains': {
                const valueStr = value != null ? String(value) : '';
                const conditionStr = conditionValue != null ? String(conditionValue) : '';
                return valueStr.includes(conditionStr);
            }

            default:
                return true;
        }
    }

    /**
     * Compare two values as numbers with validation
     * Reusable number comparison utility
     * @param value1 First value
     * @param value2 Second value
     * @param compareFn Comparison function
     * @returns Comparison result
     */
    private _compareNumbers(
        value1: unknown,
        value2: unknown,
        compareFn: (a: number, b: number) => boolean
    ): boolean {
        const num1 = typeof value1 === 'number' ? value1 : Number(value1);
        const num2 = typeof value2 === 'number' ? value2 : Number(value2);

        // Both must be valid numbers
        if (Number.isNaN(num1) || Number.isNaN(num2)) {
            return false;
        }

        return compareFn(num1, num2);
    }

    // #endregion
}
