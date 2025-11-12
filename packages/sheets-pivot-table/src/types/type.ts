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

import type { IRange } from '@univerjs/core';
import type { PivotValuePosition } from '../models/pivot-engine';
import type { AggregationType } from './enum';

/**
 * Source fields configuration for pivot table
 */
export interface ISourceFields {
    /** Column index in the source range */
    sourceColumnIndex: number;
    /** Range key */
    rangeKey: string;
}

/**
 * Source range information for pivot table data
 */
export interface ISourceRangeInfo {
    /** Workbook unit ID */
    unitId: string;
    /** Worksheet subunit ID */
    subUnitId: string;
    /** Data range */
    range: IRange;
}

/**
 * Target cell information for pivot table output
 */
export interface ITargetCellInfo {
    /** Workbook unit ID */
    unitId: string;
    /** Worksheet subunit ID */
    subUnitId: string;
    /** Starting row */
    row: number;
    /** Starting column */
    col: number;
}

/**
 * Pivot field configuration
 */
export interface IPivotField {
    /** Field unique identifier */
    id: string;
    /** Source column index */
    sourceColumnIndex: number;
    /** Field display name */
    name: string;
    /** Field area (row, column, value, filter) */
    // area?: PivotFieldAreaType;
    /** Aggregation function (for value fields) */
    aggregation?: AggregationType;
    /** Filter criteria (for filter fields) */
    filter?: IPivotFilterCriteria;
}

/**
 * Filter criteria for pivot fields
 */
export interface IPivotFilterCriteria {
    /** Filter type */
    type: 'value' | 'condition';
    /** Selected values (for value filter) */
    values?: string[];
    /** Condition operator (for condition filter) */
    operator?: 'equals' | 'notEquals' | 'greaterThan' | 'lessThan' | 'contains';
    /** Condition value */
    conditionValue?: string | number;
}

/**
 * Fields configuration for pivot table
 */
export interface IFieldsConfig {
    valueFields: IPivotField[];
    rowFields: IPivotField[];
    columnFields: IPivotField[];
    filterFields: IPivotField[];
    /** Value position (Column or Row) */
    valuePosition: PivotValuePosition;
}

/**
 * Source configuration for pivot table
 */
export interface ISourceConfig {
    sourceFields: IPivotField[];
}

/**
 * Complete pivot table configuration
 */
export interface IPivotTableConfig {
    /** Pivot table ID */
    id: string;
    /** Pivot table name */
    name: string;
    /** Source range information */
    sourceRangeInfo: ISourceRangeInfo;
    /** Target cell information */
    targetCellInfo: ITargetCellInfo;
    /** Fields configuration */
    fieldsConfig: IFieldsConfig;
}

/**
 * Pivot table configuration resource for serialization
 */
export interface IPivotTableConfigResource {
    /** Pivot table configurations by unitId -> subUnitId -> pivotTableId */
    pivotTableConfigs: Record<string, Record<string, Record<string, IPivotTableConfig>>>;
}

/**
 * Calculated pivot table data
 */
export interface IPivotTableCalculatedData {
    /** Row headers (multi-dimensional array for multiple row fields) */
    rowHeaders: string[][];
    /** Column headers (multi-dimensional array for multiple column fields) */
    columnHeaders: string[][];
    /** Data values matrix */
    values: (number | string | null)[][];
    /** Grand total row (optional) */
    grandTotalRow?: (number | string | null)[];
    /** Grand total column (optional) */
    grandTotalColumn?: (number | string | null)[];
}

/**
 * Event payload for pivot table output range change
 * This represents the actual range occupied by the pivot table output,
 * starting from targetCell and extending based on calculated data dimensions
 */
export interface IPivotTableRangeChangedEvent {
    /** Workbook unit ID */
    unitId: string;
    /** Worksheet subunit ID */
    subUnitId: string;
    /** Pivot table ID */
    tableId: string;
    /** Pivot table output range (from targetCell to end of calculated output) */
    range: IRange;
}

/**
 * Event payload for pivot table source range change
 */
export interface IPivotTableSourceRangeChangedEvent {
    /** Workbook unit ID */
    unitId: string;
    /** Worksheet subunit ID */
    subUnitId: string;
    /** Pivot table ID */
    tableId: string;
    /** New source range info */
    sourceRangeInfo: ISourceRangeInfo;
    /** Old source range info (if exists) */
    oldSourceRangeInfo?: ISourceRangeInfo;
}

/**
 * Event payload for pivot table target cell change
 */
export interface IPivotTableTargetCellChangedEvent {
    /** Workbook unit ID */
    unitId: string;
    /** Worksheet subunit ID */
    subUnitId: string;
    /** Pivot table ID */
    tableId: string;
    /** New target cell info */
    targetCellInfo: ITargetCellInfo;
}

/**
 * Event payload for pivot table fields config change
 */
export interface IPivotTableFieldsConfigChangedEvent {
    /** Workbook unit ID */
    unitId: string;
    /** Worksheet subunit ID */
    subUnitId: string;
    /** Pivot table ID */
    tableId: string;
    /** New fields config */
    fieldsConfig: IFieldsConfig;
}
