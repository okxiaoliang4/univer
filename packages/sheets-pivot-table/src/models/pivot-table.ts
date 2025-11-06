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

import type { ICellData, IObjectMatrixPrimitiveType, IRange, Nullable, Workbook } from '@univerjs/core';
import type { IFieldsConfig, ISourceRangeInfo, ITargetCellInfo } from '../types/type';
import { Disposable, ObjectMatrix } from '@univerjs/core';
import { Pivot } from './pivot';

/**
 * Simplified PivotTable implementation for MVP
 * This class handles the core pivot table calculation logic
 */
export class PivotTable extends Disposable {
    private _id: string;
    private _name: string;
    private _sourceRangeInfo: ISourceRangeInfo;
    private _targetCellInfo: ITargetCellInfo;
    private _fieldsConfig: IFieldsConfig;

    private _pivot: Pivot;

    constructor(
        id: string,
        name: string,
        sourceRangeInfo: ISourceRangeInfo,
        targetCellInfo: ITargetCellInfo,
        fieldsConfig: IFieldsConfig
    ) {
        super();
        this._id = id;
        this._name = name;
        this._sourceRangeInfo = sourceRangeInfo;
        this._targetCellInfo = targetCellInfo;
        this._fieldsConfig = fieldsConfig;

        this._pivot = new Pivot({
            valueFields: fieldsConfig.valueFields,
            rowFields: fieldsConfig.rowFields,
            columnFields: fieldsConfig.columnFields,
            filterFields: fieldsConfig.filterFields,
            sourceData: {},
        });

        this.disposeWithMe(this._pivot);
    }

    getId(): string {
        return this._id;
    }

    getName(): string {
        return this._name;
    }

    setName(name: string): void {
        this._name = name;
    }

    getSourceRangeInfo(): ISourceRangeInfo {
        return this._sourceRangeInfo;
    }

    setSourceRangeInfo(sourceRangeInfo: ISourceRangeInfo): void {
        this._sourceRangeInfo = sourceRangeInfo;
    }

    getTargetCellInfo(): ITargetCellInfo {
        return this._targetCellInfo;
    }

    setTargetCellInfo(targetCellInfo: ITargetCellInfo): void {
        this._targetCellInfo = targetCellInfo;
    }

    getFieldsConfig(): IFieldsConfig {
        return this._fieldsConfig;
    }

    setFieldsConfig(fieldsConfig: IFieldsConfig): void {
        this._fieldsConfig = fieldsConfig;
    }

    getValueFields(): string[] {
        return this._pivot.getValueFields();
    }

    getRowFields(): string[] {
        return this._pivot.getRowFields();
    }

    getColumnFields(): string[] {
        return this._pivot.getColumnFields();
    }

    getFilterFields(): string[] {
        return this._pivot.getFilterFields();
    }

    setValueFields(valueFields: string[]): void {
        this._pivot.setValueFields(valueFields);
    }

    setRowFields(rowFields: string[]): void {
        this._pivot.setRowFields(rowFields);
    }

    setColumnFields(columnFields: string[]): void {
        this._pivot.setColumnFields(columnFields);
    }

    setFilters(filters: string[]): void {
        this._pivot.setFilters(filters);
    }

    private _moveMatrix(matrix: IObjectMatrixPrimitiveType<Nullable<ICellData>>, targetCellInfo: ITargetCellInfo): ObjectMatrix<Nullable<ICellData>> {
        const targetObjectMatrix = new ObjectMatrix<Nullable<ICellData>>();
        new ObjectMatrix(matrix).forValue((row, col, value) => {
            targetObjectMatrix.setValue(row + targetCellInfo.row, col + targetCellInfo.col, value);
        });
        return targetObjectMatrix;
    }

    /**
     * Get the output range of the pivot table based on calculated data
     * Returns the range from target cell to the end of calculated output (including grand totals)
     * @returns Output range or null if not calculated yet
     */
    getOutputRange(): IRange | null {
        const calculatedData = this._pivot.getCalculatedData();
        if (!calculatedData) {
            return null;
        }
        return this._moveMatrix(calculatedData, this._targetCellInfo).getDataRange();
    }

    /**
     * Generate full cell matrix for pivot table output
     * Includes headers, values, and totals in the correct layout
     * @returns ObjectMatrix with all cell values positioned relative to target cell, or null if not calculated
     */
    getOutputCellMatrix(): IObjectMatrixPrimitiveType<Nullable<ICellData>> | null {
        const targetMatrix = this._pivot.getCalculatedData();
        if (!targetMatrix) {
            return null;
        }
        return this._moveMatrix(targetMatrix, this._targetCellInfo).getMatrix();
    }

    /**
     * Calculate pivot table data from source worksheet
     * @param workbook - The workbook containing source data
     * @returns Calculated pivot table data
     */
    calculate(workbook: Workbook) {
        const worksheet = workbook.getSheetBySheetId(this._sourceRangeInfo.subUnitId);
        if (!worksheet) {
            return null;
        }

        const range = worksheet.getRange(this._sourceRangeInfo.range);
        const matrix = range.getMatrix().getMatrix();

        this._pivot.setSourceData(matrix);
        return this.getOutputCellMatrix();
    }

    /**
     * Serialize to JSON
     */
    toJSON() {
        return {
            id: this._id,
            name: this._name,
            sourceRangeInfo: this._sourceRangeInfo,
            targetCellInfo: this._targetCellInfo,
            fieldsConfig: this._fieldsConfig,
        };
    }

    /**
     * Create from JSON
     */
    static fromJSON(json: {
        id: string;
        name: string;
        sourceRangeInfo: ISourceRangeInfo;
        targetCellInfo: ITargetCellInfo;
        fieldsConfig: IFieldsConfig;
    }): PivotTable {
        return new PivotTable(
            json.id,
            json.name,
            json.sourceRangeInfo,
            json.targetCellInfo,
            json.fieldsConfig
        );
    }
}
