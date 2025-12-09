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
import { AggregationType, PivotValuePosition } from '../../types/enum';
import { PivotEngineV2 } from '../pivot-engine-v2';

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
        expect(PivotEngineV2).toBeDefined();
    });

    describe('should empty result', () => {
        it('should empty result', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
            });
            const result = engine.getCalculatedCellMatrix();
            expect(result).toEqual(defaultPlaceholderMatrix);
        });

        it('should not empty result rowFields', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
            });
            const result = engine.getCalculatedCellMatrix();
            expect(result).not.toEqual(defaultPlaceholderMatrix);
        });

        it('should not empty result columnFields', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 3)],
                valueFields: [],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
            });
            const result = engine.getCalculatedCellMatrix();
            expect(result).not.toEqual(defaultPlaceholderMatrix);
        });

        it('should not empty result with valueFields', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
            });
            const result = engine.getCalculatedCellMatrix();
            expect(result).not.toEqual(defaultPlaceholderMatrix);
        });

        it('should empty result with filter', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [],
                filterFields: [createFilterField('Region', 0, { type: 'value', values: ['华北'] })],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
            });
            const result = engine.getCalculatedCellMatrix();
            expect(result).toEqual(defaultPlaceholderMatrix);
        });
    });

    describe('should only valueFields', () => {
        it('should single valueFields in column position', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
            });
            const output = toObjectMatrix([
                [sumOfSales],
                [121700],
            ]);
            const result = engine.getCalculatedCellMatrix();
            expect(result).toEqual(output);
        });

        it('should multiple valueFields in column position', () => {
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [],
                valueFields: [
                    createValueField('Sales', 4),
                    createValueField('Category', 1, AggregationType.COUNT),
                ],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
            });
            const result = engine.getCalculatedCellMatrix();

            const output = toObjectMatrix([
                [sumOfSales, countOfCategory],
                [121700, 2],
            ]);
            expect(result).toEqual(output);
        });
    });

    describe('should combine rowFields and valueFields', () => {
        it('should single rowFields and single valueFields in column position', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
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
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
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
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
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
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3)],
                columnFields: [],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
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
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 3)],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
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
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 3), createColumnField('Region', 0)],
                valueFields: [createValueField('Sales', 4)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
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
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [createColumnField('Region', 0)],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
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
            const engine = new PivotEngineV2({
                rowFields: [],
                columnFields: [createColumnField('Quarter', 3), createColumnField('Region', 0)],
                valueFields: [createValueField('Sales', 4), createValueField('Category', 1, AggregationType.COUNT)],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
            });
            const result = engine.getCalculatedCellMatrix();
            const output = toObjectMatrix([
                ['Quarter', 'Region', '值', '', '', '', '', '', '', '', '', '', ''],
                ['Q1', '', '', '', 'Q2', '', '', '', 'Q3', 'Q4', '', '', ''],
                ['华北', '', '华东', '', '华北', '', '华南', '', '华南', '华北', '', '华东', ''],
                [sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory, sumOfSales, countOfCategory],
                [2500, 1, 35000, 3, 24500, 2, 14000, 1, 13000, 2, 7200, 1, 25500, 2],
            ]);

            expect(result).toEqual(output);
        });
    });

    describe('should combine rowFields and columnFields', () => {
        it('should single rowFields and single columnFields in column position', () => {
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Quarter', 3)],
                valueFields: [],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
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
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3)],
                columnFields: [createColumnField('Category', 1)],
                valueFields: [],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
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
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Category', 1), createColumnField('Quarter', 3)],
                valueFields: [],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
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
            const engine = new PivotEngineV2({
                rowFields: [createRowField('Region', 0), createRowField('Quarter', 3)],
                columnFields: [createColumnField('Category', 1), createColumnField('Channel', 2)],
                valueFields: [],
                filterFields: [],
                sourceData,
                valuePosition: PivotValuePosition.COLUMN,
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
});
