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
import type { Observable } from 'rxjs';
import type { IFieldsConfig, IPivotField, IPivotTableConfig, ISourceRangeInfo, ITargetCellInfo } from '../types/type';
import type { PivotValuePosition } from './pivot-engine';
import { Disposable, ObjectMatrix } from '@univerjs/core';
import { BehaviorSubject, combineLatest, debounceTime, distinctUntilChanged } from 'rxjs';
import { defaultPlaceholderMatrix } from '../common/default-pivot-table';
import { PivotEngine } from './pivot-engine';

/**
 * Simplified PivotTable implementation for MVP
 * This class handles the core pivot table calculation logic
 */
export class PivotTable extends Disposable {
    private _id: string;
    private _name: string;
    private _sourceRangeInfo$: BehaviorSubject<ISourceRangeInfo>;
    private _targetCellInfo: ITargetCellInfo;

    private _pivotEngine: PivotEngine;

    private _valueFields$: BehaviorSubject<IPivotField[]>;
    private _rowFields$: BehaviorSubject<IPivotField[]>;
    private _columnFields$: BehaviorSubject<IPivotField[]>;
    private _filterFields$: BehaviorSubject<IPivotField[]>;
    private _valuePosition$: BehaviorSubject<PivotValuePosition>;

    private _sourceData$: BehaviorSubject<IObjectMatrixPrimitiveType<Nullable<ICellData>>>;
    private _calculatedData$: BehaviorSubject<IObjectMatrixPrimitiveType<Nullable<ICellData>> | null>;

    sourceRangeInfo$: Observable<ISourceRangeInfo>;
    valueFields$: Observable<IPivotField[]>;
    rowFields$: Observable<IPivotField[]>;
    columnFields$: Observable<IPivotField[]>;
    filterFields$: Observable<IPivotField[]>;
    valuePosition$: Observable<PivotValuePosition>;

    sourceData$: Observable<IObjectMatrixPrimitiveType<Nullable<ICellData>>>;
    calculatedData$: Observable<IObjectMatrixPrimitiveType<Nullable<ICellData>> | null>;

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
        this._sourceRangeInfo$ = new BehaviorSubject(sourceRangeInfo);
        this._targetCellInfo = targetCellInfo;

        this._pivotEngine = new PivotEngine({
            valueFields: fieldsConfig.valueFields || [],
            rowFields: fieldsConfig.rowFields || [],
            columnFields: fieldsConfig.columnFields || [],
            filterFields: fieldsConfig.filterFields || [],
            sourceData: {},
            valuePosition: fieldsConfig.valuePosition,
        });

        this._valueFields$ = new BehaviorSubject(fieldsConfig.valueFields);
        this._rowFields$ = new BehaviorSubject(fieldsConfig.rowFields);
        this._columnFields$ = new BehaviorSubject(fieldsConfig.columnFields);
        this._filterFields$ = new BehaviorSubject(fieldsConfig.filterFields);
        this._valuePosition$ = new BehaviorSubject(fieldsConfig.valuePosition);
        this._sourceData$ = new BehaviorSubject({});
        this._calculatedData$ = new BehaviorSubject<IObjectMatrixPrimitiveType<Nullable<ICellData>> | null>(null);

        this.sourceRangeInfo$ = this._sourceRangeInfo$.asObservable();
        this.valueFields$ = this._valueFields$.asObservable();
        this.rowFields$ = this._rowFields$.asObservable();
        this.columnFields$ = this._columnFields$.asObservable();
        this.filterFields$ = this._filterFields$.asObservable();
        this.valuePosition$ = this._valuePosition$.asObservable();
        this.sourceData$ = this._sourceData$.asObservable();
        this.calculatedData$ = this._calculatedData$.asObservable();

        this._initListeners();

        this.disposeWithMe(this._pivotEngine);

        this.disposeWithMe(() => {
            this._sourceRangeInfo$.complete();
            this._valueFields$.complete();
            this._rowFields$.complete();
            this._columnFields$.complete();
            this._filterFields$.complete();
            this._valuePosition$.complete();
            this._sourceData$.complete();
            this._calculatedData$.complete();
        });
    }

    private _initListeners(): void {
        this.disposeWithMe(
            combineLatest([
                this.valueFields$,
                this.rowFields$,
                this.columnFields$,
                this.filterFields$,
                this.valuePosition$,
                this.sourceData$,
            ])
                .pipe(
                    distinctUntilChanged(),
                    debounceTime(0)
                )
                .subscribe(() => {
                    // Get output cell matrix
                    const cellValue = this.getOutputCellMatrix();
                    this._calculatedData$.next(cellValue);
                })
        );
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
        return this._sourceRangeInfo$.value;
    }

    setSourceRangeInfo(sourceRangeInfo: ISourceRangeInfo): void {
        this._sourceRangeInfo$.next(sourceRangeInfo);
    }

    getTargetCellInfo(): ITargetCellInfo {
        return this._targetCellInfo;
    }

    setTargetCellInfo(targetCellInfo: ITargetCellInfo): void {
        this._targetCellInfo = targetCellInfo;
    }

    isDirty(): boolean {
        return this._pivotEngine.isDirty();
    }

    getValueFields(): IPivotField[] {
        return this._pivotEngine.getValueFields();
    }

    getRowFields(): IPivotField[] {
        return this._pivotEngine.getRowFields();
    }

    getColumnFields(): IPivotField[] {
        return this._pivotEngine.getColumnFields();
    }

    getFilterFields(): IPivotField[] {
        return this._pivotEngine.getFilterFields();
    }

    getValuePosition(): PivotValuePosition {
        return this._pivotEngine.valuePosition;
    }

    setValueFields(valueFields: IPivotField[]): void {
        this._pivotEngine.setValueFields(valueFields);
        this._valueFields$.next(valueFields);
    }

    setRowFields(rowFields: IPivotField[]): void {
        this._pivotEngine.setRowFields(rowFields);
        this._rowFields$.next(rowFields);
    }

    setColumnFields(columnFields: IPivotField[]): void {
        this._pivotEngine.setColumnFields(columnFields);
        this._columnFields$.next(columnFields);
    }

    setFilterFields(filterFields: IPivotField[]): void {
        this._pivotEngine.setFilterFields(filterFields);
        this._filterFields$.next(filterFields);
    }

    setValuePosition(valuePosition: PivotValuePosition): void {
        this._pivotEngine.setValuePosition(valuePosition);
        this._valuePosition$.next(valuePosition);
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
    getOutputRange(): IRange {
        const outputCellMatrix = this.getOutputCellMatrix();
        return new ObjectMatrix(outputCellMatrix).getDataRange();
    }

  /**
   * Generate full cell matrix for pivot table output
   * Includes headers, values, and totals in the correct layout
   * @returns ObjectMatrix with all cell values positioned relative to target cell, or null if not calculated
   */
    getOutputCellMatrix(): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        const targetMatrix = this._pivotEngine.getCalculatedData();
        if (!targetMatrix) {
            return this._moveMatrix(defaultPlaceholderMatrix, this._targetCellInfo).getMatrix();
        }
        return this._moveMatrix(targetMatrix, this._targetCellInfo).getMatrix();
    }

    setSourceData(sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>): void {
        this._pivotEngine.setSourceData(sourceData);
        this._sourceData$.next(sourceData);
    }

    getSourceData(): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        return this._sourceData$.value;
    }

  /**
   * Calculate pivot table data
   * @param workbook - The workbook containing source data
   * @returns Calculated pivot table data
   */
    setSourceDataFromWorkbook(workbook: Workbook) {
        const worksheet = workbook.getSheetBySheetId(this.getSourceRangeInfo().subUnitId);
        if (!worksheet) {
            return null;
        }
        const range = worksheet.getRange(this.getSourceRangeInfo().range);
        const matrix = range.getMatrix().getMatrix();
        this.setSourceData(matrix);
    }

  /**
   * Serialize to JSON
   */
    toJSON(): IPivotTableConfig {
        return {
            id: this._id,
            name: this._name,
            sourceRangeInfo: this.getSourceRangeInfo(),
            targetCellInfo: this._targetCellInfo,
            fieldsConfig: {
                valueFields: this.getValueFields(),
                rowFields: this.getRowFields(),
                columnFields: this.getColumnFields(),
                filterFields: this.getFilterFields(),
                valuePosition: this.getValuePosition(),
            },
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
