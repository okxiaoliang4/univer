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

import type { IDisposable, IRange } from '@univerjs/core';
import type { ISetRangeValuesMutationParams } from '@univerjs/sheets';
import { Disposable, ICommandService, Inject, IUniverInstanceService, RTree, toDisposable } from '@univerjs/core';
import { SetRangeValuesMutation } from '@univerjs/sheets';
import { SheetsPivotDataSourceModel } from '../models/sheets-pivot-data-source-model';

/**
 * Service interface for pivot table range management using spatial indexing
 */
export interface IPivotTableRangeService {
    /**
     * Register a pivot table output range in the spatial index
     * @param unitId - Workbook ID
     * @param subUnitId - Worksheet ID
     * @param pivotTableId - Pivot table ID
     * @param range - Output range (absolute coordinates)
     */
    registerPivotRange(unitId: string, subUnitId: string, pivotTableId: string, range: IRange): void;

    /**
     * Update pivot table output range (removes old range, inserts new range)
     * @param unitId - Workbook ID
     * @param subUnitId - Worksheet ID
     * @param pivotTableId - Pivot table ID
     * @param oldRange - Previous output range (absolute coordinates)
     * @param newRange - New output range (absolute coordinates)
     */
    updatePivotRange(unitId: string, subUnitId: string, pivotTableId: string, oldRange: IRange, newRange: IRange): void;

    /**
     * Remove pivot table output range from spatial index
     * @param unitId - Workbook ID
     * @param subUnitId - Worksheet ID
     * @param pivotTableId - Pivot table ID
     * @param range - Output range to remove (absolute coordinates)
     */
    removePivotRange(unitId: string, subUnitId: string, pivotTableId: string, range: IRange): void;

    /**
     * Check if a cell is within any pivot table output range using O(log n) spatial query
     * @param unitId - Workbook ID
     * @param subUnitId - Worksheet ID
     * @param row - Cell row index
     * @param col - Cell column index
     * @returns true if cell is in any pivot output range
     */
    isPivotOutputCell(unitId: string, subUnitId: string, row: number, col: number): boolean;

    /**
     * Get pivot table ID containing the specified cell using O(log n) spatial query
     * @param unitId - Workbook ID
     * @param subUnitId - Worksheet ID
     * @param row - Cell row index
     * @param col - Cell column index
     * @returns Pivot table ID if cell is in output range, null otherwise
     */
    getPivotTableIdByCell(unitId: string, subUnitId: string, row: number, col: number): string | null;
}

export const IPivotTableRangeService = 'sheets-pivot-table.pivot-table-range-service';

/**
 * Pivot table range service using RTree spatial indexing for O(log n) range queries
 * Manages pivot table output ranges to optimize performance of cell containment checks
 */
export class PivotTableRangeService extends Disposable implements IPivotTableRangeService {
    private _rTree = new RTree();

    // Map to store subscriptions for each pivot table's calculatedData$
    // Key format: unitId|subUnitId|pivotTableId
    private _pivotTableSubscriptions = new Map<string, IDisposable>();

    constructor(
        @Inject(SheetsPivotDataSourceModel) private readonly _pivotTableManager: SheetsPivotDataSourceModel,
        @Inject(IUniverInstanceService) private readonly _univerInstanceService: IUniverInstanceService,
        @Inject(ICommandService) private readonly _commandService: ICommandService
    ) {
        super();
        this._initEventListeners();
    }

    private _getSubscriptionKey(unitId: string, subUnitId: string, pivotTableId: string): string {
        return `${unitId}|${subUnitId}|${pivotTableId}`;
    }

    private _initEventListeners(): void {
        // Listen to pivot table added events
        this.disposeWithMe(
            this._pivotTableManager.pivotTableAdded$.subscribe((event) => {
                const { pivotTableId, unitId, subUnitId } = event;
                const pivotTable = this._pivotTableManager.getPivotTableInstance(unitId, subUnitId, pivotTableId);
                if (!pivotTable) {
                    return;
                }

                // Subscribe to calculatedData$ to update range when data changes
                this._subscribeToCalculatedData(unitId, subUnitId, pivotTableId);

                const outputRange = pivotTable.getOutputRange();
                if (outputRange) {
                    // Convert relative range to absolute range
                    const targetCellInfo = pivotTable.getTargetCellInfo();
                    const absoluteRange = {
                        startRow: outputRange.startRow + targetCellInfo.row,
                        endRow: outputRange.endRow + targetCellInfo.row,
                        startColumn: outputRange.startColumn + targetCellInfo.col,
                        endColumn: outputRange.endColumn + targetCellInfo.col,
                    };
                    this.registerPivotRange(unitId, subUnitId, pivotTableId, absoluteRange);
                }
            })
        );

        // Listen to pivot table removed events
        this.disposeWithMe(
            this._pivotTableManager.pivotTableRemoved$.subscribe((event) => {
                const { pivotTableId, unitId, subUnitId } = event;

                // Clean up calculatedData$ subscription
                const key = this._getSubscriptionKey(unitId, subUnitId, pivotTableId);
                const subscription = this._pivotTableSubscriptions.get(key);
                if (subscription) {
                    subscription.dispose();
                    this._pivotTableSubscriptions.delete(key);
                }

                // Find and remove the range from RTree
                this._clearPivotTableRange(unitId, subUnitId, pivotTableId);
            })
        );

        // Listen to pivot table range changed events
        this.disposeWithMe(
            this._pivotTableManager.tableRangeChanged$.subscribe((event) => {
                const { tableId: pivotTableId, unitId, subUnitId } = event;
                const pivotTable = this._pivotTableManager.getPivotTableInstance(unitId, subUnitId, pivotTableId);
                if (!pivotTable) {
                    return;
                }

                // Clear the old range before updating
                this._clearPivotTableRange(unitId, subUnitId, pivotTableId);

                // Get new range and register it
                const newOutputRange = pivotTable.getOutputRange();
                if (newOutputRange) {
                    // Convert relative range to absolute range
                    const targetCellInfo = pivotTable.getTargetCellInfo();
                    const newAbsoluteRange = {
                        startRow: newOutputRange.startRow + targetCellInfo.row,
                        endRow: newOutputRange.endRow + targetCellInfo.row,
                        startColumn: newOutputRange.startColumn + targetCellInfo.col,
                        endColumn: newOutputRange.endColumn + targetCellInfo.col,
                    };

                    this.registerPivotRange(unitId, subUnitId, pivotTableId, newAbsoluteRange);
                }
            })
        );
    }

    /**
     * Subscribe to a pivot table's calculatedData$ to update range when data changes
     */
    private _subscribeToCalculatedData(unitId: string, subUnitId: string, pivotTableId: string): void {
        const pivotTable = this._pivotTableManager.getPivotTableInstance(unitId, subUnitId, pivotTableId);
        if (!pivotTable) {
            return;
        }

        const key = this._getSubscriptionKey(unitId, subUnitId, pivotTableId);

        // Clean up existing subscription if any
        const existingSubscription = this._pivotTableSubscriptions.get(key);
        if (existingSubscription) {
            existingSubscription.dispose();
        }

        // Subscribe to calculatedData$ changes (skip initial value)
        const subscription = pivotTable.recalculated$.subscribe(() => {
                // Clear old range and register new range
            this._clearPivotTableRange(unitId, subUnitId, pivotTableId);

            const outputRange = pivotTable.getAbsoluteOutputRange();
            if (outputRange) {
                this.registerPivotRange(unitId, subUnitId, pivotTableId, outputRange);
            }
        });

        this._pivotTableSubscriptions.set(key, toDisposable(() => subscription.unsubscribe()));
    }

    registerPivotRange(unitId: string, subUnitId: string, pivotTableId: string, range: IRange): void {
        // First, remove any existing ranges for this pivot table to avoid duplicates
        const existingRanges = this._findRangesByPivotTableId(unitId, subUnitId, pivotTableId);
        for (const existingRange of existingRanges) {
            this._rTree.remove({
                unitId,
                sheetId: subUnitId,
                id: pivotTableId,
                range: existingRange,
            });
        }

        // Then insert the new range
        this._rTree.insert({
            unitId,
            sheetId: subUnitId,
            id: pivotTableId,
            range,
        });
    }

    updatePivotRange(unitId: string, subUnitId: string, pivotTableId: string, oldRange: IRange, newRange: IRange): void {
        // Remove old range
        this._rTree.remove({
            unitId,
            sheetId: subUnitId,
            id: pivotTableId,
            range: oldRange,
        });

        // Insert new range
        this._rTree.insert({
            unitId,
            sheetId: subUnitId,
            id: pivotTableId,
            range: newRange,
        });
    }

    removePivotRange(unitId: string, subUnitId: string, pivotTableId: string, range: IRange): void {
        this._rTree.remove({
            unitId,
            sheetId: subUnitId,
            id: pivotTableId,
            range,
        });
    }

    isPivotOutputCell(unitId: string, subUnitId: string, row: number, col: number): boolean {
        const pointRange = {
            startRow: row,
            endRow: row,
            startColumn: col,
            endColumn: col,
        };

        const results = Array.from(this._rTree.bulkSearch([{
            unitId,
            sheetId: subUnitId,
            range: pointRange,
        }]));

        return results.length > 0;
    }

    getPivotTableIdByCell(unitId: string, subUnitId: string, row: number, col: number): string | null {
        const pointRange = {
            startRow: row,
            endRow: row,
            startColumn: col,
            endColumn: col,
        };

        const results = Array.from(this._rTree.bulkSearch([{
            unitId,
            sheetId: subUnitId,
            range: pointRange,
        }]));

        // Return the first matching pivot table ID
        return results.length > 0 ? String(results[0]) : null;
    }

    /**
     * Clear pivot table range by executing SetRangeValuesMutation and removing from RTree
     * This removes old data when ranges change or pivot tables are deleted
     */
    private _clearPivotTableRange(unitId: string, subUnitId: string, pivotTableId: string): void {
        // Find the current range in RTree and clear it
        const rangesToClear = this._findRangesByPivotTableId(unitId, subUnitId, pivotTableId);

        for (const range of rangesToClear) {
            // Remove from RTree first to prevent detection during clearing
            this.removePivotRange(unitId, subUnitId, pivotTableId, range);
            // Then clear the cell data
            this._executeClearMutation(unitId, subUnitId, range);
        }
    }

    /**
     * Find all ranges for a specific pivot table ID
     */
    private _findRangesByPivotTableId(unitId: string, subUnitId: string, pivotTableId: string): IRange[] {
        const ranges: IRange[] = [];

        // Get the RBush tree for this unitId and subUnitId
        // Use a type assertion to access the private getTree method
        const tree = this._rTree.getTree(unitId, subUnitId);
        if (!tree) {
            return ranges;
        }

        // Get all items from the tree and filter by pivot table ID
        const allItems = tree.all();
        for (const item of allItems) {
            if (item.id === pivotTableId) {
                ranges.push({
                    startRow: item.minY,
                    startColumn: item.minX,
                    endRow: item.maxY,
                    endColumn: item.maxX,
                });
            }
        }

        return ranges;
    }

    /**
     * Execute SetRangeValuesMutation to clear a range
     */
    private _executeClearMutation(unitId: string, subUnitId: string, range: IRange): void {
        const clearParams: ISetRangeValuesMutationParams = {
            unitId,
            subUnitId,
            cellValue: this._createEmptyCellMatrix(range),
        };

        // Execute with onlyLocal: true to avoid broadcasting
        this._commandService.executeCommand(SetRangeValuesMutation.id, {
            ...clearParams,
            options: { onlyLocal: true },
        });
    }

    /**
     * Create empty cell matrix for clearing
     */
    private _createEmptyCellMatrix(range: IRange): Record<number, Record<number, null>> {
        const matrix: Record<number, Record<number, null>> = {};

        for (let row = range.startRow; row <= range.endRow; row++) {
            matrix[row] = {};
            for (let col = range.startColumn; col <= range.endColumn; col++) {
                matrix[row][col] = null; // Clear cell value
            }
        }

        return matrix;
    }
}
