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
            insert: 'ピボットテーブル',
        },
        dialog: {
            createTitle: 'ピボットテーブルの作成',
            sourceRangeLabel: '元データ範囲を選択',
            targetRangeLabel: 'ピボットテーブルを配置する場所を選択',
            targetRangeTypeNew: '新しいシート',
            targetRangeTypeExisting: '既存のシート',
            sourceRangeSingleCellError: '元データ範囲は複数のセルを含める必要があります',
            sourceRangeSingleRowError: '元データ範囲は少なくとも2行（ヘッダーとデータ）を含める必要があります',
            sourceRangeWithMergeError: '元データ範囲は結合されたセルと重なってはいけません',
            targetRangeSingleCellError: '配置場所は単一のセルである必要があります',
            invalidWorksheet: '無効なワークシート',
            cancel: 'キャンセル',
            confirm: 'OK',
        },
        editor: {
            title: 'ピボットテーブルのフィールド',
            availableFields: '利用可能なフィールド',
            filters: 'フィルター',
            columns: '列',
            rows: '行',
            values: '値',
            dragFieldsHere: 'ここにフィールドをドラッグ',
            remove: '削除',
            add: '追加',
            aggregationMethodLabel: '集計方法',
            aggregationType: {
                sum: '合計',
                count: 'カウント',
                average: '平均',
                min: '最小値',
                max: '最大値',
            },
            containerLabel: '列 {0}',
            containerLabels: {
                sourceFields: '利用可能なフィールド',
                filterFields: 'フィルター',
                columnFields: '列',
                rowFields: '行',
                valueFields: '値',
            },
        },
        panel: {
            sourceRangeLabel: '元データ範囲:',
            fieldConfigurationLabel: 'フィールドの設定:',
        },
    },
};

export default locale;
