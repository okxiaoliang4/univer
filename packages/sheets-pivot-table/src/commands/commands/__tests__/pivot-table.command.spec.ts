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

import { ICommandService, IUniverInstanceService } from '@univerjs/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ISheetsPivotTableService } from '../../../services/pivot-table.service';
import { CreatePivotTableCommand, DeletePivotTableCommand, RefreshPivotTableCommand, UpdatePivotTableFieldsCommand } from '../pivot-table.command';

describe('Pivot Table Commands', () => {
    let mockPivotTableService: any;
    let mockCommandService: any;
    let mockUniversInstanceService: any;
    let accessor: any;

    beforeEach(() => {
        mockPivotTableService = {
            createPivotTable: vi.fn(() => 'test-pivot-id'),
            getPivotTableConfig: vi.fn(() => ({
                id: 'test-pivot-id',
                name: 'Test Pivot',
                sourceRangeInfo: {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    range: { startRow: 0, startColumn: 0, endRow: 10, endColumn: 3 },
                },
                targetCellInfo: {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    row: 0,
                    col: 5,
                },
                fieldsConfig: {
                    rowFields: ['field1'],
                    columnFields: [],
                    valueFields: ['field2'],
                    filterFields: [],
                },
            })),
            updateFieldsConfig: vi.fn(),
            deletePivotTable: vi.fn(),
            markDirty: vi.fn(),
            getPivotTable: vi.fn(() => ({
                calculate: vi.fn(),
            })),
        };

        mockCommandService = {
            executeCommand: vi.fn(() => Promise.resolve(true)),
        };

        mockUniversInstanceService = {
            getUnit: vi.fn(() => ({
                getUnitId: () => 'unit1',
            })),
        };

        accessor = {
            get: vi.fn((token) => {
                if (token === ISheetsPivotTableService) {
                    return mockPivotTableService;
                }
                if (token === ICommandService) {
                    return mockCommandService;
                }
                if (token === IUniverInstanceService) {
                    return mockUniversInstanceService;
                }
                return null;
            }),
        };
    });

    describe('CreatePivotTableCommand', () => {
        it('should create a new pivot table', async () => {
            const params = {
                unitId: 'unit1',
                subUnitId: 'subunit1',
                name: 'New Pivot',
                sourceRangeInfo: {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    range: { startRow: 0, startColumn: 0, endRow: 10, endColumn: 3 },
                },
                targetCellInfo: {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    row: 0,
                    col: 5,
                },
                fieldsConfig: {
                    rowFields: ['field1'],
                    columnFields: [],
                    valueFields: ['field2'],
                    filterFields: [],
                },
            };

            const result = await CreatePivotTableCommand.handler(accessor, params);

            expect(result).toBe(true);
            expect(mockPivotTableService.createPivotTable).toHaveBeenCalledWith(
                params.unitId,
                params.subUnitId,
                expect.objectContaining({
                    name: params.name,
                    sourceRangeInfo: params.sourceRangeInfo,
                    targetCellInfo: params.targetCellInfo,
                    fieldsConfig: params.fieldsConfig,
                })
            );
        });

        it('should return false when params are missing', async () => {
            const result = await CreatePivotTableCommand.handler(accessor, undefined as any);

            expect(result).toBe(false);
            expect(mockPivotTableService.createPivotTable).not.toHaveBeenCalled();
        });
    });

    describe('UpdatePivotTableFieldsCommand', () => {
        it('should update pivot table fields configuration', async () => {
            const params = {
                unitId: 'unit1',
                subUnitId: 'subunit1',
                pivotTableId: 'test-pivot-id',
                fieldsConfig: {
                    rowFields: ['field1', 'field3'],
                    columnFields: ['field4'],
                    valueFields: ['field2'],
                    filterFields: [],
                },
            };

            const result = await UpdatePivotTableFieldsCommand.handler(accessor, params);

            expect(result).toBe(true);
            expect(mockPivotTableService.updateFieldsConfig).toHaveBeenCalledWith(
                params.unitId,
                params.subUnitId,
                params.pivotTableId,
                params.fieldsConfig
            );
        });

        it('should return false when pivot table does not exist', async () => {
            mockPivotTableService.getPivotTableConfig.mockReturnValue(null);

            const params = {
                unitId: 'unit1',
                subUnitId: 'subunit1',
                pivotTableId: 'non-existent-id',
                fieldsConfig: {
                    rowFields: [],
                    columnFields: [],
                    valueFields: [],
                    filterFields: [],
                },
            };

            const result = await UpdatePivotTableFieldsCommand.handler(accessor, params);

            expect(result).toBe(false);
        });
    });

    describe('DeletePivotTableCommand', () => {
        it('should delete a pivot table', async () => {
            const params = {
                unitId: 'unit1',
                subUnitId: 'subunit1',
                pivotTableId: 'test-pivot-id',
            };

            const result = await DeletePivotTableCommand.handler(accessor, params);

            expect(result).toBe(true);
            expect(mockPivotTableService.deletePivotTable).toHaveBeenCalledWith(
                params.unitId,
                params.subUnitId,
                params.pivotTableId
            );
        });

        it('should return false when pivot table does not exist', async () => {
            mockPivotTableService.getPivotTableConfig.mockReturnValue(null);

            const params = {
                unitId: 'unit1',
                subUnitId: 'subunit1',
                pivotTableId: 'non-existent-id',
            };

            const result = await DeletePivotTableCommand.handler(accessor, params);

            expect(result).toBe(false);
        });
    });

    describe('RefreshPivotTableCommand', () => {
        it('should refresh a pivot table', async () => {
            const params = {
                unitId: 'unit1',
                subUnitId: 'subunit1',
                pivotTableId: 'test-pivot-id',
            };

            const result = await RefreshPivotTableCommand.handler(accessor, params);

            expect(result).toBe(true);
            expect(mockPivotTableService.markDirty).toHaveBeenCalledWith(
                params.unitId,
                params.subUnitId,
                params.pivotTableId
            );
        });

        it('should return false when workbook does not exist', async () => {
            mockUniversInstanceService.getUnit.mockReturnValue(null);

            const params = {
                unitId: 'unit1',
                subUnitId: 'subunit1',
                pivotTableId: 'test-pivot-id',
            };

            const result = await RefreshPivotTableCommand.handler(accessor, params);

            expect(result).toBe(false);
        });

        it('should return false when pivot table does not exist', async () => {
            mockPivotTableService.getPivotTable.mockReturnValue(null);

            const params = {
                unitId: 'unit1',
                subUnitId: 'subunit1',
                pivotTableId: 'non-existent-id',
            };

            const result = await RefreshPivotTableCommand.handler(accessor, params);

            expect(result).toBe(false);
        });
    });
});
