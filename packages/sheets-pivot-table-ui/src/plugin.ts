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

import type { IUniverSheetsPivotTableUIConfig } from './controllers/config.schema';
import { DependentOn, IConfigService, Inject, Injector, merge, Plugin, registerDependencies, touchDependencies, UniverInstanceType } from '@univerjs/core';
import { UniverSheetsPivotTablePlugin } from '@univerjs/sheets-pivot-table';
import { UniverSheetsUIPlugin } from '@univerjs/sheets-ui';
import { ComponentManager } from '@univerjs/ui';
import { PLUGIN_NAME } from './const/const';
import { defaultPluginConfig, SHEETS_PIVOT_TABLE_UI_PLUGIN_CONFIG_KEY } from './controllers/config.schema';
import { PivotTablePermissionUIController } from './controllers/pivot-table-permission-ui.controller';
import { PivotTableRenderController } from './controllers/pivot-table-render.controller';
import { PivotTableUIDesktopController } from './controllers/pivot-table-ui-desktop.controller';
import { PivotTableController } from './controllers/pivot-table.controller';
import { IntelligentFieldPlacementService } from './services/intelligent-field-placement.service';
import { ISheetsPivotTablePanelService, SheetsPivotTablePanelService } from './services/pivot-table-panel.service';
import { IPivotTableStyleService, PivotTableStyleService } from './services/pivot-table-style.service';
import { registerPivotTableComponents } from './views/menu';

@DependentOn(UniverSheetsUIPlugin, UniverSheetsPivotTablePlugin)
export class UniverSheetsPivotTableUIPlugin extends Plugin {
    static override pluginName = PLUGIN_NAME;
    static override type = UniverInstanceType.UNIVER_SHEET;

    constructor(
        private readonly _config: Partial<IUniverSheetsPivotTableUIConfig> = defaultPluginConfig,
        @Inject(Injector) protected override _injector: Injector,
        @IConfigService private readonly _configService: IConfigService,
        @Inject(ComponentManager) private readonly _componentManager: ComponentManager
    ) {
        super();

        // Manage the plugin configuration
        const { menu, ...rest } = merge(
            {},
            defaultPluginConfig,
            this._config
        );

        if (menu) {
            this._configService.setConfig('menu', menu, { merge: true });
        }

        this._configService.setConfig(SHEETS_PIVOT_TABLE_UI_PLUGIN_CONFIG_KEY, rest);
    }

    override onStarting(): void {
        // Register services and controllers
        registerDependencies(this._injector, [
            [ISheetsPivotTablePanelService, { useClass: SheetsPivotTablePanelService }],
            [IPivotTableStyleService, { useClass: PivotTableStyleService }],
        ]);
    }

    override onReady(): void {
        // Touch panel service to ensure initialization
        this._injector.get(ISheetsPivotTablePanelService);
    }

    override onRendered(): void {
        // Register components
        registerPivotTableComponents(this._componentManager);

        // Register desktop controller and permission UI controller
        registerDependencies(this._injector, [
            [IntelligentFieldPlacementService],
            [PivotTableController],
            [PivotTableUIDesktopController],
            [PivotTablePermissionUIController],
            [PivotTableRenderController],
        ]);

      // Touch controllers to ensure initialization
        touchDependencies(this._injector, [
            [PivotTableController],
            [PivotTableUIDesktopController],
            [PivotTablePermissionUIController],
            [PivotTableRenderController],
        ]);
    }
}
