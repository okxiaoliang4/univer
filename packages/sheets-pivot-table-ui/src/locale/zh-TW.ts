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

import type enUS from './en-US';

const locale: typeof enUS = {
    pivotTable: {
        menu: {
            insert: '資料透視表',
        },
        dialog: {
            createTitle: '建立資料透視表',
            sourceRangeLabel: '選擇來源資料範圍',
            targetRangeLabel: '選擇資料透視表放置位置',
            targetRangeTypeNew: '新工作表',
            targetRangeTypeExisting: '現有工作表',
            sourceRangeSingleCellError: '來源範圍必須包含多個儲存格',
            sourceRangeSingleRowError: '來源範圍必須至少包含兩行(標題和資料)',
            sourceRangeWithMergeError: '來源範圍不能與合併儲存格重疊',
            targetRangeSingleCellError: '目標位置必須是單一儲存格',
            invalidWorksheet: '無效的工作表',
            cancel: '取消',
            confirm: '確定',
        },
        editor: {
            title: '資料透視表欄位',
            availableFields: '可用欄位',
            filters: '篩選',
            columns: '欄',
            rows: '列',
            values: '值',
            dragFieldsHere: '將欄位拖到此處',
            remove: '移除',
            add: '新增',
            aggregationMethodLabel: '彙總方式',
            aggregationType: {
                sum: '求和',
                count: '計數',
                average: '平均值',
                min: '最小值',
                max: '最大值',
            },
            containerLabel: '欄 {0}',
            containerLabels: {
                sourceFields: '可用欄位',
                filterFields: '篩選',
                columnFields: '欄',
                rowFields: '列',
                valueFields: '值',
            },
        },
        panel: {
            sourceRangeLabel: '來源範圍:',
            fieldConfigurationLabel: '欄位設定:',
            valuePositionLabel: '值位置:',
            valuePositionRow: '列',
            valuePositionColumn: '欄',
        },
    },
};

export default locale;
