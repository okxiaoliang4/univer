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

import type { IMutation } from '@univerjs/core';
import type { PivotValuePosition } from '../../models/pivot-engine';
import type { IFieldsConfig, ISourceRangeInfo, ITargetCellInfo } from '../../types/type';
import { CommandType } from '@univerjs/core';
import { PivotTable } from '../../models/pivot-table';
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
        const dataSourceModel = pivotTableService.getDataSourceModel();

        const { unitId, subUnitId, pivotTableId, config } = params;

        // Create new pivot table
        const pivotTable = new PivotTable(
            pivotTableId,
            config.name,
            config.sourceRangeInfo,
            config.targetCellInfo,
            config.fieldsConfig
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
