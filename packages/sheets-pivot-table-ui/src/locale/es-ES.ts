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
            insert: 'Tabla dinámica',
        },
        dialog: {
            createTitle: 'Crear tabla dinámica',
            sourceRangeLabel: 'Selecciona el rango de datos fuente',
            targetRangeLabel: 'Elige dónde colocar la tabla dinámica',
            targetRangeTypeNew: 'Nueva hoja',
            targetRangeTypeExisting: 'Hoja existente',
            sourceRangeSingleCellError: 'El rango fuente debe contener más de una celda',
            sourceRangeSingleRowError: 'El rango fuente debe contener al menos dos filas (encabezado y datos)',
            sourceRangeWithMergeError: 'El rango fuente no puede solaparse con celdas fusionadas',
            targetRangeSingleCellError: 'La ubicación de destino debe ser una sola celda',
            invalidWorksheet: 'Hoja de trabajo no válida',
            cancel: 'Cancelar',
            confirm: 'Aceptar',
        },
        editor: {
            title: 'Campos de tabla dinámica',
            availableFields: 'Campos disponibles',
            filters: 'Filtros',
            columns: 'Columnas',
            rows: 'Filas',
            values: 'Valores',
            dragFieldsHere: 'Arrastra los campos aquí',
            remove: 'Eliminar',
            aggregationMethodLabel: 'Método de agregación',
            aggregationType: {
                sum: 'Suma',
                count: 'Contar',
                average: 'Promedio',
                min: 'Mínimo',
                max: 'Máximo',
            },
            containerLabel: 'Columna {0}',
            containerLabels: {
                sourceFields: 'Campos disponibles',
                filterFields: 'Filtros',
                columnFields: 'Columnas',
                rowFields: 'Filas',
                valueFields: 'Valores',
            },
        },
        panel: {
            sourceRangeLabel: 'Rango fuente:',
            fieldConfigurationLabel: 'Configuración de campos:',
            valuePositionLabel: 'Posición del valor:',
            valuePositionRow: 'Fila',
            valuePositionColumn: 'Columna',
        },
    },
};

export default locale;
