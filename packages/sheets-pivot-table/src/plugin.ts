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

import type { Dependency } from '@univerjs/core';
import type { IUniverSheetsPivotTableConfig } from './controllers/config.schema';
import { DependentOn, ICommandService, IConfigService, Inject, Injector, merge, Optional, Plugin, touchDependencies, UniverInstanceType } from '@univerjs/core';
import { UniverFormulaEnginePlugin } from '@univerjs/engine-formula';
import { DataSyncPrimaryController } from '@univerjs/rpc';
import { CreatePivotTableCommand, RemovePivotTableCommand, SetPivotTableSourceRangeCommand, UpdatePivotTableFieldsCommand } from './commands/commands/pivot-table.command';
import {
    AddPivotTableMutation,
    RemovePivotTableMutation,
    SetPivotTableCalculatedDataMutation,
    SetPivotTableFieldsConfigMutation,
    SetPivotTableSourceRangeMutation,
    SetPivotTableTargetCellMutation,
} from './commands/mutations/pivot-table.mutation';
import { defaultPluginConfig, SHEETS_PIVOT_TABLE_PLUGIN_CONFIG_KEY } from './controllers/config.schema';
import { PivotTableFormulaController } from './controllers/pivot-table-formula.controller';
import { PivotTablePermissionController } from './controllers/pivot-table-permission.controller';
import { SheetPviotTableRangeController } from './controllers/pivot-table-range.controller';
import { SheetPivotTableController } from './controllers/pivot-table.controller';
import { SheetsPivotDataSourceModel } from './models/sheets-pivot-data-source-model';
import { IPivotTableRangeService, PivotTableRangeService } from './services/pivot-table-range.service';
import { ISheetsPivotTableService, SHEET_PIVOT_TABLE_PLUGIN, SheetsPivotTableService } from './services/pivot-table.service';

@DependentOn(UniverFormulaEnginePlugin)
export class UniverSheetsPivotTablePlugin extends Plugin {
    static override pluginName = SHEET_PIVOT_TABLE_PLUGIN;
    static override type = UniverInstanceType.UNIVER_SHEET;

    constructor(
        private readonly _config: Partial<IUniverSheetsPivotTableConfig> = defaultPluginConfig,
        @Inject(Injector) protected readonly _injector: Injector,
        @IConfigService private readonly _configService: IConfigService,
        @ICommandService private readonly _commandService: ICommandService,
        @Optional(DataSyncPrimaryController) private readonly _dataSyncPrimaryController?: DataSyncPrimaryController
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
        const dependencies: Dependency[] = [
            [ISheetsPivotTableService, { useClass: SheetsPivotTableService }],
            [IPivotTableRangeService, { useClass: PivotTableRangeService }],
            [SheetPivotTableController],
            [SheetPviotTableRangeController],
            [PivotTablePermissionController],
            [SheetsPivotDataSourceModel],
        ];

        // Only register PivotTableFormulaController if notExecuteFormula is false
        if (!this._config.notExecuteFormula) {
            dependencies.push([PivotTableFormulaController]);
        }

        dependencies.forEach((dependency) => {
            this._injector.add(dependency);
        });

        // Register commands
        [
            CreatePivotTableCommand,
            UpdatePivotTableFieldsCommand,
            RemovePivotTableCommand,
            SetPivotTableSourceRangeCommand,
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
            SetPivotTableCalculatedDataMutation,
        ].forEach((mutation) => {
            this._commandService.registerCommand(mutation);
            // Register mutations to be synced to the worker thread
            this._dataSyncPrimaryController?.registerSyncingMutations(mutation);
        });
    }

    override onReady(): void {
        // Touch dependencies to ensure they are initialized
        touchDependencies(this._injector, [
            [SheetPviotTableRangeController],
            [PivotTablePermissionController],
        ]);

        // Only touch PivotTableFormulaController if notExecuteFormula is false
        if (!this._config.notExecuteFormula) {
            touchDependencies(this._injector, [
                [PivotTableFormulaController],
            ]);
        }

        touchDependencies(this._injector, [
            [ISheetsPivotTableService],
            [SheetPivotTableController],
        ]);
    }
}
