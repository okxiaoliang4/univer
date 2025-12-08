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

import type { IPivotTableConfigResource } from '../types/type';
import { Disposable, IResourceManagerService, IUniverInstanceService, UniverInstanceType } from '@univerjs/core';
import { ISheetsPivotTableService, SHEET_PIVOT_TABLE_PLUGIN } from '../services/pivot-table.service';

/**
 * Pivot table controller
 * Coordinates pivot table lifecycle and handles workbook/worksheet events
 */
export class SheetPivotTableController extends Disposable {
    constructor(
        @IUniverInstanceService private readonly _univerInstanceService: IUniverInstanceService,
        @IResourceManagerService private readonly _resourceManagerService: IResourceManagerService,
        @ISheetsPivotTableService private readonly _pivotTableService: ISheetsPivotTableService
    ) {
        super();

        this._initSnapshot();
        this._initWorkbookListeners();
    }

  /**
   * Initialize snapshot handlers for save/load
   */
    private _initSnapshot(): void {
        const toJson = (unitId: string) => {
            const dataSourceModel = this._pivotTableService.getDataSourceModel();
            if (dataSourceModel) {
                return JSON.stringify(dataSourceModel.toJSON(unitId));
            }
            return '';
        };
        const parseJson = (json: string): IPivotTableConfigResource => {
            if (!json) {
                return { pivotTableConfigs: {}, pivotData: {} };
            }
            try {
                return JSON.parse(json);
            } catch {
                return { pivotTableConfigs: {}, pivotData: {} };
            }
        };

        this.disposeWithMe(
            this._resourceManagerService.registerPluginResource<IPivotTableConfigResource>({
                pluginName: SHEET_PIVOT_TABLE_PLUGIN,
                businesses: [UniverInstanceType.UNIVER_SHEET],
                toJson,
                parseJson,
                onUnLoad: (unitID) => {
                    const dataSourceModel = this._pivotTableService.getDataSourceModel();
                    dataSourceModel.deleteUnitId(unitID);
                },
                onLoad: (_unitID, resource) => {
                    const dataSourceModel = this._pivotTableService.getDataSourceModel();
                    dataSourceModel.fromJSON(resource);
                },
            })
        );
    }

  /**
   * Initialize listeners for workbook and worksheet events
   */
    private _initWorkbookListeners(): void {
    // Listen for workbook disposal
        this.disposeWithMe(
            this._univerInstanceService.getTypeOfUnitDisposed$(UniverInstanceType.UNIVER_SHEET).subscribe((workbook) => {
                const unitId = workbook.getUnitId();
                const dataSourceModel = this._pivotTableService.getDataSourceModel();
                dataSourceModel.deleteUnitId(unitId);
            })
        );

    // TODO: Add listeners for cell data changes to mark pivot tables dirty
    // This will be implemented when we add reactive updates
    }
}
