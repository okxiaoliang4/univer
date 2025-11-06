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
import type { Observable } from 'rxjs';
import { Disposable, toDisposable } from '@univerjs/core';
import { BehaviorSubject, combineLatest } from 'rxjs';
import { debounceTime } from 'rxjs/operators';
import { createAggregator } from '../model/aggregation/functions';
import { AggregationType } from '../types/enum';

interface IPivotConfig {
    valueFields: string[];
    rowFields: string[];
    columnFields: string[];
    filterFields: string[];
    sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>;
}

interface IPivot {
    valueFields$: Observable<string[]>;
    rowFields$: Observable<string[]>;
    filters$: Observable<string[]>;
    columnFields$: Observable<string[]>;
    sourceData$: Observable<IObjectMatrixPrimitiveType<Nullable<ICellData>>>;
    calculatedData$: Observable<IObjectMatrixPrimitiveType<Nullable<ICellData>> | null>;

    setValueFields: (valueFields: string[]) => void;
    setRowFields: (rowFields: string[]) => void;
    setColumnFields: (columnFields: string[]) => void;
    setFilters: (filters: string[]) => void;
    setSourceData: (sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>) => void;
}

export class Pivot extends Disposable implements IPivot {
    private _valueFields$: BehaviorSubject<string[]>;
    public valueFields$: Observable<string[]>;
    private _rowFields$: BehaviorSubject<string[]>;
    public rowFields$: Observable<string[]>;
    private _filters$: BehaviorSubject<string[]>;
    public filters$: Observable<string[]>;
    private _columnFields$: BehaviorSubject<string[]>;
    public columnFields$: Observable<string[]>;
    private _sourceData$: BehaviorSubject<IObjectMatrixPrimitiveType<Nullable<ICellData>>>;
    public sourceData$: Observable<IObjectMatrixPrimitiveType<Nullable<ICellData>>>;
    private _calculatedData$: BehaviorSubject<IObjectMatrixPrimitiveType<Nullable<ICellData>> | null>;
    public calculatedData$: Observable<IObjectMatrixPrimitiveType<Nullable<ICellData>> | null>;

  // Dirty flag and cache management
    private _isDirty: boolean = true;

    constructor(config: IPivotConfig) {
        super();

        this._valueFields$ = new BehaviorSubject<string[]>(config.valueFields);
        this._rowFields$ = new BehaviorSubject<string[]>(config.rowFields);
        this._columnFields$ = new BehaviorSubject<string[]>(config.columnFields);
        this._filters$ = new BehaviorSubject<string[]>(config.filterFields);
        this._sourceData$ = new BehaviorSubject<IObjectMatrixPrimitiveType<Nullable<ICellData>>>(config.sourceData);
        this._calculatedData$ = new BehaviorSubject<IObjectMatrixPrimitiveType<Nullable<ICellData>> | null>(null);

        this.valueFields$ = this._valueFields$.asObservable();
        this.rowFields$ = this._rowFields$.asObservable();
        this.filters$ = this._filters$.asObservable();
        this.columnFields$ = this._columnFields$.asObservable();
        this.sourceData$ = this._sourceData$.asObservable();
        this.calculatedData$ = this._calculatedData$.asObservable();

    // Set up debounced calculation pipeline
        this._setupDebouncedCalculation();

    // Initial calculation
        this._markDirty();

        this.disposeWithMe(
            toDisposable(() => {
                this._valueFields$.complete();
                this._rowFields$.complete();
                this._filters$.complete();
                this._columnFields$.complete();
                this._sourceData$.complete();
                this._calculatedData$.complete();
            })
        );
    }

  /**
   * Set up event-driven debounced calculation pipeline
   */
    private _setupDebouncedCalculation(): void {
        this.disposeWithMe(
            combineLatest([
                this._valueFields$,
                this._rowFields$,
                this._columnFields$,
                this._filters$,
                this._sourceData$,
            ])
                .pipe(debounceTime(0))
                .subscribe(() => {
                    this.getCalculatedData();
                })
        );
    }

    getValueFields(): string[] {
        return this._valueFields$.value;
    }

    getRowFields(): string[] {
        return this._rowFields$.value;
    }

    getColumnFields(): string[] {
        return this._columnFields$.value;
    }

    getFilterFields(): string[] {
        return this._filters$.value;
    }

    setValueFields(valueFields: string[]): void {
        this._markDirty();
        this._valueFields$.next(valueFields);
    }

    setRowFields(rowFields: string[]): void {
        this._markDirty();
        this._rowFields$.next(rowFields);
    }

    setColumnFields(columnFields: string[]): void {
        this._markDirty();
        this._columnFields$.next(columnFields);
    }

    setFilters(filters: string[]): void {
        this._markDirty();
        this._filters$.next(filters);
    }

    setSourceData(sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>): void {
        this._markDirty();
        this._sourceData$.next(sourceData);
    }

  /**
   * Mark the pivot as dirty and schedule a debounced recalculation
   */
    private _markDirty(): void {
        this._isDirty = true;
    }

  /**
   * Force immediate calculation without debounce
   * Useful when you need the result immediately (e.g., before saving)
   */
    getCalculatedData(): IObjectMatrixPrimitiveType<Nullable<ICellData>> | null {
        if (this._isDirty) {
            const result = this._calculate();
            this._calculatedData$.next(result);
            this._isDirty = false;
            return result;
        }
        return this._calculatedData$.value || null;
    }

  /**
   * Check if the pivot is currently dirty
   */
    isDirty(): boolean {
        return this._isDirty;
    }

    private _calculate(): IObjectMatrixPrimitiveType<Nullable<ICellData>> | null {
        const sourceData = this._sourceData$.value;
        const rowFields = this._rowFields$.value;
        const columnFields = this._columnFields$.value;
        const valueFields = this._valueFields$.value;

    // Extract header row and data rows
        const { headerRow, dataRows } = this._extractData(sourceData);

        if (dataRows.length === 0 || valueFields.length === 0) {
            return null;
        }

    // Get field indices from header
        const getFieldIndex = (fieldName: string): number => {
            const index = headerRow.findIndex((cell) => cell?.v?.toString() === fieldName);
            return index >= 0 ? index : -1;
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
            return this._calculate2D(dataRows, rowFieldIndices, columnFieldIndices, valueFieldIndices, rowFields, columnFields, valueFields);
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
        rowFields: string[],
        valueFields: string[]
    ): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        const result: IObjectMatrixPrimitiveType<Nullable<ICellData>> = {};

        if (rowFieldIndices.length === 0) {
      // No row fields: aggregate all data into one row
            let currentRow = 0;

      // Add header row if there are multiple value fields
            if (valueFields.length > 1) {
                result[currentRow] = {};
                for (let i = 0; i < valueFields.length; i++) {
                    result[currentRow][i] = { v: `Sum of ${valueFields[i]}` };
                }
                currentRow++;
            }

      // Calculate aggregated values
            result[currentRow] = {};
            for (let i = 0; i < valueFieldIndices.length; i++) {
                const valueFieldIndex = valueFieldIndices[i];
        // TODO: 根据配置选择聚合函数
                const aggregator = createAggregator(AggregationType.SUM);
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

    // Group data by row fields
        const groups = this._groupByFields(dataRows, rowFieldIndices);

    // Build result matrix
        let currentRow = 0;

    // Add header row for multiple value fields
        if (valueFields.length > 1) {
            result[currentRow] = {};
      // Row field headers
            for (let i = 0; i < rowFields.length; i++) {
                result[currentRow][i] = { v: rowFields[i] };
            }
      // Value field headers
            for (let i = 0; i < valueFields.length; i++) {
                result[currentRow][rowFields.length + i] = { v: `Sum of ${valueFields[i]}` };
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
                const aggregator = createAggregator(AggregationType.SUM);
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
        _rowFields: string[],
        _columnFields: string[],
        _valueFields: string[]
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
                const aggregator = createAggregator(AggregationType.SUM);
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
}
