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

// Commands
export {
    CreatePivotTableCommand,
    RemovePivotTableCommand as DeletePivotTableCommand,
    type ICreatePivotTableCommandParams,
    type IRemovePivotTableCommandParams as IDeletePivotTableCommandParams,
    type IUpdatePivotTableFieldsCommandParams,
    UpdatePivotTableFieldsCommand,
} from './commands/commands/pivot-table.command';

// Mutations
export {
    AddPivotTableMutation,
    type IAddPivotTableMutationParams,
    type IRemovePivotTableMutationParams,
    type ISetPivotTableFieldsConfigMutationParams,
    type ISetPivotTableSourceRangeMutationParams,
    type ISetPivotTableTargetCellMutationParams,
    RemovePivotTableMutation,
    SetPivotTableFieldsConfigMutation,
    SetPivotTableSourceRangeMutation,
    SetPivotTableTargetCellMutation,
} from './commands/mutations/pivot-table.mutation';

// Config
export type { IUniverSheetsPivotTableConfig } from './controllers/config.schema';

// Models
export { PivotTable } from './models/pivot-table';

// Plugin
export { UniverSheetsPivotTablePlugin } from './plugin';

// Services
export { ISheetsPivotTableService, SHEET_PIVOT_TABLE_PLUGIN as SHEET_PIVOT_TABLE_SNAPSHOT_ID } from './services/pivot-table.service';

export { AggregationType, PivotFieldAreaType, PivotSortOrder } from './types/enum';

// Types
export type {
    IFieldsConfig,
    IPivotField,
    IPivotFilterCriteria,
    IPivotTableCalculatedData,
    IPivotTableConfig,
    IPivotTableConfigResource,
    IPivotTableFieldsConfigChangedEvent,
    IPivotTableRangeChangedEvent,
    IPivotTableSourceRangeChangedEvent,
    IPivotTableTargetCellChangedEvent,
    ISourceRangeInfo,
    ITargetCellInfo,
} from './types/type';
