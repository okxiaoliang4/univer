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

/**
 * Configuration for the Univer Sheets Pivot Table Plugin
 */
export interface IUniverSheetsPivotTableConfig {
    /**
     * Enable automatic recalculation when source data changes
     * @default true
     */
    autoRefresh?: boolean;

    /**
     * Maximum number of rows to process in pivot table calculation
     * @default 10000
     */
    maxRows?: number;

    /**
     * Enable caching of calculation results
     * @default true
     */
    enableCache?: boolean;
}

/**
 * Default plugin configuration
 */
export const defaultPluginConfig: IUniverSheetsPivotTableConfig = {
    autoRefresh: true,
    maxRows: 10000,
    enableCache: true,
};

/**
 * Configuration key for the plugin
 */
export const SHEETS_PIVOT_TABLE_PLUGIN_CONFIG_KEY = 'sheets-pivot-table.config';
