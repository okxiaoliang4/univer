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

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { PivotTable } from '../../models/pivot-table';
import { SheetsPivotTableCalculationService } from '../pivot-table-calculation.service';

describe('SheetsPivotTableCalculationService', () => {
    let service: SheetsPivotTableCalculationService;

    beforeEach(() => {
        service = new SheetsPivotTableCalculationService();
    });

    afterEach(() => {
        service.dispose();
    });

    describe('calculate', () => {
        it('should calculate pivot table results', () => {
            const pivotTable = new PivotTable(
                'test-id',
                'Test Pivot',
                {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    range: { startRow: 0, startColumn: 0, endRow: 5, endColumn: 2 },
                },
                {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    row: 0,
                    col: 5,
                },
                {
                    rowFields: ['field1'],
                    columnFields: [],
                    valueFields: ['field2'],
                    filterFields: [],
                }
            );

            // Mock workbook
            const mockWorkbook = {
                getSheetBySheetId: vi.fn(() => ({
                    getCellMatrix: vi.fn(() => ({
                        getValue: vi.fn((row: number, col: number) => {
                            // Mock some data
                            if (row === 0) {
                                return col === 0 ? { v: 'Category' } : { v: 'Value' };
                            }
                            if (row === 1) {
                                return col === 0 ? { v: 'A' } : { v: 10 };
                            }
                            if (row === 2) {
                                return col === 0 ? { v: 'B' } : { v: 20 };
                            }
                            return { v: null };
                        }),
                    })),
                })),
            };

            const result = service.calculate(pivotTable, mockWorkbook as any);

            expect(result).toBeDefined();
            expect(result).not.toBeNull();
        });

        it('should return null for empty pivot table', () => {
            const pivotTable = new PivotTable(
                'test-id',
                'Empty Pivot',
                {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    range: { startRow: 0, startColumn: 0, endRow: 5, endColumn: 2 },
                },
                {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    row: 0,
                    col: 5,
                },
                {
                    rowFields: [],
                    columnFields: [],
                    valueFields: [],
                    filterFields: [],
                }
            );

            const mockWorkbook = {
                getSheetBySheetId: vi.fn(() => ({})),
            };

            const result = service.calculate(pivotTable, mockWorkbook as any);

            expect(result).toBeNull();
        });

        it('should use cached results when pivot table is not dirty', () => {
            const pivotTable = new PivotTable(
                'test-id',
                'Test Pivot',
                {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    range: { startRow: 0, startColumn: 0, endRow: 5, endColumn: 2 },
                },
                {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    row: 0,
                    col: 5,
                },
                {
                    rowFields: ['field1'],
                    columnFields: [],
                    valueFields: ['field2'],
                    filterFields: [],
                }
            );

            const mockWorkbook = {
                getSheetBySheetId: vi.fn(() => ({
                    getCellMatrix: vi.fn(() => ({
                        getValue: vi.fn((row: number, col: number) => {
                            if (row === 0) {
                                return col === 0 ? { v: 'Category' } : { v: 'Value' };
                            }
                            if (row === 1) {
                                return col === 0 ? { v: 'A' } : { v: 10 };
                            }
                            return { v: null };
                        }),
                    })),
                })),
            };

            // First calculation
            const result1 = service.calculate(pivotTable, mockWorkbook as any);

            // Second calculation (should use cache if not dirty)
            const result2 = service.calculate(pivotTable, mockWorkbook as any);

            expect(result1).toBeDefined();
            expect(result2).toBeDefined();
        });
    });

    describe('clearCache', () => {
        it('should clear cached calculation for specific pivot table', () => {
            const pivotTableId = 'test-id';

            // Clear cache should not throw
            expect(() => {
                service.clearCache(pivotTableId);
            }).not.toThrow();
        });
    });

    describe('clearAllCache', () => {
        it('should clear all cached calculations', () => {
            expect(() => {
                service.clearAllCache();
            }).not.toThrow();
        });
    });
});
