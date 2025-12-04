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

import type { ICellData, IObjectMatrixPrimitiveType, IRange, IUnitRange, Nullable } from '@univerjs/core';
import type { IFeatureCalculationManagerParam, IFeatureDirtyRangeType, IRuntimeUnitDataType } from '@univerjs/engine-formula';
import type { PivotTable } from '../models/pivot-table';
import { Disposable, ICommandService, Inject, IUniverInstanceService, ObjectMatrix, Rectangle } from '@univerjs/core';
import { IFeatureCalculationManagerService, RemoveFeatureCalculationMutation, SetFeatureCalculationMutation, SetFormulaCalculationStartMutation } from '@univerjs/engine-formula';
import { skip } from 'rxjs';
import { SheetsPivotDataSourceModel } from '../models/sheets-pivot-data-source-model';
import { ISheetsPivotTableService } from '../services/pivot-table.service';

/**
 * Controller for pivot table formula integration
 * Manages the registration of pivot tables as formula engine features
 * to enable formulas to reference pivot output cells
 */
export class PivotTableFormulaController extends Disposable {
    constructor(
        @Inject(IFeatureCalculationManagerService)
        private readonly _featureCalculationManagerService: IFeatureCalculationManagerService,
        @Inject(ICommandService)
        private readonly _commandService: ICommandService,
        @Inject(IUniverInstanceService)
        private readonly _univerInstanceService: IUniverInstanceService,
        @Inject(ISheetsPivotTableService)
        private readonly _pivotTableService: ISheetsPivotTableService,
        @Inject(SheetsPivotDataSourceModel)
        private readonly _dataSourceModel: SheetsPivotDataSourceModel
    ) {
        super();

        this._init();
    }

    override dispose(): void {
        // Clean up all pivot table subscriptions
        this._pivotTableSubscriptions.forEach((unitMap) => {
            unitMap.forEach((subUnitMap) => {
                subUnitMap.forEach((subscription) => {
                    subscription.dispose();
                });
            });
        });
        this._pivotTableSubscriptions.clear();

        super.dispose();
    }

    /**
     * Map to track pivot table data change subscriptions
     * unitId -> subUnitId -> pivotTableId -> subscription
     */
    private _pivotTableSubscriptions: Map<string, Map<string, Map<string, { dispose: () => void }>>> = new Map();

    /**
     * Map to track previous pivot table output ranges
     * unitId -> subUnitId -> pivotTableId -> lastOutputRange
     * Used to mark the entire affected range (including shrunk areas) as dirty when range changes
     */
    private _previousPivotRanges: Map<string, Map<string, Map<string, IRange>>> = new Map();

    /**
     * Initialize controller lifecycle
     */
    private _init(): void {
        this._initPivotTableEventListeners();
    }

    /**
     * Initialize pivot table event listeners
     */
    private _initPivotTableEventListeners(): void {
        // Listen to pivot table creation events
        this.disposeWithMe(
            this._dataSourceModel.pivotTableAdded$.subscribe((event) => {
                this._registerPivotFeature(event.unitId, event.subUnitId, event.pivotTableId);
            })
        );

        // Listen to pivot table deletion events
        this.disposeWithMe(
            this._dataSourceModel.pivotTableRemoved$.subscribe((event) => {
                this._unregisterPivotFeature(event.unitId, event.subUnitId, event.pivotTableId);
            })
        );

        // Listen to pivot table range changes to update feature dependency ranges
        this.disposeWithMe(
            this._dataSourceModel.tableRangeChanged$.subscribe((event) => {
                // When pivot table range changes, we need to update the feature registration
                // with new dependency ranges
                this._updatePivotFeature(event.unitId, event.subUnitId, event.tableId);
            })
        );
    }

    /**
     * Register a pivot table as a formula feature
     */
    private _registerPivotFeature(unitId: string, subUnitId: string, pivotTableId: string): void {
        const pivotTable = this._pivotTableService.getPivotTable(unitId, subUnitId, pivotTableId);
        if (!pivotTable) {
            return;
        }

        // Use absolute output range for dependency tracking
        // This ensures formula engine knows the exact cells affected by the pivot table
        const absoluteOutputRange = pivotTable.getAbsoluteOutputRange();
        const dependencyRanges: IUnitRange[] = [{
            unitId,
            sheetId: subUnitId,
            range: absoluteOutputRange,
        }];

        const calculationParam: IFeatureCalculationManagerParam = {
            unitId,
            subUnitId,
            dependencyRanges,
            getDirtyData: (_dirtyData, _runtimeData) => {
                return this._buildRuntimeCellData(unitId, subUnitId, pivotTableId);
            },
        };

        // Register the feature
        this._featureCalculationManagerService.register(unitId, subUnitId, pivotTableId, calculationParam);

        // Execute the mutation to notify the formula engine
        this._commandService.executeCommand(SetFeatureCalculationMutation.id, {
            featureId: pivotTableId,
            calculationParam,
        });

        // Listen to pivot table data changes to trigger formula recalculation
        this._listenToPivotTableDataChanges(unitId, subUnitId, pivotTableId, pivotTable, absoluteOutputRange);

        // Store the current range for future change detection
        this._setPreviousRange(unitId, subUnitId, pivotTableId, absoluteOutputRange);
    }

    /**
     * Unregister a pivot table feature
     */
    private _unregisterPivotFeature(unitId: string, subUnitId: string, pivotTableId: string): void {
        // Clean up the subscription for this pivot table
        this._cleanupPivotTableSubscription(unitId, subUnitId, pivotTableId);

        // Clean up the stored previous range
        const unitMap = this._previousPivotRanges.get(unitId);
        if (unitMap) {
            const subUnitMap = unitMap.get(subUnitId);
            if (subUnitMap) {
                subUnitMap.delete(pivotTableId);
            }
        }

        // Remove the feature from the manager
        this._featureCalculationManagerService.remove(unitId, subUnitId, [pivotTableId]);

        // Execute the mutation to notify the formula engine
        this._commandService.executeCommand(RemoveFeatureCalculationMutation.id, {
            unitId,
            subUnitId,
            featureIds: [pivotTableId],
        });
    }

    /**
     * Listen to pivot table data changes and trigger formula recalculation
     */
    private _listenToPivotTableDataChanges(unitId: string, subUnitId: string, pivotTableId: string, pivotTable: PivotTable, _outputRange: IRange): void {
        // Clean up any existing subscription
        this._cleanupPivotTableSubscription(unitId, subUnitId, pivotTableId);

        // Listen to calculatedData changes (skip the first emission which is the initial null value)
        const subscription = pivotTable.calculatedData$.pipe(skip(1)).subscribe((newData: IObjectMatrixPrimitiveType<Nullable<ICellData>> | null) => {
            if (newData) {
                // Get the current output range (may have changed)
                // Note: We don't use the _outputRange parameter as it's stale when range changes
                const currentOutputRange = pivotTable.getAbsoluteOutputRange();
                // Trigger formula recalculation for the current pivot table output range
                this._triggerFormulaRecalculation(unitId, subUnitId, currentOutputRange);
            }
        });

        // Store the subscription for cleanup
        this._storePivotTableSubscription(unitId, subUnitId, pivotTableId, {
            dispose: () => subscription.unsubscribe(),
        });
    }

    /**
     * Trigger formula recalculation for a specific range
     */
    private _triggerFormulaRecalculation(unitId: string, subUnitId: string, dirtyRange: IRange): void {
        const dirtyRanges = [{
            unitId,
            sheetId: subUnitId,
            range: dirtyRange,
        }];

        // Execute formula calculation start mutation with dirty ranges
        this._commandService.executeCommand(SetFormulaCalculationStartMutation.id, {
            forceCalculation: false,
            dirtyRanges,
            dirtyNameMap: {},
            dirtyDefinedNameMap: {},
            dirtyUnitFeatureMap: {},
            dirtyUnitOtherFormulaMap: {},
            clearDependencyTreeCache: {},
        });
    }

    /**
     * Store pivot table subscription for cleanup
     */
    private _storePivotTableSubscription(unitId: string, subUnitId: string, pivotTableId: string, subscription: { dispose: () => void }): void {
        let unitMap = this._pivotTableSubscriptions.get(unitId);
        if (!unitMap) {
            unitMap = new Map();
            this._pivotTableSubscriptions.set(unitId, unitMap);
        }

        let subUnitMap = unitMap.get(subUnitId);
        if (!subUnitMap) {
            subUnitMap = new Map();
            unitMap.set(subUnitId, subUnitMap);
        }

        subUnitMap.set(pivotTableId, subscription);
    }

    /**
     * Clean up pivot table subscription
     */
    private _cleanupPivotTableSubscription(unitId: string, subUnitId: string, pivotTableId: string): void {
        const unitMap = this._pivotTableSubscriptions.get(unitId);
        if (unitMap) {
            const subUnitMap = unitMap.get(subUnitId);
            if (subUnitMap) {
                const subscription = subUnitMap.get(pivotTableId);
                if (subscription) {
                    subscription.dispose();
                    subUnitMap.delete(pivotTableId);
                }
            }
        }
    }

    /**
     * Update a pivot table feature (re-register with new parameters)
     * When range changes, mark the union of old and new ranges as dirty to ensure
     * formula engine recalculates all affected formulas
     */
    private _updatePivotFeature(unitId: string, subUnitId: string, pivotTableId: string): void {
        const pivotTable = this._pivotTableService.getPivotTable(unitId, subUnitId, pivotTableId);
        if (!pivotTable) {
            return;
        }

        const newRange = pivotTable.getAbsoluteOutputRange();
        const previousRange = this._getPreviousRange(unitId, subUnitId, pivotTableId);

        // If range changed, mark the union of old and new ranges as dirty
        if (previousRange && !Rectangle.equals(previousRange, newRange)) {
            const unionRange = Rectangle.union(previousRange, newRange);
            this._triggerFormulaRecalculation(unitId, subUnitId, unionRange);
        }

        // Store the new range
        this._setPreviousRange(unitId, subUnitId, pivotTableId, newRange);

        // For updates, we need to unregister and re-register the feature
        // since there's no direct update method in the feature calculation manager
        this._unregisterPivotFeature(unitId, subUnitId, pivotTableId);
        this._registerPivotFeature(unitId, subUnitId, pivotTableId);
    }

    /**
     * Get the previously stored output range for a pivot table
     */
    private _getPreviousRange(unitId: string, subUnitId: string, pivotTableId: string): IRange | undefined {
        return this._previousPivotRanges.get(unitId)?.get(subUnitId)?.get(pivotTableId);
    }

    /**
     * Store the current output range for a pivot table
     */
    private _setPreviousRange(unitId: string, subUnitId: string, pivotTableId: string, range: IRange): void {
        let unitMap = this._previousPivotRanges.get(unitId);
        if (!unitMap) {
            unitMap = new Map();
            this._previousPivotRanges.set(unitId, unitMap);
        }

        let subUnitMap = unitMap.get(subUnitId);
        if (!subUnitMap) {
            subUnitMap = new Map();
            unitMap.set(subUnitId, subUnitMap);
        }

        subUnitMap.set(pivotTableId, range);
    }

    /**
     * Build runtime cell data from pivot table output
     */
    private _buildRuntimeCellData(unitId: string, subUnitId: string, pivotTableId: string): { runtimeCellData: IRuntimeUnitDataType; dirtyRanges: IFeatureDirtyRangeType } {
        const pivotTable = this._pivotTableService.getPivotTable(unitId, subUnitId, pivotTableId);
        if (!pivotTable) {
            return {
                runtimeCellData: {},
                dirtyRanges: {},
            };
        }

        const outputMatrix = pivotTable.getOutputCellMatrix();
        const targetCellInfo = pivotTable.getTargetCellInfo();
        const outputRange = pivotTable.getAbsoluteOutputRange();

        // Create runtime cell data structure
        const runtimeCellData: IRuntimeUnitDataType = {
            [unitId]: {
                [subUnitId]: new ObjectMatrix<ICellData>(),
            },
        };

        // Transform output matrix to absolute positions
        const unitMatrix = runtimeCellData[unitId]![subUnitId]!;

        // Copy data from output matrix to absolute positions
        for (const [relativeRow, rowData] of Object.entries(outputMatrix)) {
            const absoluteRow = Number.parseInt(relativeRow, 10) + targetCellInfo.row;
            for (const [relativeCol, cellData] of Object.entries(rowData)) {
                const absoluteCol = Number.parseInt(relativeCol, 10) + targetCellInfo.col;
                if (cellData) {
                    unitMatrix.setValue(absoluteRow, absoluteCol, cellData);
                }
            }
        }

        // Create dirty ranges for the output area
        const dirtyRanges: IFeatureDirtyRangeType = {
            [unitId]: {
                [subUnitId]: [outputRange],
            },
        };

        return {
            runtimeCellData,
            dirtyRanges,
        };
    }
}
