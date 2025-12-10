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
import { PivotEngineV3 } from '../pivot-engine-v3';

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
    it('should be defined', () => {
        expect(PivotEngineV3).toBeDefined();
    });

    describe('helpers: getRowInfo/getColumnInfo/getCellInfo', () => {
        it('should expose row/column/cell info for basic pivot', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });

            engine.getCalculatedData(); // trigger calculation

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

        it('determineCellType should rely on headers for header rows and row data otherwise', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });

            engine.getCalculatedData();

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

    describe('should empty result', () => {
        it('should empty result', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            expect(result).toEqual(defaultPlaceholderMatrix);
        });

        it('should not empty result rowFields', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            expect(result).not.toEqual(defaultPlaceholderMatrix);
        });

        it('should not empty result columnFields', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 3)],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            expect(result).not.toEqual(defaultPlaceholderMatrix);
        });

        it('should not empty result with valueFields', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            expect(result).not.toEqual(defaultPlaceholderMatrix);
        });

        it('should empty result with filter', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [],
                valueFields: [],
                filterFields: [createFilterField('Region', 0, { type: 'value', values: ['华北'] })],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            expect(result).toEqual(defaultPlaceholderMatrix);
        });
    });

    describe('should only valueFields', () => {
        it('should single valueFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });
            const output = toObjectMatrix([
                [sumOfSales],
                [121700],
            ]);
            const result = engine.getCalculatedCellMatrix();
            expect(result).toEqual(output);
        });

        it('should multiple valueFields in column position', () => {
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
            const result = engine.getCalculatedCellMatrix();

            const output = toObjectMatrix([
                [sumOfSales, countOfCategory],
                [121700, 12],
            ]);
            expect(result).toEqual(output);
        });
    });

    describe('should combine rowFields and valueFields', () => {
        it('should single rowFields and single valueFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['Region', sumOfSales],
                ['华北', 34200],
                ['华东', 60500],
                ['华南', 27000],
            ]);
            expect(result).toEqual(output);
        });

        it('should multiple rowFields and single valueFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['Region', 'Quarter', sumOfSales],
                ['华北', 'Q1', 2500],
                ['', 'Q2', 24500],
                ['', 'Q4', 7200],
                ['华东', 'Q1', 35000],
                ['', 'Q4', 25500],
                ['华南', 'Q2', 14000],
                ['', 'Q3', 13000],
            ]);
            expect(result).toEqual(output);
        });

        it('should single rowFields and multiple valueFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['Region', sumOfSales, countOfCategory],
                ['华北', 34200, 4],
                ['华东', 60500, 5],
                ['华南', 27000, 3],
            ]);
            expect(result).toEqual(output);
        });

        it('should multiple rowFields and multiple valueFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['Region', 'Quarter', sumOfSales, countOfCategory],
                ['华北', 'Q1', 2500, 1],
                ['', 'Q2', 24500, 2],
                ['', 'Q4', 7200, 1],
                ['华东', 'Q1', 35000, 3],
                ['', 'Q4', 25500, 2],
                ['华南', 'Q2', 14000, 1],
                ['', 'Q3', 13000, 2],
            ]);
            expect(result).toEqual(output);
        });
    });

    describe('should combine columnFields and valueFields', () => {
        it('should single columnFields and single valueFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 3)],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['', 'Quarter', '', '', ''],
                ['', 'Q1', 'Q2', 'Q3', 'Q4'],
                [sumOfSales, 37500, 38500, 13000, 32700],
            ]);
            expect(result).toEqual(output);
        });

        it('should multiple columnFields and single valueFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 3), createColumnField('Region', 0)],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['', 'Quarter', 'Region', '', '', '', '', ''],
                ['', 'Q1', '', 'Q2', '', 'Q3', 'Q4', ''],
                ['', '华北', '华东', '华北', '华南', '华南', '华北', '华东'],
                [sumOfSales, 2500, 35000, 24500, 14000, 13000, 7200, 25500],
            ]);
            expect(result).toEqual(output);
        });

        it('should single columnFields and multiple valueFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [createColumnField('Region', 0)],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['Region', '值', '', '', '', ''],
                ['华北', '', '华东', '', '华南', ''],
                [sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory],
                [34200, 4, 60500, 5, 27000, 3],
            ]);
            expect(result).toEqual(output);
        });

        it('should multiple columnFields and multiple valueFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 3), createColumnField('Region', 0)],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['Quarter', 'Region', '值', '', '', '', '', '', '', '', '', '', ''],
                ['Q1', '', '', '', 'Q2', '', '', '', 'Q3', '', 'Q4', '', '', ''],
                ['华北', '', '华东', '', '华北', '', '华南', '', '华南', '', '华北', '', '华东', ''],
                [sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory],
                [2500, 1, 35000, 3, 24500, 2, 14000, 1, 13000, 2, 7200, 1, 25500, 2],
            ]);
            expect(result).toEqual(output);
        });
    });

    describe('should combine rowFields and columnFields', () => {
        it('should single rowFields and single columnFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Quarter', 3)],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['', 'Quarter', '', '', ''],
                ['Region', 'Q1', 'Q2', 'Q3', 'Q4'],
                ['华北', '', '', '', ''],
                ['华东', '', '', '', ''],
                ['华南', '', '', '', ''],
            ]);
            expect(result).toEqual(output);
        });

        it('should multiple rowFields and single columnFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3)],
                columnFields: [createColumnField('Category', 1)],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['', '', 'Category', '', '', ''],
                ['Region', 'Quarter', '笔记本电脑', '配件', '手机'],
                ['华北', 'Q1', '', '', '', ''],
                ['', 'Q2', '', '', '', ''],
                ['', 'Q4', '', '', '', ''],
                ['华东', 'Q1', '', '', '', ''],
                ['', 'Q4', '', '', '', ''],
                ['华南', 'Q2', '', '', '', ''],
                ['', 'Q3', '', '', '', ''],
            ]);
            expect(result).toEqual(output);
        });

        it('should single rowFields and multiple columnFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Category', 1), createColumnField('Quarter', 3)],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['', 'Category', 'Quarter', '', '', '', '', '', '', ''],
                ['', '笔记本电脑', '', '', '配件', '', '手机', '', '', ''],
                ['Region', 'Q1', 'Q2', 'Q4', 'Q1', 'Q3', 'Q1', 'Q2', 'Q3', 'Q4'],
                ['华北', '', '', '', '', '', '', '', '', ''],
                ['华东', '', '', '', '', '', '', '', '', ''],
                ['华南', '', '', '', '', '', '', '', '', ''],
            ]);
            expect(result).toEqual(output);
        });

        it('should multiple rowFields and multiple columnFields in column position', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3)],
                columnFields: [createColumnField('Category', 1), createColumnField('Channel', 2)],
                valueFields: [],
                filterFields: [],
                sourceData,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['', '', 'Category', 'Channel', '', '', '', ''],
                ['', '', '笔记本电脑', '', '配件', '', '手机', ''],
                ['Region', 'Quarter', '线上商城', '线下门店', '线上商城', '线下门店', '线上商城', '线下门店'],
                ['华北', 'Q1', '', '', '', '', '', ''],
                ['', 'Q2', '', '', '', '', '', ''],
                ['', 'Q4', '', '', '', '', '', ''],
                ['华东', 'Q1', '', '', '', '', '', ''],
                ['', 'Q4', '', '', '', '', '', ''],
                ['华南', 'Q2', '', '', '', '', '', ''],
                ['', 'Q3', '', '', '', '', '', ''],
            ]);
            expect(result).toEqual(output);
        });
    });

    describe('totals and subtotals', () => {
        it('getCalculatedCellMatrix should include row/column case 1', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0, true), createRowField('Quarter', 3)],
                columnFields: [createColumnField('Category', 1, true), createColumnField('Channel', 2, true)],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });

            const result = engine.getCalculatedCellMatrix();
            const expected = toObjectMatrix([
                ['', '', 'Category', 'Channel', '', '', '', '', '', '', '', ''],
                ['', '', '笔记本电脑', '', '笔记本电脑 总计', '配件', '', '配件 总计', '手机', '', '手机 总计', '总计'],
                ['Region', 'Quarter', '线上商城', '线下门店', '', '线上商城', '线下门店', '', '线上商城', '线下门店', '', ''],
                ['华北', 'Q1', '', '', '', 2500, '', 2500, '', '', '', 2500],
                ['', 'Q2', 18000, '', 18000, '', '', '', '', 6500, 6500, 24500],
                ['', 'Q4', '', '', '', '', '', '', '', 7200, 7200, 7200],
                ['华东', 'Q1', 15000, 12000, 27000, '', '', '', 8000, '', 8000, 35000],
                ['', 'Q4', 16500, '', 16500, '', '', '', '', 9000, 9000, 25500],
                ['华南', 'Q2', 14000, '', 14000, '', '', '', '', '', '', 14000],
                ['', 'Q3', '', '', '', '', 3000, 3000, 10000, '', 10000, 13000],
                ['总计', '', 63500, 12000, 75500, 2500, 3000, 5500, 18000, 22700, 40700, 121700],
            ]);
            expect(result).toMatchObject(expected);
        });

        it('getCalculatedCellMatrix should include row/column case 2', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0, true), createRowField('Quarter', 3, true)],
                columnFields: [createColumnField('Category', 1, true), createColumnField('Channel', 2, true)],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
            });

            const result = engine.getCalculatedCellMatrix();
            const expected = toObjectMatrix([
                ['', '', 'Category', 'Channel', '', '', '', '', '', '', '', ''],
                ['', '', '笔记本电脑', '', '笔记本电脑 总计', '配件', '', '配件 总计', '手机', '', '手机 总计', '总计'],
                ['Region', 'Quarter', '线上商城', '线下门店', '', '线上商城', '线下门店', '', '线上商城', '线下门店', '', ''],
                ['华北', 'Q1', '', '', '', 2500, '', 2500, '', '', '', 2500],
                ['', 'Q2', 18000, '', 18000, '', '', '', '', 6500, 6500, 24500],
                ['', 'Q4', '', '', '', '', '', '', '', 7200, 7200, 7200],
                ['华北 总计', '', 18000, '', 18000, 2500, '', 2500, '', 13700, 13700, 34200],
                ['华东', 'Q1', 15000, 12000, 27000, '', '', '', 8000, '', 8000, 35000],
                ['', 'Q4', 16500, '', 16500, '', '', '', '', 9000, 9000, 25500],
                ['华东 总计', '', 31500, 12000, 43500, '', '', '', 8000, 9000, 17000, 60500],
                ['华南', 'Q2', 14000, '', 14000, '', '', '', '', '', '', 14000],
                ['', 'Q3', '', '', '', '', 3000, 3000, 10000, '', 10000, 13000],
                ['华南 总计', '', 14000, '', 14000, '', 3000, 3000, 10000, '', 10000, 27000],
                ['总计', '', 63500, 12000, 75500, 2500, 3000, 5500, 18000, 22700, 40700, 121700],
            ]);
            expect(result).toMatchObject(expected);
        });

        it('getCalculatedCellMatrix should include row/column grand totals when single row field has showSubTotals and multiple column field has showSubTotals', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0, true), createRowField('Quarter', 3)],
                columnFields: [createColumnField('Category', 1, true), createColumnField('Channel', 2, true)],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });

            const result = engine.getCalculatedCellMatrix();
            const expected = toObjectMatrix([
                ['', '', 'Category', 'Channel', '值', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', ''],
                ['', '', '笔记本电脑', '', '', '', '笔记本电脑 总计', '', '配件', '', '', '', '配件 总计', '', '手机', '', '', '', '手机 总计', '', '总计'],
                ['', '', '线上商城', '', '线下门店', '', '', '', '线上商城', '', '线下门店', '', '', '', '线上商城', '', '线下门店', '', '', ''],
                ['Region', 'Quarter', sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory],
                ['华北', 'Q1', '', '', '', '', '', '', 2500, 1, '', '', 2500, 1, '', '', '', '', '', '', 2500, 1],
                ['', 'Q2', 18000, 1, '', '', 18000, 1, '', '', '', '', '', '', '', '', 6500, 1, 6500, 1, 24500, 2],
                ['', 'Q4', '', '', '', '', '', '', '', '', '', '', '', '', '', '', 7200, 1, 7200, 1, 7200, 1],
                ['华东', 'Q1', 15000, 1, 12000, 1, 27000, 2, '', '', '', '', '', '', 8000, 1, '', '', 8000, 1, 35000, 3],
                ['', 'Q4', 16500, 1, '', '', 16500, 1, '', '', '', '', '', '', '', '', 9000, 1, 9000, 1, 25500, 2],
                ['华南', 'Q2', 14000, 1, '', '', 14000, 1, '', '', '', '', '', '', '', '', '', '', '', '', 14000, 1],
                ['', 'Q3', '', '', '', '', '', '', '', '', 3000, 1, 3000, 1, 10000, 1, '', '', 10000, 1, 13000, 2],
                ['总计', '', 63500, 4, 12000, 1, 75500, 5, 2500, 1, 3000, 1, 5500, 2, 18000, 2, 22700, 3, 40700, 5, 121700, 12],
            ]);
            expect(result).toMatchObject(expected);
        });

        it('getCalculatedCellMatrix should include row/column grand totals when multiple column field has showSubTotals and single row field has showSubTotals', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3, true)],
                columnFields: [createColumnField('Category', 1, true), createColumnField('Channel', 2, true)],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });

            const result = engine.getCalculatedCellMatrix();

            const expected = toObjectMatrix([
                ['', '', 'Category', 'Channel', '值', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', ''],
                ['', '', '笔记本电脑', '', '', '', '笔记本电脑 总计', '', '配件', '', '', '', '配件 总计', '', '手机', '', '', '', '手机 总计', '', '总计'],
                ['', '', '线上商城', '', '线下门店', '', '', '', '线上商城', '', '线下门店', '', '', '', '线上商城', '', '线下门店', '', '', ''],
                ['Region', 'Quarter', sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory],
                ['华北', 'Q1', '', '', '', '', '', '', 2500, 1, '', '', 2500, 1, '', '', '', '', '', '', 2500, 1],
                ['', 'Q2', 18000, 1, '', '', 18000, 1, '', '', '', '', '', '', '', '', 6500, 1, 6500, 1, 24500, 2],
                ['', 'Q4', '', '', '', '', '', '', '', '', '', '', '', '', '', '', 7200, 1, 7200, 1, 7200, 1],
                ['华北 总计', '', 18000, 1, '', '', 18000, 1, 2500, 1, '', '', 2500, 1, '', '', 13700, 2, 13700, 2, 34200, 4],
                ['华东', 'Q1', 15000, 1, 12000, 1, 27000, 2, '', '', '', '', '', '', 8000, 1, '', '', 8000, 1, 35000, 3],
                ['', 'Q4', 16500, 1, '', '', 16500, 1, '', '', '', '', '', '', '', '', 9000, 1, 9000, 1, 25500, 2],
                ['华东 总计', '', 31500, 2, 12000, 1, 43500, 3, '', '', '', '', '', '', 8000, 1, 9000, 1, 17000, 2, 60500, 5],
                ['华南', 'Q2', 14000, 1, '', '', 14000, 1, '', '', '', '', '', '', '', '', '', '', '', '', 14000, 1],
                ['', 'Q3', '', '', '', '', '', '', '', '', 3000, 1, 3000, 1, 10000, 1, '', '', 10000, 1, 13000, 2],
                ['华南 总计', '', 14000, 1, '', '', 14000, 1, '', '', 3000, 1, 3000, 1, 10000, 1, '', '', 10000, 1, 27000, 3],
            ]);

            expect(result).toMatchObject(expected);
        });

        it('getCalculatedCellMatrix should include row/column grand totals when multiple row field has showSubTotals and multiple column field has showSubTotals', () => {
            const engine = new PivotEngineV3({
                rowFields: [createRowField('Region', 0, true), createRowField('Quarter', 3, true)],
                columnFields: [createColumnField('Category', 1, true), createColumnField('Channel', 2, true)],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
            });

            const result = engine.getCalculatedCellMatrix();
            const expected = toObjectMatrix([
                ['', '', 'Category', 'Channel', '值', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', '', ''],
                ['', '', '笔记本电脑', '', '', '', '笔记本电脑 总计', '', '配件', '', '', '', '配件 总计', '', '手机', '', '', '', '手机 总计', '', '总计'],
                ['', '', '线上商城', '', '线下门店', '', '', '', '线上商城', '', '线下门店', '', '', '', '线上商城', '', '线下门店', '', '', ''],
                ['Region', 'Quarter', sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory],
                ['华北', 'Q1', '', '', '', '', '', '', 2500, 1, '', '', 2500, 1, '', '', '', '', '', '', 2500, 1],
                ['', 'Q2', 18000, 1, '', '', 18000, 1, '', '', '', '', '', '', '', '', 6500, 1, 6500, 1, 24500, 2],
                ['', 'Q4', '', '', '', '', '', '', '', '', '', '', '', '', '', '', 7200, 1, 7200, 1, 7200, 1],
                ['华北 总计', '', 18000, 1, '', '', 18000, 1, 2500, 1, '', '', 2500, 1, '', '', 13700, 2, 13700, 2, 34200, 4],
                ['华东', 'Q1', 15000, 1, 12000, 1, 27000, 2, '', '', '', '', '', '', 8000, 1, '', '', 8000, 1, 35000, 3],
                ['', 'Q4', 16500, 1, '', '', 16500, 1, '', '', '', '', '', '', '', '', 9000, 1, 9000, 1, 25500, 2],
                ['华东 总计', '', 31500, 2, 12000, 1, 43500, 3, '', '', '', '', '', '', 8000, 1, 9000, 1, 17000, 2, 60500, 5],
                ['华南', 'Q2', 14000, 1, '', '', 14000, 1, '', '', '', '', '', '', '', '', '', '', '', '', 14000, 1],
                ['', 'Q3', '', '', '', '', '', '', '', '', 3000, 1, 3000, 1, 10000, 1, '', '', 10000, 1, 13000, 2],
                ['华南 总计', '', 14000, 1, '', '', 14000, 1, '', '', 3000, 1, 3000, 1, 10000, 1, '', '', 10000, 1, 27000, 3],
                ['总计', '', 63500, 4, 12000, 1, 75500, 5, 2500, 1, 3000, 1, 5500, 2, 18000, 2, 22700, 3, 40700, 5, 121700, 12],
            ]);

            expect(result).toMatchObject(expected);
        });
    });
});
