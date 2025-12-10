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
import type { PivotValuePosition } from '../types/enum';
import type { IFieldsConfig, IPivotField, IPivotTableConfig, IPivotTableCrossTabData, ISourceFields, ISourceRangeInfo, ITargetCellInfo } from '../types/type';
import { Disposable, ObjectMatrix, Rectangle } from '@univerjs/core';
import { deserializeRangeWithSheetWithCache, serializeRangeToRefString, serializeRangeWithSpreadsheet } from '@univerjs/engine-formula';
import { BehaviorSubject, combineLatest, debounceTime, distinctUntilChanged, pairwise } from 'rxjs';
import { PivotEngineV3 } from './pivot-engine-v3';

/**
 * Configuration options for PivotTable
 */
export interface IPivotTableOptions {
    /**
     * If true, the pivot table will not auto-calculate when fields or source data change.
     * Instead, it will wait for calculated data to be set via setCalculatedData().
     * This is used in RPC environment where the main thread receives calculated data
     * from the Worker thread via mutation.
     * @default false
     */
    skipAutoCalculation?: boolean;
}

/**
 * Simplified PivotTable implementation for MVP
 * This class handles the core pivot table calculation logic
 */
export class PivotTable extends Disposable {
    private _id: string;
    private _name: string;
    private _sourceRangeInfo$: BehaviorSubject<ISourceRangeInfo>;
    private _targetCellInfo: ITargetCellInfo;
    private _options: IPivotTableOptions;

    private _pivotEngine: PivotEngineV3;
    private _recalculated$: BehaviorSubject<boolean>;

    private _valueFields$: BehaviorSubject<IPivotField[]>;
    private _rowFields$: BehaviorSubject<IPivotField[]>;
    private _columnFields$: BehaviorSubject<IPivotField[]>;
    private _filterFields$: BehaviorSubject<IPivotField[]>;
    private _valuePosition$: BehaviorSubject<PivotValuePosition>;

    private _sourceData$: BehaviorSubject<IObjectMatrixPrimitiveType<Nullable<ICellData>>>;
    private _sourceFields$: BehaviorSubject<IPivotField[]>;

    recalculated$: Observable<boolean>;
    sourceRangeInfo$: Observable<ISourceRangeInfo>;
    valueFields$: Observable<IPivotField[]>;
    rowFields$: Observable<IPivotField[]>;
    columnFields$: Observable<IPivotField[]>;
    filterFields$: Observable<IPivotField[]>;
    valuePosition$: Observable<PivotValuePosition>;

    sourceData$: Observable<IObjectMatrixPrimitiveType<Nullable<ICellData>>>;
    sourceFields$: Observable<IPivotField[]>;

    constructor(
        id: string,
        name: string,
        sourceRangeInfo: ISourceRangeInfo,
        targetCellInfo: ITargetCellInfo,
        fieldsConfig: IFieldsConfig,
        options: IPivotTableOptions = {}
    ) {
        super();
        this._id = id;
        this._name = name;
        this._sourceRangeInfo$ = new BehaviorSubject(sourceRangeInfo);
        this._targetCellInfo = targetCellInfo;
        this._options = options;

        this._pivotEngine = new PivotEngineV3({
            rowFields: fieldsConfig.rowFields || [],
            columnFields: fieldsConfig.columnFields || [],
            valueFields: fieldsConfig.valueFields || [],
            filterFields: fieldsConfig.filterFields || [],
            sourceData: {},
            valuePosition: fieldsConfig.valuePosition,
        });

        this._recalculated$ = new BehaviorSubject(false);
        this._valueFields$ = new BehaviorSubject(fieldsConfig.valueFields);
        this._rowFields$ = new BehaviorSubject(fieldsConfig.rowFields);
        this._columnFields$ = new BehaviorSubject(fieldsConfig.columnFields);
        this._filterFields$ = new BehaviorSubject(fieldsConfig.filterFields);
        this._valuePosition$ = new BehaviorSubject(fieldsConfig.valuePosition);
        this._sourceData$ = new BehaviorSubject({});
        this._sourceFields$ = new BehaviorSubject<IPivotField[]>([]);

        this.recalculated$ = this._recalculated$.asObservable();
        this.sourceRangeInfo$ = this._sourceRangeInfo$.asObservable();
        this.valueFields$ = this._valueFields$.asObservable();
        this.rowFields$ = this._rowFields$.asObservable();
        this.columnFields$ = this._columnFields$.asObservable();
        this.filterFields$ = this._filterFields$.asObservable();
        this.valuePosition$ = this._valuePosition$.asObservable();
        this.sourceData$ = this._sourceData$.asObservable();
        this.sourceFields$ = this._sourceFields$.asObservable();

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
        });
    }

    private _initListeners(): void {
        // Only auto-calculate if skipAutoCalculation is not set
        // In RPC environment, main thread sets skipAutoCalculation: true
        // and receives calculated data from Worker via mutation
        if (!this._options.skipAutoCalculation) {
            this._initCalculatedDataListener();
        }
        this._initSourceFieldsListener();
        this._initFieldsUpdateListener();
    }

    /**
     * Listen to field changes and update calculated data
     * This listener is only active when skipAutoCalculation is false (default)
     */
    private _initCalculatedDataListener(): void {
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
                    this._pivotEngine.getCalculatedData();
                    this._recalculated$.next(true);
                })
        );
    }

    /**
     * Listen to source range info and data changes, update source fields
     */
    private _initSourceFieldsListener(): void {
        this.disposeWithMe(
            combineLatest([
                this.sourceRangeInfo$.pipe(distinctUntilChanged((prev, curr) => {
                    return prev.unitId === curr.unitId && prev.subUnitId === curr.subUnitId && Rectangle.equals(prev.range, curr.range);
                })),
                this.sourceData$,
            ]).pipe(debounceTime(0)).subscribe(([sourceRangeInfo, sourceData]) => {
                const fields: ISourceFields[] = [];
                const { unitId, subUnitId, range } = sourceRangeInfo;
                for (let i = 0; i < range.endColumn - range.startColumn + 1; i++) {
                    const columnIndex = i + range.startColumn;
                    fields.push({
                        sourceColumnIndex: i,
                        rangeKey: serializeRangeWithSpreadsheet(unitId, subUnitId, {
                            ...range,
                            startColumn: columnIndex,
                            endColumn: columnIndex,
                        }),
                    });
                }

                const sourceFields = fields.map((field, index) => {
                    const range = deserializeRangeWithSheetWithCache(field.rangeKey);
                    return {
                        id: serializeRangeToRefString(range),
                        name: String(sourceData[range.range.startRow]?.[range.range.startColumn]?.v as string) || `Field ${index + 1}`,
                        sourceColumnIndex: field.sourceColumnIndex,
                    } satisfies IPivotField;
                });
                this._sourceFields$.next(sourceFields);
            })
        );
    }

    /**
     * Listen to source fields changes, filter and update all pivot fields
     */
    private _initFieldsUpdateListener(): void {
        this.disposeWithMe(this.sourceFields$.pipe(pairwise()).subscribe(([_prev, curr]) => {
            // Create a map of source fields by id for quick lookup
            const sourceFieldMap = new Map<string, IPivotField>();
            for (const sourceField of curr) {
                sourceFieldMap.set(sourceField.id, sourceField);
            }

            // Helper function to filter and update fields in a single pass
            const filterAndUpdateFields = (fields: IPivotField[]): IPivotField[] => {
                const result: IPivotField[] = [];
                for (const field of fields) {
                    // Extract rangeKey from field id (format: prefix_rangeKey) - only once
                    const rangeKey = field.id.split('_').slice(1).join('_');
                    const sourceField = sourceFieldMap.get(rangeKey);
                    if (sourceField) {
                        result.push({
                            ...field,
                            name: sourceField.name,
                        });
                    }
                }
                return result;
            };

            // Update all field types
            this.setValueFields(filterAndUpdateFields(this.getValueFields()));
            this.setRowFields(filterAndUpdateFields(this.getRowFields()));
            this.setColumnFields(filterAndUpdateFields(this.getColumnFields()));
            this.setFilterFields(filterAndUpdateFields(this.getFilterFields()));
        }));
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

    getSourceFields(): IPivotField[] {
        return this._sourceFields$.value;
    }

    setSourceFields(sourceFields: IPivotField[]): void {
        this._sourceFields$.next(sourceFields);
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

  /**
   * Get the output range of the pivot table based on calculated data
   * Returns the range from target cell to the end of calculated output (including grand totals)
   * @returns Output range or null if not calculated yet
   */
    getOutputRange(): IRange {
        const calculatedData = this._pivotEngine.getCalculatedCellMatrix();
        return new ObjectMatrix(calculatedData || {}).getDataRange();
    }

  /**
   * Get the absolute output range of the pivot table in the worksheet
   * Applies target cell offset to the relative output range
   * @returns Absolute output range in worksheet coordinates
   */
    getAbsoluteOutputRange(): IRange {
        const relativeRange = this.getOutputRange();
        return {
            startRow: relativeRange.startRow + this._targetCellInfo.row,
            endRow: relativeRange.endRow + this._targetCellInfo.row,
            startColumn: relativeRange.startColumn + this._targetCellInfo.col,
            endColumn: relativeRange.endColumn + this._targetCellInfo.col,
        };
    }

    getEngine(): PivotEngineV3 {
        return this._pivotEngine;
    }

    setSourceData(sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>): void {
        this._pivotEngine.setSourceData(sourceData);
        this._sourceData$.next(sourceData);
    }

    getSourceData(): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        return this._sourceData$.value;
    }

    /**
     * Directly set calculated data (used in RPC environment)
     * This method allows the main thread to receive calculated data from Worker
     * via mutation without triggering recalculation.
     */
    setCalculatedData(data: IPivotTableCrossTabData): void {
        this._pivotEngine.setCalculatedData(data, false);
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
   * @param json - The JSON configuration
   * @param options - Optional configuration options (e.g., skipAutoCalculation for RPC environment)
   */
    static fromJSON(json: {
        id: string;
        name: string;
        sourceRangeInfo: ISourceRangeInfo;
        targetCellInfo: ITargetCellInfo;
        fieldsConfig: IFieldsConfig;
    }, options?: IPivotTableOptions): PivotTable {
        return new PivotTable(
            json.id,
            json.name,
            json.sourceRangeInfo,
            json.targetCellInfo,
            json.fieldsConfig,
            options
        );
    }
}
