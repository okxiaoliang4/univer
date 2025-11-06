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

import type { IAccessor, IOperation, IRange } from '@univerjs/core';
import type { ICreatePivotTableCommandParams } from '@univerjs/sheets-pivot-table';
import { CommandType, ICommandService, IUniverInstanceService, LocaleService } from '@univerjs/core';
import { expandToContinuousRange, getSheetCommandTarget, isSingleCellSelection, SheetsSelectionsService } from '@univerjs/sheets';
import { CreatePivotTableCommand } from '@univerjs/sheets-pivot-table';
import { IDialogService, ISidebarService } from '@univerjs/ui';
import { CREATE_PIVOT_TABLE_DIALOG } from '../../const/const';
import { ISheetsPivotTablePanelService } from '../../services/pivot-table-panel.service';
import { PivotTablePanel } from '../../views/components/PivotTablePanel';

export interface IPivotTableSelectionInfo {
    unitId: string;
    subUnitId: string;
    sourceRange: IRange;
    targetRange: IRange;
}

/**
 * Operation to open the create pivot table dialog
 */
export const OpenCreatePivotTableDialogOperation: IOperation<IPivotTableSelectionInfo, Promise<boolean>> = {
    type: CommandType.OPERATION,
    id: 'sheet.operation.open-create-pivot-table-dialog',

    async handler(accessor: IAccessor): Promise<boolean> {
        const univerInstanceService = accessor.get(IUniverInstanceService);
        const commandService = accessor.get(ICommandService);
        const target = getSheetCommandTarget(univerInstanceService);
        if (!target) {
            return false;
        }

        const { unitId, subUnitId, worksheet } = target;
        const sheetsSelectionsService = accessor.get(SheetsSelectionsService);

        const lastSelection = sheetsSelectionsService.getCurrentLastSelection();
        const range = lastSelection?.range ?? { startRow: 0, endRow: 0, startColumn: 0, endColumn: 0 };

        const isSingleCell = isSingleCellSelection(lastSelection);
        const extendedRange = isSingleCell ? expandToContinuousRange(range, { up: true, left: true, right: true, down: true }, worksheet) : range;

        const pivotInfo = await openPivotTableDialog(accessor, unitId, subUnitId, extendedRange);
        if (!pivotInfo) {
            return false;
        }

        commandService.executeCommand(CreatePivotTableCommand.id, {
            unitId,
            subUnitId,
            name: 'New Pivot Table',
            sourceRangeInfo: {
                range: pivotInfo.sourceRange,
                subUnitId,
                unitId,
            },
            targetCellInfo: {
                row: pivotInfo.targetRange.startRow,
                col: pivotInfo.targetRange.startColumn,
                subUnitId,
                unitId,
            },
            fieldsConfig: {
                rowFields: [],
                columnFields: [],
                valueFields: [],
                filterFields: [],
            },
        } satisfies ICreatePivotTableCommandParams);

        return true;
    },
};

export async function openPivotTableDialog(
    accessor: IAccessor,
    unitId: string,
    subUnitId: string,
    sourceRange: IRange
): Promise<IPivotTableSelectionInfo | null> {
    const dialogService = accessor.get(IDialogService);
    const localeService = accessor.get(LocaleService);

    return new Promise((resolve) => {
        const dialogProps = {
            unitId,
            subUnitId,
            sourceRange,
            targetRange: { startRow: 0, endRow: 0, startColumn: 0, endColumn: 0 },
            onConfirm: (info: IPivotTableSelectionInfo) => {
                resolve(info);
                dialogService.close(CREATE_PIVOT_TABLE_DIALOG);
            },
            onCancel: () => {
                resolve(null);
                dialogService.close(CREATE_PIVOT_TABLE_DIALOG);
            },
        };

        dialogService.open({
            id: CREATE_PIVOT_TABLE_DIALOG,
            title: { title: localeService.t('pivotTable.dialog.createTitle') },
            draggable: true,
            destroyOnClose: true,
            mask: false,
            maskClosable: false,
            children: {
                label: {
                    name: CREATE_PIVOT_TABLE_DIALOG,
                    props: dialogProps,
                },
            },
            width: 400,
            onClose: () => {
                resolve(null);
                dialogService.close(CREATE_PIVOT_TABLE_DIALOG);
            },
        });
    });
}

/**
 * Operation to show the pivot table panel
 */
export interface IShowPivotTablePanelOperationParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
}

export const ShowPivotTablePanelOperation: IOperation<IShowPivotTablePanelOperationParams> = {
    type: CommandType.OPERATION,
    id: 'sheet.operation.show-pivot-table-panel',

    handler: (accessor, params) => {
        const { unitId, subUnitId, pivotTableId } = params;
        if (!params) {
            return false;
        }

        const sidebarService = accessor.get(ISidebarService);
        sidebarService.open({
            id: 'pivot-table-panel',
            header: { title: 'Pivot Table Panel' },
            children: {
                label: {
                    name: PivotTablePanel.componentKey,
                    props: {
                        unitId,
                        subUnitId,
                        pivotTableId,
                    },
                },
            },
            width: 400,
            onClose: () => {
                sidebarService.close('pivot-table-panel');
            },
        });
        // const panelService = accessor.get(ISheetsPivotTablePanelService);
        // panelService.openPanel(params.unitId, params.subUnitId, params.pivotTableId);
        return true;
    },
};

/**
 * Operation to hide the pivot table panel
 */
export const HidePivotTablePanelOperation: IOperation = {
    type: CommandType.OPERATION,
    id: 'sheet.operation.hide-pivot-table-panel',

    handler: (accessor) => {
        const panelService = accessor.get(ISheetsPivotTablePanelService);
        panelService.closePanel();
        return true;
    },
};
