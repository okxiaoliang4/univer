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
            insert: 'Сводная таблица',
        },
        dialog: {
            createTitle: 'Создать сводную таблицу',
            sourceRangeLabel: 'Выберите диапазон исходных данных',
            targetRangeLabel: 'Выберите место размещения сводной таблицы',
            targetRangeTypeNew: 'Новый лист',
            targetRangeTypeExisting: 'Существующий лист',
            sourceRangeSingleCellError: 'Исходный диапазон должен содержать более одной ячейки',
            sourceRangeSingleRowError: 'Исходный диапазон должен содержать хотя бы две строки (заголовок и данные)',
            sourceRangeWithMergeError: 'Исходный диапазон не может пересекаться с объединенными ячейками',
            targetRangeSingleCellError: 'Целевое местоположение должно быть одной ячейкой',
            invalidWorksheet: 'Недействительный рабочий лист',
            cancel: 'Отменить',
            confirm: 'ОК',
        },
        editor: {
            title: 'Поля сводной таблицы',
            availableFields: 'Доступные поля',
            filters: 'Фильтры',
            columns: 'Столбцы',
            rows: 'Строки',
            values: 'Значения',
            dragFieldsHere: 'Перетащите поля сюда',
            remove: 'Удалить',
            add: 'Добавить',
            aggregationMethodLabel: 'Метод агрегации',
            aggregationType: {
                sum: 'Сумма',
                count: 'Количество',
                average: 'Среднее',
                min: 'Минимум',
                max: 'Максимум',
            },
            containerLabel: 'Столбец {0}',
            containerLabels: {
                sourceFields: 'Доступные поля',
                filterFields: 'Фильтры',
                columnFields: 'Столбцы',
                rowFields: 'Строки',
                valueFields: 'Значения',
            },
        },
        panel: {
            sourceRangeLabel: 'Исходный диапазон:',
            fieldConfigurationLabel: 'Конфигурация полей:',
            valuePositionLabel: 'Позиция значения:',
            valuePositionRow: 'Строка',
            valuePositionColumn: 'Столбец',
        },
    },
};

export default locale;
