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
            insert: 'جدول محوری',
        },
        dialog: {
            createTitle: 'ایجاد جدول محوری',
            sourceRangeLabel: 'انتخاب محدوده داده منبع',
            targetRangeLabel: 'انتخاب محل قرارگیری جدول محوری',
            targetRangeTypeNew: 'ورق جدید',
            targetRangeTypeExisting: 'ورق موجود',
            sourceRangeSingleCellError: 'محدوده منبع باید شامل بیش از یک سلول باشد',
            sourceRangeSingleRowError: 'محدوده منبع باید حداقل شامل دو ردیف (هدر و داده) باشد',
            sourceRangeWithMergeError: 'محدوده منبع نمی‌تواند با سلول‌های ادغام شده همپوشانی داشته باشد',
            targetRangeSingleCellError: 'مکان هدف باید یک سلول واحد باشد',
            invalidWorksheet: 'ورق نامعتبر',
            cancel: 'انصراف',
            confirm: 'تایید',
        },
        editor: {
            title: 'فیلدهای جدول محوری',
            availableFields: 'فیلدهای موجود',
            filters: 'فیلترها',
            columns: 'ستون‌ها',
            rows: 'ردیف‌ها',
            values: 'مقادیر',
            dragFieldsHere: 'فیلدها را اینجا بکشید',
            remove: 'حذف',
            add: 'اضافه کردن',
            aggregationMethodLabel: 'روش تجمیع',
            aggregationType: {
                sum: 'جمع',
                count: 'شمارش',
                average: 'میانگین',
                min: 'حداقل',
                max: 'حداکثر',
            },
            containerLabel: 'ستون {0}',
            containerLabels: {
                sourceFields: 'فیلدهای موجود',
                filterFields: 'فیلترها',
                columnFields: 'ستون‌ها',
                rowFields: 'ردیف‌ها',
                valueFields: 'مقادیر',
            },
        },
        panel: {
            sourceRangeLabel: 'محدوده منبع:',
            fieldConfigurationLabel: 'پیکربندی فیلد:',
            valuePositionLabel: 'موقعیت مقدار:',
            valuePositionRow: 'ردیف',
            valuePositionColumn: 'ستون',
        },
    },
};

export default locale;
