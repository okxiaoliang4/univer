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

import type { ICommand, IMutationInfo } from '@univerjs/core';
import type { IFieldsConfig, ISourceRangeInfo, ITargetCellInfo } from '../../types/type';
import type { IAddPivotTableMutationParams, IRemovePivotTableMutationParams, ISetPivotTableFieldsConfigMutationParams, ISetPivotTableSourceRangeMutationParams } from '../mutations/pivot-table.mutation';
import { CommandType, generateRandomId, ICommandService, IUndoRedoService, Rectangle, sequenceExecute } from '@univerjs/core';
import { ISheetsPivotTableService } from '../../services/pivot-table.service';
import { AddPivotTableMutation, RemovePivotTableMutation, SetPivotTableFieldsConfigMutation, SetPivotTableSourceRangeMutation } from '../mutations/pivot-table.mutation';

/**
 * Command to create a new pivot table
 */
export interface ICreatePivotTableCommandParams {
    unitId: string;
    subUnitId: string;
    pivotTableId?: string;
    config: {
        /** Pivot table name */
        name: string;
        /** Source range information */
        sourceRangeInfo: ISourceRangeInfo;
        /** Target cell information */
        targetCellInfo: ITargetCellInfo;
        /** Fields configuration */
        fieldsConfig: IFieldsConfig;
    };
}

export const CreatePivotTableCommand: ICommand<ICreatePivotTableCommandParams> = {
    type: CommandType.COMMAND,
    id: 'sheet.command.create-pivot-table',

    handler: async (accessor, params) => {
        if (!params) {
            return false;
        }

        const commandService = accessor.get(ICommandService);
        const undoRedoService = accessor.get(IUndoRedoService);

        const { unitId, subUnitId, pivotTableId = generateRandomId() } = params;

        const redos: IMutationInfo[] = [];
        const undos: IMutationInfo[] = [];

        redos.push({ id: AddPivotTableMutation.id, params: { ...params, pivotTableId } satisfies IAddPivotTableMutationParams });
        undos.push({ id: RemovePivotTableMutation.id, params: { unitId, subUnitId, pivotTableId } satisfies IRemovePivotTableMutationParams });

        const res = sequenceExecute(redos, commandService);

        if (res) {
            undoRedoService.pushUndoRedo({
                unitID: params.unitId,
                undoMutations: undos,
                redoMutations: redos,
            });
        }
        return true;
    },
};

/**
 * Command to update pivot table fields configuration
 */
export interface IUpdatePivotTableFieldsCommandParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
    fieldsConfig: IFieldsConfig;
}

export const UpdatePivotTableFieldsCommand: ICommand<IUpdatePivotTableFieldsCommandParams> = {
    type: CommandType.COMMAND,
    id: 'sheet.command.update-pivot-table-fields',

    handler: async (accessor, params) => {
        if (!params) {
            return false;
        }

        const pivotTableService = accessor.get(ISheetsPivotTableService);
        const commandService = accessor.get(ICommandService);
        const undoRedoService = accessor.get(IUndoRedoService);

        const { unitId, subUnitId, pivotTableId } = params;

        const currentConfig = pivotTableService.getPivotTableConfig(unitId, subUnitId, pivotTableId);

        const redos: IMutationInfo[] = [];
        const undos: IMutationInfo[] = [];
        redos.push({ id: SetPivotTableFieldsConfigMutation.id, params: { ...params } satisfies ISetPivotTableFieldsConfigMutationParams });
        undos.push({ id: SetPivotTableFieldsConfigMutation.id, params: { unitId, subUnitId, pivotTableId, fieldsConfig: currentConfig?.fieldsConfig || {
            valueFields: [],
            rowFields: [],
            columnFields: [],
            filterFields: [],
        } satisfies IFieldsConfig } satisfies ISetPivotTableFieldsConfigMutationParams });

        const res = sequenceExecute(redos, commandService);

        if (res) {
            undoRedoService.pushUndoRedo({
                unitID: params.unitId,
                undoMutations: undos,
                redoMutations: redos,
            });
        }
        return true;
    },
};

/**
 * Command to remove a pivot table
 */
export interface IRemovePivotTableCommandParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
}

export const RemovePivotTableCommand: ICommand<IRemovePivotTableCommandParams> = {
    type: CommandType.COMMAND,
    id: 'sheet.command.remove-pivot-table',

    handler: async (accessor, params) => {
        if (!params) {
            return false;
        }

        const pivotTableService = accessor.get(ISheetsPivotTableService);
        const commandService = accessor.get(ICommandService);
        const undoRedoService = accessor.get(IUndoRedoService);

        const { unitId, subUnitId, pivotTableId } = params;

        const config = pivotTableService.getPivotTableConfig(unitId, subUnitId, pivotTableId);

        if (!config) {
            throw new Error('[PivotTableService]: Pivot table not found');
        }

        const redos: IMutationInfo[] = [];
        const undos: IMutationInfo[] = [];

        redos.push({ id: RemovePivotTableMutation.id, params: { unitId, subUnitId, pivotTableId } satisfies IRemovePivotTableMutationParams });
        undos.push({ id: AddPivotTableMutation.id, params: { ...params, pivotTableId, config } satisfies IAddPivotTableMutationParams });

        const res = sequenceExecute(redos, commandService);

        if (res) {
            undoRedoService.pushUndoRedo({
                unitID: params.unitId,
                undoMutations: undos,
                redoMutations: redos,
            });
        }
        return true;
    },
};

export interface ISetPivotTableSourceRangeCommandParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
    sourceRangeInfo: ISourceRangeInfo;
}

export const SetPivotTableSourceRangeCommand: ICommand<ISetPivotTableSourceRangeCommandParams> = {
    type: CommandType.COMMAND,
    id: 'sheet.command.set-pivot-table-source-range',

    handler: async (accessor, params) => {
        if (!params) {
            return false;
        }

        const pivotTableService = accessor.get(ISheetsPivotTableService);
        const commandService = accessor.get(ICommandService);
        const undoRedoService = accessor.get(IUndoRedoService);

        const { unitId, subUnitId, pivotTableId, sourceRangeInfo } = params;

        const currentPivotTable = pivotTableService.getPivotTable(unitId, subUnitId, pivotTableId);
        if (!currentPivotTable) {
            throw new Error('[PivotTableService]: Pivot table not found');
        }

        const currentSourceRangeInfo = currentPivotTable.getSourceRangeInfo();

        if (
            currentSourceRangeInfo.unitId === sourceRangeInfo.unitId
          && currentSourceRangeInfo.subUnitId === sourceRangeInfo.subUnitId
          && Rectangle.equals(currentSourceRangeInfo.range, sourceRangeInfo.range)
        ) {
            return true;
        }

        const redos: IMutationInfo[] = [];
        const undos: IMutationInfo[] = [];

        redos.push({ id: SetPivotTableSourceRangeMutation.id, params: { unitId, subUnitId, pivotTableId, sourceRangeInfo } satisfies ISetPivotTableSourceRangeMutationParams });
        undos.push({ id: SetPivotTableSourceRangeMutation.id, params: { unitId, subUnitId, pivotTableId, sourceRangeInfo: currentSourceRangeInfo } satisfies ISetPivotTableSourceRangeMutationParams });

        const res = sequenceExecute(redos, commandService);
        if (res) {
            undoRedoService.pushUndoRedo({
                unitID: params.unitId,
                undoMutations: undos,
                redoMutations: redos,
            });
        }
        return true;
    },

};
