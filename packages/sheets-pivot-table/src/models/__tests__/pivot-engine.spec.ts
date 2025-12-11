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
import type { IPivotField, IPivotFilterCriteria } from '../../types/type';
import { ObjectMatrix } from '@univerjs/core';
import { describe, expect, it } from 'vitest';
import { defaultPlaceholderMatrix } from '../../common/default-pivot-table';
import { AggregationType } from '../../types/enum';
import { PivotEngineV3 } from '../pivot-engine';

const sourceDataArray = [
    ['Region', 'Category', 'Channel', 'Quarter', 'Sales'],
    ['华东', '笔记本电脑', '线上商城', 'Q1', '15000'],
    ['华东', '笔记本电脑', '线下门店', 'Q1', '12000'],
    ['华东', '手机', '线上商城', 'Q1', '8000'],
    ['华北', '笔记本电脑', '线上商城', 'Q2', '18000'],
    ['华北', '手机', '线下门店', 'Q2', '6500'],
    ['华南', '手机', '线上商城', 'Q3', '10000'],
    ['华南', '配件', '线下门店', 'Q3', '3000'],
    ['华东', '笔记本电脑', '线上商城', 'Q4', '16500'],
    ['华东', '手机', '线下门店', 'Q4', '9000'],
    ['华北', '配件', '线上商城', 'Q1', '2500'],
    ['华南', '笔记本电脑', '线上商城', 'Q2', '14000'],
    ['华北', '手机', '线下门店', 'Q4', '7200'],
];

function toObjectMatrix(data: (string | number | null)[][]): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
    const matrix = new ObjectMatrix<Nullable<ICellData>>();
    data.forEach((row, rowIndex) => {
        row.forEach((cell, colIndex) => {
            if (cell === undefined || cell === null || cell === '') {
                return;
            }
            matrix.setValue(rowIndex, colIndex, { v: cell });
        });
    });
    return matrix.getMatrix();
}

function matrixToCsv(matrix: IObjectMatrixPrimitiveType<Nullable<ICellData>>): string {
    let csv = '';
    const objMatrix = new ObjectMatrix<Nullable<ICellData>>(matrix);
    objMatrix.toArray().forEach((row) => {
        csv += `${row.map((cell) => cell?.v ?? '').join(',')}\n`;
    });
    return csv.trim();
}

function getAggregationLabel(name: string, aggregation = AggregationType.SUM): string {
    return {
        [AggregationType.SUM]: `用于“${name}”的 SUM`,
        [AggregationType.COUNT]: `用于“${name}”的 COUNT`,
        [AggregationType.AVERAGE]: `用于“${name}”的 AVERAGE`,
        [AggregationType.MAX]: `用于“${name}”的 MAX`,
        [AggregationType.MIN]: `用于“${name}”的 MIN`,
    }[aggregation];
}

const sumOfSales = getAggregationLabel('Sales');
const countOfCategory = getAggregationLabel('Category', AggregationType.COUNT);

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

describe('PivotEngine', () => {
    const sourceData = toObjectMatrix(sourceDataArray);
    it('should construct pivot engine V3', () => {
        expect(PivotEngineV3).toBeDefined();
    });

    describe('helpers: getRowInfo/getColumnInfo/getCellInfo', () => {
        it('row/col/cell info for basic pivot', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });

            engine.getPivotModel(); // trigger calculation

            const rowInfo = engine.getRowInfo(0);
            expect(rowInfo.type).toBe('data');
            expect(rowInfo.headers).toEqual(['华北']);

            const colInfo = engine.getColumnInfo(0);
            expect(colInfo.type).toBe('data');
            expect(colInfo.headers).toEqual([sumOfSales]);

            const cellInfo = engine.getCellInfo(0, 0);
            expect(cellInfo.value).toBe(34200);
            expect(cellInfo.rowInfo.type).toBe('data');
            expect(cellInfo.columnInfo.type).toBe('data');
        });

        it('determineCellType uses header rows and data rows', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });

            const headerRows = engine.getHeaderRowsCount();

            // Matrix header row (row 0, col 0) should be header
            const headerCell = engine.determineCellType(0, 0);
            expect(headerCell.type).toBe('header');

            // First data row header cell (rowHeader) after header rows
            const rowHeaderCell = engine.determineCellType(headerRows, 0);
            expect(rowHeaderCell.type).toBe('rowHeader');

            // First data value cell
            const dataCell = engine.determineCellType(headerRows, 1);
            expect(dataCell.type).toBe('data');
        });
    });

    describe('empty result', () => {
        it('empty when no fields', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            const result = engine.getOutputMatrix();
            expect(result).toEqual(defaultPlaceholderMatrix);
        });

        it('non empty with rowFields', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            const result = engine.getOutputMatrix();
            expect(result).not.toEqual(defaultPlaceholderMatrix);
        });

        it('non empty with columnFields', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 3)],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            const result = engine.getOutputMatrix();
            expect(result).not.toEqual(defaultPlaceholderMatrix);
        });

        it('non empty with valueFields', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });
            const result = engine.getOutputMatrix();
            expect(result).not.toEqual(defaultPlaceholderMatrix);
        });

        it('empty with filter', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [],
                valueFields: [],
                filterFields: [createFilterField('Region', 0, { type: 'value', values: ['华北'] })],
                sourceData,
            });
            const result = engine.getOutputMatrix();
            expect(result).toEqual(defaultPlaceholderMatrix);
        });
    });

    describe('only valueFields', () => {
        it('single valueField', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('value-only-single');
        });

        it('multiple valueFields', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [],
                valueFields: [
                    createValueField('Sales', 4),
                    createValueField('Category', 1, AggregationType.COUNT),
                ],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('value-only-multi');
        });
    });

    describe('rowFields + valueFields', () => {
        it('single row field + single value field', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('row1-value1');
        });

        it('multiple row fields + single value field', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('rowN-value1');
        });

        it('single row field + multiple value fields', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('row1-valueN');
        });

        it('multiple row fields + multiple value fields', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('rowN-valueN');
        });
    });

    describe('columnFields + valueFields', () => {
        it('single column field + single value field', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 3)],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('col1-value1');
        });

        it('multiple column fields + single value field', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 3), createColumnField('Region', 0)],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('colN-value1');
        });

        it('single column field + multiple value fields', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [createColumnField('Region', 0)],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('col1-valueN');
        });

        it('multiple column fields + multiple value fields', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 3), createColumnField('Region', 0)],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('colN-valueN');
        });
    });

    describe('rowFields + columnFields (no value fields)', () => {
        it('single row + single column', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Quarter', 3)],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('row1-col1');
        });

        it('multiple row + single column', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3)],
                columnFields: [createColumnField('Category', 1)],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('rowN-col1');
        });

        it('single row + multiple columns', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Category', 1), createColumnField('Quarter', 3)],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('row1-colN');
        });

        it('multiple rows + multiple columns', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3)],
                columnFields: [createColumnField('Category', 1), createColumnField('Channel', 2)],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            expect(engine.getOutputMatrix()).toMatchSnapshot('rowN-colN');
        });
    });

    describe('totals and subtotals', () => {
        it('subtotals: row fields (Region, Quarter) + column fields (Category, Channel) + single value', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0, true), createRowField('Quarter', 3)],
                columnFields: [createColumnField('Category', 1, true), createColumnField('Channel', 2, true)],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });

            expect(engine.getOutputMatrix()).toMatchSnapshot('subtotal-row-col-single-value');
        });

        it('subtotals: row fields (Region, Quarter) + column fields (Category, Channel) + single value, row subtotals enabled', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0, true), createRowField('Quarter', 3, true)],
                columnFields: [createColumnField('Category', 1, true), createColumnField('Channel', 2, true)],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });

            expect(engine.getOutputMatrix()).toMatchSnapshot('subtotal-row-enabled-single-value');
        });

        it('grand totals: row subtotals (Region) + col subtotals (Category, Channel) + multi values', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0, true), createRowField('Quarter', 3)],
                columnFields: [createColumnField('Category', 1, true), createColumnField('Channel', 2, true)],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });

            expect(engine.getOutputMatrix()).toMatchSnapshot('grand-row-col-multi-value');
        });

        it('grand totals: col subtotals (Category, Channel) + row subtotals (Quarter) + multi values', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3, true)],
                columnFields: [createColumnField('Category', 1, true), createColumnField('Channel', 2, true)],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });

            expect(engine.getOutputMatrix()).toMatchSnapshot('grand-col-rowSub-multi-value');
        });

        it('grand totals: row subtotals (Region, Quarter) + col subtotals (Category, Channel) + multi values', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0, true), createRowField('Quarter', 3, true)],
                columnFields: [createColumnField('Category', 1, true), createColumnField('Channel', 2, true)],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });

            expect(engine.getOutputMatrix()).toMatchSnapshot('grand-row-col-subtotals-multi-value');
        });
    });
});
