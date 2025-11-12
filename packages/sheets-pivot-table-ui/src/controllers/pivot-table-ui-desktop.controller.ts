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

import type { ISetSelectionsOperationParams } from '@univerjs/sheets';
import type { IRemovePivotTableMutationParams } from '@univerjs/sheets-pivot-table';
import type { IShowPivotTablePanelOperationParams } from '../commands/operations/pivot-table.operation';
import { Disposable, ICommandService, Inject, Injector } from '@univerjs/core';
import { PivotTableIcon } from '@univerjs/icons';
import { SetSelectionsOperation } from '@univerjs/sheets';
import { ISheetsPivotTableService, RemovePivotTableMutation } from '@univerjs/sheets-pivot-table';
import { ComponentManager, IMenuManagerService, IShortcutService } from '@univerjs/ui';
import { HidePivotTablePanelOperation, OpenCreatePivotTableDialogOperation, ShowPivotTablePanelOperation } from '../commands/operations/pivot-table.operation';
import { PivotTablePanel } from '../views/components/PivotTablePanel';
import { menuSchema } from './menu.schema';
import { CreatePivotTableShortcut } from './pivot-table.shortcut';

/**
 * Desktop UI controller for pivot table
 * Coordinates UI interactions and registers UI components
 */
export class PivotTableUIDesktopController extends Disposable {
    constructor(
        @Inject(Injector) private readonly _injector: Injector,
        @ICommandService private readonly _commandService: ICommandService,
        @IMenuManagerService private readonly _menuManagerService: IMenuManagerService,
        @IShortcutService private readonly _shortcutService: IShortcutService,
        @Inject(ComponentManager) private readonly _componentManager: ComponentManager
    ) {
        super();

        this._initCommands();
        this._initComponents();
        this._initMenus();
        this._initShortcuts();
        this._initListeners();
    }

    /**
     * Register UI operations
     */
    private _initCommands(): void {
        [
            OpenCreatePivotTableDialogOperation,
            ShowPivotTablePanelOperation,
            HidePivotTablePanelOperation,
        ].forEach((operation) => {
            this.disposeWithMe(this._commandService.registerCommand(operation));
        });
    }

    private _initComponents(): void {
        ([
            [PivotTablePanel.componentKey, PivotTablePanel],
            ['PivotTableIcon', PivotTableIcon],
        ] as const).forEach(([key, comp]) => {
            this.disposeWithMe(this._componentManager.register(key, comp));
        });
    }

    /**
     * Register menu items
     */
    private _initMenus(): void {
        this._menuManagerService.mergeMenu(menuSchema);
    }

    /**
     * Register keyboard shortcuts
     */
    private _initShortcuts(): void {
        this.disposeWithMe(
            this._shortcutService.registerShortcut(CreatePivotTableShortcut)
        );
    }

    private _initListeners(): void {
        this.disposeWithMe(
            this._commandService.onCommandExecuted((command) => {
                if (command.id === SetSelectionsOperation.id) {
                    const params = command.params as ISetSelectionsOperationParams;
                    const pivotTableService = this._injector.get(ISheetsPivotTableService);
                    const primarySelection = params.selections.find((selection) => selection.primary);
                    if (!primarySelection) {
                        return;
                    }
                    const pivotTable = pivotTableService.getPivotTableByTargetRange(params.unitId, params.subUnitId, primarySelection.range);
                    if (!pivotTable) {
                        return;
                    }
                    this._commandService.executeCommand(ShowPivotTablePanelOperation.id, {
                        unitId: params.unitId,
                        subUnitId: params.subUnitId,
                        pivotTableId: pivotTable.getId(),
                    } satisfies IShowPivotTablePanelOperationParams);
                } else if (command.id === RemovePivotTableMutation.id) {
                    const params = command.params as IRemovePivotTableMutationParams;
                    this._commandService.executeCommand(HidePivotTablePanelOperation.id, {
                        unitId: params.unitId,
                        subUnitId: params.subUnitId,
                        pivotTableId: params.pivotTableId,
                    } satisfies IShowPivotTablePanelOperationParams);
                }
            })
        );
    }
}
