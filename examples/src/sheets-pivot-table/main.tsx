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

univer.registerPlugin(UniverFormulaEnginePlugin);
univer.registerPlugin(UniverRenderEnginePlugin);
univer.registerPlugin(UniverUIPlugin, {
    container: 'app',
  // ribbonType:'simple'
});
univer.registerPlugin(UniverDocsPlugin);
univer.registerPlugin(UniverDocsUIPlugin);

// sheets plugin
univer.registerPlugin(UniverSheetsPlugin);
univer.registerPlugin(UniverSheetsUIPlugin);
univer.registerPlugin(UniverSheetsFormulaUIPlugin);
// sheet feature plugins
univer.registerPlugin(UniverSheetsNumfmtPlugin);
univer.registerPlugin(UniverSheetsNumfmtUIPlugin);
univer.registerPlugin(UniverSheetsFormulaPlugin);
univer.registerPlugin(UniverSheetsPivotTablePlugin);
univer.registerPlugin(UniverSheetsPivotTableUIPlugin);

univer.createUnit(UniverInstanceType.UNIVER_SHEET, {
    id: 'H1p5SR',
    sheetOrder: [
        'ZCGh3uiogN11CwYQlPhHo',
        'L_3zPqk5AbeQDuxJiDgPs',
    ],
    name: '',
    appVersion: '0.10.12',
    locale: 'zhCN',
    styles: {
        esEZKE: {
            ff: 'Calibri, sans-serif',
            fs: 12,
            it: 0,
            bl: 0,
            ul: {
                s: 0,
                cl: {
                    rgb: 'rgb(0,0,0)',
                },
            },
            st: {
                s: 0,
                cl: {
                    rgb: 'rgb(0,0,0)',
                },
            },
            ol: {
                s: 0,
                cl: {
                    rgb: 'rgb(0,0,0)',
                },
            },
            tr: {
                a: 0,
                v: 0,
            },
            td: 0,
            cl: {
                rgb: 'rgb(0,0,0)',
            },
            ht: 0,
            vt: 3,
            tb: 1,
            pd: {
                t: 0,
                b: 2,
                l: 2,
                r: 2,
            },
        },
    },
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
                        v: 'Product',
                        t: 1,
                        s: 'esEZKE',
                    },
                    1: {
                        v: 'East',
                        t: 1,
                        s: 'esEZKE',
                    },
                    2: {
                        s: 'esEZKE',
                        v: 'West',
                        t: 1,
                    },
                },
                1: {
                    0: {
                        v: 'A',
                        t: 1,
                        s: 'esEZKE',
                    },
                    1: {
                        s: 'esEZKE',
                        v: 213,
                        t: 2,
                    },
                    2: {
                        s: 'esEZKE',
                        v: 213,
                        t: 2,
                    },
                },
                2: {
                    0: {
                        v: 'B',
                        t: 1,
                        s: 'esEZKE',
                    },
                    1: {
                        s: 'esEZKE',
                        v: 23,
                        t: 2,
                    },
                    2: {
                        s: 'esEZKE',
                        v: 123,
                        t: 2,
                    },
                },
                3: {
                    0: {
                        s: 'esEZKE',
                        v: 'A',
                        t: 1,
                    },
                    1: {
                        s: 'esEZKE',
                        v: 232,
                        t: 2,
                    },
                    2: {
                        s: 'esEZKE',
                        v: 323,
                        t: 2,
                    },
                },
                4: {
                    0: {
                        s: 'esEZKE',
                        v: 'A',
                        t: 1,
                    },
                    1: {
                        s: 'esEZKE',
                        v: 23,
                        t: 2,
                    },
                    2: {
                        s: 'esEZKE',
                        v: 23,
                        t: 2,
                    },
                },
                5: {
                    0: {
                        s: 'esEZKE',
                        v: 'B',
                        t: 1,
                    },
                    1: {
                        s: 'esEZKE',
                        v: 123213,
                        t: 2,
                    },
                    2: {
                        s: 'esEZKE',
                        v: 23,
                        t: 2,
                    },
                },
            },
            rowData: {
                0: {
                    h: 21,
                },
                1: {
                    h: 21,
                },
                2: {
                    h: 21,
                },
                3: {
                    h: 21,
                },
                4: {
                    h: 21,
                },
                5: {
                    h: 21,
                },
            },
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
        L_3zPqk5AbeQDuxJiDgPs: {
            id: 'L_3zPqk5AbeQDuxJiDgPs',
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
            cellData: {},
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
            data: '{}',
        },
    ],
});

window.univer = univer;
window.univerAPI = FUniver.newAPI(univer);
