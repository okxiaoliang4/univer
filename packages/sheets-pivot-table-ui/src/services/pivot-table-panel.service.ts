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

import { createIdentifier, Disposable } from '@univerjs/core';
import { BehaviorSubject } from 'rxjs';

/**
 * Panel state
 */
export interface IPivotTablePanelState {
    /** Whether the creation dialog is open */
    creationDialogOpen: boolean;
    /** Whether the field configuration panel is open */
    panelOpen: boolean;
    /** Active pivot table info */
    activePivotTable?: {
        unitId: string;
        subUnitId: string;
        pivotTableId: string;
    };
}

/**
 * Service interface for pivot table panel management
 */
export interface ISheetsPivotTablePanelService {
    /**
     * Observable of panel state
     */
    readonly panelState$: BehaviorSubject<IPivotTablePanelState>;

    /**
     * Open the creation dialog
     */
    openCreationDialog(): void;

    /**
     * Close the creation dialog
     */
    closeCreationDialog(): void;

    /**
     * Open the field configuration panel
     */
    openPanel(unitId: string, subUnitId: string, pivotTableId: string): void;

    /**
     * Close the field configuration panel
     */
    closePanel(): void;

    /**
     * Check if creation dialog is open
     */
    isCreationDialogOpen(): boolean;

    /**
     * Check if panel is open
     */
    isPanelOpen(): boolean;

    /**
     * Get active pivot table
     */
    getActivePivotTable(): IPivotTablePanelState['activePivotTable'] | undefined;
}

export const ISheetsPivotTablePanelService = createIdentifier<ISheetsPivotTablePanelService>('sheets-pivot-table-ui.panel-service');

/**
 * Service for managing pivot table panel state
 */
export class SheetsPivotTablePanelService extends Disposable implements ISheetsPivotTablePanelService {
    private readonly _panelState$ = new BehaviorSubject<IPivotTablePanelState>({
        creationDialogOpen: false,
        panelOpen: false,
    });

    readonly panelState$ = this._panelState$;

    openCreationDialog(): void {
        this._panelState$.next({
            ...this._panelState$.value,
            creationDialogOpen: true,
        });
    }

    closeCreationDialog(): void {
        this._panelState$.next({
            ...this._panelState$.value,
            creationDialogOpen: false,
        });
    }

    openPanel(unitId: string, subUnitId: string, pivotTableId: string): void {
        this._panelState$.next({
            ...this._panelState$.value,
            panelOpen: true,
            activePivotTable: {
                unitId,
                subUnitId,
                pivotTableId,
            },
        });
    }

    closePanel(): void {
        this._panelState$.next({
            ...this._panelState$.value,
            panelOpen: false,
            activePivotTable: undefined,
        });
    }

    isCreationDialogOpen(): boolean {
        return this._panelState$.value.creationDialogOpen;
    }

    isPanelOpen(): boolean {
        return this._panelState$.value.panelOpen;
    }

    getActivePivotTable(): IPivotTablePanelState['activePivotTable'] | undefined {
        return this._panelState$.value.activePivotTable;
    }

    override dispose(): void {
        super.dispose();
        this._panelState$.complete();
    }
}
