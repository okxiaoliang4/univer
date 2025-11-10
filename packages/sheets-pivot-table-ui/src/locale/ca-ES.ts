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
            insert: 'Taula dinàmica',
        },
        dialog: {
            createTitle: 'Crear taula dinàmica',
            sourceRangeLabel: 'Selecciona el rang de dades font',
            targetRangeLabel: 'Tria on col·locar la taula dinàmica',
            targetRangeTypeNew: 'Full nou',
            targetRangeTypeExisting: 'Full existent',
            sourceRangeSingleCellError: 'El rang font ha de contenir més d\'una cel·la',
            sourceRangeSingleRowError: 'El rang font ha de contenir almenys dues files (capçalera i dades)',
            sourceRangeWithMergeError: 'El rang font no pot solapar-se amb cel·les fusionades',
            targetRangeSingleCellError: 'La ubicació de destinació ha de ser una sola cel·la',
            invalidWorksheet: 'Full de treball no vàlid',
            cancel: 'Cancel·lar',
            confirm: 'D\'acord',
        },
        editor: {
            title: 'Camps de la taula dinàmica',
            availableFields: 'Camps disponibles',
            filters: 'Filtres',
            columns: 'Columnes',
            rows: 'Files',
            values: 'Valors',
            dragFieldsHere: 'Arrossega els camps aquí',
            remove: 'Eliminar',
            aggregationMethodLabel: 'Mètode d\'agregació',
            aggregationType: {
                sum: 'Suma',
                count: 'Comptar',
                average: 'Mitjana',
                min: 'Mínim',
                max: 'Màxim',
            },
            containerLabel: 'Columna {0}',
            containerLabels: {
                sourceFields: 'Camps disponibles',
                filterFields: 'Filtres',
                columnFields: 'Columnes',
                rowFields: 'Files',
                valueFields: 'Valors',
            },
        },
        panel: {
            sourceRangeLabel: 'Rang font:',
            fieldConfigurationLabel: 'Configuració de camps:',
            valuePositionLabel: 'Posició del valor:',
            valuePositionRow: 'Fila',
            valuePositionColumn: 'Columna',
        },
    },
};

export default locale;
