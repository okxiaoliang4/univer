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

import type { ICellData, IRange, Nullable } from '@univerjs/core';
import type { ISetRangeValuesMutationParams } from '@univerjs/sheets';
import type { IFieldsConfig, IPivotTableConfig, ISourceRangeInfo, ITargetCellInfo } from '../types/type';
import { createIdentifier, Disposable, generateRandomId, ICommandService, Inject, ObjectMatrix } from '@univerjs/core';
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
}

export const ISheetsPivotTableService = createIdentifier<ISheetsPivotTableService>('sheets-pivot-table.pivot-table-service');

/**
 * Pivot table service implementation
 * Manages pivot table lifecycle and provides API for operations
 */
export class SheetsPivotTableService extends Disposable implements ISheetsPivotTableService {
    constructor(
        @Inject(SheetsPivotDataSourceModel) private readonly _dataSourceModel: SheetsPivotDataSourceModel,
        @Inject(ICommandService) private readonly _commandService: ICommandService
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

            // 监听output变动，更新数据
            this.disposeWithMe(pivotTable.calculatedData$.pipe(pairwise()).subscribe(([prev, next]) => {
                const updateCellData = new ObjectMatrix<Nullable<ICellData>>({});
                if (prev) {
                    // 将原来的值设置为null
                    new ObjectMatrix(prev).forValue((row, col, value) => {
                        updateCellData.setValue(row, col, {
                            ...value,
                            v: null,
                        });
                    });
                }

                if (next) {
                    // 将新的值覆盖到原来的值
                    new ObjectMatrix(next).forValue((row, col, value) => {
                        updateCellData.setValue(row, col, value);
                    });
                }

                // Apply the cell matrix to the worksheet
                this._commandService.executeCommand(SetRangeValuesMutation.id, {
                    unitId,
                    subUnitId,
                    cellValue: updateCellData.getMatrix(),
                } satisfies ISetRangeValuesMutationParams, {
                    onlyLocal: true, // NOTE: 不记录到协同中，每个用户自己本地计算，如果放开的话会出现undo，redo记录上这个操作
                });
            }));
        }));
    }

    createPivotTable(
        unitId: string,
        subUnitId: string,
        config: Omit<IPivotTableConfig, 'id'>
    ): string {
        const pivotTableId = generateRandomId();
        const fullConfig: IPivotTableConfig = {
            ...config,
            id: pivotTableId,
        };

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
            pivotTable,
            fullConfig
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
}
