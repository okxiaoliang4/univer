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
import type { IShowPivotTablePanelOperationParams } from '../commands/operations/pivot-table.operation';
import { Disposable, ICommandService, Inject, Injector } from '@univerjs/core';
import { SetSelectionsOperation } from '@univerjs/sheets';
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
        this.disposeWithMe(this._componentManager.register(PivotTablePanel.componentKey, PivotTablePanel));
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
                // TODO: 找到targetRange匹配的tableId
                    this._commandService.executeCommand(ShowPivotTablePanelOperation.id, {
                        unitId: params.unitId,
                        subUnitId: params.subUnitId,
                        pivotTableId: 'JzeRdFgNWuU2aPXNeM765',
                    } satisfies IShowPivotTablePanelOperationParams);
                }
            })
        );
    }
}
