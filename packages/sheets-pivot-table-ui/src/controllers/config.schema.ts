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

import type { MenuConfig } from '@univerjs/ui';

/**
 * Configuration for the Univer Sheets Pivot Table UI Plugin
 */
export interface IUniverSheetsPivotTableUIConfig {
    /**
     * Menu configuration
     */
    menu?: MenuConfig;

    /**
     * Show pivot table panel by default
     * @default false
     */
    showPanelByDefault?: boolean;
}

/**
 * Default plugin configuration
 */
export const defaultPluginConfig: IUniverSheetsPivotTableUIConfig = {
    showPanelByDefault: false,
};

/**
 * Configuration key for the plugin
 */
export const SHEETS_PIVOT_TABLE_UI_PLUGIN_CONFIG_KEY = 'sheets-pivot-table-ui.config';
