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
import type { IInsertSheetCommandParams } from '@univerjs/sheets';
import type { ICreatePivotTableCommandParams, ISourceFields } from '@univerjs/sheets-pivot-table';
import { BooleanNumber, CommandType, generateRandomId, ICommandService, IUniverInstanceService, LocaleService } from '@univerjs/core';
import { serializeRangeWithSpreadsheet } from '@univerjs/engine-formula';
import { expandToContinuousRange, getSheetCommandTarget, InsertSheetCommand, isSingleCellSelection, SheetsSelectionsService } from '@univerjs/sheets';
import { CreatePivotTableCommand, PivotValuePosition } from '@univerjs/sheets-pivot-table';
import { IDialogService, ISidebarService } from '@univerjs/ui';
import { CREATE_PIVOT_TABLE_DIALOG } from '../../const/const';
import { PivotTablePanel } from '../../views/components/PivotTablePanel';

/**
 * Operation to show the pivot table panel
 */
export interface IShowPivotTablePanelOperationParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
}

const PIVOT_TABLE_PANEL_ID = 'pivot-table-panel';

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
            id: PIVOT_TABLE_PANEL_ID,
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
        });
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
        const sidebarService = accessor.get(ISidebarService);
        sidebarService.close(PIVOT_TABLE_PANEL_ID);
        return true;
    },
};

export interface IPivotTableSelectionInfo {
    unitId: string;
    subUnitId: string;
    sourceRange: IRange;
    targetRangeType: 'new' | 'existing';
    targetRange?: IRange;
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
        if (!target) return false;

        const { unitId, subUnitId, worksheet } = target;
        const sheetsSelectionsService = accessor.get(SheetsSelectionsService);

        const lastSelection = sheetsSelectionsService.getCurrentLastSelection();
        const range = lastSelection?.range ?? { startRow: 0, endRow: 0, startColumn: 0, endColumn: 0 };

        const isSingleCell = isSingleCellSelection(lastSelection);
        const extendedRange = isSingleCell ? expandToContinuousRange(range, { up: true, left: true, right: true, down: true }, worksheet) : range;

        const pivotInfo = await openPivotTableDialog(accessor, unitId, subUnitId, extendedRange);
        if (!pivotInfo) return false;

        let targetRange = pivotInfo.targetRange;
        let targetSheetId = pivotInfo.subUnitId;
        if (pivotInfo.targetRangeType === 'new') {
            targetSheetId = generateRandomId();
            const success = await commandService.executeCommand(InsertSheetCommand.id, {
                unitId,
                sheet: {
                    id: targetSheetId,
                    showGridlines: BooleanNumber.FALSE,
                },
            } satisfies IInsertSheetCommandParams);
            if (!success) {
                return false;
            }
            targetRange = {
                startRow: 0,
                endRow: 0,
                startColumn: 0,
                endColumn: 0,
            };
        }

        if (!targetRange) {
            return false;
        }

        const pivotTableId = generateRandomId();
        const fields = (() => {
            const fields: ISourceFields[] = [];
            for (let i = 0; i < pivotInfo.sourceRange.endColumn - pivotInfo.sourceRange.startColumn + 1; i++) {
                fields.push({
                    sourceColumnIndex: i,
                    rangeKey: serializeRangeWithSpreadsheet(unitId, subUnitId, {
                        ...pivotInfo.sourceRange,
                        startColumn: i + pivotInfo.sourceRange.startColumn,
                        endColumn: i + pivotInfo.sourceRange.startColumn,
                    }),
                });
            }
            return fields;
        })();

        await commandService.executeCommand(CreatePivotTableCommand.id, {
            unitId,
            subUnitId: targetSheetId,
            pivotTableId,
            config: {
                name: 'New Pivot Table',
                sourceRangeInfo: {
                    range: pivotInfo.sourceRange,
                    subUnitId,
                    unitId,
                    fields,
                },
                targetCellInfo: {
                    row: targetRange.startRow,
                    col: targetRange.startColumn,
                    subUnitId: targetSheetId,
                    unitId,
                },
                fieldsConfig: {
                    rowFields: [],
                    columnFields: [],
                    valueFields: [],
                    filterFields: [],
                    valuePosition: PivotValuePosition.Column,
                },
            },
        } satisfies ICreatePivotTableCommandParams);
        await commandService.executeCommand(ShowPivotTablePanelOperation.id, {
            unitId,
            subUnitId: targetSheetId,
            pivotTableId,
        }) satisfies IShowPivotTablePanelOperationParams;

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
            targetRangeType: 'new' as const,
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
