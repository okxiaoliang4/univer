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

import type { ICellDataForSheetInterceptor } from '@univerjs/core';
import { Disposable, Inject, InterceptorEffectEnum } from '@univerjs/core';
import { INTERCEPTOR_POINT, SheetInterceptorService } from '@univerjs/sheets';
import { IPivotTableRangeService, ISheetsPivotTableService } from '@univerjs/sheets-pivot-table';
import { IPivotTableStyleService } from '../services/pivot-table-style.service';

/**
 * Controller that handles pivot table rendering through interceptors
 *
 * This controller injects pivot table values and styles during the cell rendering process.
 * It replaces the functionality that was previously in PivotTablePermissionController.
 *
 * Architecture:
 * - Uses SheetInterceptorService to intercept cell content rendering
 * - Depends on PivotTableRangeService for efficient range detection
 * - Uses PivotTableStyleService for consistent styling
 * - Injects both values and styles with appropriate priority
 */
export class PivotTableRenderController extends Disposable {
    constructor(
        @Inject(SheetInterceptorService) private readonly _sheetInterceptorService: SheetInterceptorService,
        @Inject(ISheetsPivotTableService) private readonly _pivotTableService: ISheetsPivotTableService,
        @Inject(IPivotTableRangeService) private readonly _rangeService: IPivotTableRangeService,
        @Inject(IPivotTableStyleService) private readonly _styleService: IPivotTableStyleService
    ) {
        super();
        this._initCellContentInterceptor();
    }

    /**
     * Initialize cell content interceptor to inject pivot table values and styles
     * This runs during cell rendering to provide dynamic pivot table content
     */
    private _initCellContentInterceptor(): void {
        this.disposeWithMe(
            this._sheetInterceptorService.intercept(INTERCEPTOR_POINT.CELL_CONTENT, {
                // Set priority lower than worksheet/range protection (999) but higher than general styles
                priority: 900,
                effect: InterceptorEffectEnum.Value | InterceptorEffectEnum.Style,
                handler: (cell, context, next) => {
                    const { unitId, subUnitId, row, col } = context;

                    // Check if cell is in any pivot table output range using spatial indexing
                    if (!this._rangeService.isPivotOutputCell(unitId, subUnitId, row, col)) {
                        return next(cell);
                    }

                    // Get pivot table ID for this cell
                    const pivotTableId = this._rangeService.getPivotTableIdByCell(unitId, subUnitId, row, col);
                    if (!pivotTableId) {
                        return next(cell);
                    }

                    // Get pivot table instance
                    const pivotTable = this._pivotTableService.getPivotTable(unitId, subUnitId, pivotTableId);
                    if (!pivotTable) {
                        return next(cell);
                    }

                    // Clone cell data to avoid modifying original
                    const _cellData = ((!cell || cell === context.rawData) ? { ...context.rawData } : cell) as ICellDataForSheetInterceptor;

                    // Inject pivot output value using relative positioning
                    const pivotEngine = pivotTable.getEngine();
                    const outputMatrix = pivotEngine.getOutputMatrix();
                    const targetCellInfo = pivotTable.getTargetCellInfo();

                    // Convert absolute position to relative position in the matrix
                    const relativeRow = row - targetCellInfo.row;
                    const relativeCol = col - targetCellInfo.col;

                    // Get value from relative position
                    const cellValue = outputMatrix?.[relativeRow]?.[relativeCol];

                    // Inject pivot output value (including null, 0, false, empty string as valid values)
                    if (cellValue !== null) {
                        _cellData.v = cellValue?.v;
                        _cellData.t = cellValue?.t;
                        // Copy other cell properties as needed
                        if (cellValue?.s) {
                            _cellData.s = cellValue.s;
                        }
                    } else {
                        // Explicitly set to null if cell value is null
                        _cellData.v = null;
                    }

                    // Inject pivot table style
                    const cellRenderInfo = pivotEngine.determineCellType(relativeRow, relativeCol);

                    const style = this._styleService.getCellStyle(
                        pivotTableId,
                        relativeRow,
                        relativeCol,
                        cellRenderInfo.type,
                        cellRenderInfo.level
                    );
                    if (style) {
                        _cellData.s = { ...(_cellData.s && typeof _cellData.s === 'object' ? _cellData.s : {}), ...style };
                    }

                    return next(_cellData);
                },
            })
        );
    }
}
