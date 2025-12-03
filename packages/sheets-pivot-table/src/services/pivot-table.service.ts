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

import type { ICellData, IRange, Nullable, Workbook } from '@univerjs/core';
import type { ISetRangeValuesMutationParams } from '@univerjs/sheets';
import type { IFieldsConfig, IPivotTableConfig, ISourceRangeInfo, ITargetCellInfo } from '../types/type';
import { createIdentifier, Disposable, generateRandomId, ICommandService, Inject, IUniverInstanceService, ObjectMatrix, Rectangle } from '@univerjs/core';
import { SetRangeValuesMutation } from '@univerjs/sheets';
import { pairwise } from 'rxjs';
import { PivotTable } from '../models/pivot-table';
import { SheetsPivotDataSourceModel } from '../models/sheets-pivot-data-source-model';

/**
 * Resource key for pivot table snapshot
 */
export const SHEET_PIVOT_TABLE_PLUGIN = 'SHEET_PIVOT_TABLE_PLUGIN';

/**
 * Service interface for pivot table management
 */
export interface ISheetsPivotTableService {
    /**
     * Create a new pivot table
     */
    createPivotTable(
        unitId: string,
        subUnitId: string,
        config: Omit<IPivotTableConfig, 'id'>
    ): string;

    /**
     * Get pivot table by ID
     */
    getPivotTable(unitId: string, subUnitId: string, pivotTableId: string): PivotTable | undefined;

    /**
     * Get pivot table config by ID
     */
    getPivotTableConfig(unitId: string, subUnitId: string, pivotTableId: string): IPivotTableConfig | undefined;

    /**
     * Update pivot table source range (atomic operation)
     */
    updateSourceRange(
        unitId: string,
        subUnitId: string,
        pivotTableId: string,
        sourceRangeInfo: ISourceRangeInfo
    ): void;

    /**
     * Update pivot table target cell (atomic operation)
     */
    updateTargetCell(
        unitId: string,
        subUnitId: string,
        pivotTableId: string,
        targetCellInfo: ITargetCellInfo
    ): void;

    /**
     * Update pivot table fields configuration (atomic operation)
     */
    updateFieldsConfig(
        unitId: string,
        subUnitId: string,
        pivotTableId: string,
        fieldsConfig: IFieldsConfig
    ): void;

    /**
     * Delete pivot table
     */
    deletePivotTable(unitId: string, subUnitId: string, pivotTableId: string): void;

    /**
     * Get all pivot tables for a worksheet
     */
    getWorksheetPivotTables(unitId: string, subUnitId: string): Map<string, PivotTable> | undefined;

    /**
     * Get pivot table by target range
     */
    getPivotTableByTargetRange(unitId: string, subUnitId: string, targetRange: IRange): PivotTable | undefined;

    /**
     * Get the data source model
     */
    getDataSourceModel(): SheetsPivotDataSourceModel;

    /**
     * Check if a cell is within any pivot table output range
     * Used by permission system to protect pivot output cells
     * @param unitId - Workbook ID
     * @param subUnitId - Worksheet ID
     * @param row - Cell row index
     * @param col - Cell column index
     * @returns true if cell is in any pivot output range
     */
    isPivotOutputCell(unitId: string, subUnitId: string, row: number, col: number): boolean;

    /**
     * Get pivot table by output cell position
     * Returns the pivot table that contains the specified cell in its output range
     * @param unitId - Workbook ID
     * @param subUnitId - Worksheet ID
     * @param row - Cell row index
     * @param col - Cell column index
     * @returns PivotTable if cell is in output range, undefined otherwise
     */
    getPivotTableByOutputCell(unitId: string, subUnitId: string, row: number, col: number): PivotTable | undefined;
}

export const ISheetsPivotTableService = createIdentifier<ISheetsPivotTableService>('sheets-pivot-table.pivot-table-service');

/**
 * Pivot table service implementation
 * Manages pivot table lifecycle and provides API for operations
 */
export class SheetsPivotTableService extends Disposable implements ISheetsPivotTableService {
    constructor(
        @Inject(SheetsPivotDataSourceModel) private readonly _dataSourceModel: SheetsPivotDataSourceModel,
        @Inject(ICommandService) private readonly _commandService: ICommandService,
        @Inject(IUniverInstanceService) private readonly _univerInstanceService: IUniverInstanceService
    ) {
        super();

        this._initListeners();
    }

    private _initListeners(): void {
        this.disposeWithMe(this._dataSourceModel.pivotTableAdded$.subscribe((event) => {
            const { unitId, subUnitId, pivotTableId } = event;
            const pivotTable = this.getPivotTable(unitId, subUnitId, pivotTableId);
            if (!pivotTable) {
                return;
            }
            this._initPivotTableSourceValueChange(pivotTable);
            this._initPivotTableOutputValueChange(pivotTable);
        }));
    }

    private _initPivotTableOutputValueChange(pivotTable: PivotTable): void {
        // 监听output变动，更新数据
        this.disposeWithMe(pivotTable.calculatedData$.pipe(pairwise()).subscribe(([prev, next]) => {
            const updateCellData = new ObjectMatrix<Nullable<ICellData>>({});
            if (prev) {
                // 将原来的值设置为null
                new ObjectMatrix(prev).forValue((row, col) => {
                    updateCellData.setValue(row, col, null);
                });
            }

            if (next) {
                // 将新的值覆盖到原来的值
                new ObjectMatrix(next).forValue((row, col, value) => {
                    updateCellData.setValue(row, col, value);
                });
            }

            const targetCellInfo = pivotTable.getTargetCellInfo();

            // Apply the cell matrix to the worksheet
            this._commandService.executeCommand(SetRangeValuesMutation.id, {
                unitId: targetCellInfo.unitId,
                subUnitId: targetCellInfo.subUnitId,
                cellValue: updateCellData.getMatrix(),
            } satisfies ISetRangeValuesMutationParams, {
                onlyLocal: true, // NOTE: 不记录到协同中，每个用户自己本地计算，如果放开的话会出现undo，redo记录上这个操作
            });
        }));
    }

    private _initPivotTableSourceValueChange(pivotTable: PivotTable): void {
        this.disposeWithMe(this._commandService.onCommandExecuted((commandInfo) => {
            if (commandInfo.id === SetRangeValuesMutation.id) {
                const params = commandInfo.params as ISetRangeValuesMutationParams;
                const sourceRangeInfo = pivotTable.getSourceRangeInfo();
                if (
                    sourceRangeInfo.unitId !== params.unitId || sourceRangeInfo.subUnitId !== params.subUnitId
                ) {
                    return;
                }
                const matrix = new ObjectMatrix(params.cellValue);
                if (matrix.getSizeOf() <= 0) {
                    return;
                }
                const mutateRange = matrix.getDataRange();
                if (
                    mutateRange && Rectangle.intersects(sourceRangeInfo.range, mutateRange)
                ) {
                    // Calculate pivot table data first
                    const workbook = this._univerInstanceService.getUnit(sourceRangeInfo.unitId) as Workbook;
                    // Set source data from workbook
                    pivotTable.setSourceDataFromWorkbook(workbook);
                }
            }
        }));
    }

    createPivotTable(
        unitId: string,
        subUnitId: string,
        config: Omit<IPivotTableConfig, 'id'>
    ): string {
        const pivotTableId = generateRandomId();

        const pivotTable = new PivotTable(
            pivotTableId,
            config.name,
            config.sourceRangeInfo,
            config.targetCellInfo,
            config.fieldsConfig
        );

        this._dataSourceModel.addPivotTable(
            unitId,
            subUnitId,
            pivotTableId,
            pivotTable
        );

        return pivotTableId;
    }

    getPivotTable(unitId: string, subUnitId: string, pivotTableId: string): PivotTable | undefined {
        return this._dataSourceModel.getPivotTableInstance(unitId, subUnitId, pivotTableId);
    }

    getPivotTableConfig(unitId: string, subUnitId: string, pivotTableId: string): IPivotTableConfig | undefined {
        return this._dataSourceModel.getPivotTableConfig(unitId, subUnitId, pivotTableId);
    }

    updateSourceRange(
        unitId: string,
        subUnitId: string,
        pivotTableId: string,
        sourceRangeInfo: ISourceRangeInfo
    ): void {
        this._dataSourceModel.setSourceRangeInfo(unitId, subUnitId, pivotTableId, sourceRangeInfo);
    }

    updateTargetCell(
        unitId: string,
        subUnitId: string,
        pivotTableId: string,
        targetCellInfo: ITargetCellInfo
    ): void {
        this._dataSourceModel.setTargetCellInfo(unitId, subUnitId, pivotTableId, targetCellInfo);
    }

    updateFieldsConfig(
        unitId: string,
        subUnitId: string,
        pivotTableId: string,
        fieldsConfig: IFieldsConfig
    ): void {
        this._dataSourceModel.setFieldsConfig(unitId, subUnitId, pivotTableId, fieldsConfig);
    }

    deletePivotTable(unitId: string, subUnitId: string, pivotTableId: string): void {
        this._dataSourceModel.removePivotTable(unitId, subUnitId, pivotTableId);
    }

    getWorksheetPivotTables(unitId: string, subUnitId: string): Map<string, PivotTable> | undefined {
        return this._dataSourceModel.getSubUnitPivotTables(unitId, subUnitId);
    }

    getPivotTableByTargetRange(unitId: string, subUnitId: string, targetRange: IRange): PivotTable | undefined {
        return this._dataSourceModel.getPivotTableByTargetRange(unitId, subUnitId, targetRange);
    }

    getDataSourceModel(): SheetsPivotDataSourceModel {
        return this._dataSourceModel;
    }

    /**
     * Check if a cell is within any pivot table output range
     * Used by permission system to protect pivot output cells from manual editing
     */
    isPivotOutputCell(unitId: string, subUnitId: string, row: number, col: number): boolean {
        const pivotTables = this.getWorksheetPivotTables(unitId, subUnitId);
        if (!pivotTables) {
            return false;
        }

        for (const pivotTable of pivotTables.values()) {
            const targetCellInfo = pivotTable.getTargetCellInfo();
            if (targetCellInfo.unitId !== unitId || targetCellInfo.subUnitId !== subUnitId) {
                continue;
            }

            const outputRange = pivotTable.getOutputRange();
            if (outputRange) {
                // Adjust output range to absolute position using target cell
                const absoluteRange = {
                    startRow: outputRange.startRow + targetCellInfo.row,
                    endRow: outputRange.endRow + targetCellInfo.row,
                    startColumn: outputRange.startColumn + targetCellInfo.col,
                    endColumn: outputRange.endColumn + targetCellInfo.col,
                };

                if (Rectangle.contains(absoluteRange, { startRow: row, endRow: row, startColumn: col, endColumn: col })) {
                    return true;
                }
            }
        }

        return false;
    }

    /**
     * Get pivot table by output cell position
     * Returns the first pivot table that contains the specified cell in its output range
     */
    getPivotTableByOutputCell(unitId: string, subUnitId: string, row: number, col: number): PivotTable | undefined {
        const pivotTables = this.getWorksheetPivotTables(unitId, subUnitId);
        if (!pivotTables) {
            return undefined;
        }

        for (const pivotTable of pivotTables.values()) {
            const targetCellInfo = pivotTable.getTargetCellInfo();
            if (targetCellInfo.unitId !== unitId || targetCellInfo.subUnitId !== subUnitId) {
                continue;
            }

            const outputRange = pivotTable.getOutputRange();
            if (outputRange) {
                // Adjust output range to absolute position using target cell
                const absoluteRange = {
                    startRow: outputRange.startRow + targetCellInfo.row,
                    endRow: outputRange.endRow + targetCellInfo.row,
                    startColumn: outputRange.startColumn + targetCellInfo.col,
                    endColumn: outputRange.endColumn + targetCellInfo.col,
                };

                if (Rectangle.contains(absoluteRange, { startRow: row, endRow: row, startColumn: col, endColumn: col })) {
                    return pivotTable;
                }
            }
        }

        return undefined;
    }
}
