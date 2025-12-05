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
import { Disposable, ICommandService, Inject, IUniverInstanceService } from '@univerjs/core';
import { IExclusiveRangeService } from '@univerjs/sheets';
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
                // Update exclusive ranges when pivot table range changes
                // Note: Source data is already loaded by PivotTableFormulaController._registerPivotFeature
                // so we don't need to call setSourceDataFromWorkbook here
                this._exclusiveRangeService.clearExclusiveRangesByGroupId(unitId, subUnitId, FEATURE_PIVOT_TABLE_ID, tableId);
                this._exclusiveRangeService.addExclusiveRange(unitId, subUnitId, FEATURE_PIVOT_TABLE_ID, [{
                    range: { ...range },
                    groupId: tableId,
                }]);
            })
        );
        this.disposeWithMe(
            this._pivotTableManager.pivotTableAdded$.subscribe((event) => {
                const { pivotTableId, unitId, subUnitId } = event;
                const pivotTable = this._pivotTableManager.getPivotTableInstance(unitId, subUnitId, pivotTableId);
                if (!pivotTable) {
                    return;
                }

                // Load source data into the pivot table
                // This is essential for:
                // 1. Main thread: PivotTableFormulaController is not initialized when notExecuteFormula: true,
                //    so this is the only place that loads source data for field names to display
                // 2. Worker thread: This loads data first, then PivotTableFormulaController subscribes to calculatedData$
                const workbook = this._univerInstanceService.getUnit(unitId) as Workbook;
                if (workbook) {
                    pivotTable.setSourceDataFromWorkbook(workbook);
                }

                // Set up placeholder exclusive range for immediate protection
                // The actual range will be updated when tableRangeChanged$ emits after calculation
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
