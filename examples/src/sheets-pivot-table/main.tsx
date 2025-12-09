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

import { LocaleType, LogLevel, Univer, UniverInstanceType } from '@univerjs/core';
import { FUniver } from '@univerjs/core/facade';
import { UniverDocsPlugin } from '@univerjs/docs';
import { UniverDocsUIPlugin } from '@univerjs/docs-ui';
import { UniverFormulaEnginePlugin } from '@univerjs/engine-formula';
import { UniverRenderEnginePlugin } from '@univerjs/engine-render';
import zhCN from '@univerjs/mockdata/locales/zh-CN';
import { UniverRPCMainThreadPlugin } from '@univerjs/rpc';
import { UniverSheetsPlugin } from '@univerjs/sheets';
import { UniverSheetsFormulaPlugin } from '@univerjs/sheets-formula';
import { UniverSheetsFormulaUIPlugin } from '@univerjs/sheets-formula-ui';
import { UniverSheetsNumfmtPlugin } from '@univerjs/sheets-numfmt';
import { UniverSheetsNumfmtUIPlugin } from '@univerjs/sheets-numfmt-ui';
// import { UniverSheetsPivotTablePlugin } from '@univerjs-pro/sheets-pivot';
// import { UniverSheetsPivotTableUIPlugin } from '@univerjs-pro/sheets-pivot-ui';
import { UniverSheetsPivotTablePlugin } from '@univerjs/sheets-pivot-table';
import { UniverSheetsPivotTableUIPlugin } from '@univerjs/sheets-pivot-table-ui';

import { UniverSheetsUIPlugin } from '@univerjs/sheets-ui';
import { UniverUIPlugin } from '@univerjs/ui';
import '@univerjs/sheets/facade';
import '@univerjs/ui/facade';
import '@univerjs/docs-ui/facade';
import '@univerjs/sheets-ui/facade';
import '@univerjs/sheets-data-validation/facade';
import '@univerjs/engine-formula/facade';
import '@univerjs/sheets-filter/facade';
import '@univerjs/sheets-formula/facade';
import '@univerjs/sheets-numfmt/facade';
import '@univerjs/sheets-hyper-link-ui/facade';
import '@univerjs/sheets-thread-comment/facade';
import '@univerjs/sheets-conditional-formatting/facade';
import '@univerjs/sheets-find-replace/facade';
import '@univerjs/sheets-drawing-ui/facade';
import '@univerjs/sheets-zen-editor/facade';
import '@univerjs/sheets-crosshair-highlight/facade';
import '@univerjs/sheets-formula-ui/facade';
import '@univerjs/sheets-table/facade';
import '@univerjs/sheets-sort/facade';
import '@univerjs/network/facade';
import '@univerjs/sheets-note/facade';
import '../global.css';

const univer = new Univer({
    locale: LocaleType.ZH_CN,
    locales: {
        [LocaleType.ZH_CN]: zhCN,
    },
    logLevel: LogLevel.VERBOSE,
});

const worker = new Worker(new URL('./worker.js', import.meta.url), { type: 'module' });
univer.registerPlugin(UniverRPCMainThreadPlugin, { workerURL: worker });
univer.onDispose(() => worker.terminate());

univer.registerPlugin(UniverFormulaEnginePlugin, { notExecuteFormula: true });
univer.registerPlugin(UniverRenderEnginePlugin);
univer.registerPlugin(UniverUIPlugin, {
    container: 'app',
  // ribbonType:'simple'
});
univer.registerPlugin(UniverDocsPlugin);
univer.registerPlugin(UniverDocsUIPlugin);

// sheets plugin
univer.registerPlugin(UniverSheetsPlugin, { notExecuteFormula: true });
univer.registerPlugin(UniverSheetsUIPlugin);
univer.registerPlugin(UniverSheetsFormulaUIPlugin);
// sheet feature plugins
univer.registerPlugin(UniverSheetsNumfmtPlugin);
univer.registerPlugin(UniverSheetsNumfmtUIPlugin);
univer.registerPlugin(UniverSheetsFormulaPlugin, { notExecuteFormula: true });
univer.registerPlugin(UniverSheetsPivotTablePlugin, { notExecuteFormula: true });
univer.registerPlugin(UniverSheetsPivotTableUIPlugin);

univer.createUnit(UniverInstanceType.UNIVER_SHEET, {
    id: 'H1p5SR',
    sheetOrder: [
        'ZCGh3uiogN11CwYQlPhHo',
        'M2HmbxqfnSZky1RnARn0g',
    ],
    name: '',
    appVersion: '0.10.12',
    locale: 'zhCN',
    styles: {},
    sheets: {
        ZCGh3uiogN11CwYQlPhHo: {
            id: 'ZCGh3uiogN11CwYQlPhHo',
            name: 'Sheet1',
            tabColor: '',
            hidden: 0,
            rowCount: 1000,
            columnCount: 20,
            zoomRatio: 1,
            freeze: {
                xSplit: 0,
                ySplit: 0,
                startRow: -1,
                startColumn: -1,
            },
            scrollTop: 0,
            scrollLeft: 0,
            defaultColumnWidth: 88,
            defaultRowHeight: 24,
            mergeData: [],
            cellData: {
                0: {
                    0: {
                        v: '区域 (Region)',
                        t: 1,
                    },
                    1: {
                        v: '产品类别 (Category)',
                        t: 1,
                    },
                    2: {
                        v: '销售渠道 (Channel)',
                        t: 1,
                    },
                    3: {
                        v: '季度 (Quarter)',
                        t: 1,
                    },
                    4: {
                        v: '销售额 (Sales Amount)',
                        t: 1,
                    },
                },
                1: {
                    0: {
                        v: '华东',
                        t: 1,
                    },
                    1: {
                        v: '笔记本电脑',
                        t: 1,
                    },
                    2: {
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        v: 'Q1',
                        t: 1,
                    },
                    4: {
                        v: 15000,
                        t: 2,
                    },
                },
                2: {
                    0: {
                        v: '华东',
                        t: 1,
                    },
                    1: {
                        v: '笔记本电脑',
                        t: 1,
                    },
                    2: {
                        v: '线下门店',
                        t: 1,
                    },
                    3: {
                        v: 'Q1',
                        t: 1,
                    },
                    4: {
                        v: 12000,
                        t: 2,
                    },
                },
                3: {
                    0: {
                        v: '华东',
                        t: 1,
                    },
                    1: {
                        v: '手机',
                        t: 1,
                    },
                    2: {
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        v: 'Q1',
                        t: 1,
                    },
                    4: {
                        v: 8000,
                        t: 2,
                    },
                },
                4: {
                    0: {
                        v: '华北',
                        t: 1,
                    },
                    1: {
                        v: '笔记本电脑',
                        t: 1,
                    },
                    2: {
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        v: 'Q2',
                        t: 1,
                    },
                    4: {
                        v: 18000,
                        t: 2,
                    },
                },
                5: {
                    0: {
                        v: '华北',
                        t: 1,
                    },
                    1: {
                        v: '手机',
                        t: 1,
                    },
                    2: {
                        v: '线下门店',
                        t: 1,
                    },
                    3: {
                        v: 'Q2',
                        t: 1,
                    },
                    4: {
                        v: 6500,
                        t: 2,
                    },
                },
                6: {
                    0: {
                        v: '华南',
                        t: 1,
                    },
                    1: {
                        v: '手机',
                        t: 1,
                    },
                    2: {
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        v: 'Q3',
                        t: 1,
                    },
                    4: {
                        v: 10000,
                        t: 2,
                    },
                },
                7: {
                    0: {
                        v: '华南',
                        t: 1,
                    },
                    1: {
                        v: '配件',
                        t: 1,
                    },
                    2: {
                        v: '线下门店',
                        t: 1,
                    },
                    3: {
                        v: 'Q3',
                        t: 1,
                    },
                    4: {
                        v: 3000,
                        t: 2,
                    },
                },
                8: {
                    0: {
                        v: '华东',
                        t: 1,
                    },
                    1: {
                        v: '笔记本电脑',
                        t: 1,
                    },
                    2: {
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        v: 'Q4',
                        t: 1,
                    },
                    4: {
                        v: 16500,
                        t: 2,
                    },
                },
                9: {
                    0: {
                        v: '华东',
                        t: 1,
                    },
                    1: {
                        v: '手机',
                        t: 1,
                    },
                    2: {
                        v: '线下门店',
                        t: 1,
                    },
                    3: {
                        v: 'Q4',
                        t: 1,
                    },
                    4: {
                        v: 9000,
                        t: 2,
                    },
                },
                10: {
                    0: {
                        v: '华北',
                        t: 1,
                    },
                    1: {
                        v: '配件',
                        t: 1,
                    },
                    2: {
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        v: 'Q1',
                        t: 1,
                    },
                    4: {
                        v: 2500,
                        t: 2,
                    },
                },
                11: {
                    0: {
                        v: '华南',
                        t: 1,
                    },
                    1: {
                        v: '笔记本电脑',
                        t: 1,
                    },
                    2: {
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        v: 'Q2',
                        t: 1,
                    },
                    4: {
                        v: 14000,
                        t: 2,
                    },
                },
                12: {
                    0: {
                        v: '华北',
                        t: 1,
                    },
                    1: {
                        v: '手机',
                        t: 1,
                    },
                    2: {
                        v: '线下门店',
                        t: 1,
                    },
                    3: {
                        v: 'Q4',
                        t: 1,
                    },
                    4: {
                        v: 7200,
                        t: 2,
                    },
                },
            },
            rowData: {},
            columnData: {},
            showGridlines: 1,
            rowHeader: {
                width: 46,
                hidden: 0,
            },
            columnHeader: {
                height: 20,
                hidden: 0,
            },
            rightToLeft: 0,
        },
        M2HmbxqfnSZky1RnARn0g: {
            id: 'M2HmbxqfnSZky1RnARn0g',
            showGridlines: 0,
            name: 'Sheet11',
            tabColor: '',
            hidden: 0,
            rowCount: 1000,
            columnCount: 20,
            zoomRatio: 1,
            freeze: {
                xSplit: 0,
                ySplit: 0,
                startRow: -1,
                startColumn: -1,
            },
            scrollTop: 0,
            scrollLeft: 0,
            defaultColumnWidth: 88,
            defaultRowHeight: 24,
            mergeData: [],
            cellData: {
                15: {
                    0: {
                        f: '=C8',
                        v: 2500,
                        t: 2,
                    },
                },
            },
            rowData: {},
            columnData: {},
            rowHeader: {
                width: 46,
                hidden: 0,
            },
            columnHeader: {
                height: 20,
                hidden: 0,
            },
            rightToLeft: 0,
        },
    },
    resources: [
        {
            name: 'SHEET_RANGE_PROTECTION_PLUGIN',
            data: '',
        },
        {
            name: 'SHEET_AuthzIoMockService_PLUGIN',
            data: '{}',
        },
        {
            name: 'SHEET_WORKSHEET_PROTECTION_PLUGIN',
            data: '{}',
        },
        {
            name: 'SHEET_WORKSHEET_PROTECTION_POINT_PLUGIN',
            data: '{}',
        },
        {
            name: 'SHEET_DEFINED_NAME_PLUGIN',
            data: '{}',
        },
        {
            name: 'SHEET_RANGE_THEME_MODEL_PLUGIN',
            data: '{}',
        },
        {
            name: 'SHEET_PIVOT_TABLE_PLUGIN',
            data: '{"pivotTableConfigs":{"H1p5SR":{"M2HmbxqfnSZky1RnARn0g":{"w5OLpWxLxvkC3i-xnTWTd":{"id":"w5OLpWxLxvkC3i-xnTWTd","name":"New Pivot Table","sourceRangeInfo":{"range":{"startRow":0,"startColumn":0,"endRow":12,"endColumn":4,"startAbsoluteRefType":0,"endAbsoluteRefType":0,"rangeType":0},"subUnitId":"ZCGh3uiogN11CwYQlPhHo","unitId":"H1p5SR"},"targetCellInfo":{"row":0,"col":0,"subUnitId":"M2HmbxqfnSZky1RnARn0g","unitId":"H1p5SR"},"fieldsConfig":{"valueFields":[{"id":"MpVVvJ_[H1p5SR]ZCGh3uiogN11CwYQlPhHo!E1:E13","name":"销售额 (Sales Amount)","sourceColumnIndex":4,"aggregation":"sum"}],"rowFields":[{"id":"aSXFU4_[H1p5SR]ZCGh3uiogN11CwYQlPhHo!A1:A13","name":"区域 (Region)","sourceColumnIndex":0},{"id":"gxb5-p_[H1p5SR]ZCGh3uiogN11CwYQlPhHo!B1:B13","name":"产品类别 (Category)","sourceColumnIndex":1}],"columnFields":[{"id":"hkTk5Q_[H1p5SR]ZCGh3uiogN11CwYQlPhHo!C1:C13","name":"销售渠道 (Channel)","sourceColumnIndex":2},{"id":"T--5Nk_[H1p5SR]ZCGh3uiogN11CwYQlPhHo!D1:D13","name":"季度 (Quarter)","sourceColumnIndex":3}],"filterFields":[],"valuePosition":1}}}}},"pivotData":{"H1p5SR":{"M2HmbxqfnSZky1RnARn0g":{"w5OLpWxLxvkC3i-xnTWTd":{"isEmpty":false,"dimensions":{"totalRows":8,"totalColumns":8,"dataRowCount":8,"dataColumnCount":8,"valueFieldCount":1},"structure":{"rowHeaders":[["华东","手机"],["华东","笔记本电脑"],["华北","手机"],["华北","笔记本电脑"],["华北","配件"],["华南","手机"],["华南","笔记本电脑"],["华南","配件"]],"columnHeaders":[["线上商城","Q1"],["线上商城","Q2"],["线上商城","Q3"],["线上商城","Q4"],["线下门店","Q1"],["线下门店","Q2"],["线下门店","Q3"],["线下门店","Q4"]],"values":[[[8000],[null],[null],[null],[null],[null],[null],[9000]],[[15000],[null],[null],[16500],[12000],[null],[null],[null]],[[null],[null],[null],[null],[null],[6500],[null],[7200]],[[null],[18000],[null],[null],[null],[null],[null],[null]],[[2500],[null],[null],[null],[null],[null],[null],[null]],[[null],[null],[10000],[null],[null],[null],[null],[null]],[[null],[14000],[null],[null],[null],[null],[null],[null]],[[null],[null],[null],[null],[null],[null],[3000],[null]]],"rowTypes":["data","data","data","data","data","data","data","data"],"columnTypes":["data","data","data","data","data","data","data","data"],"rowGroups":[{"groupId":"row-0-华东|手机","firstRowIndex":0,"lastRowIndex":0,"level":0,"fieldIndex":0,"value":"华东","expanded":true},{"groupId":"row-0-华东|笔记本电脑","firstRowIndex":1,"lastRowIndex":1,"level":0,"fieldIndex":0,"value":"华东","expanded":true},{"groupId":"row-0-华北|手机","firstRowIndex":2,"lastRowIndex":2,"level":0,"fieldIndex":0,"value":"华北","expanded":true},{"groupId":"row-0-华北|笔记本电脑","firstRowIndex":3,"lastRowIndex":3,"level":0,"fieldIndex":0,"value":"华北","expanded":true},{"groupId":"row-0-华北|配件","firstRowIndex":4,"lastRowIndex":4,"level":0,"fieldIndex":0,"value":"华北","expanded":true},{"groupId":"row-0-华南|手机","firstRowIndex":5,"lastRowIndex":5,"level":0,"fieldIndex":0,"value":"华南","expanded":true},{"groupId":"row-0-华南|笔记本电脑","firstRowIndex":6,"lastRowIndex":6,"level":0,"fieldIndex":0,"value":"华南","expanded":true},{"groupId":"row-0-华南|配件","firstRowIndex":7,"lastRowIndex":7,"level":0,"fieldIndex":0,"value":"华南","expanded":true}],"columnGroups":[{"groupId":"col-0-线上商城|Q1","firstColumnIndex":0,"lastColumnIndex":0,"level":0,"fieldIndex":0,"value":"线上商城","expanded":true},{"groupId":"col-0-线上商城|Q2","firstColumnIndex":1,"lastColumnIndex":1,"level":0,"fieldIndex":0,"value":"线上商城","expanded":true},{"groupId":"col-0-线上商城|Q3","firstColumnIndex":2,"lastColumnIndex":2,"level":0,"fieldIndex":0,"value":"线上商城","expanded":true},{"groupId":"col-0-线上商城|Q4","firstColumnIndex":3,"lastColumnIndex":3,"level":0,"fieldIndex":0,"value":"线上商城","expanded":true},{"groupId":"col-0-线下门店|Q1","firstColumnIndex":4,"lastColumnIndex":4,"level":0,"fieldIndex":0,"value":"线下门店","expanded":true},{"groupId":"col-0-线下门店|Q2","firstColumnIndex":5,"lastColumnIndex":5,"level":0,"fieldIndex":0,"value":"线下门店","expanded":true},{"groupId":"col-0-线下门店|Q3","firstColumnIndex":6,"lastColumnIndex":6,"level":0,"fieldIndex":0,"value":"线下门店","expanded":true},{"groupId":"col-0-线下门店|Q4","firstColumnIndex":7,"lastColumnIndex":7,"level":0,"fieldIndex":0,"value":"线下门店","expanded":true}],"rowLevelMap":{"0":["row-0-华东|手机"],"1":["row-0-华东|笔记本电脑"],"2":["row-0-华北|手机"],"3":["row-0-华北|笔记本电脑"],"4":["row-0-华北|配件"],"5":["row-0-华南|手机"],"6":["row-0-华南|笔记本电脑"],"7":["row-0-华南|配件"]},"columnLevelMap":{"0":["col-0-线上商城|Q1"],"1":["col-0-线上商城|Q2"],"2":["col-0-线上商城|Q3"],"3":["col-0-线上商城|Q4"],"4":["col-0-线下门店|Q1"],"5":["col-0-线下门店|Q2"],"6":["col-0-线下门店|Q3"],"7":["col-0-线下门店|Q4"]}}}}}}}',
        },
    ],
});

window.univer = univer;
window.univerAPI = FUniver.newAPI(univer);
