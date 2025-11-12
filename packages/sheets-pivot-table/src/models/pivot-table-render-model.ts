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

import type { IPivotSubtotalInfo, IPivotTableCrossTabData } from '../types/type';

/**
 * Pivot table render business logic model
 * Provides business logic for rendering Cross-Tabulation pivot tables
 */
export class PivotTableRenderModel {
    private data: IPivotTableCrossTabData;
    private collapsedRowGroups: Set<string> = new Set();
    private collapsedColumnGroups: Set<string> = new Set();

    constructor(data: IPivotTableCrossTabData) {
        this.data = data;
        // Initialize: sync all expanded states to collapsedGroups
        this._initializeCollapsedGroups();
    }

    /**
     * Initialize collapsed state
     */
    private _initializeCollapsedGroups(): void {
        if (this.data.structure.rowGroups) {
            this.data.structure.rowGroups.forEach((group) => {
                if (!group.expanded) {
                    this.collapsedRowGroups.add(group.groupId);
                }
            });
        }
        if (this.data.structure.columnGroups) {
            this.data.structure.columnGroups.forEach((group) => {
                if (!group.expanded) {
                    this.collapsedColumnGroups.add(group.groupId);
                }
            });
        }
    }

    // ========== Row related business logic ==========

    /**
     * Get all visible row indices (considering collapse state)
     */
    getVisibleRowIndices(): number[] {
        const visibleIndices: number[] = [];
        const totalRows = this.data.structure.rowHeaders.length;

        for (let i = 0; i < totalRows; i++) {
            if (this.isRowVisible(i)) {
                visibleIndices.push(i);
            }
        }

        return visibleIndices;
    }

    /**
     * Check if a row is visible (considering collapse state)
     */
    isRowVisible(rowIndex: number): boolean {
        const rowType = this.data.structure.rowTypes[rowIndex];

        // Grand total row is always visible (it's a subtotal with level=0, fieldIndex=0)
        const subtotalRow = this.data.structure.subtotalRows?.find((sr) => sr.rowIndex === rowIndex);
        if (subtotalRow && subtotalRow.level === 0 && subtotalRow.fieldIndex === 0) {
            return true;
        }

        // Check if this row belongs to any collapsed group
        const groupIds = this.data.structure.rowLevelMap?.[rowIndex];
        if (!groupIds || groupIds.length === 0) {
            return true; // No group information, default visible
        }

        // Check if any parent group is collapsed
        for (const groupId of groupIds) {
            if (this.collapsedRowGroups.has(groupId)) {
                return false;
            }
        }

        return true;
    }

    /**
     * Get row display information
     */
    getRowInfo(rowIndex: number): {
        headers: string[];
        type: 'data' | 'subtotal';
        isVisible: boolean;
        groupInfo?: Array<{
            groupId: string;
            level: number;
            fieldIndex: number;
            value: string;
        }>;
    } {
        const headers = this.data.structure.rowHeaders[rowIndex] || [];
        const type = this.data.structure.rowTypes[rowIndex] || 'data';
        const isVisible = this.isRowVisible(rowIndex);

        const groupIds = this.data.structure.rowLevelMap?.[rowIndex] || [];
        const groupInfo = groupIds.map((groupId) => {
            const group = this.data.structure.rowGroups?.find((g) => g.groupId === groupId);
            return group
                ? {
                    groupId: group.groupId,
                    level: group.level,
                    fieldIndex: group.fieldIndex,
                    value: group.value,
                }
                : undefined;
        }).filter(Boolean) as Array<{
            groupId: string;
            level: number;
            fieldIndex: number;
            value: string;
        }>;

        return {
            headers,
            type,
            isVisible,
            groupInfo,
        };
    }

    /**
     * Get row group information (for displaying collapse/expand buttons)
     */
    getRowGroupInfo(rowIndex: number): {
        groupId: string;
        level: number;
        hasChildren: boolean;
        isExpanded: boolean;
        canCollapse: boolean;
    } | null {
        const groupIds = this.data.structure.rowLevelMap?.[rowIndex];
        if (!groupIds || groupIds.length === 0) {
            return null;
        }

        // Return outermost (level=0) group information
        const topLevelGroupId = groupIds[0];
        const group = this.data.structure.rowGroups?.find((g) => g.groupId === topLevelGroupId);

        if (!group) {
            return null;
        }

        const hasChildren = (group.childGroupIds?.length || 0) > 0;
        const isExpanded = !this.collapsedRowGroups.has(group.groupId);
        const canCollapse = hasChildren || ((group.lastRowIndex ?? 0) - (group.firstRowIndex ?? 0)) > 0;

        return {
            groupId: group.groupId,
            level: group.level,
            hasChildren,
            isExpanded,
            canCollapse,
        };
    }

    // ========== Column related business logic ==========

    /**
     * Get all visible column indices (considering collapse state)
     */
    getVisibleColumnIndices(): number[] {
        const visibleIndices: number[] = [];
        const totalColumns = this.data.structure.columnHeaders.length;

        for (let i = 0; i < totalColumns; i++) {
            if (this.isColumnVisible(i)) {
                visibleIndices.push(i);
            }
        }

        return visibleIndices;
    }

    /**
     * Check if a column is visible (considering collapse state)
     */
    isColumnVisible(columnIndex: number): boolean {
        const columnType = this.data.structure.columnTypes[columnIndex];

        // Grand total column is always visible
        const subtotalColumn = this.data.structure.subtotalColumns?.find((sc) => sc.columnIndex === columnIndex);
        if (subtotalColumn && subtotalColumn.level === 0 && subtotalColumn.fieldIndex === 0) {
            return true;
        }

        const groupIds = this.data.structure.columnLevelMap?.[columnIndex];
        if (!groupIds || groupIds.length === 0) {
            return true;
        }

        for (const groupId of groupIds) {
            if (this.collapsedColumnGroups.has(groupId)) {
                return false;
            }
        }

        return true;
    }

    /**
     * Get column display information
     */
    getColumnInfo(columnIndex: number): {
        headers: string[];
        type: 'data' | 'subtotal';
        isVisible: boolean;
        groupInfo?: Array<{
            groupId: string;
            level: number;
            fieldIndex: number;
            value: string;
        }>;
    } {
        const headers = this.data.structure.columnHeaders[columnIndex] || [];
        const type = this.data.structure.columnTypes[columnIndex] || 'data';
        const isVisible = this.isColumnVisible(columnIndex);

        const groupIds = this.data.structure.columnLevelMap?.[columnIndex] || [];
        const groupInfo = groupIds.map((groupId) => {
            const group = this.data.structure.columnGroups?.find((g) => g.groupId === groupId);
            return group
                ? {
                    groupId: group.groupId,
                    level: group.level,
                    fieldIndex: group.fieldIndex,
                    value: group.value,
                }
                : undefined;
        }).filter(Boolean) as Array<{
            groupId: string;
            level: number;
            fieldIndex: number;
            value: string;
        }>;

        return {
            headers,
            type,
            isVisible,
            groupInfo,
        };
    }

    /**
     * Get column group information (for displaying collapse/expand buttons)
     */
    getColumnGroupInfo(columnIndex: number): {
        groupId: string;
        level: number;
        hasChildren: boolean;
        isExpanded: boolean;
        canCollapse: boolean;
    } | null {
        const groupIds = this.data.structure.columnLevelMap?.[columnIndex];
        if (!groupIds || groupIds.length === 0) {
            return null;
        }

        const topLevelGroupId = groupIds[0];
        const group = this.data.structure.columnGroups?.find((g) => g.groupId === topLevelGroupId);

        if (!group) {
            return null;
        }

        const hasChildren = (group.childGroupIds?.length || 0) > 0;
        const isExpanded = !this.collapsedColumnGroups.has(group.groupId);
        const canCollapse = hasChildren || ((group.lastColumnIndex ?? 0) - (group.firstColumnIndex ?? 0)) > 0;

        return {
            groupId: group.groupId,
            level: group.level,
            hasChildren,
            isExpanded,
            canCollapse,
        };
    }

    // ========== Cell related business logic ==========

    /**
     * Get cell value
     * @param rowIndex Original row index (position in rowHeaders)
     * @param columnIndex Original column index (position in columnHeaders)
     * @param valueFieldIndex Value field index (if multiple value fields)
     */
    getCellValue(
        rowIndex: number,
        columnIndex: number,
        valueFieldIndex: number = 0
    ): number | string | null {
        const rowValues = this.data.structure.values[rowIndex];
        if (!rowValues) {
            return null;
        }

        const columnValues = rowValues[columnIndex];
        if (!columnValues) {
            return null;
        }

        return columnValues[valueFieldIndex] ?? null;
    }

    /**
     * Get complete cell information (for rendering)
     */
    getCellInfo(
        rowIndex: number,
        columnIndex: number,
        valueFieldIndex: number = 0
    ): {
        value: number | string | null;
        rowType: 'data' | 'subtotal';
        columnType: 'data' | 'subtotal';
        isVisible: boolean;
        rowHeaders: string[];
        columnHeaders: string[];
    } {
        const value = this.getCellValue(rowIndex, columnIndex, valueFieldIndex);
        const rowType = this.data.structure.rowTypes[rowIndex] || 'data';
        const columnType = this.data.structure.columnTypes[columnIndex] || 'data';
        const isVisible = this.isRowVisible(rowIndex) && this.isColumnVisible(columnIndex);
        const rowHeaders = this.data.structure.rowHeaders[rowIndex] || [];
        const columnHeaders = this.data.structure.columnHeaders[columnIndex] || [];

        return {
            value,
            rowType,
            columnType,
            isVisible,
            rowHeaders,
            columnHeaders,
        };
    }

    // ========== Collapse/Expand operations ==========

    /**
     * Toggle row group collapse/expand state
     */
    toggleRowGroup(groupId: string): void {
        if (this.collapsedRowGroups.has(groupId)) {
            this.collapsedRowGroups.delete(groupId);
        } else {
            this.collapsedRowGroups.add(groupId);
        }
    }

    /**
     * Toggle column group collapse/expand state
     */
    toggleColumnGroup(groupId: string): void {
        if (this.collapsedColumnGroups.has(groupId)) {
            this.collapsedColumnGroups.delete(groupId);
        } else {
            this.collapsedColumnGroups.add(groupId);
        }
    }

    /**
     * Expand all row groups
     */
    expandAllRows(): void {
        this.collapsedRowGroups.clear();
    }

    /**
     * Collapse all row groups
     */
    collapseAllRows(): void {
        if (this.data.structure.rowGroups) {
            this.data.structure.rowGroups.forEach((group) => {
                if (group.level === 0) { // Only collapse outermost level
                    this.collapsedRowGroups.add(group.groupId);
                }
            });
        }
    }

    /**
     * Expand all column groups
     */
    expandAllColumns(): void {
        this.collapsedColumnGroups.clear();
    }

    /**
     * Collapse all column groups
     */
    collapseAllColumns(): void {
        if (this.data.structure.columnGroups) {
            this.data.structure.columnGroups.forEach((group) => {
                if (group.level === 0) {
                    this.collapsedColumnGroups.add(group.groupId);
                }
            });
        }
    }

    // ========== Subtotal and Grand Total related business logic ==========

    /**
     * Get subtotal row information
     */
    getSubtotalRowInfo(rowIndex: number): IPivotSubtotalInfo | null {
        return this.data.structure.subtotalRows?.find(
            (sr) => sr.rowIndex === rowIndex
        ) || null;
    }

    /**
     * Get subtotal column information
     */
    getSubtotalColumnInfo(columnIndex: number): IPivotSubtotalInfo | null {
        return this.data.structure.subtotalColumns?.find(
            (sc) => sc.columnIndex === columnIndex
        ) || null;
    }

    /**
     * Check if row is grand total row
     * Grand total = first field's subtotal (level === 0 && fieldIndex === 0)
     */
    isGrandTotalRow(rowIndex: number): boolean {
        const subtotalRow = this.data.structure.subtotalRows?.find(
            (sr) => sr.rowIndex === rowIndex
        );

        if (!subtotalRow) {
            return false;
        }

        // First field (fieldIndex === 0) outermost subtotal (level === 0) is grand total row
        return subtotalRow.level === 0 && subtotalRow.fieldIndex === 0;
    }

    /**
     * Check if column is grand total column
     */
    isGrandTotalColumn(columnIndex: number): boolean {
        const subtotalColumn = this.data.structure.subtotalColumns?.find(
            (sc) => sc.columnIndex === columnIndex
        );

        if (!subtotalColumn) {
            return false;
        }

        return subtotalColumn.level === 0 && subtotalColumn.fieldIndex === 0;
    }

    // ========== Value field related business logic ==========

    /**
     * Get value field headers
     */
    getValueFieldHeaders(): string[] {
        return this.data.structure.valueFieldHeaders || [];
    }

    /**
     * Get value field count
     */
    getValueFieldCount(): number {
        return this.data.dimensions.valueFieldCount;
    }

    // ========== Utility methods ==========

    /**
     * Get visible area dimensions (for calculating scroll area)
     */
    getVisibleDimensions(): {
        visibleRowCount: number;
        visibleColumnCount: number;
        totalRowCount: number;
        totalColumnCount: number;
    } {
        const visibleRowIndices = this.getVisibleRowIndices();
        const visibleColumnIndices = this.getVisibleColumnIndices();

        return {
            visibleRowCount: visibleRowIndices.length,
            visibleColumnCount: visibleColumnIndices.length,
            totalRowCount: this.data.structure.rowHeaders.length,
            totalColumnCount: this.data.structure.columnHeaders.length,
        };
    }

    /**
     * Map original index to visible index
     * @param originalIndex Original row/column index
     * @param isRow Whether it's a row index
     */
    getVisibleIndex(originalIndex: number, isRow: boolean): number {
        const visibleIndices = isRow
            ? this.getVisibleRowIndices()
            : this.getVisibleColumnIndices();

        return visibleIndices.indexOf(originalIndex);
    }

    /**
     * Map visible index back to original index
     * @param visibleIndex Visible row/column index
     * @param isRow Whether it's a row index
     */
    getOriginalIndex(visibleIndex: number, isRow: boolean): number {
        const visibleIndices = isRow
            ? this.getVisibleRowIndices()
            : this.getVisibleColumnIndices();

        return visibleIndices[visibleIndex] ?? -1;
    }
}
