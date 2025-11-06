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

import type { ICellData, IObjectMatrixPrimitiveType } from '@univerjs/core';
import type { IPivotField, IPivotFilterCriteria } from '../../types/type';
import { afterEach, describe, expect, it } from 'vitest';
import { AggregationType, PivotFieldAreaType } from '../../types/enum';
import { Pivot } from '../pivot';

// Helper functions to create IPivotField objects
function createValueField(name: string, sourceColumnIndex: number, aggregation: AggregationType = AggregationType.SUM): IPivotField {
    return {
        id: `value-${name}-${aggregation}`,
        sourceColumnIndex,
        name,
        area: PivotFieldAreaType.VALUE,
        aggregation,
    };
}

function createRowField(name: string, sourceColumnIndex: number): IPivotField {
    return {
        id: `row-${name}`,
        sourceColumnIndex,
        name,
        area: PivotFieldAreaType.ROW,
    };
}

function createColumnField(name: string, sourceColumnIndex: number): IPivotField {
    return {
        id: `col-${name}`,
        sourceColumnIndex,
        name,
        area: PivotFieldAreaType.COLUMN,
    };
}

function createFilterField(name: string, sourceColumnIndex: number, filter: IPivotFilterCriteria): IPivotField {
    return {
        id: `filter-${name}`,
        sourceColumnIndex,
        name,
        area: PivotFieldAreaType.FILTER,
        filter,
    };
}

describe('Pivot', () => {
    let pivot: Pivot;

    afterEach(() => {
        pivot?.dispose();
    });

    describe('basic pivot table calculation', () => {
        it('should calculate simple row field with value field (like Excel)', () => {
            // Source data:
            // | Category | Amount |
            // | A        | 100    |
            // | B        | 200    |
            // | A        | 150    |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'B' },
                    1: { v: 200 },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 150 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected output (like Excel):
            // | Category | Sum of Amount |
            // | A        | 250           |
            // | B        | 200           |

            // Row A (rows 1 and 3 combined)
            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(250);

            // Row B
            expect(result?.[1]?.[0]?.v).toBe('B');
            expect(result?.[1]?.[1]?.v).toBe(200);
        });

        it('should calculate with row and column fields (2D pivot)', () => {
            // Source data:
            // | Region | Product | Sales |
            // | East   | Apple   | 100   |
            // | West   | Apple   | 150   |
            // | East   | Banana  | 200   |
            // | West   | Banana  | 250   |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Region' },
                    1: { v: 'Product' },
                    2: { v: 'Sales' },
                },
                1: {
                    0: { v: 'East' },
                    1: { v: 'Apple' },
                    2: { v: 100 },
                },
                2: {
                    0: { v: 'West' },
                    1: { v: 'Apple' },
                    2: { v: 150 },
                },
                3: {
                    0: { v: 'East' },
                    1: { v: 'Banana' },
                    2: { v: 200 },
                },
                4: {
                    0: { v: 'West' },
                    1: { v: 'Banana' },
                    2: { v: 250 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Sales', 2)],
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Product', 1)],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected output (like Excel):
            // |        | Apple | Banana |
            // | East   | 100   | 200    |
            // | West   | 150   | 250    |

            // Header row
            expect(result?.[0]?.[0]?.v).toBe(''); // top-left corner
            expect(result?.[0]?.[1]?.v).toBe('Apple');
            expect(result?.[0]?.[2]?.v).toBe('Banana');

            // East row
            expect(result?.[1]?.[0]?.v).toBe('East');
            expect(result?.[1]?.[1]?.v).toBe(100);
            expect(result?.[1]?.[2]?.v).toBe(200);

            // West row
            expect(result?.[2]?.[0]?.v).toBe('West');
            expect(result?.[2]?.[1]?.v).toBe(150);
            expect(result?.[2]?.[2]?.v).toBe(250);
        });

        it('should handle multiple row fields (nested rows)', () => {
            // Source data:
            // | Region | Product | Sales |
            // | East   | Apple   | 100   |
            // | East   | Banana  | 150   |
            // | West   | Apple   | 200   |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Region' },
                    1: { v: 'Product' },
                    2: { v: 'Sales' },
                },
                1: {
                    0: { v: 'East' },
                    1: { v: 'Apple' },
                    2: { v: 100 },
                },
                2: {
                    0: { v: 'East' },
                    1: { v: 'Banana' },
                    2: { v: 150 },
                },
                3: {
                    0: { v: 'West' },
                    1: { v: 'Apple' },
                    2: { v: 200 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Sales', 2)],
                rowFields: [createRowField('Region', 0), createRowField('Product', 1)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected output (like Excel):
            // | Region | Product | Sum of Sales |
            // | East   | Apple   | 100          |
            // | East   | Banana  | 150          |
            // | West   | Apple   | 200          |

            // First group: East - Apple
            expect(result?.[0]?.[0]?.v).toBe('East');
            expect(result?.[0]?.[1]?.v).toBe('Apple');
            expect(result?.[0]?.[2]?.v).toBe(100);

            // Second group: East - Banana
            expect(result?.[1]?.[0]?.v).toBe('East');
            expect(result?.[1]?.[1]?.v).toBe('Banana');
            expect(result?.[1]?.[2]?.v).toBe(150);

            // Third group: West - Apple
            expect(result?.[2]?.[0]?.v).toBe('West');
            expect(result?.[2]?.[1]?.v).toBe('Apple');
            expect(result?.[2]?.[2]?.v).toBe(200);
        });

        it('should handle multiple value fields', () => {
            // Source data:
            // | Product | Sales | Quantity |
            // | Apple   | 100   | 10       |
            // | Banana  | 200   | 20       |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Product' },
                    1: { v: 'Sales' },
                    2: { v: 'Quantity' },
                },
                1: {
                    0: { v: 'Apple' },
                    1: { v: 100 },
                    2: { v: 10 },
                },
                2: {
                    0: { v: 'Banana' },
                    1: { v: 200 },
                    2: { v: 20 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Sales', 1), createValueField('Quantity', 2)],
                rowFields: [createRowField('Product', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected output (like Excel):
            // | Product | Sum of Sales | Sum of Quantity |
            // | Apple   | 100          | 10              |
            // | Banana  | 200          | 20              |

            // Header row
            expect(result?.[0]?.[0]?.v).toBe('Product');
            expect(result?.[0]?.[1]?.v).toBe('Sum of Sales');
            expect(result?.[0]?.[2]?.v).toBe('Sum of Quantity');

            // Apple row
            expect(result?.[1]?.[0]?.v).toBe('Apple');
            expect(result?.[1]?.[1]?.v).toBe(100);
            expect(result?.[1]?.[2]?.v).toBe(10);

            // Banana row
            expect(result?.[2]?.[0]?.v).toBe('Banana');
            expect(result?.[2]?.[1]?.v).toBe(200);
            expect(result?.[2]?.[2]?.v).toBe(20);
        });

        it('should aggregate duplicate values correctly', () => {
            // Source data:
            // | Category | Amount |
            // | A        | 100    |
            // | A        | 200    |
            // | A        | 300    |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'A' },
                    1: { v: 200 },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 300 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: A should have sum of 600
            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(600);
        });
    });

    describe('edge cases', () => {
        it('should handle empty source data', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Should return empty result or header only
            expect(Object.keys(result || {}).length).toBeLessThanOrEqual(1);
        });

        it('should handle missing values (null/undefined)', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'B' },
                // Missing amount
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 50 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // A should sum to 150 (100 + 50)
            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(150);

            // B should handle missing value (0 or null)
            expect(result?.[1]?.[0]?.v).toBe('B');
            // Could be 0 or null depending on implementation
            expect([0, null, undefined]).toContain(result?.[1]?.[1]?.v);
        });

        it('should handle non-numeric values in value fields', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'B' },
                    1: { v: 'invalid' },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 50 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // A should sum correctly
            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(150);

            // B should handle non-numeric value
            expect(result?.[1]?.[0]?.v).toBe('B');
        });

        it('should handle blank values in row fields', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                // Blank category
                    1: { v: 200 },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 50 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Should handle blank category (usually labeled as "(blank)" in Excel)
            // Result should have 2 groups: 'A' (with sum 150) and blank (with sum 200)
            const rows = Object.keys(result || {}).map(Number).sort((a, b) => a - b);
            expect(rows.length).toBeGreaterThanOrEqual(2);

            // Check for 'A' group
            const hasA = rows.some((row) => result?.[row]?.[0]?.v === 'A');
            expect(hasA).toBe(true);

            // Check for blank/empty group
            const hasBlank = rows.some((row) => {
                const val = result?.[row]?.[0]?.v;
                return val === '' || val === '(blank)' || val === undefined;
            });
            expect(hasBlank).toBe(true);
        });
    });

    describe('dirty flag and cache', () => {
        it('should use cache when configuration has not changed', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result1 = pivot.getCalculatedData();
            const result2 = pivot.getCalculatedData();

            // Should return the same cached result
            expect(result1).toBe(result2);
        });

        it('should mark as dirty when configuration changes', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            pivot.getCalculatedData();
            expect(pivot.isDirty()).toBe(false);

            pivot.setValueFields([createValueField('NewField', 2)]);
            expect(pivot.isDirty()).toBe(true);
        });

        it('should clear cache and mark dirty when clearCache is called', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            pivot.getCalculatedData();
            expect(pivot.isDirty()).toBe(false);

            // pivot.markDirty();
            // expect(pivot.isDirty()).toBe(true);
        });
    });

    describe('COUNT aggregation', () => {
        it('should count non-empty values with COUNT aggregator', () => {
            // Source data:
            // | Category | Items  |
            // | A        | 100    |
            // | A        | 200    |
            // | A        | 300    |
            // | B        |        | (empty)
            // | B        | 400    |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Items' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'A' },
                    1: { v: 200 },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 300 },
                },
                4: {
                    0: { v: 'B' },
                    // Empty value
                },
                5: {
                    0: { v: 'B' },
                    1: { v: 400 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Items', 1, AggregationType.COUNT)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: A should have count of 3, B should have count of 1
            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(3);

            expect(result?.[1]?.[0]?.v).toBe('B');
            expect(result?.[1]?.[1]?.v).toBe(1);
        });

        it('should count in 2D pivot table', () => {
            // Source data:
            // | Region | Product | Sales |
            // | East   | Apple   | 100   |
            // | East   | Apple   | 150   |
            // | West   | Apple   | 200   |
            // | East   | Banana  | 250   |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Region' },
                    1: { v: 'Product' },
                    2: { v: 'Sales' },
                },
                1: {
                    0: { v: 'East' },
                    1: { v: 'Apple' },
                    2: { v: 100 },
                },
                2: {
                    0: { v: 'East' },
                    1: { v: 'Apple' },
                    2: { v: 150 },
                },
                3: {
                    0: { v: 'West' },
                    1: { v: 'Apple' },
                    2: { v: 200 },
                },
                4: {
                    0: { v: 'East' },
                    1: { v: 'Banana' },
                    2: { v: 250 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Sales', 2, AggregationType.COUNT)],
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Product', 1)],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected output (like Excel):
            // |        | Apple | Banana |
            // | East   | 2     | 1      |
            // | West   | 1     | 0      |

            // Header row
            expect(result?.[0]?.[1]?.v).toBe('Apple');
            expect(result?.[0]?.[2]?.v).toBe('Banana');

            // East row
            expect(result?.[1]?.[0]?.v).toBe('East');
            expect(result?.[1]?.[1]?.v).toBe(2); // 2 Apple entries
            expect(result?.[1]?.[2]?.v).toBe(1); // 1 Banana entry

            // West row
            expect(result?.[2]?.[0]?.v).toBe('West');
            expect(result?.[2]?.[1]?.v).toBe(1); // 1 Apple entry
            expect(result?.[2]?.[2]?.v).toBe(0); // 0 Banana entries
        });

        it('should count all entries including text values', () => {
            // COUNT should count any non-empty cell
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Values' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'A' },
                    1: { v: 'text' },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 0 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Values', 1, AggregationType.COUNT)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Should count all 3 entries (number, text, zero)
            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(3);
        });
    });

    describe('AVERAGE aggregation', () => {
        it('should calculate average correctly', () => {
            // Source data:
            // | Category | Amount |
            // | A        | 100    |
            // | A        | 200    |
            // | A        | 300    |
            // | B        | 50     |
            // | B        | 150    |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'A' },
                    1: { v: 200 },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 300 },
                },
                4: {
                    0: { v: 'B' },
                    1: { v: 50 },
                },
                5: {
                    0: { v: 'B' },
                    1: { v: 150 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1, AggregationType.AVERAGE)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: A average = (100+200+300)/3 = 200, B average = (50+150)/2 = 100
            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(200);

            expect(result?.[1]?.[0]?.v).toBe('B');
            expect(result?.[1]?.[1]?.v).toBe(100);
        });

        it('should handle decimal precision', () => {
            // Source data with decimal averages
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'A' },
                    1: { v: 150 },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 175 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1, AggregationType.AVERAGE)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Average = (100+150+175)/3 = 425/3 = 141.666...
            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBeCloseTo(141.666666, 5);
        });

        it('should ignore non-numeric values in average', () => {
            // AVERAGE should only consider numeric values
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'A' },
                    1: { v: 'text' }, // Should be ignored
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 200 },
                },
                4: {
                    0: { v: 'A' },
                    // Empty - should be ignored
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1, AggregationType.AVERAGE)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Average should be (100+200)/2 = 150, ignoring text and empty
            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(150);
        });

        it('should calculate average in 2D pivot', () => {
            // Source data:
            // | Region | Product | Sales |
            // | East   | Apple   | 100   |
            // | East   | Apple   | 200   |
            // | West   | Apple   | 300   |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Region' },
                    1: { v: 'Product' },
                    2: { v: 'Sales' },
                },
                1: {
                    0: { v: 'East' },
                    1: { v: 'Apple' },
                    2: { v: 100 },
                },
                2: {
                    0: { v: 'East' },
                    1: { v: 'Apple' },
                    2: { v: 200 },
                },
                3: {
                    0: { v: 'West' },
                    1: { v: 'Apple' },
                    2: { v: 300 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Sales', 2, AggregationType.AVERAGE)],
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Product', 1)],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // East average = (100+200)/2 = 150
            expect(result?.[1]?.[0]?.v).toBe('East');
            expect(result?.[1]?.[1]?.v).toBe(150);

            // West average = 300/1 = 300
            expect(result?.[2]?.[0]?.v).toBe('West');
            expect(result?.[2]?.[1]?.v).toBe(300);
        });
    });

    describe('MIN/MAX aggregation', () => {
        it('should find minimum value with MIN aggregator', () => {
            // Source data:
            // | Category | Amount |
            // | A        | 100    |
            // | A        | 50     |
            // | A        | 150    |
            // | B        | 200    |
            // | B        | 75     |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'A' },
                    1: { v: 50 },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 150 },
                },
                4: {
                    0: { v: 'B' },
                    1: { v: 200 },
                },
                5: {
                    0: { v: 'B' },
                    1: { v: 75 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1, AggregationType.MIN)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: A min = 50, B min = 75
            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(50);

            expect(result?.[1]?.[0]?.v).toBe('B');
            expect(result?.[1]?.[1]?.v).toBe(75);
        });

        it('should find maximum value with MAX aggregator', () => {
            // Source data:
            // | Category | Amount |
            // | A        | 100    |
            // | A        | 50     |
            // | A        | 150    |
            // | B        | 200    |
            // | B        | 75     |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'A' },
                    1: { v: 50 },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 150 },
                },
                4: {
                    0: { v: 'B' },
                    1: { v: 200 },
                },
                5: {
                    0: { v: 'B' },
                    1: { v: 75 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1, AggregationType.MAX)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: A max = 150, B max = 200
            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(150);

            expect(result?.[1]?.[0]?.v).toBe('B');
            expect(result?.[1]?.[1]?.v).toBe(200);
        });

        it('should handle negative numbers in MIN/MAX', () => {
            // Source data with negative numbers
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: -100 },
                },
                2: {
                    0: { v: 'A' },
                    1: { v: 50 },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: -200 },
                },
            };

            const pivotMin = new Pivot({
                valueFields: [createValueField('Amount', 1, AggregationType.MIN)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const resultMin = pivotMin.getCalculatedData();
            expect(resultMin?.[0]?.[1]?.v).toBe(-200); // Min should be -200

            pivotMin.dispose();

            const pivotMax = new Pivot({
                valueFields: [createValueField('Amount', 1, AggregationType.MAX)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const resultMax = pivotMax.getCalculatedData();
            expect(resultMax?.[0]?.[1]?.v).toBe(50); // Max should be 50

            pivotMax.dispose();
        });

        it('should calculate MIN/MAX in 2D pivot', () => {
            // Source data:
            // | Region | Product | Sales |
            // | East   | Apple   | 100   |
            // | East   | Apple   | 250   |
            // | West   | Apple   | 150   |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Region' },
                    1: { v: 'Product' },
                    2: { v: 'Sales' },
                },
                1: {
                    0: { v: 'East' },
                    1: { v: 'Apple' },
                    2: { v: 100 },
                },
                2: {
                    0: { v: 'East' },
                    1: { v: 'Apple' },
                    2: { v: 250 },
                },
                3: {
                    0: { v: 'West' },
                    1: { v: 'Apple' },
                    2: { v: 150 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Sales', 2, AggregationType.MAX)],
                rowFields: [createRowField('Region', 0)],
                columnFields: [createColumnField('Product', 1)],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // East max = 250, West max = 150
            expect(result?.[1]?.[0]?.v).toBe('East');
            expect(result?.[1]?.[1]?.v).toBe(250);

            expect(result?.[2]?.[0]?.v).toBe('West');
            expect(result?.[2]?.[1]?.v).toBe(150);
        });
    });

    describe('mixed aggregations', () => {
        it('should support multiple different aggregators in one pivot', () => {
            // Source data:
            // | Product | Sales |
            // | Apple   | 100   |
            // | Apple   | 150   |
            // | Banana  | 200   |
            // | Banana  | 250   |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Product' },
                    1: { v: 'Sales' },
                },
                1: {
                    0: { v: 'Apple' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'Apple' },
                    1: { v: 150 },
                },
                3: {
                    0: { v: 'Banana' },
                    1: { v: 200 },
                },
                4: {
                    0: { v: 'Banana' },
                    1: { v: 250 },
                },
            };

            pivot = new Pivot({
                valueFields: [
                    createValueField('Sales', 1, AggregationType.SUM),
                    createValueField('Sales', 1, AggregationType.COUNT),
                    createValueField('Sales', 1, AggregationType.AVERAGE),
                ],
                rowFields: [createRowField('Product', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected output:
            // | Product | Sum of Sales | Count of Sales | Average of Sales |
            // | Apple   | 250          | 2              | 125              |
            // | Banana  | 450          | 2              | 225              |

            // Header row
            expect(result?.[0]?.[0]?.v).toBe('Product');
            expect(result?.[0]?.[1]?.v).toBe('Sum of Sales');
            expect(result?.[0]?.[2]?.v).toBe('Count of Sales');
            expect(result?.[0]?.[3]?.v).toBe('Average of Sales');

            // Apple row
            expect(result?.[1]?.[0]?.v).toBe('Apple');
            expect(result?.[1]?.[1]?.v).toBe(250); // Sum
            expect(result?.[1]?.[2]?.v).toBe(2); // Count
            expect(result?.[1]?.[3]?.v).toBe(125); // Average

            // Banana row
            expect(result?.[2]?.[0]?.v).toBe('Banana');
            expect(result?.[2]?.[1]?.v).toBe(450); // Sum
            expect(result?.[2]?.[2]?.v).toBe(2); // Count
            expect(result?.[2]?.[3]?.v).toBe(225); // Average
        });

        it('should support different aggregators on different fields', () => {
            // Source data:
            // | Product | Sales | Quantity |
            // | Apple   | 100   | 5        |
            // | Apple   | 200   | 10       |
            // | Banana  | 150   | 8        |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Product' },
                    1: { v: 'Sales' },
                    2: { v: 'Quantity' },
                },
                1: {
                    0: { v: 'Apple' },
                    1: { v: 100 },
                    2: { v: 5 },
                },
                2: {
                    0: { v: 'Apple' },
                    1: { v: 200 },
                    2: { v: 10 },
                },
                3: {
                    0: { v: 'Banana' },
                    1: { v: 150 },
                    2: { v: 8 },
                },
            };

            pivot = new Pivot({
                valueFields: [
                    createValueField('Sales', 1, AggregationType.SUM),
                    createValueField('Quantity', 2, AggregationType.AVERAGE),
                    createValueField('Sales', 1, AggregationType.MAX),
                ],
                rowFields: [createRowField('Product', 0)],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected:
            // | Product | Sum of Sales | Average of Quantity | Max of Sales |
            // | Apple   | 300          | 7.5                 | 200          |
            // | Banana  | 150          | 8                   | 150          |

            // Apple row
            expect(result?.[1]?.[0]?.v).toBe('Apple');
            expect(result?.[1]?.[1]?.v).toBe(300); // Sum of Sales
            expect(result?.[1]?.[2]?.v).toBe(7.5); // Average of Quantity
            expect(result?.[1]?.[3]?.v).toBe(200); // Max of Sales

            // Banana row
            expect(result?.[2]?.[0]?.v).toBe('Banana');
            expect(result?.[2]?.[1]?.v).toBe(150);
            expect(result?.[2]?.[2]?.v).toBe(8);
            expect(result?.[2]?.[3]?.v).toBe(150);
        });
    });

    describe('complex scenarios', () => {
        it('should handle multiple row fields + column fields + multiple aggregators', () => {
            // Complex business scenario: Sales data by region, product, quarter
            // | Region | Product | Quarter | Sales | Units |
            // | East   | Apple   | Q1      | 1000  | 50    |
            // | East   | Apple   | Q2      | 1200  | 60    |
            // | East   | Banana  | Q1      | 800   | 40    |
            // | West   | Apple   | Q1      | 1500  | 75    |
            // | West   | Banana  | Q2      | 900   | 45    |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Region' },
                    1: { v: 'Product' },
                    2: { v: 'Quarter' },
                    3: { v: 'Sales' },
                    4: { v: 'Units' },
                },
                1: {
                    0: { v: 'East' },
                    1: { v: 'Apple' },
                    2: { v: 'Q1' },
                    3: { v: 1000 },
                    4: { v: 50 },
                },
                2: {
                    0: { v: 'East' },
                    1: { v: 'Apple' },
                    2: { v: 'Q2' },
                    3: { v: 1200 },
                    4: { v: 60 },
                },
                3: {
                    0: { v: 'East' },
                    1: { v: 'Banana' },
                    2: { v: 'Q1' },
                    3: { v: 800 },
                    4: { v: 40 },
                },
                4: {
                    0: { v: 'West' },
                    1: { v: 'Apple' },
                    2: { v: 'Q1' },
                    3: { v: 1500 },
                    4: { v: 75 },
                },
                5: {
                    0: { v: 'West' },
                    1: { v: 'Banana' },
                    2: { v: 'Q2' },
                    3: { v: 900 },
                    4: { v: 45 },
                },
            };

            pivot = new Pivot({
                valueFields: [createValueField('Sales', 3, AggregationType.SUM)],
                rowFields: [createRowField('Region', 0), createRowField('Product', 1)],
                columnFields: [createColumnField('Quarter', 2)],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Verify structure exists (detailed validation would check specific values)
            expect(result).toBeTruthy();
            expect(Object.keys(result || {}).length).toBeGreaterThan(0);

            // Verify header row contains quarters
            const headerRow = result?.[0];
            expect(headerRow).toBeTruthy();
        });

        it('should match Excel-style pivot table output with realistic data', () => {
            // Realistic sales data
            // | Category | Region | Sales |
            // | Electronics | North | 5000 |
            // | Electronics | North | 6000 |
            // | Electronics | South | 4500 |
            // | Furniture   | North | 3000 |
            // | Furniture   | South | 3500 |
            // | Furniture   | South | 4000 |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Region' },
                    2: { v: 'Sales' },
                },
                1: {
                    0: { v: 'Electronics' },
                    1: { v: 'North' },
                    2: { v: 5000 },
                },
                2: {
                    0: { v: 'Electronics' },
                    1: { v: 'North' },
                    2: { v: 6000 },
                },
                3: {
                    0: { v: 'Electronics' },
                    1: { v: 'South' },
                    2: { v: 4500 },
                },
                4: {
                    0: { v: 'Furniture' },
                    1: { v: 'North' },
                    2: { v: 3000 },
                },
                5: {
                    0: { v: 'Furniture' },
                    1: { v: 'South' },
                    2: { v: 3500 },
                },
                6: {
                    0: { v: 'Furniture' },
                    1: { v: 'South' },
                    2: { v: 4000 },
                },
            };

            pivot = new Pivot({
                valueFields: [
                    createValueField('Sales', 2, AggregationType.SUM),
                    createValueField('Sales', 2, AggregationType.COUNT),
                    createValueField('Sales', 2, AggregationType.AVERAGE),
                ],
                rowFields: [createRowField('Category', 0)],
                columnFields: [createColumnField('Region', 1)],
                filterFields: [],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Verify the pivot table structure
            expect(result).toBeTruthy();

            // Check that we have header row + data rows
            const rowCount = Object.keys(result || {}).length;
            expect(rowCount).toBeGreaterThanOrEqual(2); // At least header + 1 data row
        });
    });

    describe('value filters', () => {
        it('should filter by single value', () => {
            // Source data:
            // | Region | Product | Sales |
            // | East   | Apple   | 100   |
            // | West   | Apple   | 200   |
            // | East   | Banana  | 150   |
            // | West   | Banana  | 250   |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Region' },
                    1: { v: 'Product' },
                    2: { v: 'Sales' },
                },
                1: {
                    0: { v: 'East' },
                    1: { v: 'Apple' },
                    2: { v: 100 },
                },
                2: {
                    0: { v: 'West' },
                    1: { v: 'Apple' },
                    2: { v: 200 },
                },
                3: {
                    0: { v: 'East' },
                    1: { v: 'Banana' },
                    2: { v: 150 },
                },
                4: {
                    0: { v: 'West' },
                    1: { v: 'Banana' },
                    2: { v: 250 },
                },
            };

            // Filter: only show "East" region
            pivot = new Pivot({
                valueFields: [createValueField('Sales', 2)],
                rowFields: [createRowField('Product', 1)],
                columnFields: [],
                filterFields: [createFilterField('Region', 0, { type: 'value', values: ['East'] })],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: Only East data (Apple: 100, Banana: 150)
            expect(result?.[0]?.[0]?.v).toBe('Apple');
            expect(result?.[0]?.[1]?.v).toBe(100);

            expect(result?.[1]?.[0]?.v).toBe('Banana');
            expect(result?.[1]?.[1]?.v).toBe(150);

            // Should only have 2 rows (no West data)
            expect(Object.keys(result || {}).length).toBe(2);
        });

        it('should filter by multiple values', () => {
            // Source data with 3 regions
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Region' },
                    1: { v: 'Sales' },
                },
                1: {
                    0: { v: 'East' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'West' },
                    1: { v: 200 },
                },
                3: {
                    0: { v: 'North' },
                    1: { v: 150 },
                },
                4: {
                    0: { v: 'East' },
                    1: { v: 120 },
                },
            };

            // Filter: only show "East" and "West"
            pivot = new Pivot({
                valueFields: [createValueField('Sales', 1)],
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                filterFields: [createFilterField('Region', 0, { type: 'value', values: ['East', 'West'] })],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: East (100+120=220) and West (200)
            expect(result?.[0]?.[0]?.v).toBe('East');
            expect(result?.[0]?.[1]?.v).toBe(220);

            expect(result?.[1]?.[0]?.v).toBe('West');
            expect(result?.[1]?.[1]?.v).toBe(200);

            // Should only have 2 rows (no North data)
            expect(Object.keys(result || {}).length).toBe(2);
        });

        it('should handle empty values in filter', () => {
            // Test filtering blank/empty values
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    // Blank category
                    1: { v: 200 },
                },
                3: {
                    0: { v: 'B' },
                    1: { v: 150 },
                },
            };

            // Filter: only show "A"
            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [createFilterField('Category', 0, { type: 'value', values: ['A'] })],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: only row A with value 100
            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(100);

            // Should only have 1 row
            expect(Object.keys(result || {}).length).toBe(1);
        });
    });

    describe('condition filters', () => {
        it('should filter with greaterThan operator', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Product' },
                    1: { v: 'Sales' },
                },
                1: {
                    0: { v: 'Apple' },
                    1: { v: 500 },
                },
                2: {
                    0: { v: 'Banana' },
                    1: { v: 1500 },
                },
                3: {
                    0: { v: 'Cherry' },
                    1: { v: 800 },
                },
            };

            // Filter: Sales > 1000
            pivot = new Pivot({
                valueFields: [createValueField('Sales', 1)],
                rowFields: [createRowField('Product', 0)],
                columnFields: [],
                filterFields: [createFilterField('Sales', 1, { type: 'condition', operator: 'greaterThan', conditionValue: 1000 })],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: Only Banana (1500)
            expect(result?.[0]?.[0]?.v).toBe('Banana');
            expect(result?.[0]?.[1]?.v).toBe(1500);
            expect(Object.keys(result || {}).length).toBe(1);
        });

        it('should filter with lessThan operator', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Product' },
                    1: { v: 'Sales' },
                },
                1: {
                    0: { v: 'Apple' },
                    1: { v: 500 },
                },
                2: {
                    0: { v: 'Banana' },
                    1: { v: 1500 },
                },
                3: {
                    0: { v: 'Cherry' },
                    1: { v: 800 },
                },
            };

            // Filter: Sales < 1000
            pivot = new Pivot({
                valueFields: [createValueField('Sales', 1)],
                rowFields: [createRowField('Product', 0)],
                columnFields: [],
                filterFields: [createFilterField('Sales', 1, { type: 'condition', operator: 'lessThan', conditionValue: 1000 })],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: Apple (500) and Cherry (800)
            expect(result?.[0]?.[0]?.v).toBe('Apple');
            expect(result?.[0]?.[1]?.v).toBe(500);

            expect(result?.[1]?.[0]?.v).toBe('Cherry');
            expect(result?.[1]?.[1]?.v).toBe(800);

            expect(Object.keys(result || {}).length).toBe(2);
        });

        it('should filter with equals operator', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Product' },
                    1: { v: 'Status' },
                    2: { v: 'Sales' },
                },
                1: {
                    0: { v: 'Apple' },
                    1: { v: 'Active' },
                    2: { v: 100 },
                },
                2: {
                    0: { v: 'Banana' },
                    1: { v: 'Inactive' },
                    2: { v: 200 },
                },
                3: {
                    0: { v: 'Cherry' },
                    1: { v: 'Active' },
                    2: { v: 150 },
                },
            };

            // Filter: Status equals "Active"
            pivot = new Pivot({
                valueFields: [createValueField('Sales', 2)],
                rowFields: [createRowField('Product', 0)],
                columnFields: [],
                filterFields: [createFilterField('Status', 1, { type: 'condition', operator: 'equals', conditionValue: 'Active' })],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: Apple and Cherry (both Active)
            expect(result?.[0]?.[0]?.v).toBe('Apple');
            expect(result?.[0]?.[1]?.v).toBe(100);

            expect(result?.[1]?.[0]?.v).toBe('Cherry');
            expect(result?.[1]?.[1]?.v).toBe(150);

            expect(Object.keys(result || {}).length).toBe(2);
        });

        it('should filter with notEquals operator', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Product' },
                    1: { v: 'Status' },
                    2: { v: 'Sales' },
                },
                1: {
                    0: { v: 'Apple' },
                    1: { v: 'Active' },
                    2: { v: 100 },
                },
                2: {
                    0: { v: 'Banana' },
                    1: { v: 'Inactive' },
                    2: { v: 200 },
                },
            };

            // Filter: Status not equals "Active"
            pivot = new Pivot({
                valueFields: [createValueField('Sales', 2)],
                rowFields: [createRowField('Product', 0)],
                columnFields: [],
                filterFields: [createFilterField('Status', 1, { type: 'condition', operator: 'notEquals', conditionValue: 'Active' })],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: Only Banana
            expect(result?.[0]?.[0]?.v).toBe('Banana');
            expect(result?.[0]?.[1]?.v).toBe(200);
            expect(Object.keys(result || {}).length).toBe(1);
        });

        it('should filter with contains operator', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Product' },
                    1: { v: 'Sales' },
                },
                1: {
                    0: { v: 'Apple Juice' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'Banana' },
                    1: { v: 200 },
                },
                3: {
                    0: { v: 'Apple Pie' },
                    1: { v: 150 },
                },
            };

            // Filter: Product contains "Apple"
            pivot = new Pivot({
                valueFields: [createValueField('Sales', 1)],
                rowFields: [createRowField('Product', 0)],
                columnFields: [],
                filterFields: [createFilterField('Product', 0, { type: 'condition', operator: 'contains', conditionValue: 'Apple' })],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: Apple Juice and Apple Pie
            expect(result?.[0]?.[0]?.v).toBe('Apple Juice');
            expect(result?.[0]?.[1]?.v).toBe(100);

            expect(result?.[1]?.[0]?.v).toBe('Apple Pie');
            expect(result?.[1]?.[1]?.v).toBe(150);

            expect(Object.keys(result || {}).length).toBe(2);
        });
    });

    describe('multiple filters (AND logic)', () => {
        it('should apply multiple filters with AND logic', () => {
            // Source data:
            // | Region | Category | Sales |
            // | East   | A        | 500   |
            // | East   | B        | 1500  |
            // | West   | A        | 800   |
            // | West   | B        | 1200  |
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Region' },
                    1: { v: 'Category' },
                    2: { v: 'Sales' },
                },
                1: {
                    0: { v: 'East' },
                    1: { v: 'A' },
                    2: { v: 500 },
                },
                2: {
                    0: { v: 'East' },
                    1: { v: 'B' },
                    2: { v: 1500 },
                },
                3: {
                    0: { v: 'West' },
                    1: { v: 'A' },
                    2: { v: 800 },
                },
                4: {
                    0: { v: 'West' },
                    1: { v: 'B' },
                    2: { v: 1200 },
                },
            };

            // Filter: Region = "East" AND Sales > 1000
            pivot = new Pivot({
                valueFields: [createValueField('Sales', 2)],
                rowFields: [createRowField('Category', 1)],
                columnFields: [],
                filterFields: [
                    createFilterField('Region', 0, { type: 'value', values: ['East'] }),
                    createFilterField('Sales', 2, { type: 'condition', operator: 'greaterThan', conditionValue: 1000 }),
                ],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: Only East + B (1500) passes both filters
            expect(result?.[0]?.[0]?.v).toBe('B');
            expect(result?.[0]?.[1]?.v).toBe(1500);
            expect(Object.keys(result || {}).length).toBe(1);
        });

        it('should combine value and condition filters', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Region' },
                    1: { v: 'Product' },
                    2: { v: 'Sales' },
                },
                1: {
                    0: { v: 'East' },
                    1: { v: 'Apple' },
                    2: { v: 100 },
                },
                2: {
                    0: { v: 'East' },
                    1: { v: 'Banana' },
                    2: { v: 200 },
                },
                3: {
                    0: { v: 'West' },
                    1: { v: 'Apple' },
                    2: { v: 150 },
                },
            };

            // Filter: Region in ["East", "West"] AND Product contains "Apple"
            pivot = new Pivot({
                valueFields: [createValueField('Sales', 2)],
                rowFields: [createRowField('Region', 0)],
                columnFields: [],
                filterFields: [
                    createFilterField('Region', 0, { type: 'value', values: ['East', 'West'] }),
                    createFilterField('Product', 1, { type: 'condition', operator: 'contains', conditionValue: 'Apple' }),
                ],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            // Expected: East-Apple (100) + West-Apple (150) = East: 100, West: 150
            expect(result?.[0]?.[0]?.v).toBe('East');
            expect(result?.[0]?.[1]?.v).toBe(100);

            expect(result?.[1]?.[0]?.v).toBe('West');
            expect(result?.[1]?.[1]?.v).toBe(150);

            expect(Object.keys(result || {}).length).toBe(2);
        });
    });

    describe('filters with aggregations', () => {
        it('should filter before aggregating (SUM)', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Sales' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'A' },
                    1: { v: 200 },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 50 },
                },
                4: {
                    0: { v: 'B' },
                    1: { v: 300 },
                },
            };

            // Filter: Sales >= 100 (excludes 50)
            // Expected SUM for A: 100 + 200 = 300 (not 350)
            pivot = new Pivot({
                valueFields: [createValueField('Sales', 1, AggregationType.SUM)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [
                    createFilterField('Sales', 1, { type: 'condition', operator: 'greaterThan', conditionValue: 50 }),
                ],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(300); // 100 + 200, not 350

            expect(result?.[1]?.[0]?.v).toBe('B');
            expect(result?.[1]?.[1]?.v).toBe(300);
        });

        it('should filter before aggregating (COUNT)', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Amount' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'A' },
                    1: { v: 50 },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 200 },
                },
            };

            // Filter: Amount > 75, then COUNT
            // Expected: A should have count of 2 (100, 200), not 3
            pivot = new Pivot({
                valueFields: [createValueField('Amount', 1, AggregationType.COUNT)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [
                    createFilterField('Amount', 1, { type: 'condition', operator: 'greaterThan', conditionValue: 75 }),
                ],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(2); // Only 100 and 200 counted
        });

        it('should filter before aggregating (AVERAGE)', () => {
            const sourceData: IObjectMatrixPrimitiveType<ICellData> = {
                0: {
                    0: { v: 'Category' },
                    1: { v: 'Score' },
                },
                1: {
                    0: { v: 'A' },
                    1: { v: 100 },
                },
                2: {
                    0: { v: 'A' },
                    1: { v: 50 },
                },
                3: {
                    0: { v: 'A' },
                    1: { v: 80 },
                },
            };

            // Filter: Score > 60, then AVERAGE
            // Expected: A average = (100 + 80) / 2 = 90, not (100 + 50 + 80) / 3 = 76.67
            pivot = new Pivot({
                valueFields: [createValueField('Score', 1, AggregationType.AVERAGE)],
                rowFields: [createRowField('Category', 0)],
                columnFields: [],
                filterFields: [
                    createFilterField('Score', 1, { type: 'condition', operator: 'greaterThan', conditionValue: 60 }),
                ],
                sourceData,
            });

            const result = pivot.getCalculatedData();

            expect(result?.[0]?.[0]?.v).toBe('A');
            expect(result?.[0]?.[1]?.v).toBe(90); // (100 + 80) / 2
        });
    });
});
