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

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ISheetsPivotTableService } from '../../../services/pivot-table.service';
import { AddPivotTableMutation, RemovePivotTableMutation, SetPivotTableFieldsConfigMutation, SetPivotTableSourceRangeMutation, SetPivotTableTargetCellMutation } from '../pivot-table.mutation';

describe('Pivot Table Mutations', () => {
    let mockPivotTableService: any;
    let mockDataSourceModel: any;
    let accessor: any;

    beforeEach(() => {
        mockDataSourceModel = {
            setPivotTableConfig: vi.fn(),
            addPivotTable: vi.fn(),
        };

        mockPivotTableService = {
            getPivotTableConfig: vi.fn(),
            getPivotTable: vi.fn(),
            deletePivotTable: vi.fn(),
            updateSourceRange: vi.fn(),
            updateTargetCell: vi.fn(),
            updateFieldsConfig: vi.fn(),
            getDataSourceModel: vi.fn(() => mockDataSourceModel),
        };

        accessor = {
            get: vi.fn((token) => {
                if (token === ISheetsPivotTableService) {
                    return mockPivotTableService;
                }
                return null;
            }),
        };
    });

    describe('AddPivotTableMutation', () => {
        it('should add a new pivot table when it does not exist', () => {
            mockPivotTableService.getPivotTable.mockReturnValue(null);

            const params = {
                unitId: 'unit1',
                subUnitId: 'subunit1',
                pivotTableId: 'new-pivot-id',
                config: {
                    id: 'new-pivot-id',
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
                },
            };

            const result = AddPivotTableMutation.handler(accessor, params);

            expect(result).toBe(true);
            expect(mockDataSourceModel.addPivotTable).toHaveBeenCalled();
        });

        it('should return false when pivot table already exists', () => {
            mockPivotTableService.getPivotTable.mockReturnValue({});

            const params = {
                unitId: 'unit1',
                subUnitId: 'subunit1',
                pivotTableId: 'existing-pivot-id',
                config: {
                    id: 'existing-pivot-id',
                    name: 'Existing Pivot',
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
                },
            };

            const result = AddPivotTableMutation.handler(accessor, params);

            expect(result).toBe(false);
            expect(mockDataSourceModel.addPivotTable).not.toHaveBeenCalled();
        });

        it('should return false when params are missing', () => {
            const result = AddPivotTableMutation.handler(accessor, undefined as any);

            expect(result).toBe(false);
        });
    });

    describe('RemovePivotTableMutation', () => {
        it('should remove a pivot table', () => {
            mockPivotTableService.getPivotTableConfig.mockReturnValue({
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
                    rowFields: [],
                    columnFields: [],
                    valueFields: [],
                    filterFields: [],
                },
            });

            const params = {
                unitId: 'unit1',
                subUnitId: 'subunit1',
                pivotTableId: 'test-pivot-id',
            };

            const result = RemovePivotTableMutation.handler(accessor, params);

            expect(result).toBe(true);
            expect(mockPivotTableService.deletePivotTable).toHaveBeenCalledWith(
                params.unitId,
                params.subUnitId,
                params.pivotTableId
            );
        });

        it('should store config for undo when removing', () => {
            const config = {
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
                    rowFields: [],
                    columnFields: [],
                    valueFields: [],
                    filterFields: [],
                },
            };

            mockPivotTableService.getPivotTableConfig.mockReturnValue(config);

            const params = {
                unitId: 'unit1',
                subUnitId: 'subunit1',
                pivotTableId: 'test-pivot-id',
            };

            RemovePivotTableMutation.handler(accessor, params);

            // Check that removedConfig is set in params (for undo)
            expect(params).toHaveProperty('removedConfig');
        });

        it('should return false when params are missing', () => {
            const result = RemovePivotTableMutation.handler(accessor, undefined as any);

            expect(result).toBe(false);
        });
    });

    describe('Fine-grained Mutations', () => {
        describe('SetPivotTableSourceRangeMutation', () => {
            it('should update source range and store old value for undo', () => {
                const oldSourceRange = {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    range: { startRow: 0, startColumn: 0, endRow: 10, endColumn: 3 },
                };

                mockPivotTableService.getPivotTableConfig.mockReturnValue({
                    id: 'test-pivot-id',
                    name: 'Test Pivot',
                    sourceRangeInfo: oldSourceRange,
                    targetCellInfo: {
                        unitId: 'unit1',
                        subUnitId: 'subunit1',
                        row: 0,
                        col: 5,
                    },
                    fieldsConfig: {
                        rowFields: [],
                        columnFields: [],
                        valueFields: [],
                        filterFields: [],
                    },
                });

                const newSourceRange = {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    range: { startRow: 0, startColumn: 0, endRow: 20, endColumn: 5 },
                };

                const params = {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    pivotTableId: 'test-pivot-id',
                    sourceRangeInfo: newSourceRange,
                };

                const result = SetPivotTableSourceRangeMutation.handler(accessor, params);

                expect(result).toBe(true);
                expect(mockPivotTableService.updateSourceRange).toHaveBeenCalledWith(
                    params.unitId,
                    params.subUnitId,
                    params.pivotTableId,
                    newSourceRange
                );
                expect(params.oldSourceRangeInfo).toEqual(oldSourceRange);
            });

            it('should return false when pivot table does not exist', () => {
                mockPivotTableService.getPivotTableConfig.mockReturnValue(null);

                const params = {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    pivotTableId: 'non-existent',
                    sourceRangeInfo: {
                        unitId: 'unit1',
                        subUnitId: 'subunit1',
                        range: { startRow: 0, startColumn: 0, endRow: 10, endColumn: 3 },
                    },
                };

                const result = SetPivotTableSourceRangeMutation.handler(accessor, params);

                expect(result).toBe(false);
                expect(mockPivotTableService.updateSourceRange).not.toHaveBeenCalled();
            });

            it('should return false when params are missing', () => {
                const result = SetPivotTableSourceRangeMutation.handler(accessor, undefined as any);

                expect(result).toBe(false);
            });
        });

        describe('SetPivotTableTargetCellMutation', () => {
            it('should update target cell and store old value for undo', () => {
                const oldTargetCell = {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    row: 0,
                    col: 5,
                };

                mockPivotTableService.getPivotTableConfig.mockReturnValue({
                    id: 'test-pivot-id',
                    name: 'Test Pivot',
                    sourceRangeInfo: {
                        unitId: 'unit1',
                        subUnitId: 'subunit1',
                        range: { startRow: 0, startColumn: 0, endRow: 10, endColumn: 3 },
                    },
                    targetCellInfo: oldTargetCell,
                    fieldsConfig: {
                        rowFields: [],
                        columnFields: [],
                        valueFields: [],
                        filterFields: [],
                    },
                });

                const newTargetCell = {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    row: 2,
                    col: 10,
                };

                const params = {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    pivotTableId: 'test-pivot-id',
                    targetCellInfo: newTargetCell,
                };

                const result = SetPivotTableTargetCellMutation.handler(accessor, params);

                expect(result).toBe(true);
                expect(mockPivotTableService.updateTargetCell).toHaveBeenCalledWith(
                    params.unitId,
                    params.subUnitId,
                    params.pivotTableId,
                    newTargetCell
                );
                expect(params.oldTargetCellInfo).toEqual(oldTargetCell);
            });

            it('should return false when pivot table does not exist', () => {
                mockPivotTableService.getPivotTableConfig.mockReturnValue(null);

                const params = {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    pivotTableId: 'non-existent',
                    targetCellInfo: {
                        unitId: 'unit1',
                        subUnitId: 'subunit1',
                        row: 0,
                        col: 5,
                    },
                };

                const result = SetPivotTableTargetCellMutation.handler(accessor, params);

                expect(result).toBe(false);
                expect(mockPivotTableService.updateTargetCell).not.toHaveBeenCalled();
            });

            it('should return false when params are missing', () => {
                const result = SetPivotTableTargetCellMutation.handler(accessor, undefined as any);

                expect(result).toBe(false);
            });
        });

        describe('SetPivotTableFieldsConfigMutation', () => {
            it('should update fields config and store old value for undo', () => {
                const oldFieldsConfig = {
                    rowFields: ['field1'],
                    columnFields: [],
                    valueFields: ['field2'],
                    filterFields: [],
                };

                mockPivotTableService.getPivotTableConfig.mockReturnValue({
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
                    fieldsConfig: oldFieldsConfig,
                });

                const newFieldsConfig = {
                    rowFields: ['field1', 'field3'],
                    columnFields: ['field4'],
                    valueFields: ['field2'],
                    filterFields: [],
                };

                const params = {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    pivotTableId: 'test-pivot-id',
                    fieldsConfig: newFieldsConfig,
                };

                const result = SetPivotTableFieldsConfigMutation.handler(accessor, params);

                expect(result).toBe(true);
                expect(mockPivotTableService.updateFieldsConfig).toHaveBeenCalledWith(
                    params.unitId,
                    params.subUnitId,
                    params.pivotTableId,
                    newFieldsConfig
                );
                expect(params.oldFieldsConfig).toEqual(oldFieldsConfig);
            });

            it('should return false when pivot table does not exist', () => {
                mockPivotTableService.getPivotTableConfig.mockReturnValue(null);

                const params = {
                    unitId: 'unit1',
                    subUnitId: 'subunit1',
                    pivotTableId: 'non-existent',
                    fieldsConfig: {
                        rowFields: [],
                        columnFields: [],
                        valueFields: [],
                        filterFields: [],
                    },
                };

                const result = SetPivotTableFieldsConfigMutation.handler(accessor, params);

                expect(result).toBe(false);
                expect(mockPivotTableService.updateFieldsConfig).not.toHaveBeenCalled();
            });

            it('should return false when params are missing', () => {
                const result = SetPivotTableFieldsConfigMutation.handler(accessor, undefined as any);

                expect(result).toBe(false);
            });
        });
    });
});
