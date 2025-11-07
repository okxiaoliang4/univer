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
import type { IFieldsConfig, IPivotTableConfig, IPivotTableConfigResource, IPivotTableFieldsConfigChangedEvent, IPivotTableRangeChangedEvent, IPivotTableSourceRangeChangedEvent, IPivotTableTargetCellChangedEvent, ISourceRangeInfo, ITargetCellInfo } from '../types/type';
import { Disposable, ICommandService, IResourceManagerService, IUniverInstanceService, ObjectMatrix, Rectangle, toDisposable } from '@univerjs/core';
import { SetRangeValuesMutation } from '@univerjs/sheets';
import { Subject } from 'rxjs';
import { PivotTable } from './pivot-table';

/**
 * Simplified data source model for pivot tables (MVP version)
 * Manages pivot table instances and their lifecycle
 */
export class SheetsPivotDataSourceModel extends Disposable {
    private _pivotTableMap: Map<string, Map<string, Map<string, PivotTable>>> = new Map();

    private _pivotTableAdded$ = new Subject<{ unitId: string; subUnitId: string; pivotTableId: string }>();
    public readonly pivotTableAdded$ = this._pivotTableAdded$.asObservable();

    private _pivotTableRemoved$ = new Subject<{ unitId: string; subUnitId: string; pivotTableId: string }>();
    public readonly pivotTableRemoved$ = this._pivotTableRemoved$.asObservable();

    private _pivotTableUpdated$ = new Subject<{ unitId: string; subUnitId: string; pivotTableId: string }>();
    public readonly pivotTableUpdated$ = this._pivotTableUpdated$.asObservable();

    private _tableRangeChanged$ = new Subject<IPivotTableRangeChangedEvent>();
    public readonly tableRangeChanged$ = this._tableRangeChanged$.asObservable();

    private _sourceRangeChanged$ = new Subject<IPivotTableSourceRangeChangedEvent>();
    public readonly sourceRangeChanged$ = this._sourceRangeChanged$.asObservable();

    private _targetCellChanged$ = new Subject<IPivotTableTargetCellChangedEvent>();
    public readonly targetCellChanged$ = this._targetCellChanged$.asObservable();

    private _fieldsConfigChanged$ = new Subject<IPivotTableFieldsConfigChangedEvent>();
    public readonly fieldsConfigChanged$ = this._fieldsConfigChanged$.asObservable();

    constructor(
        @IUniverInstanceService private readonly _univerInstanceService: IUniverInstanceService,
        @IResourceManagerService private readonly _resourceManagerService: IResourceManagerService,
        @ICommandService private readonly _commandService: ICommandService
    ) {
        super();

        this.disposeWithMe(
            toDisposable(() => {
                this._pivotTableMap.clear();
            })
        );
    }

    /**
     * Add a pivot table
     */
    addPivotTable(
        unitId: string,
        subUnitId: string,
        pivotTableId: string,
        pivotTable: PivotTable,
        config: IPivotTableConfig
    ): void {
        this._ensureMaps(unitId, subUnitId);

        this._pivotTableMap.get(unitId)?.get(subUnitId)?.set(pivotTableId, pivotTable);

        this._pivotTableAdded$.next({ unitId, subUnitId, pivotTableId });
        this.disposeWithMe(this._commandService.onCommandExecuted((commandInfo) => {
            if (commandInfo.id === SetRangeValuesMutation.id) {
                const params = commandInfo.params as ISetRangeValuesMutationParams;
                const sourceRangeInfo = pivotTable.getSourceRangeInfo();
                if (
                    sourceRangeInfo.unitId !== params.unitId ||
                  sourceRangeInfo.subUnitId !== params.subUnitId
                ) {
                    return;
                }
                const matrix = new ObjectMatrix(params.cellValue);
                if (matrix.getSizeOf() <= 0) {
                    return;
                }
                const mutateRange = matrix.getDataRange();
                if (
                    mutateRange &&
                  Rectangle.intersects(sourceRangeInfo.range, mutateRange)
                ) {
                  // Calculate pivot table data first
                    const workbook = this._univerInstanceService.getUnit(unitId) as Workbook;
                  // 获取当前的输出单元格矩阵
                    const outputCellMatrix = pivotTable.getOutputCellMatrix();
                    pivotTable.calculate(workbook);

                  // Get full cell matrix including headers, values, and totals
                    const cellValue = pivotTable.getOutputCellMatrix();

                    const updateCellData = new ObjectMatrix<Nullable<ICellData>>({});

                    if (outputCellMatrix) {
                    // 将原来的值设置为null
                        new ObjectMatrix(outputCellMatrix).forValue((row, col, value) => {
                            updateCellData.setValue(row, col, {
                                ...value,
                                v: null,
                            });
                        });
                    }

                    if (cellValue) {
                    // 将新的值覆盖到原来的值
                        new ObjectMatrix(cellValue).forValue((row, col, value) => {
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
                }
            }
        }));
    }

    /**
     * Remove a pivot table
     */
    removePivotTable(unitId: string, subUnitId: string, pivotTableId: string): void {
        const pivotTable = this.getPivotTableInstance(unitId, subUnitId, pivotTableId);
        if (pivotTable) {
            pivotTable.dispose();
        }

        this._pivotTableMap.get(unitId)?.get(subUnitId)?.delete(pivotTableId);

        this._pivotTableRemoved$.next({ unitId, subUnitId, pivotTableId });
    }

    /**
     * Get pivot table instance
     */
    getPivotTableInstance(unitId: string, subUnitId: string, pivotTableId: string): PivotTable | undefined {
        return this._pivotTableMap.get(unitId)?.get(subUnitId)?.get(pivotTableId);
    }

    /**
     * Get pivot table config
     */
    getPivotTableConfig(unitId: string, subUnitId: string, pivotTableId: string): IPivotTableConfig | undefined {
        return this._pivotTableMap.get(unitId)?.get(subUnitId)?.get(pivotTableId)?.toJSON();
    }

    /**
     * Set source range info for a pivot table
     */
    setSourceRangeInfo(
        unitId: string,
        subUnitId: string,
        pivotTableId: string,
        sourceRangeInfo: ISourceRangeInfo
    ): void {
        const oldConfig = this.getPivotTableConfig(unitId, subUnitId, pivotTableId);
        if (!oldConfig) {
            throw new Error(`Pivot table ${pivotTableId} not found`);
        }

        const pivotTable = this.getPivotTableInstance(unitId, subUnitId, pivotTableId);
        if (!pivotTable) {
            return;
        }

        // Check if actually changed
        const changed = (
            oldConfig.sourceRangeInfo.range.startRow !== sourceRangeInfo.range.startRow ||
            oldConfig.sourceRangeInfo.range.startColumn !== sourceRangeInfo.range.startColumn ||
            oldConfig.sourceRangeInfo.range.endRow !== sourceRangeInfo.range.endRow ||
            oldConfig.sourceRangeInfo.range.endColumn !== sourceRangeInfo.range.endColumn ||
            oldConfig.sourceRangeInfo.unitId !== sourceRangeInfo.unitId ||
            oldConfig.sourceRangeInfo.subUnitId !== sourceRangeInfo.subUnitId
        );

        if (!changed) {
            return;
        }

        // Update pivot table instance
        pivotTable.setSourceRangeInfo(sourceRangeInfo);

        // Emit events
        this._sourceRangeChanged$.next({
            unitId,
            subUnitId,
            tableId: pivotTableId,
            sourceRangeInfo,
            oldSourceRangeInfo: oldConfig.sourceRangeInfo,
        });

        // Emit range change event (output range may have changed)
        const outputRange = pivotTable.getOutputRange();
        if (outputRange) {
            this._tableRangeChanged$.next({
                unitId,
                subUnitId,
                tableId: pivotTableId,
                range: outputRange,
            });
        }

        this._pivotTableUpdated$.next({ unitId, subUnitId, pivotTableId });
    }

    /**
     * Set target cell info for a pivot table
     */
    setTargetCellInfo(
        unitId: string,
        subUnitId: string,
        pivotTableId: string,
        targetCellInfo: ITargetCellInfo
    ): void {
        const pivotTable = this.getPivotTableInstance(unitId, subUnitId, pivotTableId);
        if (!pivotTable) {
            return;
        }

        // Update pivot table instance
        pivotTable.setTargetCellInfo(targetCellInfo);

        // Emit events
        this._targetCellChanged$.next({
            unitId,
            subUnitId,
            tableId: pivotTableId,
            targetCellInfo,
        });

        // Emit range change event (output range position changed)
        const outputRange = pivotTable.getOutputRange();
        if (outputRange) {
            this._tableRangeChanged$.next({
                unitId,
                subUnitId,
                tableId: pivotTableId,
                range: outputRange,
            });
        }

        this._pivotTableUpdated$.next({ unitId, subUnitId, pivotTableId });
    }

    /**
     * Set fields config for a pivot table
     */
    setFieldsConfig(
        unitId: string,
        subUnitId: string,
        pivotTableId: string,
        fieldsConfig: IFieldsConfig
    ): void {
        const pivotTable = this.getPivotTableInstance(unitId, subUnitId, pivotTableId);
        if (!pivotTable) {
            return;
        }

        // Update pivot table instance
        pivotTable.setFieldsConfig(fieldsConfig);

        // Emit event
        this._fieldsConfigChanged$.next({
            unitId,
            subUnitId,
            tableId: pivotTableId,
            fieldsConfig,
        });

        this._pivotTableUpdated$.next({ unitId, subUnitId, pivotTableId });
    }

    /**
     * Notify that a pivot table's output range has changed
     * Should be called after calculation completes
     */
    notifyRangeChanged(unitId: string, subUnitId: string, pivotTableId: string): void {
        const pivotTable = this.getPivotTableInstance(unitId, subUnitId, pivotTableId);
        if (pivotTable) {
            const outputRange = pivotTable.getOutputRange();
            if (outputRange) {
                this._tableRangeChanged$.next({
                    unitId,
                    subUnitId,
                    tableId: pivotTableId,
                    range: outputRange,
                });
            }
        }
    }

    /**
     * Get all pivot tables in a subunit
     */
    getSubUnitPivotTables(unitId: string, subUnitId: string): Map<string, PivotTable> | undefined {
        return this._pivotTableMap.get(unitId)?.get(subUnitId);
    }

    /**
     * Get pivot table by target range
     */
    getPivotTableByTargetRange(unitId: string, subUnitId: string, targetRange: IRange): PivotTable | undefined {
        const pivotTables = this.getSubUnitPivotTables(unitId, subUnitId);
        if (!pivotTables) {
            return undefined;
        }

        return Array.from(pivotTables.values()).find((pivotTable) => {
            const outputRange = pivotTable.getOutputRange();
            if (!outputRange) {
                return false;
            }
            return Rectangle.intersects(outputRange, targetRange);
        });
    }

    /**
     * Delete all pivot tables for a unit
     */
    deleteUnitId(unitId: string): void {
        const unitMap = this._pivotTableMap.get(unitId);
        if (unitMap) {
            unitMap.forEach((subUnitMap) => {
                subUnitMap.forEach((pivotTable) => {
                    pivotTable.dispose();
                });
            });
        }

        this._pivotTableMap.delete(unitId);
    }

    /**
     * Serialize to JSON
     */
    toJSON(unitId: string): IPivotTableConfigResource {
        const result: IPivotTableConfigResource = {
            pivotTableConfigs: {},
        };

        const unitConfigMap = this._pivotTableMap.get(unitId);
        if (!unitConfigMap) {
            return result;
        }

        const unitResult: Record<string, Record<string, IPivotTableConfig>> = {};

        unitConfigMap.forEach((subUnitMap, subUnitId) => {
            const subUnitResult: Record<string, IPivotTableConfig> = {};

            subUnitMap.forEach((pivotTable, pivotTableId) => {
                subUnitResult[pivotTableId] = pivotTable.toJSON();
            });

            unitResult[subUnitId] = subUnitResult;
        });

        result.pivotTableConfigs[unitId] = unitResult;

        return result;
    }

    /**
     * Deserialize from JSON
     */
    fromJSON(data: IPivotTableConfigResource): void {
        const pivotConfigs = data.pivotTableConfigs;

        Object.keys(pivotConfigs).forEach((unitId) => {
            const unitData = pivotConfigs[unitId];

            Object.keys(unitData).forEach((subUnitId) => {
                const subUnitData = unitData[subUnitId];

                Object.keys(subUnitData).forEach((pivotTableId) => {
                    const config = subUnitData[pivotTableId];

                    // Create PivotTable instance
                    const pivotTable = new PivotTable(
                        pivotTableId,
                        `Pivot_${pivotTableId}`,
                        config.sourceRangeInfo,
                        config.targetCellInfo,
                        config.fieldsConfig
                    );

                    this.addPivotTable(unitId, subUnitId, pivotTableId, pivotTable, config);
                });
            });
        });
    }

    private _ensureMaps(unitId: string, subUnitId: string): void {
        if (!this._pivotTableMap.has(unitId)) {
            this._pivotTableMap.set(unitId, new Map());
        }
        if (!this._pivotTableMap.get(unitId)!.has(subUnitId)) {
            this._pivotTableMap.get(unitId)!.set(subUnitId, new Map());
        }
    }
}
