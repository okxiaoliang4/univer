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

import type { ICellData, IMutation, IObjectMatrixPrimitiveType, Nullable } from '@univerjs/core';
import type { IUniverSheetsPivotTableConfig } from '../../controllers/config.schema';
import type { PivotValuePosition } from '../../types/enum';
import type { IFieldsConfig, ISourceRangeInfo, ITargetCellInfo } from '../../types/type';
import { CommandType, IConfigService } from '@univerjs/core';
import { SHEETS_PIVOT_TABLE_PLUGIN_CONFIG_KEY } from '../../controllers/config.schema';
import { PivotTable } from '../../models/pivot-table';
import { SheetsPivotDataSourceModel } from '../../models/sheets-pivot-data-source-model';
import { ISheetsPivotTableService } from '../../services/pivot-table.service';

/**
 * Mutation to add a new pivot table
 */
export interface IAddPivotTableMutationParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
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

export const AddPivotTableMutation: IMutation<IAddPivotTableMutationParams> = {
    type: CommandType.MUTATION,
    id: 'sheet.mutation.add-pivot-table',

    handler: (accessor, params) => {
        if (!params) {
            return false;
        }

        const pivotTableService = accessor.get(ISheetsPivotTableService);
        const configService = accessor.get(IConfigService);
        const dataSourceModel = pivotTableService.getDataSourceModel();

        const { unitId, subUnitId, pivotTableId, config } = params;

        // Check if auto-calculation should be skipped (RPC environment)
        const pluginConfig = configService.getConfig<IUniverSheetsPivotTableConfig>(SHEETS_PIVOT_TABLE_PLUGIN_CONFIG_KEY);
        const skipAutoCalculation = pluginConfig?.notExecuteFormula ?? false;

        // Create new pivot table with skipAutoCalculation option
        // In RPC environment, main thread skips auto-calculation
        // and receives calculated data from Worker via mutation
        const pivotTable = new PivotTable(
            pivotTableId,
            config.name,
            config.sourceRangeInfo,
            config.targetCellInfo,
            config.fieldsConfig,
            { skipAutoCalculation }
        );

        dataSourceModel.addPivotTable(unitId, subUnitId, pivotTableId, pivotTable);

        return true;
    },
};

/**
 * Mutation to remove a pivot table
 */
export interface IRemovePivotTableMutationParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
}

export const RemovePivotTableMutation: IMutation<IRemovePivotTableMutationParams> = {
    type: CommandType.MUTATION,
    id: 'sheet.mutation.remove-pivot-table',

    handler: (accessor, params) => {
        if (!params) {
            return false;
        }

        const pivotTableService = accessor.get(ISheetsPivotTableService);

        const { unitId, subUnitId, pivotTableId } = params;

        // Remove pivot table
        pivotTableService.deletePivotTable(unitId, subUnitId, pivotTableId);

        return true;
    },
};

/**
 * Mutation to update pivot table source range (atomic operation)
 */
export interface ISetPivotTableSourceRangeMutationParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
    sourceRangeInfo: ISourceRangeInfo;
}

export const SetPivotTableSourceRangeMutation: IMutation<ISetPivotTableSourceRangeMutationParams> = {
    type: CommandType.MUTATION,
    id: 'sheet.mutation.set-pivot-table-source-range',

    handler: (accessor, params) => {
        if (!params) {
            return false;
        }

        const pivotTableService = accessor.get(ISheetsPivotTableService);
        const { unitId, subUnitId, pivotTableId, sourceRangeInfo } = params;

        // Update source range
        pivotTableService.updateSourceRange(unitId, subUnitId, pivotTableId, sourceRangeInfo);

        return true;
    },
};

/**
 * Mutation to update pivot table target cell (atomic operation)
 */
export interface ISetPivotTableTargetCellMutationParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
    targetCellInfo: ITargetCellInfo;
}

export const SetPivotTableTargetCellMutation: IMutation<ISetPivotTableTargetCellMutationParams> = {
    type: CommandType.MUTATION,
    id: 'sheet.mutation.set-pivot-table-target-cell',

    handler: (accessor, params) => {
        if (!params) {
            return false;
        }

        const pivotTableService = accessor.get(ISheetsPivotTableService);
        const { unitId, subUnitId, pivotTableId, targetCellInfo } = params;

        // Update target cell
        pivotTableService.updateTargetCell(unitId, subUnitId, pivotTableId, targetCellInfo);
        return true;
    },
};

/**
 * Mutation to update pivot table fields config (atomic operation)
 */
export interface ISetPivotTableFieldsConfigMutationParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
    fieldsConfig: IFieldsConfig;
}

export const SetPivotTableFieldsConfigMutation: IMutation<ISetPivotTableFieldsConfigMutationParams> = {
    type: CommandType.MUTATION,
    id: 'sheet.mutation.set-pivot-table-fields-config',

    handler: (accessor, params) => {
        if (!params) {
            return false;
        }

        const pivotTableService = accessor.get(ISheetsPivotTableService);
        const { unitId, subUnitId, pivotTableId, fieldsConfig } = params;

        // Update fields config
        pivotTableService.updateFieldsConfig(unitId, subUnitId, pivotTableId, fieldsConfig);

        return true;
    },
};

/**
 * Mutation to update pivot table value position (atomic operation)
 */
export interface ISetPivotTableValuePositionMutationParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
    valuePosition: PivotValuePosition;
}

export const SetPivotTableValuePositionMutation: IMutation<ISetPivotTableValuePositionMutationParams> = {
    type: CommandType.MUTATION,
    id: 'sheet.mutation.set-pivot-table-value-position',

    handler: (accessor, params) => {
        if (!params) {
            return false;
        }

        const pivotTableService = accessor.get(ISheetsPivotTableService);
        const { unitId, subUnitId, pivotTableId, valuePosition } = params;

        const pivotTable = pivotTableService.getPivotTable(unitId, subUnitId, pivotTableId);
        if (!pivotTable) {
            return false;
        }

        // Update value position
        pivotTable.setValuePosition(valuePosition);

        // Update fields config to persist the change
        const currentFieldsConfig = pivotTableService.getPivotTableConfig(unitId, subUnitId, pivotTableId)?.fieldsConfig;
        if (currentFieldsConfig) {
            pivotTableService.updateFieldsConfig(unitId, subUnitId, pivotTableId, {
                ...currentFieldsConfig,
                valuePosition,
            });
        }

        return true;
    },
};

/**
 * Mutation to sync calculated data from Worker thread to Main thread
 * This mutation is used in RPC environment where:
 * - Worker thread performs pivot table calculations
 * - Worker thread executes this mutation to sync results
 * - Main thread receives the mutation via RPC and updates the model
 *
 * The calculated data is serializable (IObjectMatrixPrimitiveType) so it can be
 * passed through postMessage without DataCloneError.
 *
 * IMPORTANT: This mutation only updates the model on the Main thread (notExecuteFormula: true).
 * On the Worker thread (notExecuteFormula: false), the model already has the correct data
 * from the auto-calculation listener, so we skip the update to prevent infinite loops.
 */
export interface ISetPivotTableCalculatedDataMutationParams {
    unitId: string;
    subUnitId: string;
    pivotTableId: string;
    calculatedData: IObjectMatrixPrimitiveType<Nullable<ICellData>>;
}

export const SetPivotTableCalculatedDataMutation: IMutation<ISetPivotTableCalculatedDataMutationParams> = {
    type: CommandType.MUTATION,
    id: 'sheet.mutation.set-pivot-table-calculated-data',

    handler: (accessor, params) => {
        if (!params) {
            return false;
        }

        const configService = accessor.get(IConfigService);
        const pluginConfig = configService.getConfig<IUniverSheetsPivotTableConfig>(SHEETS_PIVOT_TABLE_PLUGIN_CONFIG_KEY);

        // Only update the model on the Main thread (notExecuteFormula: true).
        // On the Worker thread (notExecuteFormula: false), the data is already correct
        // from the auto-calculation listener. Updating here would trigger calculatedData$
        // which would trigger _listenToPivotTableDataChanges which would execute this
        // mutation again, causing an infinite loop.
        const isCalculate = pluginConfig?.notExecuteFormula ?? false;
        const { unitId, subUnitId, pivotTableId, calculatedData } = params;

        if (!isCalculate) {
            return true;
        }

        const dataSourceModel = accessor.get(SheetsPivotDataSourceModel);

        // Update calculated data in the model (Main thread only)
        dataSourceModel.setCalculatedData(unitId, subUnitId, pivotTableId, calculatedData);

        return true;
    },
};
