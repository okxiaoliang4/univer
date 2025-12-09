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

import type { ICellData, IObjectArrayPrimitiveType, IObjectMatrixPrimitiveType, Nullable } from '@univerjs/core';
import type { IPivotField, IPivotFilterCriteria, IPivotGroupInfo, IPivotSubtotalInfo, IPivotTableCrossTabConfig, IPivotTableCrossTabData } from '../types/type';
import { Disposable } from '@univerjs/core';
import { createAggregator } from '../common/aggregation/functions';
import { defaultPlaceholderMatrix } from '../common/default-pivot-table';
import { AggregationType, PivotValuePosition } from '../types/enum';

// Constants
const BLANK_VALUE_PLACEHOLDER = '(blank)';
const GROUP_KEY_SEPARATOR = '|';

/**
 * Pivot table calculation engine V2
 * Provides Cross-Tabulation format output optimized for UI rendering
 */
export class PivotEngineV2 extends Disposable {
    private _rowFields: IPivotField[];
    private _columnFields: IPivotField[];
    private _valueFields: IPivotField[];
    private _filterFields: IPivotField[];
    private _sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>;
    public valuePosition: PivotValuePosition;

    /** Cached calculated result */
    private _calculatedData: IPivotTableCrossTabData | null = null;

    /** Dirty flag for cache invalidation */
    private _isDirty: boolean = true;

    /** Cached cell matrix result */
    private _cachedCellMatrix: IObjectMatrixPrimitiveType<Nullable<ICellData>> | null = null;

    /** Reference to the calculated data used for cell matrix cache */
    private _cellMatrixCacheDataRef: IPivotTableCrossTabData | null = null;

    constructor(config: IPivotTableCrossTabConfig) {
        super();

        this._rowFields = config.rowFields;
        this._columnFields = config.columnFields;
        this._valueFields = config.valueFields;
        this._filterFields = config.filterFields;
        this._sourceData = config.sourceData;
        this.valuePosition = config.valuePosition;
    }

    // #region Public API Methods

    /**
     * Get the calculated pivot data
     * Uses caching to avoid redundant calculations
     * @returns The calculated Cross-Tabulation data
     */
    getCalculatedData(): IPivotTableCrossTabData {
        if (this._isDirty) {
            this._calculatedData = this.calculate();
            this._isDirty = false;
        }
        if (!this._calculatedData) {
            return this._createEmptyResult();
        }
        return this._calculatedData;
    }

    /**
     * Get the calculated data as cell matrix format (for backward compatibility with PivotEngine)
     * Converts Cross-Tabulation format to the old matrix format
     * This method is used by PivotTable to get output in the same format as PivotEngine
     * Uses caching to avoid redundant calculations when _calculatedData hasn't changed
     * @returns Cell matrix or null if empty
     */
    // eslint-disable-next-line max-lines-per-function
    getCalculatedCellMatrix(): IObjectMatrixPrimitiveType<Nullable<ICellData>> | null {
        const crossTabData = this.getCalculatedData();

        // Check if cache is valid (same data reference)
        if (this._cachedCellMatrix !== null && this._cellMatrixCacheDataRef === crossTabData) {
            return this._cachedCellMatrix;
        }

        if (crossTabData.isEmpty) {
            this._cachedCellMatrix = null;
            this._cellMatrixCacheDataRef = crossTabData;
            return defaultPlaceholderMatrix;
        }

        const matrix: IObjectMatrixPrimitiveType<Nullable<ICellData>> = {};
        const { structure } = crossTabData;
        let rowIndex = 0;

        // Build column headers (including value field headers)
        const columnHeaderRows: Nullable<ICellData>[][] = [];
        const hasMultipleValueFields = structure.valueFieldHeaders && structure.valueFieldHeaders.length > 1;
        const rowHeaderDepth = structure.rowHeaders[0]?.length || 0;
        const columnHeaderDepth = structure.columnHeaders[0]?.length || 0;
        const hasColumnFields = columnHeaderDepth > 0;

        // Determine if we need to show value field headers row
        // Show value field headers when: (1) multiple value fields OR (2) single value field with column fields
        const showValueFieldHeadersRow = hasColumnFields && (hasMultipleValueFields || this._valueFields.length >= 1);

        // Build header rows for each level of column headers
        // For multiple column fields, we need multiple rows:
        // Row 0: First column field values (e.g., Q1, Q2)
        // Row 1: Second column field values (e.g., 线下门店, 线上商店)
        // Row 2: Row field names + Value field names (e.g., Region | Product | Sum of Sales | ...)

        // Build column field header rows (one row per column field level)
        for (let colFieldLevel = 0; colFieldLevel < columnHeaderDepth; colFieldLevel++) {
            const headerRow: Nullable<ICellData>[] = [];

            // Empty cells for row header columns in all column field rows
            for (let i = 0; i < rowHeaderDepth; i++) {
                headerRow.push({
                    v: '',
                });
            }

            // Add column headers for this level
            if (hasColumnFields) {
                // Track last value and parent context for deduplication
                // For level 0: no parent, deduplicate across entire row
                // For level > 0: deduplicate within each parent group
                let lastParentKey: string | null = null;
                let lastValue: string | null = null;

                for (let colIdx = 0; colIdx < structure.columnHeaders.length; colIdx++) {
                    const colHeader = structure.columnHeaders[colIdx];
                    const headerValue = colHeader[colFieldLevel] || '';

                    // Build parent key from all previous levels
                    // This groups values by their parent context
                    const parentKey = colFieldLevel > 0
                        ? colHeader.slice(0, colFieldLevel).join(GROUP_KEY_SEPARATOR)
                        : '';

                    // Calculate how many columns this header spans
                    // If this is not the last level, we need to span across child columns
                    let spanCount = 1;
                    if (colFieldLevel < columnHeaderDepth - 1) {
                        // Count how many columns share the same prefix up to this level
                        let count = 1;
                        for (let nextIdx = colIdx + 1; nextIdx < structure.columnHeaders.length; nextIdx++) {
                            const nextHeader = structure.columnHeaders[nextIdx];
                            let matches = true;
                            for (let level = 0; level <= colFieldLevel; level++) {
                                if (colHeader[level] !== nextHeader[level]) {
                                    matches = false;
                                    break;
                                }
                            }
                            if (matches) {
                                count++;
                            } else {
                                break;
                            }
                        }
                        spanCount = count;
                    }

                    // Apply span for value fields
                    const valueFieldCount = hasMultipleValueFields && structure.valueFieldHeaders
                        ? structure.valueFieldHeaders.length
                        : 1;
                    const totalSpan = spanCount * valueFieldCount;

                    // Determine if this is the first occurrence:
                    // - First column always shows
                    // - Parent changed: show value (new parent group)
                    // - Same parent but value changed: show value
                    // - Same parent and same value: hide (duplicate)
                    const isFirstOccurrence = colIdx === 0
                        || parentKey !== lastParentKey
                        || (parentKey === lastParentKey && headerValue !== lastValue);

                    // Add header value repeated for each spanned column
                    // Only show value in the very first cell of this value group
                    for (let span = 0; span < totalSpan; span++) {
                        const shouldShowValue = isFirstOccurrence && span === 0;
                        headerRow.push({
                            v: shouldShowValue ? headerValue : '',
                        });
                    }

                    // Update tracking
                    if (isFirstOccurrence) {
                        lastParentKey = parentKey;
                        lastValue = headerValue;
                    }

                    // Skip columns that are spanned by this header
                    if (spanCount > 1) {
                        colIdx += spanCount - 1;
                    }
                }
            }

            if (headerRow.length > 0) {
                columnHeaderRows.push(headerRow);
            }
        }

        // Build value field headers row with row field names
        // Row field names should be in the same row as value field names
        if (showValueFieldHeadersRow || !hasColumnFields) {
            const valueFieldHeaderRow: Nullable<ICellData>[] = [];

            // Add row field names in the same row as value field names
            for (let i = 0; i < rowHeaderDepth; i++) {
                valueFieldHeaderRow.push({
                    v: this._rowFields[i]?.name || '',
                });
            }

            // Add value field headers for each column
            if (hasColumnFields) {
                if (hasMultipleValueFields && structure.valueFieldHeaders) {
                    // Multiple value fields: repeat value field headers for each column
                    for (let colIdx = 0; colIdx < structure.columnHeaders.length; colIdx++) {
                        for (let vfIdx = 0; vfIdx < structure.valueFieldHeaders.length; vfIdx++) {
                            valueFieldHeaderRow.push({
                                v: structure.valueFieldHeaders[vfIdx],
                            });
                        }
                    }
                } else if (this._valueFields.length >= 1) {
                    // Single value field: repeat value field name for each column
                    const field = this._valueFields[0];
                    const aggregation = field.aggregation || AggregationType.SUM;
                    const valueFieldLabel = this._getAggregationLabel(aggregation, field.name);
                    for (let colIdx = 0; colIdx < structure.columnHeaders.length; colIdx++) {
                        valueFieldHeaderRow.push({
                            v: valueFieldLabel,
                        });
                    }
                }
            } else {
                // No column fields: just add value field headers
                if (hasMultipleValueFields && structure.valueFieldHeaders) {
                    for (let vfIdx = 0; vfIdx < structure.valueFieldHeaders.length; vfIdx++) {
                        valueFieldHeaderRow.push({
                            v: structure.valueFieldHeaders[vfIdx],
                        });
                    }
                } else if (this._valueFields.length >= 1) {
                    const field = this._valueFields[0];
                    const aggregation = field.aggregation || AggregationType.SUM;
                    valueFieldHeaderRow.push({
                        v: this._getAggregationLabel(aggregation, field.name),
                    });
                }
            }

            if (valueFieldHeaderRow.length > 0) {
                columnHeaderRows.push(valueFieldHeaderRow);
            }
        }

        // Add header rows to matrix
        for (const headerRow of columnHeaderRows) {
            matrix[rowIndex] = {};
            headerRow.forEach((cell, colIdx) => {
                matrix[rowIndex][colIdx] = cell;
            });
            rowIndex++;
        }

        // Build data rows
        // Track last row header values for each level to avoid duplicates
        const lastRowHeaderValues: (string | null)[] = new Array(rowHeaderDepth).fill(null);

        for (let rowIdx = 0; rowIdx < structure.rowHeaders.length; rowIdx++) {
            const rowHeader = structure.rowHeaders[rowIdx];
            const row: Nullable<ICellData>[] = [];

            // Add row headers (only show value if different from previous row)
            for (let i = 0; i < rowHeader.length; i++) {
                const headerValue = rowHeader[i] || '';
                // Show value only if it's the first row or different from previous row at this level
                const shouldShowValue = rowIdx === 0 || headerValue !== lastRowHeaderValues[i];
                row.push({
                    v: shouldShowValue ? headerValue : '',
                });
                if (shouldShowValue) {
                    lastRowHeaderValues[i] = headerValue;
                }
            }

            // Pad row headers if needed
            const maxRowHeaderDepth = Math.max(...structure.rowHeaders.map((rh) => rh.length));
            while (row.length < maxRowHeaderDepth) {
                row.push({
                    v: '',
                });
            }

            // Add values
            if (structure.values[rowIdx]) {
                for (let colIdx = 0; colIdx < structure.values[rowIdx].length; colIdx++) {
                    const valueRow = structure.values[rowIdx][colIdx];
                    for (let vfIdx = 0; vfIdx < valueRow.length; vfIdx++) {
                        const value = valueRow[vfIdx];
                        row.push({
                            v: value,
                        });
                    }
                }
            }

            matrix[rowIndex] = {};
            row.forEach((cell, colIdx) => {
                matrix[rowIndex][colIdx] = cell;
            });
            rowIndex++;
        }

        // Cache the result
        this._cachedCellMatrix = matrix;
        this._cellMatrixCacheDataRef = crossTabData;

        return matrix;
    }

    /**
     * Get row fields
     */
    getRowFields(): IPivotField[] {
        return this._rowFields;
    }

    /**
     * Get column fields
     */
    getColumnFields(): IPivotField[] {
        return this._columnFields;
    }

    /**
     * Get value fields
     */
    getValueFields(): IPivotField[] {
        return this._valueFields;
    }

    /**
     * Get filter fields
     */
    getFilterFields(): IPivotField[] {
        return this._filterFields;
    }

    /**
     * Get source data
     */
    getSourceData(): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        return this._sourceData;
    }

    /**
     * Set row fields
     */
    setRowFields(rowFields: IPivotField[]): void {
        this._rowFields = rowFields;
        this._markDirty();
    }

    /**
     * Set column fields
     */
    setColumnFields(columnFields: IPivotField[]): void {
        this._columnFields = columnFields;
        this._markDirty();
    }

    /**
     * Set value fields
     */
    setValueFields(valueFields: IPivotField[]): void {
        this._valueFields = valueFields;
        this._markDirty();
    }

    /**
     * Set filter fields
     */
    setFilterFields(filterFields: IPivotField[]): void {
        this._filterFields = filterFields;
        this._markDirty();
    }

    /**
     * Set source data
     */
    setSourceData(sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>): void {
        this._sourceData = sourceData;
        this._markDirty();
    }

    /**
     * Set value position
     * Note: In Cross-Tabulation format, value fields are nested as additional dimensions.
     * This property is kept for API compatibility but may not affect the output format.
     */
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

    /**
     * Set calculated data directly (for persistence and warm start)
     * @param data The calculated Cross-Tabulation data to set
     * @param isDirty Whether the data should be marked as dirty (warm start) or clean (authoritative)
     * @returns true if data was accepted, false if validation failed
     */
    setCalculatedData(data: IPivotTableCrossTabData, isDirty = true): boolean {
        // Basic validation
        if (!data || typeof data !== 'object') {
            return false;
        }

        // Validate structure exists
        if (!data.structure || !data.dimensions) {
            return false;
        }

        // Validate dimensions match current configuration
        const expectedValueFieldCount = this._valueFields.length;
        if (data.dimensions.valueFieldCount !== expectedValueFieldCount) {
            return false;
        }

        // Validate row/column structure compatibility
        const expectedRowDepth = this._rowFields.length;
        const expectedColDepth = this._columnFields.length;

        // Allow some flexibility - data might have different row counts but same structure
        if (data.structure.rowHeaders.length > 0) {
            const actualRowDepth = data.structure.rowHeaders[0].length;
            if (actualRowDepth !== expectedRowDepth) {
                return false;
            }
        }

        if (data.structure.columnHeaders.length > 0) {
            const actualColDepth = data.structure.columnHeaders[0].length;
            if (actualColDepth !== expectedColDepth) {
                return false;
            }
        }

        // Set the data
        this._calculatedData = data;

        // Set dirty flag based on parameter
        this._isDirty = isDirty;

        // Invalidate cell matrix cache since calculated data changed
        this._cachedCellMatrix = null;
        this._cellMatrixCacheDataRef = null;

        return true;
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
     * @returns The calculated Cross-Tabulation data
     */
    calculate(): IPivotTableCrossTabData {
        // Early return if no value fields
        if (this._valueFields.length === 0) {
            return this._createEmptyResult();
        }

        // Extract and prepare data
        let { dataRows } = this._extractData(this._sourceData);
        if (dataRows.length === 0) {
            return this._createEmptyResult();
        }

        // Apply filters before aggregation (performance optimization)
        if (this._filterFields.length > 0) {
            dataRows = this._applyFilters(dataRows, this._filterFields);
            if (dataRows.length === 0) {
                return this._createEmptyResult();
            }
        }

        // Extract field indices (do this once to avoid repeated property access)
        const fieldIndices = this._extractFieldIndices();

        // Calculate Cross-Tabulation structure
        return this._calculateCrossTab(dataRows, fieldIndices);
    }

    /**
     * Create empty result structure
     */
    private _createEmptyResult(): IPivotTableCrossTabData {
        return {
            isEmpty: true,
            dimensions: {
                totalRows: 0,
                totalColumns: 0,
                dataRowCount: 0,
                dataColumnCount: 0,
                valueFieldCount: this._valueFields.length,
            },
            structure: {
                rowHeaders: [],
                columnHeaders: [],
                values: [],
                rowTypes: [],
                columnTypes: [],
            },
        };
    }

    /**
     * Extract field indices from field configurations
     * Optimizes repeated access to sourceColumnIndex
     * @returns Object containing all field indices
     */
    private _extractFieldIndices() {
        return {
            rowIndices: this._rowFields.map((f) => f.sourceColumnIndex).filter((i) => i >= 0),
            columnIndices: this._columnFields.map((f) => f.sourceColumnIndex).filter((i) => i >= 0),
            valueIndices: this._valueFields.map((f) => f.sourceColumnIndex).filter((i) => i >= 0),
        };
    }

    /**
     * Calculate Cross-Tabulation structure
     * @param dataRows Filtered data rows
     * @param fieldIndices Pre-extracted field indices
     * @returns Calculated Cross-Tabulation data
     */
    private _calculateCrossTab(
        dataRows: ICellData[][],
        fieldIndices: { rowIndices: number[]; columnIndices: number[]; valueIndices: number[] }
    ): IPivotTableCrossTabData {
        const { rowIndices, columnIndices, valueIndices } = fieldIndices;

        // Get unique column combinations (sorted)
        const columnCombos = this._getColumnCombinations(dataRows, columnIndices);

        // Group data by row fields
        const rowGroups = rowIndices.length > 0
            ? this._groupByFields(dataRows, rowIndices)
            : new Map([['', dataRows]]);

        // Build result structure
        const result = this._buildCrossTabResult(rowGroups, columnCombos, rowIndices, columnIndices, valueIndices);

        // Calculate isEmpty
        const isEmpty = this._calculateIsEmpty(result, dataRows.length);

        // Calculate dimensions
        const dimensions = this._calculateDimensions(result);

        return {
            isEmpty,
            dimensions,
            structure: result,
        };
    }

    /**
     * Build Cross-Tabulation result structure
     * Respects valuePosition to place value fields as rows or columns
     */
    private _buildCrossTabResult(
        rowGroups: Map<string, ICellData[][]>,
        columnCombos: string[][],
        rowIndices: number[],
        columnIndices: number[],
        valueIndices: number[]
    ): IPivotTableCrossTabData['structure'] {
        // Check if we should place value fields as rows instead of columns
        const useValueFieldsAsRows = this.valuePosition === PivotValuePosition.ROW;

        // Delegate to appropriate implementation based on valuePosition
        if (useValueFieldsAsRows) {
            return this._buildCrossTabResultWithRowValues(
                rowGroups,
                columnCombos,
                rowIndices,
                columnIndices,
                valueIndices
            );
        } else {
            return this._buildCrossTabResultWithColumnValues(
                rowGroups,
                columnCombos,
                rowIndices,
                columnIndices,
                valueIndices
            );
        }
    }

    /**
     * Build Cross-Tabulation with value fields as column headers (current behavior)
     * This is the original implementation
     */
    // eslint-disable-next-line max-lines-per-function
    private _buildCrossTabResultWithColumnValues(
        rowGroups: Map<string, ICellData[][]>,
        columnCombos: string[][],
        rowIndices: number[],
        columnIndices: number[],
        valueIndices: number[]
    ): IPivotTableCrossTabData['structure'] {
        const rowHeaders: string[][] = [];
        const columnHeaders: string[][] = [];
        const values: (number | string | null)[][][] = [];
        const rowTypes: ('data' | 'subtotal')[] = [];
        const columnTypes: ('data' | 'subtotal')[] = [];
        const subtotalRows: IPivotSubtotalInfo[] = [];
        const subtotalColumns: IPivotSubtotalInfo[] = [];
        const rowGroupsInfo: IPivotGroupInfo[] = [];
        const columnGroupsInfo: IPivotGroupInfo[] = [];
        const rowLevelMap: Record<number, string[]> = {};
        const columnLevelMap: Record<number, string[]> = {};

        // Build column headers
        for (const combo of columnCombos) {
            columnHeaders.push(combo.map((v) => v === BLANK_VALUE_PLACEHOLDER ? '' : v));
            columnTypes.push('data');
        }

        // Add grand total column if first column field has showSubTotals
        if (columnIndices.length > 0 && this._columnFields[0]?.showSubTotals) {
            columnHeaders.push(['总计']);
            columnTypes.push('subtotal');
            subtotalColumns.push({
                columnIndex: columnHeaders.length - 1,
                level: 0,
                fieldIndex: 0,
                value: '',
                label: '总计',
            });
        }

        // When no column fields are defined, keep a placeholder column to match the value grid
        if (columnHeaders.length === 0) {
            columnHeaders.push([]);
            columnTypes.push('data');
        }

        // Build value field headers if multiple value fields
        const valueFieldHeaders = this._valueFields.length > 1
            ? this._valueFields.map((f) => {
                const aggregation = f.aggregation || AggregationType.SUM;
                return this._getAggregationLabel(aggregation, f.name);
            })
            : undefined;

        // Process row groups
        const sortedRowKeys = Array.from(rowGroups.keys()).sort();
        let currentRowIndex = 0;

        for (const rowKey of sortedRowKeys) {
            const groupRows = rowGroups.get(rowKey);
            if (!groupRows) continue;

            const rowFieldValues = this._parseGroupKey(rowKey);
            const groupId = this._generateGroupId('row', rowFieldValues, 0);

            // Track group start
            const groupStartIndex = currentRowIndex;

            // Add data rows for this group
            if (columnIndices.length > 0) {
                // 2D pivot: calculate values for each column combination
                const rowValues: (number | string | null)[][] = [];
                for (const columnCombo of columnCombos) {
                    const filteredRows = this._filterByColumnCombo(groupRows, columnIndices, columnCombo);
                    const cellValues: (number | string | null)[] = [];
                    for (let i = 0; i < valueIndices.length; i++) {
                        const valueIndex = valueIndices[i];
                        const field = this._valueFields[i];
                        const aggregation = field.aggregation || AggregationType.SUM;
                        const aggregatedValue = this._aggregateSingleValue(filteredRows, valueIndex, aggregation);
                        cellValues.push(this._normalizeValue(aggregatedValue.v));
                    }
                    rowValues.push(cellValues);
                }

                // Add grand total column value if needed
                if (columnHeaders.length > columnCombos.length) {
                    const grandTotalValues: (number | string | null)[] = [];
                    for (let i = 0; i < valueIndices.length; i++) {
                        const valueIndex = valueIndices[i];
                        const field = this._valueFields[i];
                        const aggregation = field.aggregation || AggregationType.SUM;
                        const aggregatedValue = this._aggregateSingleValue(groupRows, valueIndex, aggregation);
                        grandTotalValues.push(this._normalizeValue(aggregatedValue.v));
                    }
                    rowValues.push(grandTotalValues);
                }

                rowHeaders.push(rowFieldValues);
                rowTypes.push('data');
                values.push(rowValues);
                currentRowIndex++;

                // Update row level map
                rowLevelMap[currentRowIndex - 1] = [groupId];
            } else {
                // Row-only pivot: aggregate all values
                const cellValues: (number | string | null)[] = [];
                for (let i = 0; i < valueIndices.length; i++) {
                    const valueIndex = valueIndices[i];
                    const field = this._valueFields[i];
                    const aggregation = field.aggregation || AggregationType.SUM;
                    const aggregatedValue = this._aggregateSingleValue(groupRows, valueIndex, aggregation);
                    cellValues.push(this._normalizeValue(aggregatedValue.v));
                }

                rowHeaders.push(rowFieldValues);
                rowTypes.push('data');
                values.push([cellValues]);
                currentRowIndex++;

                // Update row level map
                rowLevelMap[currentRowIndex - 1] = [groupId];
            }

            // Add subtotal row if first row field has showSubTotals
            if (rowIndices.length > 0 && (this._rowFields[0]?.showSubTotals)) {
                const subtotalHeader = [...rowFieldValues];
                subtotalHeader[0] = ''; // Empty for subtotal row

                const subtotalValues: (number | string | null)[][] = [];
                if (columnIndices.length > 0) {
                    // Calculate subtotal for each column
                    for (const columnCombo of columnCombos) {
                        const filteredRows = this._filterByColumnCombo(groupRows, columnIndices, columnCombo);
                        const cellValues: (number | string | null)[] = [];
                        for (let i = 0; i < valueIndices.length; i++) {
                            const valueIndex = valueIndices[i];
                            const field = this._valueFields[i];
                            const aggregation = field.aggregation || AggregationType.SUM;
                            const aggregatedValue = this._aggregateSingleValue(filteredRows, valueIndex, aggregation);
                            cellValues.push(this._normalizeValue(aggregatedValue.v));
                        }
                        subtotalValues.push(cellValues);
                    }

                    // Add grand total column value
                    if (columnHeaders.length > columnCombos.length) {
                        const grandTotalValues: (number | string | null)[] = [];
                        for (let i = 0; i < valueIndices.length; i++) {
                            const valueIndex = valueIndices[i];
                            const field = this._valueFields[i];
                            const aggregation = field.aggregation || AggregationType.SUM;
                            const aggregatedValue = this._aggregateSingleValue(groupRows, valueIndex, aggregation);
                            grandTotalValues.push(this._normalizeValue(aggregatedValue.v));
                        }
                        subtotalValues.push(grandTotalValues);
                    }
                } else {
                    // Row-only: single column of values
                    const cellValues: (number | string | null)[] = [];
                    for (let i = 0; i < valueIndices.length; i++) {
                        const valueIndex = valueIndices[i];
                        const field = this._valueFields[i];
                        const aggregation = field.aggregation || AggregationType.SUM;
                        const aggregatedValue = this._aggregateSingleValue(groupRows, valueIndex, aggregation);
                        cellValues.push(this._normalizeValue(aggregatedValue.v));
                    }
                    subtotalValues.push(cellValues);
                }

                rowHeaders.push(subtotalHeader);
                rowTypes.push('subtotal');
                values.push(subtotalValues);

                subtotalRows.push({
                    rowIndex: currentRowIndex,
                    level: 0,
                    fieldIndex: 0,
                    value: rowFieldValues[0] || '',
                    label: `${rowFieldValues[0] || ''} 总计`,
                });

                // Update row level map for subtotal row
                rowLevelMap[currentRowIndex] = [groupId];

                // Update group info
                rowGroupsInfo.push({
                    groupId,
                    firstRowIndex: groupStartIndex,
                    lastRowIndex: currentRowIndex,
                    level: 0,
                    fieldIndex: 0,
                    value: rowFieldValues[0] || '',
                    expanded: true,
                });

                currentRowIndex++;
            } else if (rowIndices.length > 0) {
                // No subtotal, but still track group
                rowGroupsInfo.push({
                    groupId,
                    firstRowIndex: groupStartIndex,
                    lastRowIndex: currentRowIndex - 1,
                    level: 0,
                    fieldIndex: 0,
                    value: rowFieldValues[0] || '',
                    expanded: true,
                });
            }
        }

        // Add grand total row if first row field has showSubTotals
        if (rowIndices.length > 0 && (this._rowFields[0]?.showSubTotals)) {
            const grandTotalHeader: string[] = [];
            for (let i = 0; i < rowIndices.length; i++) {
                grandTotalHeader.push(i === 0 ? '总计' : '');
            }

            const grandTotalValues: (number | string | null)[][] = [];
            if (columnIndices.length > 0) {
                // Calculate grand total for each column
                for (const columnCombo of columnCombos) {
                    const allRowsForColumn = Array.from(rowGroups.values())
                        .flat()
                        .filter((row) => {
                            for (let i = 0; i < columnIndices.length; i++) {
                                const fieldIndex = columnIndices[i];
                                const expectedValue = columnCombo[i];
                                const actualValue = row[fieldIndex]?.v?.toString() || BLANK_VALUE_PLACEHOLDER;
                                if (actualValue !== expectedValue) {
                                    return false;
                                }
                            }
                            return true;
                        });

                    const cellValues: (number | string | null)[] = [];
                    for (let i = 0; i < valueIndices.length; i++) {
                        const valueIndex = valueIndices[i];
                        const field = this._valueFields[i];
                        const aggregation = field.aggregation || AggregationType.SUM;
                        const aggregatedValue = this._aggregateSingleValue(allRowsForColumn, valueIndex, aggregation);
                        cellValues.push(this._normalizeValue(aggregatedValue.v));
                    }
                    grandTotalValues.push(cellValues);
                }

                // Add grand total column value
                if (columnHeaders.length > columnCombos.length) {
                    const allRows = Array.from(rowGroups.values()).flat();
                    const cellValues: (number | string | null)[] = [];
                    for (let i = 0; i < valueIndices.length; i++) {
                        const valueIndex = valueIndices[i];
                        const field = this._valueFields[i];
                        const aggregation = field.aggregation || AggregationType.SUM;
                        const aggregatedValue = this._aggregateSingleValue(allRows, valueIndex, aggregation);
                        cellValues.push(this._normalizeValue(aggregatedValue.v));
                    }
                    grandTotalValues.push(cellValues);
                }
            } else {
                // Row-only: single column
                const allRows = Array.from(rowGroups.values()).flat();
                const cellValues: (number | string | null)[] = [];
                for (let i = 0; i < valueIndices.length; i++) {
                    const valueIndex = valueIndices[i];
                    const field = this._valueFields[i];
                    const aggregation = field.aggregation || AggregationType.SUM;
                    const aggregatedValue = this._aggregateSingleValue(allRows, valueIndex, aggregation);
                    cellValues.push(this._normalizeValue(aggregatedValue.v));
                }
                grandTotalValues.push(cellValues);
            }

            rowHeaders.push(grandTotalHeader);
            rowTypes.push('subtotal');
            values.push(grandTotalValues);

            subtotalRows.push({
                rowIndex: currentRowIndex,
                level: 0,
                fieldIndex: 0,
                value: '',
                label: '总计',
            });

            rowLevelMap[currentRowIndex] = [];
            currentRowIndex++;
        }

        // Build column groups and level map
        if (columnIndices.length > 0) {
            // Group columns by their values
            const columnGroupMap = new Map<string, number[]>();
            for (let i = 0; i < columnCombos.length; i++) {
                const combo = columnCombos[i];
                const groupId = this._generateGroupId('col', combo, 0);

                if (!columnGroupMap.has(groupId)) {
                    columnGroupMap.set(groupId, []);
                }
                columnGroupMap.get(groupId)!.push(i);
            }

            // Create column group info
            for (const [groupId, indices] of columnGroupMap.entries()) {
                if (indices.length > 0) {
                    const combo = columnCombos[indices[0]];
                    columnGroupsInfo.push({
                        groupId,
                        firstColumnIndex: indices[0],
                        lastColumnIndex: indices[indices.length - 1],
                        level: 0,
                        fieldIndex: 0,
                        value: combo[0] || '',
                        expanded: true,
                    });
                }
            }

            // Build column level map
            for (let i = 0; i < columnHeaders.length; i++) {
                if (i < columnCombos.length) {
                    const combo = columnCombos[i];
                    const groupId = this._generateGroupId('col', combo, 0);
                    columnLevelMap[i] = [groupId];
                } else {
                    columnLevelMap[i] = [];
                }
            }
        } else {
            // No column fields, all columns belong to empty group
            for (let i = 0; i < columnHeaders.length; i++) {
                columnLevelMap[i] = [];
            }
        }

        return {
            rowHeaders,
            columnHeaders,
            valueFieldHeaders,
            values,
            rowTypes,
            columnTypes,
            subtotalRows: subtotalRows.length > 0 ? subtotalRows : undefined,
            subtotalColumns: subtotalColumns.length > 0 ? subtotalColumns : undefined,
            rowGroups: rowGroupsInfo.length > 0 ? rowGroupsInfo : undefined,
            columnGroups: columnGroupsInfo.length > 0 ? columnGroupsInfo : undefined,
            rowLevelMap: Object.keys(rowLevelMap).length > 0 ? rowLevelMap : undefined,
            columnLevelMap: Object.keys(columnLevelMap).length > 0 ? columnLevelMap : undefined,
        };
    }

    /**
     * Build Cross-Tabulation with value fields as row headers
     * When valuePosition === ROW, value fields become part of the row structure
     *
     * In this layout:
     * - Each original row group generates rows for each value field
     * - Value field names are appended to row headers as an additional level
     * - Column headers remain from column fields only
     * - This creates a layout where values are spread across rows instead of columns
     *
     * Example structure with 2 regions and 1 value field:
     * Row Headers: [Region, ValueFieldName]
     * Values:      [[100, 200], [150, 250]]  (North-A: 100, North-B: 200, etc.)
     */
    // eslint-disable-next-line max-lines-per-function
    private _buildCrossTabResultWithRowValues(
        rowGroups: Map<string, ICellData[][]>,
        columnCombos: string[][],
        rowIndices: number[],
        columnIndices: number[],
        valueIndices: number[]
    ): IPivotTableCrossTabData['structure'] {
        const rowHeaders: string[][] = [];
        const columnHeaders: string[][] = [];
        const values: IObjectMatrixPrimitiveType<IObjectArrayPrimitiveType<number | string | null>> = {};
        const rowTypes: ('data' | 'subtotal')[] = [];
        const columnTypes: ('data' | 'subtotal')[] = [];
        const subtotalRows: IPivotSubtotalInfo[] = [];
        const subtotalColumns: IPivotSubtotalInfo[] = [];
        const rowGroupsInfo: IPivotGroupInfo[] = [];
        const columnGroupsInfo: IPivotGroupInfo[] = [];
        const rowLevelMap: Record<number, string[]> = {};
        const columnLevelMap: Record<number, string[]> = {};

        // Build column headers (only from column fields, NOT value fields)
        for (const combo of columnCombos) {
            columnHeaders.push(combo.map((v) => v === BLANK_VALUE_PLACEHOLDER ? '' : v));
            columnTypes.push('data');
        }

        // Add grand total column if first column field has showSubTotals
        if (columnIndices.length > 0 && this._columnFields[0]?.showSubTotals) {
            columnHeaders.push(['总计']);
            columnTypes.push('subtotal');
            subtotalColumns.push({
                columnIndex: columnHeaders.length - 1,
                level: 0,
                fieldIndex: 0,
                value: '',
                label: '总计',
            });
        }

        // When there are no column fields, keep a placeholder column header so the
        // downstream structure reflects the single value column that exists.
        if (columnHeaders.length === 0) {
            columnHeaders.push([]);
            columnTypes.push('data');
        }

        // Process row groups
        const sortedRowKeys = Array.from(rowGroups.keys()).sort();
        let currentRowIndex = 0;

        for (const rowKey of sortedRowKeys) {
            const groupRows = rowGroups.get(rowKey);
            if (!groupRows) continue;

            const rowFieldValues = this._parseGroupKey(rowKey);
            const groupId = this._generateGroupId('row', rowFieldValues, 0);

            // Track group start
            const groupStartIndex = currentRowIndex;

            // For each value field, add a row
            for (let valueIdx = 0; valueIdx < this._valueFields.length; valueIdx++) {
                const valueField = this._valueFields[valueIdx];
                const valueIndex = valueIndices[valueIdx];
                const aggregation = valueField.aggregation || AggregationType.SUM;

                // Create row header: original row fields + value field name
                const valueHeader = [...rowFieldValues];
                // Append value field label as an additional level
                const aggregationLabel = this._getAggregationLabel(aggregation, valueField.name);
                valueHeader.push(aggregationLabel);

                // Calculate values for each column
                const rowValues: (number | string | null)[][] = [];
                if (columnIndices.length > 0) {
                    // For each column combination
                    for (const columnCombo of columnCombos) {
                        const filteredRows = this._filterByColumnCombo(groupRows, columnIndices, columnCombo);
                        const aggregatedValue = this._aggregateSingleValue(filteredRows, valueIndex, aggregation);
                        rowValues.push([this._normalizeValue(aggregatedValue.v)]);
                    }

                    // Add grand total column if needed
                    if (columnHeaders.length > columnCombos.length) {
                        const aggregatedValue = this._aggregateSingleValue(groupRows, valueIndex, aggregation);
                        rowValues.push([this._normalizeValue(aggregatedValue.v)]);
                    }
                } else {
                    // Row-only: single column
                    const aggregatedValue = this._aggregateSingleValue(groupRows, valueIndex, aggregation);
                    rowValues.push([this._normalizeValue(aggregatedValue.v)]);
                }

                rowHeaders.push(valueHeader);
                rowTypes.push('data');
                values.push(rowValues);

                // Update row level map
                rowLevelMap[currentRowIndex] = [groupId];
                currentRowIndex++;
            }

            // Add subtotal row if first row field has showSubTotals
            if (rowIndices.length > 0 && (this._rowFields[0]?.showSubTotals)) {
                const subtotalHeader = [...rowFieldValues];
                subtotalHeader[0] = ''; // Empty for subtotal row
                subtotalHeader.push('小计'); // Add subtotal marker

                const subtotalValues: (number | string | null)[][] = [];
                if (columnIndices.length > 0) {
                    // Calculate subtotal for each column (across all value fields)
                    for (const columnCombo of columnCombos) {
                        const filteredRows = this._filterByColumnCombo(groupRows, columnIndices, columnCombo);
                        // For ROW position, subtotals sum across value fields would be less meaningful
                        // So we just aggregate the first value field as representative
                        const firstValueIndex = valueIndices[0];
                        const firstField = this._valueFields[0];
                        const aggregation = firstField.aggregation || AggregationType.SUM;
                        const aggregatedValue = this._aggregateSingleValue(filteredRows, firstValueIndex, aggregation);
                        subtotalValues.push([this._normalizeValue(aggregatedValue.v)]);
                    }

                    // Add grand total column if needed
                    if (columnHeaders.length > columnCombos.length) {
                        const firstValueIndex = valueIndices[0];
                        const firstField = this._valueFields[0];
                        const aggregation = firstField.aggregation || AggregationType.SUM;
                        const aggregatedValue = this._aggregateSingleValue(groupRows, firstValueIndex, aggregation);
                        subtotalValues.push([this._normalizeValue(aggregatedValue.v)]);
                    }
                } else {
                    // Row-only
                    const firstValueIndex = valueIndices[0];
                    const firstField = this._valueFields[0];
                    const aggregation = firstField.aggregation || AggregationType.SUM;
                    const aggregatedValue = this._aggregateSingleValue(groupRows, firstValueIndex, aggregation);
                    subtotalValues.push([this._normalizeValue(aggregatedValue.v)]);
                }

                rowHeaders.push(subtotalHeader);
                rowTypes.push('subtotal');
                values.push(subtotalValues);

                subtotalRows.push({
                    rowIndex: currentRowIndex,
                    level: 0,
                    fieldIndex: 0,
                    value: rowFieldValues[0] || '',
                    label: `${rowFieldValues[0] || ''} 小计`,
                });

                // Update row level map for subtotal row
                rowLevelMap[currentRowIndex] = [groupId];

                // Update group info
                rowGroupsInfo.push({
                    groupId,
                    firstRowIndex: groupStartIndex,
                    lastRowIndex: currentRowIndex,
                    level: 0,
                    fieldIndex: 0,
                    value: rowFieldValues[0] || '',
                    expanded: true,
                });

                currentRowIndex++;
            } else if (rowIndices.length > 0) {
                // No subtotal, but still track group
                rowGroupsInfo.push({
                    groupId,
                    firstRowIndex: groupStartIndex,
                    lastRowIndex: currentRowIndex - 1,
                    level: 0,
                    fieldIndex: 0,
                    value: rowFieldValues[0] || '',
                    expanded: true,
                });
            }
        }

        // Add grand total row if first row field has showSubTotals
        if (rowIndices.length > 0 && (this._rowFields[0]?.showSubTotals)) {
            // Add one grand total row for each value field
            for (let valueIdx = 0; valueIdx < this._valueFields.length; valueIdx++) {
                const valueField = this._valueFields[valueIdx];
                const valueIndex = valueIndices[valueIdx];
                const aggregation = valueField.aggregation || AggregationType.SUM;

                const grandTotalHeader: string[] = [];
                for (let i = 0; i < rowIndices.length; i++) {
                    grandTotalHeader.push(i === 0 ? '总计' : '');
                }
                const aggregationLabel = this._getAggregationLabel(aggregation, valueField.name);
                grandTotalHeader.push(aggregationLabel);

                const grandTotalValues: (number | string | null)[][] = [];
                if (columnIndices.length > 0) {
                    // Calculate grand total for each column
                    for (const columnCombo of columnCombos) {
                        const allRowsForColumn = Array.from(rowGroups.values())
                            .flat()
                            .filter((row) => {
                                for (let i = 0; i < columnIndices.length; i++) {
                                    const fieldIndex = columnIndices[i];
                                    const expectedValue = columnCombo[i];
                                    const actualValue = row[fieldIndex]?.v?.toString() || BLANK_VALUE_PLACEHOLDER;
                                    if (actualValue !== expectedValue) {
                                        return false;
                                    }
                                }
                                return true;
                            });

                        const aggregatedValue = this._aggregateSingleValue(allRowsForColumn, valueIndex, aggregation);
                        grandTotalValues.push([this._normalizeValue(aggregatedValue.v)]);
                    }

                    // Add grand total column value
                    if (columnHeaders.length > columnCombos.length) {
                        const allRows = Array.from(rowGroups.values()).flat();
                        const aggregatedValue = this._aggregateSingleValue(allRows, valueIndex, aggregation);
                        grandTotalValues.push([this._normalizeValue(aggregatedValue.v)]);
                    }
                } else {
                    // Row-only: single column
                    const allRows = Array.from(rowGroups.values()).flat();
                    const aggregatedValue = this._aggregateSingleValue(allRows, valueIndex, aggregation);
                    grandTotalValues.push([this._normalizeValue(aggregatedValue.v)]);
                }

                rowHeaders.push(grandTotalHeader);
                rowTypes.push('subtotal');
                values.push(grandTotalValues);

                subtotalRows.push({
                    rowIndex: currentRowIndex,
                    level: 0,
                    fieldIndex: 0,
                    value: '',
                    label: `总计 - ${aggregationLabel}`,
                });

                rowLevelMap[currentRowIndex] = [];
                currentRowIndex++;
            }
        }

        // Build column groups and level map
        if (columnIndices.length > 0) {
            // Group columns by their values
            const columnGroupMap = new Map<string, number[]>();
            for (let i = 0; i < columnCombos.length; i++) {
                const combo = columnCombos[i];
                const groupId = this._generateGroupId('col', combo, 0);

                if (!columnGroupMap.has(groupId)) {
                    columnGroupMap.set(groupId, []);
                }
                columnGroupMap.get(groupId)!.push(i);
            }

            // Create column group info
            for (const [groupId, indices] of columnGroupMap.entries()) {
                if (indices.length > 0) {
                    const combo = columnCombos[indices[0]];
                    columnGroupsInfo.push({
                        groupId,
                        firstColumnIndex: indices[0],
                        lastColumnIndex: indices[indices.length - 1],
                        level: 0,
                        fieldIndex: 0,
                        value: combo[0] || '',
                        expanded: true,
                    });
                }
            }

            // Build column level map
            for (let i = 0; i < columnHeaders.length; i++) {
                if (i < columnCombos.length) {
                    const combo = columnCombos[i];
                    const groupId = this._generateGroupId('col', combo, 0);
                    columnLevelMap[i] = [groupId];
                } else {
                    columnLevelMap[i] = [];
                }
            }
        } else {
            // No column fields, all columns belong to empty group
            for (let i = 0; i < columnHeaders.length; i++) {
                columnLevelMap[i] = [];
            }
        }

        return {
            rowHeaders,
            columnHeaders,
            valueFieldHeaders: undefined, // Not used when values are rows
            values,
            rowTypes,
            columnTypes,
            subtotalRows: subtotalRows.length > 0 ? subtotalRows : undefined,
            subtotalColumns: subtotalColumns.length > 0 ? subtotalColumns : undefined,
            rowGroups: rowGroupsInfo.length > 0 ? rowGroupsInfo : undefined,
            columnGroups: columnGroupsInfo.length > 0 ? columnGroupsInfo : undefined,
            rowLevelMap: Object.keys(rowLevelMap).length > 0 ? rowLevelMap : undefined,
            columnLevelMap: Object.keys(columnLevelMap).length > 0 ? columnLevelMap : undefined,
        };
    }

    /**
     * Generate group ID from values
     */
    private _generateGroupId(type: 'row' | 'col', values: string[], level: number): string {
        return `${type}-${level}-${values.join(GROUP_KEY_SEPARATOR)}`;
    }

    /**
     * Calculate isEmpty flag
     */
    private _calculateIsEmpty(result: IPivotTableCrossTabData['structure'], sourceRowCount: number): boolean {
        // No value fields
        if (this._valueFields.length === 0) {
            return true;
        }

        // No data rows
        if (sourceRowCount === 0) {
            return true;
        }

        // Check if all values are null/empty
        if (result.values.length === 0) {
            return true;
        }

        for (const row of result.values) {
            for (const col of row) {
                for (const val of col) {
                    if (val !== null && val !== undefined && val !== '') {
                        return false;
                    }
                }
            }
        }

        return true;
    }

    /**
     * Calculate dimensions
     */
    private _calculateDimensions(result: IPivotTableCrossTabData['structure']): IPivotTableCrossTabData['dimensions'] {
        return {
            totalRows: result.rowHeaders.length,
            totalColumns: result.columnHeaders.length,
            dataRowCount: result.rowTypes.filter((t) => t === 'data').length,
            dataColumnCount: result.columnTypes.filter((t) => t === 'data').length,
            valueFieldCount: this._valueFields.length,
        };
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
        // When there are no column fields, return empty array (no column combinations)
        if (fieldIndices.length === 0) {
            return [];
        }

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
     * Convert cell value to number | string | null
     * Handles boolean and other types
     */
    private _normalizeValue(value: unknown): number | string | null {
        if (value === null || value === undefined) {
            return null;
        }
        if (typeof value === 'boolean') {
            return value ? 1 : 0;
        }
        if (typeof value === 'number' || typeof value === 'string') {
            return value;
        }
        return String(value);
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
