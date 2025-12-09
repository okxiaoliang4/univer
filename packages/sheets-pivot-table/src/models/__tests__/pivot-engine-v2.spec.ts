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
import type { IPivotField, IPivotFilterCriteria, IPivotTableCrossTabData } from '../../types/type';

import { afterEach, describe, expect, it, vi } from 'vitest';
import { AggregationType, PivotValuePosition } from '../../types/enum';
import { PivotEngineV2 } from '../pivot-engine-v2';
import { PivotTableRenderModel } from '../pivot-table-render-model';

// Helper functions to create test data
function createValueField(name: string, sourceColumnIndex: number, aggregation: AggregationType = AggregationType.SUM): IPivotField {
    return {
        id: `value-${name}-${aggregation}`,
        sourceColumnIndex,
        name,
        aggregation,
    };
}

function createRowField(name: string, sourceColumnIndex: number, showSubTotals?: boolean): IPivotField {
    return {
        id: `row-${name}`,
        sourceColumnIndex,
        name,
        showSubTotals,
    };
}

function createColumnField(name: string, sourceColumnIndex: number, showSubTotals?: boolean): IPivotField {
    return {
        id: `col-${name}`,
        sourceColumnIndex,
        name,
        showSubTotals,
    };
}

function createFilterField(name: string, sourceColumnIndex: number, filter: IPivotFilterCriteria): IPivotField {
    return {
        id: `filter-${name}`,
        sourceColumnIndex,
        name,
        filter,
    };
}

function createSourceData(data: (string | number)[][]): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
    const result: IObjectMatrixPrimitiveType<Nullable<ICellData>> = {};

    data.forEach((row, rowIndex) => {
        result[rowIndex] = {};
        row.forEach((cell, colIndex) => {
            result[rowIndex][colIndex] = { v: cell };
        });
    });

    return result;
}

function getRowCount(
    values: IObjectMatrixPrimitiveType<IObjectArrayPrimitiveType<number | string | null>>
): number {
    return Object.keys(values).length;
}

function getColumnCount(
    values: IObjectMatrixPrimitiveType<IObjectArrayPrimitiveType<number | string | null>>,
    rowIndex: number
): number {
    return Object.keys(values[rowIndex] || {}).length;
}

function getValueFieldCount(
    values: IObjectMatrixPrimitiveType<IObjectArrayPrimitiveType<number | string | null>>,
    rowIndex: number,
    columnIndex: number
): number {
    return Object.keys(values[rowIndex]?.[columnIndex] || {}).length;
}

describe('PivotEngineV2', () => {
    afterEach(() => {
        vi.clearAllMocks();
    });

    describe('Basic functionality', () => {
        it('should create an instance', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [],
                filterFields: [],
                sourceData: {},
                valuePosition: PivotValuePosition.COLUMN,
            });
            expect(engine).toBeDefined();
        });

        it('should return empty result when no value fields', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.isEmpty).toBe(true);
            expect(result.dimensions.valueFieldCount).toBe(0);
        });

        it('should return empty result when no data rows', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: {},
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.isEmpty).toBe(true);
        });
    });

    describe('Single row field + single value field', () => {
        it('should calculate simple row-only pivot', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['South', 200],
                    ['North', 150],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.isEmpty).toBe(false);
            expect(result.dimensions.totalRows).toBe(2);
            expect(result.dimensions.dataRowCount).toBe(2);
            expect(result.structure.rowHeaders).toEqual([
                ['North'],
                ['South'],
            ]);
            expect(result.structure.values[0][0][0]).toBe(250); // North: 100 + 150
            expect(result.structure.values[1][0][0]).toBe(200); // South: 200
        });
    });

    describe('2D Cross-Tabulation (row + column + value)', () => {
        it('should calculate 2D pivot table', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Quarter', 1)],
                valueFields: [createValueField('Sales', 2)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Quarter', 'Sales'],
                    ['North', 'Q1', 100],
                    ['North', 'Q2', 150],
                    ['South', 'Q1', 200],
                    ['South', 'Q2', 250],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.isEmpty).toBe(false);
            expect(result.dimensions.totalRows).toBe(2);
            expect(result.dimensions.totalColumns).toBe(2);

            // Check row headers
            expect(result.structure.rowHeaders).toEqual([
                ['North'],
                ['South'],
            ]);

            // Check column headers
            expect(result.structure.columnHeaders).toEqual([
                ['Q1'],
                ['Q2'],
            ]);

            // Check values: North-Q1=100, North-Q2=150, South-Q1=200, South-Q2=250
            expect(result.structure.values[0][0][0]).toBe(100);
            expect(result.structure.values[0][1][0]).toBe(150);
            expect(result.structure.values[1][0][0]).toBe(200);
            expect(result.structure.values[1][1][0]).toBe(250);
        });
    });

    describe('Subtotals', () => {
        it('should add subtotal rows when showSubTotals is true', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0, true)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['North', 150],
                    ['South', 200],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.isEmpty).toBe(false);

            // Should have: North data, North subtotal, South data, South subtotal, Grand total
            expect(result.dimensions.totalRows).toBeGreaterThan(2);
            expect(result.structure.subtotalRows).toBeDefined();
            expect(result.structure.subtotalRows!.length).toBeGreaterThan(0);

            // Check subtotal row types
            const subtotalRowIndices = result.structure.subtotalRows!.map((sr) => sr.rowIndex);
            subtotalRowIndices.forEach((index) => {
                if (index !== undefined) {
                    expect(result.structure.rowTypes[index]).toBe('subtotal');
                }
            });
        });

        it('should add grand total row when first row field has showSubTotals', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0, true)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['South', 200],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();

            // Check if grand total row exists (last row with '总计' in first column)
            const grandTotalRow = result.structure.rowHeaders.find(
                (headers, index) => headers[0] === '总计' && result.structure.rowTypes[index] === 'subtotal'
            );
            expect(grandTotalRow).toBeDefined();
        });

        it('should add grand total column when first column field has showSubTotals', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Quarter', 1, true)],
                valueFields: [createValueField('Sales', 2)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Quarter', 'Sales'],
                    ['North', 'Q1', 100],
                    ['North', 'Q2', 150],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();

            // Check if grand total column exists
            const grandTotalColumn = result.structure.columnHeaders.find(
                (headers) => headers[0] === '总计'
            );
            expect(grandTotalColumn).toBeDefined();
            expect(result.structure.subtotalColumns).toBeDefined();
        });
    });

    describe('Multiple value fields', () => {
        it('should handle multiple value fields', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [
                    createValueField('Sales', 1, AggregationType.SUM),
                    createValueField('Count', 1, AggregationType.COUNT),
                ],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['North', 150],
                    ['South', 200],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.isEmpty).toBe(false);
            expect(result.dimensions.valueFieldCount).toBe(2);
            expect(result.structure.valueFieldHeaders).toBeDefined();
            expect(result.structure.valueFieldHeaders!.length).toBe(2);

            // Check values: first value field (SUM), second value field (COUNT)
            expect(result.structure.values[0][0][0]).toBe(250); // North SUM
            expect(result.structure.values[0][0][1]).toBe(2); // North COUNT
        });

        it('should display value field headers in column headers when there are column fields', () => {
            const engine = new PivotEngineV2({
                rowFields: [
                    createRowField('Region', 0),
                    createRowField('Product', 1),
                ],
                columnFields: [createColumnField('Quarter', 2)],
                valueFields: [createValueField('Sales', 3, AggregationType.SUM)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Product', 'Quarter', 'Sales'],
                    ['华东', '手机', 'Q1', 8000],
                    ['华东', '手机', 'Q4', 9000],
                    ['华东', '笔记本电脑', 'Q1', 15000],
                    ['华东', '笔记本电脑', 'Q4', 16500],
                    ['华北', '手机', 'Q2', 6500],
                    ['华北', '手机', 'Q4', 7200],
                    ['华北', '笔记本电脑', 'Q2', 18000],
                    ['华北', '配件', 'Q1', 2500],
                    ['华南', '手机', 'Q3', 10000],
                    ['华南', '笔记本电脑', 'Q2', 14000],
                    ['华南', '配件', 'Q3', 3000],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const matrix = engine.getCalculatedCellMatrix();
            expect(matrix).not.toBeNull();

            // Check that column headers include value field name
            // First row should have column field values (Quarter values)
            // Second row should have row field names + value field name under each column
            const headerRow0 = matrix?.[0];
            const headerRow1 = matrix?.[1];

            expect(headerRow0).toBeDefined();
            expect(headerRow1).toBeDefined();

            // First header row: empty for row fields + column headers (Quarter values)
            // Duplicate values should be removed - only show once
            expect(headerRow0?.[0]?.v).toBe(''); // Empty for row field column
            expect(headerRow0?.[1]?.v).toBe(''); // Empty for row field column
            // Quarter values should appear only once (first occurrence)
            expect(headerRow0?.[2]?.v).toBe('Q1');
            expect(headerRow0?.[3]?.v).toBe('Q2');
            expect(headerRow0?.[4]?.v).toBe('Q3');
            expect(headerRow0?.[5]?.v).toBe('Q4');

            // Second header row: row field names + value field name repeated for each column
            expect(headerRow1?.[0]?.v).toBe('Region'); // First row field name
            expect(headerRow1?.[1]?.v).toBe('Product'); // Second row field name
            // Value field name should appear under each column
            expect(headerRow1?.[2]?.v).toBe('Sum of Sales'); // Value field name for Q1
            expect(headerRow1?.[3]?.v).toBe('Sum of Sales'); // Value field name for Q2
            expect(headerRow1?.[4]?.v).toBe('Sum of Sales'); // Value field name for Q3
            expect(headerRow1?.[5]?.v).toBe('Sum of Sales'); // Value field name for Q4
        });

        it('should display multiple value field headers in column headers when there are column fields', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Quarter', 1)],
                valueFields: [
                    createValueField('Sales', 2, AggregationType.SUM),
                    createValueField('Quantity', 3, AggregationType.COUNT),
                ],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Quarter', 'Sales', 'Quantity'],
                    ['North', 'Q1', 100, 5],
                    ['North', 'Q2', 150, 8],
                    ['South', 'Q1', 200, 10],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const matrix = engine.getCalculatedCellMatrix();
            expect(matrix).not.toBeNull();

            // Check that column headers include value field names
            const headerRow0 = matrix?.[0];
            const headerRow1 = matrix?.[1];

            expect(headerRow0).toBeDefined();
            expect(headerRow1).toBeDefined();

            // First header row: empty for row field + column headers (Quarter values)
            // Duplicate values should be removed - only show once
            expect(headerRow0?.[0]?.v).toBe(''); // Empty for row field column
            // Quarter values should appear only once (first occurrence), even with multiple value fields
            expect(headerRow0?.[1]?.v).toBe('Q1');
            expect(headerRow0?.[2]?.v).toBe(''); // Duplicate Q1 should be empty
            expect(headerRow0?.[3]?.v).toBe('Q2');
            expect(headerRow0?.[4]?.v).toBe(''); // Duplicate Q2 should be empty

            // Second header row: row field name + value field names repeated for each column
            expect(headerRow1?.[0]?.v).toBe('Region'); // Row field name
            // Each column should have both value field names
            expect(headerRow1?.[1]?.v).toBe('Sum of Sales'); // First value field for Q1
            expect(headerRow1?.[2]?.v).toBe('Count of Quantity'); // Second value field for Q1
            expect(headerRow1?.[3]?.v).toBe('Sum of Sales'); // First value field for Q2
            expect(headerRow1?.[4]?.v).toBe('Count of Quantity'); // Second value field for Q2
        });
    });

    describe('Filter fields', () => {
        it('should apply value filter', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [
                    createFilterField('Region', 0, {
                        type: 'value',
                        values: ['North'],
                    }),
                ],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['South', 200],
                    ['North', 150],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.isEmpty).toBe(false);

            // Should only have North rows
            expect(result.structure.rowHeaders.length).toBe(1);
            expect(result.structure.rowHeaders[0][0]).toBe('North');
            expect(result.structure.values[0][0][0]).toBe(250); // North: 100 + 150
        });

        it('should apply condition filter', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [
                    createFilterField('Sales', 1, {
                        type: 'condition',
                        operator: 'greaterThan',
                        conditionValue: 150,
                    }),
                ],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['South', 200],
                    ['North', 150],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            // Should only include rows with Sales > 150
            // Only South (200) should remain
            expect(result.structure.values[0][0][0]).toBe(200);
        });
    });

    describe('Multiple row/column fields', () => {
        it('should handle multiple row fields', () => {
            const engine = new PivotEngineV2({
                rowFields: [
                    createRowField('Region', 0),
                    createRowField('Product', 1),
                ],
                columnFields: [],
                valueFields: [createValueField('Sales', 2)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Product', 'Sales'],
                    ['North', 'A', 100],
                    ['North', 'B', 200],
                    ['South', 'A', 150],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.isEmpty).toBe(false);

            // Should have multiple rows with two-level headers
            expect(result.structure.rowHeaders.length).toBeGreaterThan(1);
            expect(result.structure.rowHeaders[0].length).toBe(2); // Region + Product
        });

        it('should handle multiple column fields', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [
                    createColumnField('Year', 0),
                    createColumnField('Quarter', 1),
                ],
                valueFields: [createValueField('Sales', 2)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Year', 'Quarter', 'Sales'],
                    ['2023', 'Q1', 100],
                    ['2023', 'Q2', 150],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.isEmpty).toBe(false);

            // Should have multiple columns with two-level headers
            expect(result.structure.columnHeaders.length).toBeGreaterThan(0);
            expect(result.structure.columnHeaders[0].length).toBe(2); // Year + Quarter
        });

        it('should display multiple column field levels in header rows', () => {
            const engine = new PivotEngineV2({
                rowFields: [
                    createRowField('Region', 0),
                    createRowField('Product', 1),
                ],
                columnFields: [
                    createColumnField('Quarter', 2),
                    createColumnField('Channel', 3),
                ],
                valueFields: [createValueField('Sales', 4, AggregationType.SUM)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Product', 'Quarter', 'Channel', 'Sales'],
                    ['华东', '手机', 'Q1', '线下门店', 8000],
                    ['华东', '手机', 'Q1', '线上商店', 9000],
                    ['华东', '手机', 'Q2', '线下门店', 10000],
                    ['华东', '手机', 'Q2', '线上商店', 11000],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const matrix = engine.getCalculatedCellMatrix();
            expect(matrix).not.toBeNull();

            // Should have multiple header rows for column fields
            // Row 0: Empty for row fields + First column field values (Q1, Q2)
            // Row 1: Empty for row fields + Second column field values (线下门店, 线上商店)
            // Row 2: Row field names + Value field name (Region | Product | Sum of Sales | ...)

            const headerRow0 = matrix?.[0];
            const headerRow1 = matrix?.[1];
            const headerRow2 = matrix?.[2];

            expect(headerRow0).toBeDefined();
            expect(headerRow1).toBeDefined();
            expect(headerRow2).toBeDefined();

            // First header row: empty for row fields + first column field values
            // Duplicate values should be removed - only show once
            expect(headerRow0?.[0]?.v).toBe('');
            expect(headerRow0?.[1]?.v).toBe('');
            // Q1 should appear only once (first occurrence), others should be empty
            expect(headerRow0?.[2]?.v).toBe('Q1');
            expect(headerRow0?.[3]?.v).toBe(''); // Duplicate Q1 should be empty
            // Q2 should appear only once (first occurrence)
            expect(headerRow0?.[4]?.v).toBe('Q2');
            expect(headerRow0?.[5]?.v).toBe(''); // Duplicate Q2 should be empty

            // Second header row: empty for row fields + second column field values
            // Note: Column order depends on how columnCombos are sorted
            // Duplicate values should be removed within each parent group
            // Q1 -> 线下门店, 线上商店 (each shown once)
            // Q2 -> 线下门店, 线上商店 (each shown once)
            expect(headerRow1?.[0]?.v).toBe('');
            expect(headerRow1?.[1]?.v).toBe('');
            // Check that values appear once per parent group
            // Under Q1: should have both 线下门店 and 线上商店
            // Under Q2: should have both 线下门店 and 线上商店 again
            const secondLevelValues = [
                headerRow1?.[2]?.v,
                headerRow1?.[3]?.v,
                headerRow1?.[4]?.v,
                headerRow1?.[5]?.v,
            ];
            // Should contain both values, appearing once per parent group
            const nonEmptyValues = secondLevelValues.filter((v) => v !== '');
            expect(nonEmptyValues).toContain('线下门店');
            expect(nonEmptyValues).toContain('线上商店');
            // Each value should appear twice (once for Q1, once for Q2)
            expect(nonEmptyValues.filter((v) => v === '线下门店').length).toBe(2);
            expect(nonEmptyValues.filter((v) => v === '线上商店').length).toBe(2);

            // Third header row: row field names + value field name
            expect(headerRow2?.[0]?.v).toBe('Region');
            expect(headerRow2?.[1]?.v).toBe('Product');
            expect(headerRow2?.[2]?.v).toBe('Sum of Sales');
            expect(headerRow2?.[3]?.v).toBe('Sum of Sales');
            expect(headerRow2?.[4]?.v).toBe('Sum of Sales');
            expect(headerRow2?.[5]?.v).toBe('Sum of Sales');
        });

        it('should handle three-level column fields with proper deduplication', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [
                    createColumnField('Quarter', 1),
                    createColumnField('Channel', 2),
                    createColumnField('Type', 3),
                ],
                valueFields: [createValueField('Sales', 4, AggregationType.SUM)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Quarter', 'Channel', 'Type', 'Sales'],
                    ['华东', 'Q1', '线下门店', 'A', 8000],
                    ['华东', 'Q1', '线下门店', 'B', 9000],
                    ['华东', 'Q1', '线上商店', 'A', 10000],
                    ['华东', 'Q1', '线上商店', 'B', 11000],
                    ['华东', 'Q2', '线下门店', 'A', 12000],
                    ['华东', 'Q2', '线下门店', 'B', 13000],
                    ['华东', 'Q2', '线上商店', 'A', 14000],
                    ['华东', 'Q2', '线上商店', 'B', 15000],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const matrix = engine.getCalculatedCellMatrix();
            expect(matrix).not.toBeNull();

            // Should have 4 header rows:
            // Row 0: Q1, Q2 (each shown once)
            // Row 1: 线下门店, 线上商店 (each shown once per Q)
            // Row 2: A, B (each shown once per Q-Channel combination)
            // Row 3: Region | Sum of Sales

            const headerRow0 = matrix?.[0];
            const headerRow1 = matrix?.[1];
            const headerRow2 = matrix?.[2];
            const headerRow3 = matrix?.[3];

            expect(headerRow0).toBeDefined();
            expect(headerRow1).toBeDefined();
            expect(headerRow2).toBeDefined();
            expect(headerRow3).toBeDefined();

            // First level: Q1, Q2 (each shown once)
            expect(headerRow0?.[1]?.v).toBe('Q1');
            expect(headerRow0?.[2]?.v).toBe(''); // Duplicate Q1 should be empty
            expect(headerRow0?.[3]?.v).toBe(''); // Duplicate Q1 should be empty
            expect(headerRow0?.[4]?.v).toBe(''); // Duplicate Q1 should be empty
            expect(headerRow0?.[5]?.v).toBe('Q2');
            expect(headerRow0?.[6]?.v).toBe(''); // Duplicate Q2 should be empty
            expect(headerRow0?.[7]?.v).toBe(''); // Duplicate Q2 should be empty
            expect(headerRow0?.[8]?.v).toBe(''); // Duplicate Q2 should be empty

            // Second level: 线下门店, 线上商店 (each shown once per Q)
            // Under Q1: 线下门店, 线上商店
            // Under Q2: 线下门店, 线上商店
            const q1Channels = [
                headerRow1?.[1]?.v,
                headerRow1?.[2]?.v,
                headerRow1?.[3]?.v,
                headerRow1?.[4]?.v,
            ];
            const q2Channels = [
                headerRow1?.[5]?.v,
                headerRow1?.[6]?.v,
                headerRow1?.[7]?.v,
                headerRow1?.[8]?.v,
            ];

            // Q1 should have both 线下门店 and 线上商店 (each once)
            const q1NonEmpty = q1Channels.filter((v) => v !== '');
            expect(q1NonEmpty).toContain('线下门店');
            expect(q1NonEmpty).toContain('线上商店');
            expect(q1NonEmpty.length).toBe(2);

            // Q2 should have both 线下门店 and 线上商店 (each once)
            const q2NonEmpty = q2Channels.filter((v) => v !== '');
            expect(q2NonEmpty).toContain('线下门店');
            expect(q2NonEmpty).toContain('线上商店');
            expect(q2NonEmpty.length).toBe(2);

            // Third level: A, B (each shown once per Q-Channel combination)
            // Q1-线下门店: A, B
            // Q1-线上商店: A, B
            // Q2-线下门店: A, B
            // Q2-线上商店: A, B
            const thirdLevelValues = [
                headerRow2?.[1]?.v,
                headerRow2?.[2]?.v,
                headerRow2?.[3]?.v,
                headerRow2?.[4]?.v,
                headerRow2?.[5]?.v,
                headerRow2?.[6]?.v,
                headerRow2?.[7]?.v,
                headerRow2?.[8]?.v,
            ];
            const thirdLevelNonEmpty = thirdLevelValues.filter((v) => v !== '');
            // Should have A and B, each appearing 4 times (once per Q-Channel combo)
            expect(thirdLevelNonEmpty.filter((v) => v === 'A').length).toBe(4);
            expect(thirdLevelNonEmpty.filter((v) => v === 'B').length).toBe(4);
        });
    });

    describe('Aggregation types', () => {
        it('should support SUM aggregation', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1, AggregationType.SUM)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['North', 150],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.structure.values[0][0][0]).toBe(250);
        });

        it('should support COUNT aggregation', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['North', 150],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.structure.values[0][0][0]).toBe(2);
        });

        it('should support AVERAGE aggregation', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1, AggregationType.AVERAGE)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['North', 200],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.structure.values[0][0][0]).toBe(150);
        });
    });

    describe('Dimensions calculation', () => {
        it('should calculate dimensions correctly', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0, true)],
                columnFields: [createColumnField('Quarter', 1)],
                valueFields: [createValueField('Sales', 2)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Quarter', 'Sales'],
                    ['North', 'Q1', 100],
                    ['South', 'Q1', 200],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.dimensions.totalRows).toBeGreaterThan(0);
            expect(result.dimensions.totalColumns).toBeGreaterThan(0);
            expect(result.dimensions.dataRowCount).toBeGreaterThan(0);
            expect(result.dimensions.dataColumnCount).toBeGreaterThan(0);
            expect(result.dimensions.valueFieldCount).toBe(1);
        });
    });

    describe('isEmpty calculation', () => {
        it('should return isEmpty=true when all values are null', () => {
            const sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>> = {
                0: { 0: { v: 'Region' }, 1: { v: 'Sales' } },
                1: { 0: { v: 'North' }, 1: { v: null } },
            };

            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            // Note: This depends on aggregation behavior - COUNT would return 0, SUM would return null
            // The actual isEmpty logic checks if all values are null/empty
            expect(result.isEmpty).toBe(true);
        });
    });

    describe('Group information', () => {
        it('should generate row groups', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0, true)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['South', 200],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.structure.rowGroups).toBeDefined();
            expect(result.structure.rowGroups!.length).toBeGreaterThan(0);
            expect(result.structure.rowLevelMap).toBeDefined();
        });

        it('should generate column groups', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 0)],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Quarter', 'Sales'],
                    ['Q1', 100],
                    ['Q2', 200],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const result = engine.getCalculatedData();
            expect(result.structure.columnGroups).toBeDefined();
            expect(result.structure.columnLevelMap).toBeDefined();
        });
    });

    describe('Setter methods', () => {
        it('should mark dirty when setRowFields is called', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            engine.getCalculatedData(); // Calculate once
            expect(engine.isDirty()).toBe(false);

            engine.setRowFields([createRowField('Region', 0)]);
            expect(engine.isDirty()).toBe(true);
        });

        it('should mark dirty when setColumnFields is called', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([['Sales'], [100]]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            engine.getCalculatedData();
            expect(engine.isDirty()).toBe(false);

            engine.setColumnFields([createColumnField('Quarter', 0)]);
            expect(engine.isDirty()).toBe(true);
        });

        it('should mark dirty when setValueFields is called', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([['Sales'], [100]]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            engine.getCalculatedData();
            expect(engine.isDirty()).toBe(false);

            engine.setValueFields([createValueField('Count', 1, AggregationType.COUNT)]);
            expect(engine.isDirty()).toBe(true);
        });

        it('should mark dirty when setSourceData is called', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([['Sales'], [100]]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            engine.getCalculatedData();
            expect(engine.isDirty()).toBe(false);

            engine.setSourceData(createSourceData([['Sales'], [200]]));
            expect(engine.isDirty()).toBe(true);
        });
    });
});

describe('PivotTableRenderModel', () => {
    describe('Basic functionality', () => {
        it('should create an instance', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const data = engine.getCalculatedData();
            const renderModel = new PivotTableRenderModel(data);
            expect(renderModel).toBeDefined();
        });
    });

    describe('Row visibility', () => {
        it('should return visible row indices', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0, true)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['South', 200],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const data = engine.getCalculatedData();
            const renderModel = new PivotTableRenderModel(data);

            const visibleIndices = renderModel.getVisibleRowIndices();
            expect(visibleIndices.length).toBeGreaterThan(0);
        });

        it('should check row visibility correctly', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0, true)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const data = engine.getCalculatedData();
            const renderModel = new PivotTableRenderModel(data);

            // All rows should be visible initially
            const totalRows = data.structure.rowHeaders.length;
            for (let i = 0; i < totalRows; i++) {
                expect(renderModel.isRowVisible(i)).toBe(true);
            }
        });
    });

    describe('Collapse/Expand', () => {
        it('should toggle row group collapse state', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0, true)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['South', 200],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const data = engine.getCalculatedData();
            const renderModel = new PivotTableRenderModel(data);

            const rowGroupInfo = renderModel.getRowGroupInfo(0);
            if (rowGroupInfo) {
                const wasExpanded = rowGroupInfo.isExpanded;
                renderModel.toggleRowGroup(rowGroupInfo.groupId);
                const newGroupInfo = renderModel.getRowGroupInfo(0);
                expect(newGroupInfo?.isExpanded).toBe(!wasExpanded);
            }
        });
    });

    describe('Cell information', () => {
        it('should get cell value', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const data = engine.getCalculatedData();
            const renderModel = new PivotTableRenderModel(data);

            const value = renderModel.getCellValue(0, 0, 0);
            expect(value).toBe(100);
        });

        it('should get complete cell info', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const data = engine.getCalculatedData();
            const renderModel = new PivotTableRenderModel(data);

            const cellInfo = renderModel.getCellInfo(0, 0, 0);
            expect(cellInfo.value).toBe(100);
            expect(cellInfo.isVisible).toBe(true);
            expect(cellInfo.rowHeaders).toEqual(['North']);
        });
    });

    describe('Value Position Support', () => {
        it('should support valuePosition COLUMN (baseline)', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Product', 2)],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales', 'Product'],
                    ['North', 100, 'A'],
                    ['North', 200, 'B'],
                    ['South', 150, 'A'],
                    ['South', 250, 'B'],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const data = engine.getCalculatedData();
            expect(data.isEmpty).toBe(false);
            expect(data.structure.rowHeaders.length).toBe(2); // North, South
            expect(data.structure.columnHeaders.length).toBe(2); // Product A, Product B
            expect(data.structure.values[0][0][0]).toBe(100); // North-A
            expect(data.structure.values[0][1][0]).toBe(200); // North-B
            expect(data.structure.values[1][0][0]).toBe(150); // South-A
            expect(data.structure.values[1][1][0]).toBe(250); // South-B
        });

        it('should store values as sparse matrix to avoid empty allocation', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Product', 2)],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales', 'Product'],
                    ['North', 100, 'A'],
                    ['North', 200, 'B'],
                    ['South', 150, 'A'],
                    ['South', 250, 'B'],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const data = engine.getCalculatedData();
            expect(Array.isArray(data.structure.values)).toBe(false);
            expect(Object.keys(data.structure.values)).toEqual(['0', '1']);
            expect(Object.keys(data.structure.values[0] || {})).toEqual(['0', '1']);
            expect(Object.keys(data.structure.values[0]?.[0] || {})).toEqual(['0']);
            expect(data.structure.values[0][0][0]).toBe(100);
        });

        it('should support valuePosition ROW', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Product', 2)],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales', 'Product'],
                    ['North', 100, 'A'],
                    ['North', 200, 'B'],
                    ['South', 150, 'A'],
                    ['South', 250, 'B'],
                ]),
                valuePosition: PivotValuePosition.ROW,
            });

            const data = engine.getCalculatedData();
            expect(data.isEmpty).toBe(false);
            // With ROW position and 1 value field:
            // Should have 2 rows (one per region, value field is just an additional level in row header)
            expect(data.structure.rowHeaders.length).toBe(2);
            // Column headers should be from column fields only (no value headers)
            expect(data.structure.columnHeaders.length).toBe(2); // Product A, Product B
            // Values should still be correct
            expect(getRowCount(data.structure.values)).toBe(2);
        });

        it('should handle valuePosition change from COLUMN to ROW', () => {
            const sourceData = createSourceData([
                ['Region', 'Sales', 'Product'],
                ['North', 100, 'A'],
                ['North', 200, 'B'],
            ]);

            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Product', 2)],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
            });

            const dataColumnPos = engine.getCalculatedData();
            const rowCountColumn = dataColumnPos.structure.rowHeaders.length;
            const colCountColumn = dataColumnPos.structure.columnHeaders.length;

            // Switch to ROW position
            engine.setValuePosition(PivotValuePosition.ROW);
            const dataRowPos = engine.getCalculatedData();

            // With single value field, row count should remain same
            // (value field just becomes an additional row header level)
            expect(dataRowPos.structure.rowHeaders.length).toBe(rowCountColumn);
            expect(dataRowPos.structure.columnHeaders.length).toBe(colCountColumn);
            expect(engine.isDirty()).toBe(false); // Should be clean after recalculation
        });

        it('should handle valuePosition change from ROW to COLUMN', () => {
            const sourceData = createSourceData([
                ['Region', 'Sales', 'Product'],
                ['North', 100, 'A'],
                ['North', 200, 'B'],
            ]);

            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Product', 2)],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.ROW,
            });

            const dataRowPos = engine.getCalculatedData();
            const rowCountRow = dataRowPos.structure.rowHeaders.length;

            // Switch to COLUMN position
            engine.setValuePosition(PivotValuePosition.COLUMN);
            const dataColumnPos = engine.getCalculatedData();

            // With single value field, row count should remain same
            expect(dataColumnPos.structure.rowHeaders.length).toBe(rowCountRow);
            expect(engine.isDirty()).toBe(false);
        });

        it('should handle multiple value fields with ROW position', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [
                    createValueField('Sales', 1),
                    createValueField('Count', 2, AggregationType.COUNT),
                ],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales', 'Count'],
                    ['North', 100, 5],
                    ['South', 200, 3],
                ]),
                valuePosition: PivotValuePosition.ROW,
            });

            const data = engine.getCalculatedData();
            expect(data.isEmpty).toBe(false);
            // Should have rows for each value field per region
            // North-Sales, North-Count, South-Sales, South-Count = 4 rows
            expect(data.structure.rowHeaders.length).toBe(4);
            // Each row should have one value column
            expect(getColumnCount(data.structure.values, 0)).toBe(1);
        });

        it('should handle multiple value fields with COLUMN position', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [
                    createValueField('Sales', 1),
                    createValueField('Count', 2, AggregationType.COUNT),
                ],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales', 'Count'],
                    ['North', 100, 5],
                    ['South', 200, 3],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const data = engine.getCalculatedData();
            expect(data.isEmpty).toBe(false);
            // Should have rows for each region
            expect(data.structure.rowHeaders.length).toBe(2);
            // Column headers should include value field names
            expect(data.structure.valueFieldHeaders).toBeDefined();
            expect(data.structure.valueFieldHeaders?.length).toBe(2);
            // When no column fields, single column, but with multiple values per cell
            expect(getColumnCount(data.structure.values, 0)).toBe(1);
            // Each cell should contain an array of value field values
            expect(getValueFieldCount(data.structure.values, 0, 0)).toBe(2);
        });

        it('should verify dirty flag is reset after recalculation', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            // First call triggers recalculation
            engine.getCalculatedData();
            // Should be clean after calculation
            expect(engine.isDirty()).toBe(false);

            // Change position
            engine.setValuePosition(PivotValuePosition.ROW);
            // Should be marked dirty
            expect(engine.isDirty()).toBe(true);

            // Recalculate should reset dirty flag
            engine.getCalculatedData();
            expect(engine.isDirty()).toBe(false);
        });

        it('should support no-change scenario (same position set twice)', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const data1 = engine.getCalculatedData();

            // Set same position again
            engine.setValuePosition(PivotValuePosition.COLUMN);

            // Should not be dirty (optimization: same position)
            // Implementation may or may not optimize this, but result should be same
            const data2 = engine.getCalculatedData();

            expect(data1.structure.rowHeaders).toEqual(data2.structure.rowHeaders);
            expect(data1.structure.columnHeaders).toEqual(data2.structure.columnHeaders);
        });

        it('should handle row-only pivot with valuePosition ROW', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['South', 200],
                ]),
                valuePosition: PivotValuePosition.ROW,
            });

            const data = engine.getCalculatedData();
            expect(data.isEmpty).toBe(false);
            // Should have rows for each region's values
            expect(data.structure.rowHeaders.length).toBeGreaterThan(0);
            expect(data.structure.columnHeaders.length).toBeGreaterThan(0);
        });

        it('should handle row-only pivot with valuePosition COLUMN', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                    ['South', 200],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const data = engine.getCalculatedData();
            expect(data.isEmpty).toBe(false);
            expect(data.structure.rowHeaders.length).toBe(2); // North, South
            expect(data.structure.columnHeaders.length).toBe(1);
            expect(data.structure.values[0][0][0]).toBe(100);
            expect(data.structure.values[1][0][0]).toBe(200);
        });

        it('should handle empty pivot with valuePosition changes', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [],
                filterFields: [],
                sourceData: {},
                valuePosition: PivotValuePosition.COLUMN,
            });

            const data1 = engine.getCalculatedData();
            expect(data1.isEmpty).toBe(true);

            engine.setValuePosition(PivotValuePosition.ROW);
            const data2 = engine.getCalculatedData();
            expect(data2.isEmpty).toBe(true);

            // Both should be empty
            expect(data1.isEmpty).toBe(data2.isEmpty);
        });

        it('should correctly update dimensions with position changes', () => {
            const sourceData = createSourceData([
                ['Region', 'Sales', 'Product'],
                ['North', 100, 'A'],
                ['North', 200, 'B'],
                ['South', 150, 'A'],
                ['South', 250, 'B'],
            ]);

            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Product', 2)],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
            });

            const dataColumn = engine.getCalculatedData();
            const dimsColumn = dataColumn.dimensions;

            engine.setValuePosition(PivotValuePosition.ROW);
            const dataRow = engine.getCalculatedData();
            const dimsRow = dataRow.dimensions;

            // Column dimensions should remain same (still from column fields)
            expect(dimsRow.totalColumns).toBe(dimsColumn.totalColumns);

            // Row dimensions should remain same with single value field
            expect(dimsRow.totalRows).toBe(dimsColumn.totalRows);

            // Value field count should remain same
            expect(dimsRow.valueFieldCount).toBe(dimsColumn.valueFieldCount);
        });
    });

    describe('setCalculatedData', () => {
        it('should set calculated data with isDirty = true for warm start', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            // Get fresh calculated data first
            const originalData = engine.getCalculatedData();
            expect(engine.isDirty()).toBe(false);

            // Set the same data back with isDirty = true (simulating warm start)
            // This tests that data can be set and engine remains dirty
            const success = engine.setCalculatedData(originalData, true);
            expect(success).toBe(true);
            expect(engine.isDirty()).toBe(true); // Should be marked dirty

            // Get data again - should recalculate because engine is dirty
            const newData = engine.getCalculatedData();
            expect(newData.structure.values[0][0][0]).toBe(100);
            expect(engine.isDirty()).toBe(false); // Should be clean after recalculation
        });

        it('should set calculated data with isDirty = false as authoritative', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            // Create mock calculated data
            const mockData = {
                isEmpty: false,
                dimensions: {
                    totalRows: 1,
                    totalColumns: 2,
                    dataRowCount: 1,
                    dataColumnCount: 1,
                    valueFieldCount: 1,
                },
                structure: {
                    rowHeaders: [['North']],
                    columnHeaders: [[]],
                    values: {
                        0: {
                            0: {
                                0: 200,
                            },
                        },
                    }, // Different value
                    rowTypes: ['data' as const],
                    columnTypes: ['data' as const],
                },
            };

            // Set calculated data with isDirty = false (authoritative)
            const success = engine.setCalculatedData(mockData, false);
            expect(success).toBe(true);
            expect(engine.isDirty()).toBe(false); // Should be clean

            // Get data - should return the set data without recalculation
            const cachedData = engine.getCalculatedData();
            expect(cachedData.structure.values[0][0][0]).toBe(200);
            expect(engine.isDirty()).toBe(false); // Should remain clean
        });

        it('should reject invalid data structures', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            // Test with null data
            expect(engine.setCalculatedData(null as unknown as IPivotTableCrossTabData, true)).toBe(false);

            // Test with missing structure
            expect(engine.setCalculatedData({} as unknown as IPivotTableCrossTabData, true)).toBe(false);

            // Test with missing dimensions
            expect(engine.setCalculatedData({ structure: {} } as unknown as IPivotTableCrossTabData, true)).toBe(false);
        });

        it('should reject incompatible field configurations', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            // Get valid data first
            const validData = engine.getCalculatedData();

            // Test with wrong value field count
            const invalidData = {
                ...validData,
                dimensions: {
                    ...validData.dimensions,
                    valueFieldCount: 2, // Wrong count
                },
            };
            expect(engine.setCalculatedData(invalidData, true)).toBe(false);

            // Test with wrong row depth
            const invalidRowData = {
                ...validData,
                structure: {
                    ...validData.structure,
                    rowHeaders: [['North', 'Extra']], // Wrong depth
                },
            };
            expect(engine.setCalculatedData(invalidRowData, true)).toBe(false);
        });

        it('should accept compatible data with different row counts', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 1)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Sales'],
                    ['North', 100],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            // Create data with different row count but same structure
            const compatibleData = {
                isEmpty: false,
                dimensions: {
                    totalRows: 2,
                    totalColumns: 2,
                    dataRowCount: 2,
                    dataColumnCount: 1,
                    valueFieldCount: 1,
                },
                structure: {
                    rowHeaders: [['North'], ['South']],
                    columnHeaders: [[]],
                    values: {
                        0: { 0: { 0: 100 } },
                        1: { 0: { 0: 200 } },
                    }, // Two rows
                    rowTypes: ['data' as const, 'data' as const],
                    columnTypes: ['data' as const],
                },
            };

            // Should accept compatible data
            expect(engine.setCalculatedData(compatibleData, true)).toBe(true);
            expect(engine.isDirty()).toBe(true);
        });
    });

    describe('determineCellType and cell metadata', () => {
        it('should infer cell types without relying on custom payloads and omit custom fields', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0, true)],
                columnFields: [createColumnField('Quarter', 1, true)],
                valueFields: [createValueField('Sales', 2)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'Quarter', 'Sales'],
                    ['North', 'Q1', 100],
                    ['North', 'Q2', 150],
                    ['South', 'Q1', 200],
                    ['South', 'Q2', 250],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const matrix = engine.getCalculatedCellMatrix();
            expect(matrix).not.toBeNull();

            Object.values(matrix || {}).forEach((row) => {
                Object.values(row || {}).forEach((cell) => {
                    if (cell) {
                        const cellRecord = cell as Record<string, unknown>;
                        expect('custom' in cellRecord).toBe(false);
                    }
                });
            });

            const renderModel = new PivotTableRenderModel(engine.getCalculatedData());

            expect(renderModel.determineCellType(0, 1)).toEqual({ type: 'columnHeader', level: 0 });
            expect(renderModel.determineCellType(0, 3)).toEqual({ type: 'grandTotal', level: 0 });
            expect(renderModel.determineCellType(1, 0)).toEqual({ type: 'rowHeader', level: 0 });
            expect(renderModel.determineCellType(1, 3)).toEqual({ type: 'grandTotal', level: 0 });
            expect(renderModel.determineCellType(2, 0)).toEqual({ type: 'rowHeader', level: 0 });
            expect(renderModel.determineCellType(2, 1)).toEqual({ type: 'data', level: 0 });
            expect(renderModel.determineCellType(2, 3)).toEqual({ type: 'grandTotal', level: 0 });
            // Data row (second data row, column 1) should be data, not grand total
            expect(renderModel.determineCellType(4, 1)).toEqual({ type: 'data', level: 0 });
            // Last row should be grand total
            expect(renderModel.determineCellType(6, 1)).toEqual({ type: 'grandTotal', level: 0 });
        });

        it('should return correct header levels for multi-level rows and columns', () => {
            const engine = new PivotEngineV2({
                rowFields: [
                    createRowField('Region', 0, true),
                    createRowField('City', 1, true),
                ],
                columnFields: [
                    createColumnField('Year', 2, true),
                    createColumnField('Quarter', 3, true),
                ],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData: createSourceData([
                    ['Region', 'City', 'Year', 'Quarter', 'Sales'],
                    ['North', 'A', '2024', 'Q1', 100],
                    ['North', 'A', '2024', 'Q2', 150],
                    ['South', 'B', '2024', 'Q1', 200],
                ]),
                valuePosition: PivotValuePosition.COLUMN,
            });

            const renderModel = new PivotTableRenderModel(engine.getCalculatedData());

          // Column headers span 2 levels -> levels 0 and 1
            expect(renderModel.determineCellType(0, 2)).toEqual({ type: 'columnHeader', level: 0 });
            expect(renderModel.determineCellType(1, 2)).toEqual({ type: 'columnHeader', level: 1 });

          // Row header depth is 2 -> levels 0 and 1
            expect(renderModel.determineCellType(0, 1)).toEqual({ type: 'rowHeader', level: 0 });
          // Value header row (row index = columnHeaderDepth) should report level = columnHeaderDepth
            expect(renderModel.determineCellType(2, 2)).toEqual({ type: 'columnHeader', level: 2 });

          // Data row row-header column 1 should still carry level 1
          // Data rows start after columnHeaderDepth + 1 (value header row)
            expect(renderModel.determineCellType(3, 1)).toEqual({ type: 'rowHeader', level: 1 });
        });
    });
});
