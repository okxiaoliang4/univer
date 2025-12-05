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
import { CustomCommandExecutionError, Disposable, FOCUSING_EDITOR_STANDALONE, ICommandService, IContextService, Inject, IUniverInstanceService, LocaleService } from '@univerjs/core';
import { IMEInputCommand, InsertCommand } from '@univerjs/docs-ui';
import { getSheetCommandTarget, SheetsSelectionsService } from '@univerjs/sheets';
import { IPivotTableRangeService } from '@univerjs/sheets-pivot-table';
import { SetCellEditVisibleOperation, SetCellEditVisibleWithF2Operation } from '@univerjs/sheets-ui';

/**
 * UI Controller that protects pivot table output ranges from UI interactions
 *
 * This controller handles UI-specific command interception for pivot table output protection.
 * It works in conjunction with PivotTablePermissionController in @sheets-pivot-table
 * to provide complete protection at both UI and business logic layers.
 *
 * Responsibilities:
 * - Block entering edit mode (double-click, F2)
 * - Block formula bar input (InsertCommand, IMEInputCommand)
 * - Display user-friendly error messages
 *
 * Business logic layer (@sheets-pivot-table) handles:
 * - Cell content interceptor (read-only metadata injection)
 * - Core edit commands (SetRangeValues, ClearSelectionContent)
 */
export class PivotTablePermissionUIController extends Disposable {
    constructor(
        @ICommandService private readonly _commandService: ICommandService,
        @Inject(IPivotTableRangeService) private readonly _pivotTableRangeService: IPivotTableRangeService,
        @IUniverInstanceService private readonly _univerInstanceService: IUniverInstanceService,
        @Inject(LocaleService) private readonly _localeService: LocaleService,
        @Inject(SheetsSelectionsService) private readonly _selectionsService: SheetsSelectionsService,
        @IContextService private readonly _contextService: IContextService
    ) {
        super();
        this._initUICommandInterceptor();
    }

    /**
     * Initialize UI command interceptor to block UI interactions with pivot output cells
     * This prevents users from entering edit mode or typing in formula bar
     */
    private _initUICommandInterceptor(): void {
        this.disposeWithMe(
            this._commandService.beforeCommandExecuted((commandInfo: ICommandInfo) => {
                // Check UI-related commands
                if (this._shouldCheckUICommand(commandInfo.id)) {
                    const hasPermission = this._checkUIPermission(commandInfo);
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
     * Determine if a UI command should be checked for pivot output protection
     */
    private _shouldCheckUICommand(commandId: string): boolean {
        const uiCommands = [
            SetCellEditVisibleOperation.id, // SetCellEditVisibleOperation
            SetCellEditVisibleWithF2Operation.id, // SetCellEditVisibleWithF2Operation
            InsertCommand.id, // InsertTextCommand - prevent direct text input
            IMEInputCommand.id, // IMEInputCommand - prevent IME input in formula bar
        ];
        return uiCommands.includes(commandId);
    }

    /**
     * Check if the UI command targets any pivot output cells
     * @returns true if command is allowed, false if it should be blocked
     */
    private _checkUIPermission(commandInfo: ICommandInfo): boolean {
        const target = getSheetCommandTarget(this._univerInstanceService);
        if (!target) {
            return true; // No active target, allow command
        }

        const { unitId, subUnitId } = target;

        // Handle SetCellEditVisibleOperation - check current selection
        if (commandInfo.id === SetCellEditVisibleOperation.id || commandInfo.id === SetCellEditVisibleWithF2Operation.id) {
            const params = commandInfo.params as { visible?: boolean };
            // Only block if trying to show editor (visible === true or undefined)
            if (params.visible === false) {
                return true; // Allow hiding editor
            }
            // Get current selection to determine which cell is being edited
            const selections = this._selectionsService.getCurrentSelections();
            if (selections && selections.length > 0) {
                const primarySelection = selections[0];
                const { startRow, startColumn } = primarySelection.range;
                // Check if the selected cell is in a pivot output range
                if (this._pivotTableRangeService.isPivotOutputCell(unitId, subUnitId, startRow, startColumn)) {
                    return false; // Block entering edit mode
                }
            }
            return true; // Allow if no selection or cell is not in pivot output
        }

        // Handle InsertCommand and IMEInputCommand - prevent input in formula bar for pivot output cells
        if (commandInfo.id === 'doc.command.insert'
            || commandInfo.id === 'doc.command.insert-text'
            || commandInfo.id === 'doc.command.ime-input') {
            // Skip check if user is editing in standalone editor (not sheet cell)
            if (this._contextService.getContextValue(FOCUSING_EDITOR_STANDALONE) === true) {
                return true;
            }
            // Get current selection to determine which cell is being edited
            const selections = this._selectionsService.getCurrentSelections();
            if (selections && selections.length > 0) {
                const primarySelection = selections[0];
                const { startRow, startColumn } = primarySelection.range;
                // Check if the selected cell is in a pivot output range
                if (this._pivotTableRangeService.isPivotOutputCell(unitId, subUnitId, startRow, startColumn)) {
                    return false; // Block input
                }
            }
            return true; // Allow if no selection or cell is not in pivot output
        }

        return true; // Allow other commands
    }
}
