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
            insert: 'Bảng tổng hợp',
        },
        dialog: {
            createTitle: 'Tạo bảng tổng hợp',
            sourceRangeLabel: 'Chọn phạm vi dữ liệu nguồn',
            targetRangeLabel: 'Chọn vị trí đặt bảng tổng hợp',
            targetRangeTypeNew: 'Sheet mới',
            targetRangeTypeExisting: 'Sheet hiện có',
            sourceRangeSingleCellError: 'Phạm vi nguồn phải chứa nhiều hơn một ô',
            sourceRangeSingleRowError: 'Phạm vi nguồn phải chứa ít nhất hai hàng (tiêu đề và dữ liệu)',
            sourceRangeWithMergeError: 'Phạm vi nguồn không thể chồng lên các ô đã hợp nhất',
            targetRangeSingleCellError: 'Vị trí đích phải là một ô duy nhất',
            invalidWorksheet: 'Bảng tính không hợp lệ',
            cancel: 'Hủy',
            confirm: 'OK',
        },
        editor: {
            title: 'Trường bảng tổng hợp',
            availableFields: 'Trường có sẵn',
            filters: 'Bộ lọc',
            columns: 'Cột',
            rows: 'Hàng',
            values: 'Giá trị',
            dragFieldsHere: 'Kéo các trường vào đây',
            remove: 'Xóa',
            add: 'Thêm',
            aggregationMethodLabel: 'Phương thức tổng hợp',
            aggregationType: {
                sum: 'Tổng',
                count: 'Đếm',
                average: 'Trung bình',
                min: 'Tối thiểu',
                max: 'Tối đa',
            },
            containerLabel: 'Cột {0}',
            containerLabels: {
                sourceFields: 'Trường có sẵn',
                filterFields: 'Bộ lọc',
                columnFields: 'Cột',
                rowFields: 'Hàng',
                valueFields: 'Giá trị',
            },
        },
        panel: {
            sourceRangeLabel: 'Phạm vi nguồn:',
            fieldConfigurationLabel: 'Cấu hình trường:',
            valuePositionLabel: 'Vị trí giá trị:',
            valuePositionRow: 'Hàng',
            valuePositionColumn: 'Cột',
        },
    },
};

export default locale;
