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

const locale = {
    pivotTable: {
        menu: {
            insert: '数据透视表',
        },
        dialog: {
            createTitle: '创建数据透视表',
            sourceRangeLabel: '选择源数据范围',
            targetRangeLabel: '选择数据透视表放置位置',
            sourceRangeSingleCellError: '源范围必须包含多个单元格',
            sourceRangeSingleRowError: '源范围必须至少包含两行(标题和数据)',
            sourceRangeWithMergeError: '源范围不能与合并单元格重叠',
            targetRangeSingleCellError: '目标位置必须是单个单元格',
            invalidWorksheet: '无效的工作表',
            cancel: '取消',
            confirm: '确定',
        },
        editor: {
            title: '透视表字段',
            availableFields: '可用字段',
            filters: '筛选',
            columns: '列',
            rows: '行',
            values: '值',
            dragFieldsHere: '将字段拖到此处',
            remove: '移除',
        },
    },
};

export default locale;
