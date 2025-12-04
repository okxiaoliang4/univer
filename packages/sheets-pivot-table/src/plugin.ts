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

import type { IAddPivotTableMutationParams, IRemovePivotTableMutationParams, ISetPivotTableFieldsConfigMutationParams, ISetPivotTableSourceRangeMutationParams, ISetPivotTableTargetCellMutationParams, ISetPivotTableValuePositionMutationParams } from './commands/mutations/pivot-table.mutation';
import type { IUniverSheetsPivotTableConfig } from './controllers/config.schema';
import type { IPivotTableConfigResource } from './types/type';
import { ICommandService, IConfigService, Inject, Injector, merge, Plugin, registerDependencies, touchDependencies, UniverInstanceType } from '@univerjs/core';
import { CreatePivotTableCommand, RemovePivotTableCommand, SetPivotTableSourceRangeCommand, SetPivotTableValuePositionCommand, UpdatePivotTableFieldsCommand } from './commands/commands/pivot-table.command';
import {
    AddPivotTableMutation,
    RemovePivotTableMutation,
    SetPivotTableFieldsConfigMutation,
    SetPivotTableSourceRangeMutation,
    SetPivotTableTargetCellMutation,
    SetPivotTableValuePositionMutation,
} from './commands/mutations/pivot-table.mutation';
import { defaultPluginConfig, SHEETS_PIVOT_TABLE_PLUGIN_CONFIG_KEY } from './controllers/config.schema';
import { PivotTableFormulaController } from './controllers/pivot-table-formula.controller';
import { PivotTablePermissionController } from './controllers/pivot-table-permission.controller';
import { SheetPviotTableRangeController } from './controllers/pivot-table-range.controller';
import { SheetPivotTableController } from './controllers/pivot-table.controller';
import { SheetsPivotDataSourceModel } from './models/sheets-pivot-data-source-model';
import { IPivotTableRangeService, PivotTableRangeService } from './services/pivot-table-range.service';
import { ISheetsPivotTableService, SHEET_PIVOT_TABLE_PLUGIN, SheetsPivotTableService } from './services/pivot-table.service';

export class UniverSheetsPivotTablePlugin extends Plugin {
    static override pluginName = SHEET_PIVOT_TABLE_PLUGIN;
    static override type = UniverInstanceType.UNIVER_SHEET;

    constructor(
        private readonly _config: Partial<IUniverSheetsPivotTableConfig> = defaultPluginConfig,
        @Inject(Injector) protected readonly _injector: Injector,
        @IConfigService private readonly _configService: IConfigService,
        @ICommandService private readonly _commandService: ICommandService
    ) {
        super();

        // Manage the plugin configuration
        const { ...rest } = merge(
            {},
            defaultPluginConfig,
            this._config
        );
        this._configService.setConfig(SHEETS_PIVOT_TABLE_PLUGIN_CONFIG_KEY, rest);
    }

    override onStarting(): void {
        // Register services and controllers
        registerDependencies(this._injector, [
            [ISheetsPivotTableService, { useClass: SheetsPivotTableService }],
            [IPivotTableRangeService, { useClass: PivotTableRangeService }],
            [SheetPivotTableController],
            [SheetPviotTableRangeController],
            [PivotTablePermissionController],
            [PivotTableFormulaController],
            [SheetsPivotDataSourceModel],
        ]);

        touchDependencies(this._injector, [
            [SheetPviotTableRangeController],
            [PivotTablePermissionController],
            [PivotTableFormulaController],
        ]);

        // Register commands
        [
            CreatePivotTableCommand,
            UpdatePivotTableFieldsCommand,
            RemovePivotTableCommand,
            SetPivotTableSourceRangeCommand,
            SetPivotTableValuePositionCommand,
        ].forEach((command) => {
            this._commandService.registerCommand(command);
        });

        // Register mutations
        [
            AddPivotTableMutation,
            RemovePivotTableMutation,
            SetPivotTableSourceRangeMutation,
            SetPivotTableFieldsConfigMutation,
            SetPivotTableTargetCellMutation,
            SetPivotTableValuePositionMutation,
        ].forEach((mutation) => {
            this._commandService.registerCommand(mutation);
        });
    }

    override onReady(): void {
        // Touch dependencies to ensure they are initialized
        touchDependencies(this._injector, [
            [ISheetsPivotTableService],
            [SheetPivotTableController],
        ]);
    }
}

// Export mutations and their types for collaboration plugin
export {
    AddPivotTableMutation,
    RemovePivotTableMutation,
    SetPivotTableFieldsConfigMutation,
    SetPivotTableSourceRangeMutation,
    SetPivotTableTargetCellMutation,
    SetPivotTableValuePositionMutation,
};

export type {
    IAddPivotTableMutationParams,
    IRemovePivotTableMutationParams,
    ISetPivotTableFieldsConfigMutationParams,
    ISetPivotTableSourceRangeMutationParams,
    ISetPivotTableTargetCellMutationParams,
    ISetPivotTableValuePositionMutationParams,
};

// Export resource types for collaboration plugin
export type { IPivotTableConfigResource };
