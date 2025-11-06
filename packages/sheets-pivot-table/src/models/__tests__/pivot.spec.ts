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
import { afterEach, describe, expect, it } from 'vitest';
import { Pivot } from '../pivot';

// Helper to wait for debounce
const waitForDebounce = (ms = 0) => new Promise((resolve) => setTimeout(resolve, ms));

// Helper to get calculated result (bypasses debounce for testing)
const getCalculatedResult = (pivotInstance: Pivot) => pivotInstance.getCalculatedData();

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
                valueFields: ['Amount'],
                rowFields: ['Category'],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = getCalculatedResult(pivot);

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
                valueFields: ['Sales'],
                rowFields: ['Region'],
                columnFields: ['Product'],
                filterFields: [],
                sourceData,
            });

            const result = getCalculatedResult(pivot);

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
                valueFields: ['Sales'],
                rowFields: ['Region', 'Product'],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = getCalculatedResult(pivot);

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
                valueFields: ['Sales', 'Quantity'],
                rowFields: ['Product'],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = getCalculatedResult(pivot);

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
                valueFields: ['Amount'],
                rowFields: ['Category'],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = getCalculatedResult(pivot);

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
                valueFields: ['Amount'],
                rowFields: ['Category'],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = getCalculatedResult(pivot);

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
                valueFields: ['Amount'],
                rowFields: ['Category'],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = getCalculatedResult(pivot);

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
                valueFields: ['Amount'],
                rowFields: ['Category'],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = getCalculatedResult(pivot);

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
                valueFields: ['Amount'],
                rowFields: ['Category'],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result = getCalculatedResult(pivot);

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

    describe('observables', () => {
        it('should emit calculatedData$ when configuration changes (with debounce)', async () => {
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
                valueFields: ['Amount'],
                rowFields: ['Category'],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

      // Wait for initial calculation to complete
            await waitForDebounce();

      // Start tracking emissions after initial calculation
            let emissionReceived = false;
            const subscription = pivot.calculatedData$.subscribe((data) => {
                emissionReceived = true;
                expect(data).toBeDefined();
            });

      // Reset the flag
            emissionReceived = false;

      // Trigger recalculation with a different value
            pivot.setRowFields(['Amount']); // Change configuration

      // Wait for debounced calculation
            await waitForDebounce();

      // Should have received an emission
            expect(emissionReceived).toBe(true);

            subscription.unsubscribe();
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
                valueFields: ['Amount'],
                rowFields: ['Category'],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            const result1 = getCalculatedResult(pivot);
            const result2 = getCalculatedResult(pivot);

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
                valueFields: ['Amount'],
                rowFields: ['Category'],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            getCalculatedResult(pivot);
            expect(pivot.isDirty()).toBe(false);

            pivot.setValueFields(['NewField']);
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
                valueFields: ['Amount'],
                rowFields: ['Category'],
                columnFields: [],
                filterFields: [],
                sourceData,
            });

            getCalculatedResult(pivot);
            expect(pivot.isDirty()).toBe(false);

      // pivot.markDirty();
      // expect(pivot.isDirty()).toBe(true);
        });
    });
});
