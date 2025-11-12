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
        'M2HmbxqfnSZky1RnARn0g',
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
        zDOqkI: {
            ff: 'Arial',
            fs: 11,
            it: 0,
            bl: 0,
            ul: {
                s: 0,
            },
            st: {
                s: 0,
            },
            ol: {
                s: 0,
            },
            tr: {
                a: 0,
                v: 0,
            },
            td: 0,
            ht: 0,
            vt: 0,
            tb: 0,
            pd: {
                t: 0,
                b: 2,
                l: 2,
                r: 2,
            },
        },
        Qf2ezt: {
            bd: {
                t: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                l: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#e1effe',
            },
        },
        '8C3-ed': {
            bd: {
                t: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#edebfe',
            },
        },
        'U0-kpb': {
            bd: {
                t: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#edebfe',
            },
        },
        'dZrD-B': {
            bd: {
                l: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#e1effe',
            },
        },
        '-_NnhX': {
            bg: {
                rgb: '#e6e6e6',
            },
        },
        '2Wv06O': {
            bd: {
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#e6e6e6',
            },
        },
        '9ChdWQ': {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                l: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#e1effe',
            },
        },
        L6xlgu: {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#e6e6e6',
            },
        },
        JT8HFG: {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#e6e6e6',
            },
        },
        qWAGQR: {
            bd: {
                t: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                l: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#e1effe',
            },
        },
        m83WXn: {
            bd: {
                t: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                l: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#edebfe',
            },
        },
        ZDRCVa: {
            bd: {
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                l: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                t: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
        },
        A3cKbS: {
            bg: {
                rgb: '#e6e6e6',
            },
            bd: {
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                l: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                t: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
        },
        AD9p4_: {
            bd: {
                t: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
        },
        '8gSjpp': {
            bd: {
                t: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
        },
        LMzh5X: {
            bd: {
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
        },
        pCQ6og: {
            bd: {
                l: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
        },
        '8ljP_v': {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                l: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
        },
        MCvSh3: {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
        },
        GX5RdF: {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
        },
        n3IWnG: {
            bg: {
                rgb: '#e1effe',
            },
        },
        '0zf8oD': {
            bd: {
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#e1effe',
            },
        },
        '0GAPjT': {
            bg: {
                rgb: '#edebfe',
            },
        },
        PVg3Dp: {
            bd: {
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#edebfe',
            },
        },
        '6lykwt': {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#edebfe',
            },
        },
        UR6l15: {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#edebfe',
            },
        },
        l4otnx: {
            bg: {
                rgb: '#cdd0d8',
            },
        },
        '9_TTtP': {
            bd: {
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#cdd0d8',
            },
        },
        DzR6JE: {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#cdd0d8',
            },
        },
        'Qgh-3O': {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#cdd0d8',
            },
        },
        bI_hFu: {
            bg: {
                rgb: '#eeeeee',
            },
        },
        qZFhkH: {
            bd: {
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#eeeeee',
            },
        },
        '57yH04': {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#eeeeee',
            },
        },
        U2djVe: {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#eeeeee',
            },
        },
        _UrEAM: {
            bg: {
                rgb: '#ffffff',
            },
        },
        jvfNbP: {
            bd: {
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#ffffff',
            },
        },
        Ao_Tif: {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#ffffff',
            },
        },
        '-P13NE': {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                r: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#ffffff',
            },
        },
        '89ry-k': {
            bd: {
                t: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                l: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#feecdc',
            },
        },
        DwdEqC: {
            bd: {
                t: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#feecdc',
            },
        },
        Eu96et: {
            bd: {
                l: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#feecdc',
            },
        },
        qgIyQ4: {
            bg: {
                rgb: '#feecdc',
            },
        },
        TvIrpW: {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
                l: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#feecdc',
            },
        },
        ukd3PM: {
            bd: {
                b: {
                    s: 1,
                    cl: {
                        rgb: '#000000',
                    },
                },
            },
            bg: {
                rgb: '#feecdc',
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
                        v: '区域 (Region)',
                        t: 1,
                        s: 'zDOqkI',
                    },
                    1: {
                        v: '产品类别 (Category)',
                        t: 1,
                        s: 'zDOqkI',
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '销售渠道 (Channel)',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: '季度 (Quarter)',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
                        v: '销售额 (Sales Amount)',
                        t: 1,
                    },
                },
                1: {
                    0: {
                        v: '华东',
                        t: 1,
                        s: 'zDOqkI',
                    },
                    1: {
                        s: 'zDOqkI',
                        v: '笔记本电脑',
                        t: 1,
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: 'Q1',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
                        v: 15000,
                        t: 2,
                    },
                },
                2: {
                    0: {
                        v: '华东',
                        t: 1,
                        s: 'zDOqkI',
                    },
                    1: {
                        s: 'zDOqkI',
                        v: '笔记本电脑',
                        t: 1,
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '线下门店',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: 'Q1',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
                        v: 12000,
                        t: 2,
                    },
                },
                3: {
                    0: {
                        s: 'zDOqkI',
                        v: '华东',
                        t: 1,
                    },
                    1: {
                        s: 'zDOqkI',
                        v: '手机',
                        t: 1,
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: 'Q1',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
                        v: 8000,
                        t: 2,
                    },
                },
                4: {
                    0: {
                        s: 'zDOqkI',
                        v: '华北',
                        t: 1,
                    },
                    1: {
                        s: 'zDOqkI',
                        v: '笔记本电脑',
                        t: 1,
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: 'Q2',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
                        v: 18000,
                        t: 2,
                    },
                },
                5: {
                    0: {
                        s: 'zDOqkI',
                        v: '华北',
                        t: 1,
                    },
                    1: {
                        s: 'zDOqkI',
                        v: '手机',
                        t: 1,
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '线下门店',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: 'Q2',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
                        v: 6500,
                        t: 2,
                    },
                },
                6: {
                    0: {
                        s: 'zDOqkI',
                        v: '华南',
                        t: 1,
                    },
                    1: {
                        s: 'zDOqkI',
                        v: '手机',
                        t: 1,
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: 'Q3',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
                        v: 10000,
                        t: 2,
                    },
                },
                7: {
                    0: {
                        s: 'zDOqkI',
                        v: '华南',
                        t: 1,
                    },
                    1: {
                        s: 'zDOqkI',
                        v: '配件',
                        t: 1,
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '线下门店',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: 'Q3',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
                        v: 3000,
                        t: 2,
                    },
                },
                8: {
                    0: {
                        s: 'zDOqkI',
                        v: '华东',
                        t: 1,
                    },
                    1: {
                        s: 'zDOqkI',
                        v: '笔记本电脑',
                        t: 1,
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: 'Q4',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
                        v: 16500,
                        t: 2,
                    },
                },
                9: {
                    0: {
                        s: 'zDOqkI',
                        v: '华东',
                        t: 1,
                    },
                    1: {
                        s: 'zDOqkI',
                        v: '手机',
                        t: 1,
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '线下门店',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: 'Q4',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
                        v: 9000,
                        t: 2,
                    },
                },
                10: {
                    0: {
                        s: 'zDOqkI',
                        v: '华北',
                        t: 1,
                    },
                    1: {
                        s: 'zDOqkI',
                        v: '配件',
                        t: 1,
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: 'Q1',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
                        v: 2500,
                        t: 2,
                    },
                },
                11: {
                    0: {
                        s: 'zDOqkI',
                        v: '华南',
                        t: 1,
                    },
                    1: {
                        s: 'zDOqkI',
                        v: '笔记本电脑',
                        t: 1,
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: 'Q2',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
                        v: 14000,
                        t: 2,
                    },
                },
                12: {
                    0: {
                        s: 'zDOqkI',
                        v: '华北',
                        t: 1,
                    },
                    1: {
                        s: 'zDOqkI',
                        v: '手机',
                        t: 1,
                    },
                    2: {
                        s: 'zDOqkI',
                        v: '线下门店',
                        t: 1,
                    },
                    3: {
                        s: 'zDOqkI',
                        v: 'Q4',
                        t: 1,
                    },
                    4: {
                        s: 'zDOqkI',
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
                0: {
                    0: {
                        v: '',
                        s: '89ry-k',
                        t: 1,
                    },
                    1: {
                        v: '',
                        s: 'DwdEqC',
                        t: 1,
                    },
                    2: {
                        s: '8C3-ed',
                        v: 'Q1',
                        t: 1,
                    },
                    3: {
                        s: '8C3-ed',
                        v: '',
                        t: 1,
                    },
                    4: {
                        s: '8C3-ed',
                        v: '',
                        t: 1,
                    },
                    5: {
                        v: '',
                        t: 1,
                        s: '8C3-ed',
                    },
                    6: {
                        v: 'Q2',
                        t: 1,
                        s: '8C3-ed',
                    },
                    7: {
                        v: '',
                        t: 1,
                        s: '8C3-ed',
                    },
                    8: {
                        v: '',
                        t: 1,
                        s: '8C3-ed',
                    },
                    9: {
                        v: '',
                        t: 1,
                        s: '8C3-ed',
                    },
                    10: {
                        v: 'Q3',
                        t: 1,
                        s: '8C3-ed',
                    },
                    11: {
                        v: '',
                        t: 1,
                        s: '8C3-ed',
                    },
                    12: {
                        v: '',
                        t: 1,
                        s: '8C3-ed',
                    },
                    13: {
                        v: '',
                        t: 1,
                        s: '8C3-ed',
                    },
                    14: {
                        v: 'Q4',
                        t: 1,
                        s: '8C3-ed',
                    },
                    15: {
                        v: '',
                        t: 1,
                        s: '8C3-ed',
                    },
                    16: {
                        v: '',
                        t: 1,
                        s: '8C3-ed',
                    },
                    17: {
                        v: '',
                        t: 1,
                        s: 'U0-kpb',
                    },
                },
                1: {
                    0: {
                        v: '',
                        s: 'Eu96et',
                        t: 1,
                    },
                    1: {
                        v: '',
                        s: 'qgIyQ4',
                        t: 1,
                    },
                    2: {
                        s: 'n3IWnG',
                        v: '线上商城',
                        t: 1,
                    },
                    3: {
                        s: 'n3IWnG',
                        v: '',
                        t: 1,
                    },
                    4: {
                        s: 'n3IWnG',
                        v: '线下门店',
                        t: 1,
                    },
                    5: {
                        v: '',
                        t: 1,
                        s: 'n3IWnG',
                    },
                    6: {
                        v: '线上商城',
                        t: 1,
                        s: 'n3IWnG',
                    },
                    7: {
                        v: '',
                        t: 1,
                        s: 'n3IWnG',
                    },
                    8: {
                        v: '线下门店',
                        t: 1,
                        s: 'n3IWnG',
                    },
                    9: {
                        v: '',
                        t: 1,
                        s: 'n3IWnG',
                    },
                    10: {
                        v: '线上商城',
                        t: 1,
                        s: 'n3IWnG',
                    },
                    11: {
                        v: '',
                        t: 1,
                        s: 'n3IWnG',
                    },
                    12: {
                        v: '线下门店',
                        t: 1,
                        s: 'n3IWnG',
                    },
                    13: {
                        v: '',
                        t: 1,
                        s: 'n3IWnG',
                    },
                    14: {
                        v: '线上商城',
                        t: 1,
                        s: 'n3IWnG',
                    },
                    15: {
                        v: '',
                        t: 1,
                        s: 'n3IWnG',
                    },
                    16: {
                        v: '线下门店',
                        t: 1,
                        s: 'n3IWnG',
                    },
                    17: {
                        v: '',
                        t: 1,
                        s: '0zf8oD',
                    },
                },
                2: {
                    0: {
                        s: 'Eu96et',
                        v: '区域 (Region)',
                        t: 1,
                    },
                    1: {
                        s: 'qgIyQ4',
                        v: '产品类别 (Category)',
                        t: 1,
                    },
                    2: {
                        s: '_UrEAM',
                        v: 'Sum of 销售额 (Sales Amount)',
                        t: 1,
                    },
                    3: {
                        s: '_UrEAM',
                        v: 'Count of 产品类别 (Category)',
                        t: 1,
                    },
                    4: {
                        s: '_UrEAM',
                        v: 'Sum of 销售额 (Sales Amount)',
                        t: 1,
                    },
                    5: {
                        v: 'Count of 产品类别 (Category)',
                        t: 1,
                        s: '_UrEAM',
                    },
                    6: {
                        v: 'Sum of 销售额 (Sales Amount)',
                        t: 1,
                        s: '_UrEAM',
                    },
                    7: {
                        v: 'Count of 产品类别 (Category)',
                        t: 1,
                        s: '_UrEAM',
                    },
                    8: {
                        v: 'Sum of 销售额 (Sales Amount)',
                        t: 1,
                        s: '_UrEAM',
                    },
                    9: {
                        v: 'Count of 产品类别 (Category)',
                        t: 1,
                        s: '_UrEAM',
                    },
                    10: {
                        v: 'Sum of 销售额 (Sales Amount)',
                        t: 1,
                        s: '_UrEAM',
                    },
                    11: {
                        v: 'Count of 产品类别 (Category)',
                        t: 1,
                        s: '_UrEAM',
                    },
                    12: {
                        v: 'Sum of 销售额 (Sales Amount)',
                        t: 1,
                        s: '_UrEAM',
                    },
                    13: {
                        v: 'Count of 产品类别 (Category)',
                        t: 1,
                        s: '_UrEAM',
                    },
                    14: {
                        v: 'Sum of 销售额 (Sales Amount)',
                        t: 1,
                        s: '_UrEAM',
                    },
                    15: {
                        v: 'Count of 产品类别 (Category)',
                        t: 1,
                        s: '_UrEAM',
                    },
                    16: {
                        v: 'Sum of 销售额 (Sales Amount)',
                        t: 1,
                        s: '_UrEAM',
                    },
                    17: {
                        v: 'Count of 产品类别 (Category)',
                        t: 1,
                        s: 'jvfNbP',
                    },
                },
                3: {
                    0: {
                        s: 'Eu96et',
                        v: '华东',
                        t: 1,
                    },
                    1: {
                        s: 'qgIyQ4',
                        v: '手机',
                        t: 1,
                    },
                    2: {
                        s: '_UrEAM',
                        v: 8000,
                        t: 2,
                    },
                    3: {
                        s: '_UrEAM',
                        v: 1,
                        t: 2,
                    },
                    4: {
                        s: '_UrEAM',
                    },
                    5: {
                        s: '_UrEAM',
                    },
                    6: {
                        s: '_UrEAM',
                    },
                    7: {
                        s: '_UrEAM',
                    },
                    8: {
                        s: '_UrEAM',
                    },
                    9: {
                        s: '_UrEAM',
                    },
                    10: {
                        s: '_UrEAM',
                    },
                    11: {
                        s: '_UrEAM',
                    },
                    12: {
                        s: '_UrEAM',
                    },
                    13: {
                        s: '_UrEAM',
                    },
                    14: {
                        s: '_UrEAM',
                    },
                    15: {
                        s: '_UrEAM',
                    },
                    16: {
                        v: 9000,
                        t: 2,
                        s: '_UrEAM',
                    },
                    17: {
                        v: 1,
                        t: 2,
                        s: 'jvfNbP',
                    },
                },
                4: {
                    0: {
                        s: 'Eu96et',
                        v: '',
                        t: 1,
                    },
                    1: {
                        s: 'qgIyQ4',
                        v: '笔记本电脑',
                        t: 1,
                    },
                    2: {
                        s: '_UrEAM',
                        v: 15000,
                        t: 2,
                    },
                    3: {
                        s: '_UrEAM',
                        v: 1,
                        t: 2,
                    },
                    4: {
                        s: '_UrEAM',
                        v: 12000,
                        t: 2,
                    },
                    5: {
                        v: 1,
                        t: 2,
                        s: '_UrEAM',
                    },
                    6: {
                        s: '_UrEAM',
                    },
                    7: {
                        s: '_UrEAM',
                    },
                    8: {
                        s: '_UrEAM',
                    },
                    9: {
                        s: '_UrEAM',
                    },
                    10: {
                        s: '_UrEAM',
                    },
                    11: {
                        s: '_UrEAM',
                    },
                    12: {
                        s: '_UrEAM',
                    },
                    13: {
                        s: '_UrEAM',
                    },
                    14: {
                        v: 16500,
                        t: 2,
                        s: '_UrEAM',
                    },
                    15: {
                        v: 1,
                        t: 2,
                        s: '_UrEAM',
                    },
                    16: {
                        s: '_UrEAM',
                    },
                    17: {
                        s: 'jvfNbP',
                    },
                },
                5: {
                    0: {
                        v: '华北',
                        t: 1,
                        s: 'Eu96et',
                    },
                    1: {
                        v: '手机',
                        t: 1,
                        s: 'qgIyQ4',
                    },
                    2: {
                        s: '_UrEAM',
                    },
                    3: {
                        s: '_UrEAM',
                    },
                    4: {
                        s: '_UrEAM',
                    },
                    5: {
                        s: '_UrEAM',
                    },
                    6: {
                        s: '_UrEAM',
                    },
                    7: {
                        s: '_UrEAM',
                    },
                    8: {
                        v: 6500,
                        t: 2,
                        s: '_UrEAM',
                    },
                    9: {
                        v: 1,
                        t: 2,
                        s: '_UrEAM',
                    },
                    10: {
                        s: '_UrEAM',
                    },
                    11: {
                        s: '_UrEAM',
                    },
                    12: {
                        s: '_UrEAM',
                    },
                    13: {
                        s: '_UrEAM',
                    },
                    14: {
                        s: '_UrEAM',
                    },
                    15: {
                        s: '_UrEAM',
                    },
                    16: {
                        v: 7200,
                        t: 2,
                        s: '_UrEAM',
                    },
                    17: {
                        v: 1,
                        t: 2,
                        s: 'jvfNbP',
                    },
                },
                6: {
                    0: {
                        v: '',
                        t: 1,
                        s: 'Eu96et',
                    },
                    1: {
                        v: '笔记本电脑',
                        t: 1,
                        s: 'qgIyQ4',
                    },
                    2: {
                        s: '_UrEAM',
                    },
                    3: {
                        s: '_UrEAM',
                    },
                    4: {
                        s: '_UrEAM',
                    },
                    5: {
                        s: '_UrEAM',
                    },
                    6: {
                        v: 18000,
                        t: 2,
                        s: '_UrEAM',
                    },
                    7: {
                        v: 1,
                        t: 2,
                        s: '_UrEAM',
                    },
                    8: {
                        s: '_UrEAM',
                    },
                    9: {
                        s: '_UrEAM',
                    },
                    10: {
                        s: '_UrEAM',
                    },
                    11: {
                        s: '_UrEAM',
                    },
                    12: {
                        s: '_UrEAM',
                    },
                    13: {
                        s: '_UrEAM',
                    },
                    14: {
                        s: '_UrEAM',
                    },
                    15: {
                        s: '_UrEAM',
                    },
                    16: {
                        s: '_UrEAM',
                    },
                    17: {
                        s: 'jvfNbP',
                    },
                },
                7: {
                    0: {
                        v: '',
                        t: 1,
                        s: 'Eu96et',
                    },
                    1: {
                        v: '配件',
                        t: 1,
                        s: 'qgIyQ4',
                    },
                    2: {
                        v: 2500,
                        t: 2,
                        s: '_UrEAM',
                    },
                    3: {
                        v: 1,
                        t: 2,
                        s: '_UrEAM',
                    },
                    4: {
                        s: '_UrEAM',
                    },
                    5: {
                        s: '_UrEAM',
                    },
                    6: {
                        s: '_UrEAM',
                    },
                    7: {
                        s: '_UrEAM',
                    },
                    8: {
                        s: '_UrEAM',
                    },
                    9: {
                        s: '_UrEAM',
                    },
                    10: {
                        s: '_UrEAM',
                    },
                    11: {
                        s: '_UrEAM',
                    },
                    12: {
                        s: '_UrEAM',
                    },
                    13: {
                        s: '_UrEAM',
                    },
                    14: {
                        s: '_UrEAM',
                    },
                    15: {
                        s: '_UrEAM',
                    },
                    16: {
                        s: '_UrEAM',
                    },
                    17: {
                        s: 'jvfNbP',
                    },
                },
                8: {
                    0: {
                        v: '华南',
                        t: 1,
                        s: 'Eu96et',
                    },
                    1: {
                        v: '手机',
                        t: 1,
                        s: 'qgIyQ4',
                    },
                    2: {
                        s: '_UrEAM',
                    },
                    3: {
                        s: '_UrEAM',
                    },
                    4: {
                        s: '_UrEAM',
                    },
                    5: {
                        s: '_UrEAM',
                    },
                    6: {
                        s: '_UrEAM',
                    },
                    7: {
                        s: '_UrEAM',
                    },
                    8: {
                        s: '_UrEAM',
                    },
                    9: {
                        s: '_UrEAM',
                    },
                    10: {
                        v: 10000,
                        t: 2,
                        s: '_UrEAM',
                    },
                    11: {
                        v: 1,
                        t: 2,
                        s: '_UrEAM',
                    },
                    12: {
                        s: '_UrEAM',
                    },
                    13: {
                        s: '_UrEAM',
                    },
                    14: {
                        s: '_UrEAM',
                    },
                    15: {
                        s: '_UrEAM',
                    },
                    16: {
                        s: '_UrEAM',
                    },
                    17: {
                        s: 'jvfNbP',
                    },
                },
                9: {
                    0: {
                        v: '',
                        t: 1,
                        s: 'Eu96et',
                    },
                    1: {
                        v: '笔记本电脑',
                        t: 1,
                        s: 'qgIyQ4',
                    },
                    2: {
                        s: '_UrEAM',
                    },
                    3: {
                        s: '_UrEAM',
                    },
                    4: {
                        s: '_UrEAM',
                    },
                    5: {
                        s: '_UrEAM',
                    },
                    6: {
                        v: 14000,
                        t: 2,
                        s: '_UrEAM',
                    },
                    7: {
                        v: 1,
                        t: 2,
                        s: '_UrEAM',
                    },
                    8: {
                        s: '_UrEAM',
                    },
                    9: {
                        s: '_UrEAM',
                    },
                    10: {
                        s: '_UrEAM',
                    },
                    11: {
                        s: '_UrEAM',
                    },
                    12: {
                        s: '_UrEAM',
                    },
                    13: {
                        s: '_UrEAM',
                    },
                    14: {
                        s: '_UrEAM',
                    },
                    15: {
                        s: '_UrEAM',
                    },
                    16: {
                        s: '_UrEAM',
                    },
                    17: {
                        s: 'jvfNbP',
                    },
                },
                10: {
                    0: {
                        v: '',
                        t: 1,
                        s: 'TvIrpW',
                    },
                    1: {
                        v: '配件',
                        t: 1,
                        s: 'ukd3PM',
                    },
                    2: {
                        s: 'Ao_Tif',
                    },
                    3: {
                        s: 'Ao_Tif',
                    },
                    4: {
                        s: 'Ao_Tif',
                    },
                    5: {
                        s: 'Ao_Tif',
                    },
                    6: {
                        s: 'Ao_Tif',
                    },
                    7: {
                        s: 'Ao_Tif',
                    },
                    8: {
                        s: 'Ao_Tif',
                    },
                    9: {
                        s: 'Ao_Tif',
                    },
                    10: {
                        s: 'Ao_Tif',
                    },
                    11: {
                        s: 'Ao_Tif',
                    },
                    12: {
                        v: 3000,
                        t: 2,
                        s: 'Ao_Tif',
                    },
                    13: {
                        v: 1,
                        t: 2,
                        s: 'Ao_Tif',
                    },
                    14: {
                        s: 'Ao_Tif',
                    },
                    15: {
                        s: 'Ao_Tif',
                    },
                    16: {
                        s: 'Ao_Tif',
                    },
                    17: {
                        s: '-P13NE',
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
            data: '{"pivotTableConfigs":{"H1p5SR":{"M2HmbxqfnSZky1RnARn0g":{"lUnl6N2A_mRfVgOyQPn6m":{"id":"lUnl6N2A_mRfVgOyQPn6m","name":"Pivot_lUnl6N2A_mRfVgOyQPn6m","sourceRangeInfo":{"range":{"startRow":0,"startColumn":0,"endRow":12,"endColumn":4,"startAbsoluteRefType":0,"endAbsoluteRefType":0,"rangeType":0},"subUnitId":"ZCGh3uiogN11CwYQlPhHo","unitId":"H1p5SR"},"targetCellInfo":{"row":0,"col":0,"subUnitId":"M2HmbxqfnSZky1RnARn0g","unitId":"H1p5SR"},"fieldsConfig":{"valueFields":[{"id":"xXBDgV_[H1p5SR]ZCGh3uiogN11CwYQlPhHo!E1:E13","name":"销售额 (Sales Amount)","sourceColumnIndex":4},{"id":"EBNK6H_[H1p5SR]ZCGh3uiogN11CwYQlPhHo!B1:B13","name":"产品类别 (Category)","sourceColumnIndex":1,"aggregation":"count"}],"rowFields":[{"id":"bSIqZ1_[H1p5SR]ZCGh3uiogN11CwYQlPhHo!A1:A13","name":"区域 (Region)","sourceColumnIndex":0},{"id":"K0WSjk_[H1p5SR]ZCGh3uiogN11CwYQlPhHo!B1:B13","name":"产品类别 (Category)","sourceColumnIndex":1}],"columnFields":[{"id":"mwBIQp_[H1p5SR]ZCGh3uiogN11CwYQlPhHo!D1:D13","name":"季度 (Quarter)","sourceColumnIndex":3},{"id":"czXb6u_[H1p5SR]ZCGh3uiogN11CwYQlPhHo!C1:C13","name":"销售渠道 (Channel)","sourceColumnIndex":2}],"filterFields":[],"valuePosition":1}}}}}}',
        },
    ],
});

window.univer = univer;
window.univerAPI = FUniver.newAPI(univer);
