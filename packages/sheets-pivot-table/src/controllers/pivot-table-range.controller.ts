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

import type { Workbook } from '@univerjs/core';
import type { ISetRangeValuesMutationParams } from '@univerjs/sheets';
import { Disposable, ICommandService, Inject, IUniverInstanceService } from '@univerjs/core';
import { IExclusiveRangeService, SetRangeValuesMutation } from '@univerjs/sheets';
import { FEATURE_PIVOT_TABLE_ID } from '../const';
import { SheetsPivotDataSourceModel } from '../models/sheets-pivot-data-source-model';

export class SheetPviotTableRangeController extends Disposable {
    constructor(
        @Inject(SheetsPivotDataSourceModel) private _pivotTableManager: SheetsPivotDataSourceModel,
        @Inject(IExclusiveRangeService) private _exclusiveRangeService: IExclusiveRangeService,
        @Inject(IUniverInstanceService) private _univerInstanceService: IUniverInstanceService,
        @Inject(ICommandService) private _commandService: ICommandService
    ) {
        super();
        this._init();
    }

    private _init() {
        this._initRangeListener();
    }

    private _initRangeListener() {
        this.disposeWithMe(
            this._pivotTableManager.tableRangeChanged$.subscribe((event) => {
                const { range, tableId, unitId, subUnitId } = event;
                this._exclusiveRangeService.clearExclusiveRangesByGroupId(unitId, subUnitId, FEATURE_PIVOT_TABLE_ID, tableId);
                this._exclusiveRangeService.addExclusiveRange(unitId, subUnitId, FEATURE_PIVOT_TABLE_ID, [{
                    range: { ...range },
                    groupId: tableId,
                }]);

                const pivotTable = this._pivotTableManager.getPivotTableInstance(unitId, subUnitId, tableId);
                if (!pivotTable) {
                    return;
                }

        // Calculate pivot table data first
                const workbook = this._univerInstanceService.getUnit(unitId) as Workbook;
                pivotTable.calculate(workbook);

        // Get full cell matrix including headers, values, and totals
                const cellValue = pivotTable.getOutputCellMatrix();
                if (!cellValue) {
                    return;
                }

        // Apply the cell matrix to the worksheet
                this._commandService.executeCommand(SetRangeValuesMutation.id, {
                    unitId,
                    subUnitId,
                    cellValue,
                } satisfies ISetRangeValuesMutationParams);
            })
        );
        this.disposeWithMe(
            this._pivotTableManager.pivotTableAdded$.subscribe((event) => {
                const { pivotTableId, unitId, subUnitId } = event;
                const pivotTable = this._pivotTableManager.getPivotTableInstance(unitId, subUnitId, pivotTableId);
                if (!pivotTable) {
                    return;
                }
                pivotTable.calculate(this._univerInstanceService.getUnit(unitId) as Workbook);

                this._pivotTableManager.notifyRangeChanged(unitId, subUnitId, pivotTableId);

                // Get output range (may be null if not calculated yet)
                const range = pivotTable.getOutputRange();
                if (!range) {
                // If no calculated range yet, use a single cell at target position as placeholder
                    const targetInfo = pivotTable.getTargetCellInfo();
                    const placeholderRange = {
                        startRow: targetInfo.row,
                        startColumn: targetInfo.col,
                        endRow: targetInfo.row,
                        endColumn: targetInfo.col,
                    };
                    this._exclusiveRangeService.addExclusiveRange(unitId, subUnitId, FEATURE_PIVOT_TABLE_ID, [{
                        range: placeholderRange,
                        groupId: pivotTableId,
                    }]);
                    return;
                }
                this._exclusiveRangeService.addExclusiveRange(unitId, subUnitId, FEATURE_PIVOT_TABLE_ID, [{
                    range: { ...range },
                    groupId: pivotTableId,
                }]);
            })
        );
        this.disposeWithMe(
            this._pivotTableManager.pivotTableRemoved$.subscribe((event) => {
                const { pivotTableId, unitId, subUnitId } = event;
        // Simply clear the exclusive range by group ID
        // No need to check pivot table instance as it may already be disposed
                this._exclusiveRangeService.clearExclusiveRangesByGroupId(unitId, subUnitId, FEATURE_PIVOT_TABLE_ID, pivotTableId);
            })
        );
    }
}
