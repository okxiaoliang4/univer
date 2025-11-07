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

import type { ICommand, Workbook } from '@univerjs/core';
import type { IFieldsConfig, ISourceRangeInfo, ITargetCellInfo } from '../../types/type';
import { CommandType, ICommandService, IUniverInstanceService } from '@univerjs/core';
import { ISheetsPivotTableService } from '../../services/pivot-table.service';
import { AddPivotTableMutation, SetPivotTableFieldsConfigMutation } from '../mutations/pivot-table.mutation';

/**
 * Command to create a new pivot table
 */
export interface ICreatePivotTableCommandParams {
    unitId: string;
    subUnitId: string;
    name: string;
    sourceRangeInfo: ISourceRangeInfo;
    targetCellInfo: ITargetCellInfo;
    fieldsConfig: IFieldsConfig;
}

export const CreatePivotTableCommand: ICommand<ICreatePivotTableCommandParams> = {
    type: CommandType.COMMAND,
    id: 'sheet.command.create-pivot-table',

    handler: async (accessor, params) => {
        if (!params) {
            return false;
        }

        const pivotTableService = accessor.get(ISheetsPivotTableService);
        const commandService = accessor.get(ICommandService);

        const { unitId, subUnitId, name, sourceRangeInfo, targetCellInfo, fieldsConfig } = params;

        // Create pivot table ID
        const pivotTableId = pivotTableService.createPivotTable(unitId, subUnitId, {
            name,
            sourceRangeInfo,
            targetCellInfo,
            fieldsConfig,
        });

        const config = pivotTableService.getPivotTableConfig(unitId, subUnitId, pivotTableId);
        if (!config) {
            return false;
        }

        // Execute mutation for undo/redo support
        return commandService.executeCommand(AddPivotTableMutation.id, {
            unitId,
            subUnitId,
            pivotTableId,
            config,
        });
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

        const { unitId, subUnitId, pivotTableId, fieldsConfig } = params;

        // Check if pivot table exists
        const existingConfig = pivotTableService.getPivotTableConfig(unitId, subUnitId, pivotTableId);
        if (!existingConfig) {
            return false;
        }

        // Update fields config using fine-grained update
        pivotTableService.updateFieldsConfig(unitId, subUnitId, pivotTableId, fieldsConfig);

        // Execute mutation for undo/redo
        return commandService.executeCommand(SetPivotTableFieldsConfigMutation.id, {
            unitId,
            subUnitId,
            pivotTableId,
            fieldsConfig,
        });
    },
};

/**
 * Command to delete a pivot table
 */
export interface IDeletePivotTableCommandParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
}

export const DeletePivotTableCommand: ICommand<IDeletePivotTableCommandParams> = {
    type: CommandType.COMMAND,
    id: 'sheet.command.delete-pivot-table',

    handler: async (accessor, params) => {
        if (!params) {
            return false;
        }

        const pivotTableService = accessor.get(ISheetsPivotTableService);
        const commandService = accessor.get(ICommandService);

        const { unitId, subUnitId, pivotTableId } = params;

        // Check if pivot table exists
        const config = pivotTableService.getPivotTableConfig(unitId, subUnitId, pivotTableId);
        if (!config) {
            return false;
        }

        // Delete pivot table
        pivotTableService.deletePivotTable(unitId, subUnitId, pivotTableId);

        // Execute remove mutation for undo/redo
        const { RemovePivotTableMutation } = await import('../mutations/pivot-table.mutation');
        return commandService.executeCommand(RemovePivotTableMutation.id, {
            unitId,
            subUnitId,
            pivotTableId,
        });
    },
};

/**
 * Command to refresh (recalculate) a pivot table
 */
export interface IRefreshPivotTableCommandParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
}

export const RefreshPivotTableCommand: ICommand<IRefreshPivotTableCommandParams> = {
    type: CommandType.COMMAND,
    id: 'sheet.command.refresh-pivot-table',

    handler: async (accessor, params) => {
        if (!params) {
            return false;
        }

        const pivotTableService = accessor.get(ISheetsPivotTableService);
        const univerInstanceService = accessor.get(IUniverInstanceService);

        const { unitId, subUnitId, pivotTableId } = params;

        // Mark as dirty to force recalculation
        // pivotTableService.markDirty(unitId, subUnitId, pivotTableId);

        // Get workbook and trigger recalculation
        const workbook = univerInstanceService.getUnit(unitId);
        if (!workbook) {
            return false;
        }

        const pivotTable = pivotTableService.getPivotTable(unitId, subUnitId, pivotTableId);
        if (!pivotTable) {
            return false;
        }

        // Trigger recalculation (actual rendering will be done by render controller)
        pivotTable.calculate(workbook as Workbook);

        return true;
    },
};
