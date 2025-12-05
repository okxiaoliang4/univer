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

import type { ICommandInfo } from '@univerjs/core';
import type { IClearSelectionContentCommandParams, ISetRangeValuesCommandParams } from '@univerjs/sheets';
import { CustomCommandExecutionError, Disposable, ICommandService, Inject, IUniverInstanceService, LocaleService, ObjectMatrix } from '@univerjs/core';
import { ClearSelectionContentCommand, getSheetCommandTarget, SetRangeValuesCommand } from '@univerjs/sheets';
import { IPivotTableRangeService } from '../services/pivot-table-range.service';
import { ISheetsPivotTableService } from '../services/pivot-table.service';

/**
 * Controller that protects pivot table output ranges from manual editing
 *
 * This controller implements runtime protection for pivot table output cells using command interceptors.
 * Protection is not persisted to snapshot as pivot output is dynamically calculated.
 *
 * Architecture:
 * - Command Interceptor: Blocks core edit commands (SetRangeValues, ClearSelectionContent) before execution
 * - Value and style injection is handled by PivotTableRenderController in @sheets-pivot-table-ui package
 * - Range detection is handled by PivotTableRangeService using spatial indexing
 *
 * UI-related command interception (edit mode, formula bar input) is handled by
 * PivotTablePermissionUIController in @sheets-pivot-table-ui package.
 *
 * The protection automatically:
 * - Applies when pivot tables are created
 * - Updates when pivot ranges change
 * - Removes when pivot tables are deleted
 */
export class PivotTablePermissionController extends Disposable {
    constructor(
        @ICommandService private readonly _commandService: ICommandService,
        @Inject(ISheetsPivotTableService) private readonly _pivotTableService: ISheetsPivotTableService,
        @Inject(IPivotTableRangeService) private readonly _pivotTableRangeService: IPivotTableRangeService,
        @IUniverInstanceService private readonly _univerInstanceService: IUniverInstanceService,
        @Inject(LocaleService) private readonly _localeService: LocaleService
    ) {
        super();
        this._initCommandInterceptor();
    }

    /**
     * Initialize command interceptor to block edit operations on pivot output ranges
     * This prevents both UI and API-driven edits to pivot output cells
     */
    private _initCommandInterceptor(): void {
        this.disposeWithMe(
            this._commandService.beforeCommandExecuted((commandInfo: ICommandInfo) => {
                // Check commands that modify cell values
                if (this._shouldCheckCommand(commandInfo.id)) {
                    const hasPermission = this._checkPermission(commandInfo);
                    if (!hasPermission) {
                        const errorMsg = this._localeService.t('pivot-table.permission.outputProtected') ||
                            'Cannot edit pivot table output. Pivot table cells are calculated and protected from manual editing.';
                        throw new CustomCommandExecutionError(errorMsg);
                    }
                }
            })
        );
    }

    /**
     * Determine if a command should be checked for pivot output protection
     * Only core business logic commands are handled here.
     * UI-related commands are handled by PivotTablePermissionUIController in @sheets-pivot-table-ui
     */
    private _shouldCheckCommand(commandId: string): boolean {
        const protectedCommands = [
            SetRangeValuesCommand.id,
            ClearSelectionContentCommand.id,
            // Add more core edit commands as needed (paste, delete, etc.)
        ];
        return protectedCommands.includes(commandId);
    }

    /**
     * Check if the command targets any pivot output cells
     * @returns true if command is allowed, false if it should be blocked
     */
    private _checkPermission(commandInfo: ICommandInfo): boolean {
        const target = getSheetCommandTarget(this._univerInstanceService);
        if (!target) {
            return true; // No active target, allow command
        }

        const { unitId, subUnitId } = target;

        // Get affected range based on command type
        const affectedRanges = this._getAffectedRanges(commandInfo, unitId, subUnitId);
        if (!affectedRanges || affectedRanges.length === 0) {
            return true; // No ranges affected, allow command
        }

        // Check if any affected cell is in a pivot output range
        for (const range of affectedRanges) {
            for (let row = range.startRow; row <= range.endRow; row++) {
                for (let col = range.startColumn; col <= range.endColumn; col++) {
                    if (this._pivotTableRangeService.isPivotOutputCell(unitId, subUnitId, row, col)) {
                        return false; // Block command - cell is in pivot output
                    }
                }
            }
        }

        return true; // Allow command - no pivot output cells affected
    }

    /**
     * Extract affected cell ranges from command parameters
     */
    private _getAffectedRanges(commandInfo: ICommandInfo, unitId: string, subUnitId: string): Array<{ startRow: number; endRow: number; startColumn: number; endColumn: number }> | null {
        switch (commandInfo.id) {
            case SetRangeValuesCommand.id: {
                const params = commandInfo.params as ISetRangeValuesCommandParams;
                if (params.unitId !== unitId || params.subUnitId !== subUnitId) {
                    return null;
                }
                // If explicit range is provided, use it
                if (params.range) {
                    return [params.range];
                }
                // Otherwise, extract range from value matrix
                if (params.value && typeof params.value === 'object') {
                    // Handle IObjectMatrixPrimitiveType<ICellData>
                    if (!Array.isArray(params.value)) {
                        const matrix = new ObjectMatrix(params.value as Record<string, Record<string, unknown>>);
                        const range = matrix.getDataRange();
                        return range ? [range] : null;
                    }
                    // Handle ICellData[][] - need to determine range from array dimensions
                    // For now, return null as we can't determine range without selection context
                    // The command will be checked at execution time via selection manager
                }
                return null;
            }
            case ClearSelectionContentCommand.id: {
                const params = commandInfo.params as IClearSelectionContentCommandParams | undefined;
                if (!params) {
                    return null;
                }
                if (params.unitId !== unitId || params.subUnitId !== subUnitId) {
                    return null;
                }
                return params.ranges || null;
            }
            default:
                return null;
        }
    }
}
