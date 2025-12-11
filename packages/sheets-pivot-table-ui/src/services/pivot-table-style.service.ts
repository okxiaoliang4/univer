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

import type { IBorderData, IColorStyle, IStyleData } from '@univerjs/core';
import type { PivotCellType } from '@univerjs/sheets-pivot-table';
import { BooleanNumber, BorderStyleTypes, createIdentifier, Disposable, Inject } from '@univerjs/core';
import { IPivotTableRangeService } from '@univerjs/sheets-pivot-table';
import tinycolor from 'tinycolor2';

/**
 * Green theme color palette for pivot table styling
 */
const PIVOT_GREEN_PALETTE = {
    // Header area - Dark green background
    headerBg: { rgb: '#2E7D32' },
    headerFg: { rgb: '#FFFFFF' },

    // Data area - Alternating light green
    dataOddBg: { rgb: '#E8F5E9' }, // Green 50
    dataEvenBg: { rgb: '#C8E6C9' }, // Green 100

    // Subtotal rows - Medium green
    subtotalBg: { rgb: '#A5D6A7' }, // Green 200

    // Grand total rows - Medium green
    grandTotalBg: { rgb: '#81C784' }, // Green 300

    // Border color
    border: { rgb: '#4CAF50' }, // Green 500
} as const;

/**
 * Service interface for pivot table style calculation
 */
export interface IPivotTableStyleService {
    /**
     * Calculate style for a pivot table cell based on its position and type
     * @param pivotTableId - The pivot table identifier
     * @param row - Row index in the output matrix (0-based)
     * @param col - Column index in the output matrix (0-based)
     * @param cellType - Type of cell (header, data, subtotal, etc.)
     * @param level - Hierarchy level (used for color graduation, defaults to 0)
     * @returns Style data object
     */
    getCellStyle(
        pivotTableId: string,
        row: number,
        col: number,
        cellType: PivotCellType,
        level?: number
    ): IStyleData;
}

/**
 * Pivot table style service implementing green theme styling
 *
 * Provides styling for different pivot table cell types with a green color scheme:
 * - Header areas: Dark green background with white text
 * - Data areas: Alternating light green backgrounds (zebra stripes)
 * - Subtotal/Total areas: Medium green backgrounds
 * - Borders: Green outlines for visual separation
 */
export class PivotTableStyleService extends Disposable implements IPivotTableStyleService {
    constructor(
        @Inject(IPivotTableRangeService) private readonly _rangeService: IPivotTableRangeService
    ) {
        super();
    }

    getCellStyle(
        pivotTableId: string,
        row: number,
        col: number,
        cellType: PivotCellType,
        level: number = 0
    ): IStyleData {
        const baseStyle: IStyleData = {};

        switch (cellType) {
            case 'header':
            case 'rowHeader':
            case 'columnHeader':
                return this._getHeaderStyle(level);
            case 'data':
                return this._getDataStyle(row);
            case 'subtotal':
                return this._getSubtotalStyle();
            case 'grandTotal':
                return this._getGrandTotalStyle();
            default:
                return baseStyle;
        }
    }

    /**
     * Get header area style with level-based color graduation
     */
    private _getHeaderStyle(level: number): IStyleData {
        // Calculate color intensity based on level (deeper levels get lighter)
        const intensity = Math.max(0, 100 - level * 20); // Decrease by 20% per level
        const bgColor = this._adjustColorIntensity(PIVOT_GREEN_PALETTE.headerBg, intensity);

        return {
            bg: bgColor,
            cl: PIVOT_GREEN_PALETTE.headerFg,
            bl: BooleanNumber.TRUE, // Bold text
            bd: this._getBorderStyle(),
        };
    }

    /**
     * Get data area style with zebra stripe pattern
     */
    private _getDataStyle(row: number): IStyleData {
        const isEvenRow = row % 2 === 0;
        const bgColor = isEvenRow ? PIVOT_GREEN_PALETTE.dataEvenBg : PIVOT_GREEN_PALETTE.dataOddBg;

        return {
            bg: bgColor,
            bd: this._getBorderStyle(),
        };
    }

    /**
     * Get subtotal row style
     */
    private _getSubtotalStyle(): IStyleData {
        return {
            bg: PIVOT_GREEN_PALETTE.subtotalBg,
            bl: BooleanNumber.TRUE, // Bold text
            bd: this._getBorderStyle(),
        };
    }

    /**
     * Get grand total row style
     */
    private _getGrandTotalStyle(): IStyleData {
        // Use the darkest header tone to make grand totals stand out
        return {
            bg: PIVOT_GREEN_PALETTE.headerBg,
            cl: PIVOT_GREEN_PALETTE.headerFg,
            bl: BooleanNumber.TRUE,
            bd: this._getBorderStyle(),
        };
    }

    /**
     * Get consistent border style for all cells
     */
    private _getBorderStyle(): IBorderData {
        return {
            t: { s: BorderStyleTypes.THIN, cl: PIVOT_GREEN_PALETTE.border },
            b: { s: BorderStyleTypes.THIN, cl: PIVOT_GREEN_PALETTE.border },
            l: { s: BorderStyleTypes.THIN, cl: PIVOT_GREEN_PALETTE.border },
            r: { s: BorderStyleTypes.THIN, cl: PIVOT_GREEN_PALETTE.border },
        };
    }

    /**
     * Adjust color intensity by mixing with white
     * @param baseColor - Original color
     * @param intensity - Intensity percentage (0-100, where 100 is full intensity)
     */
    private _adjustColorIntensity(baseColor: IColorStyle, intensity: number): IColorStyle {
        // Clamp intensity to 0-100 where 100 = full base color, 0 = white
        const clamped = Math.min(100, Math.max(0, intensity));
        const baseHex = (baseColor as { rgb?: string }).rgb || '#000000';
        const mixed = tinycolor.mix('#ffffff', baseHex, clamped);
        return { rgb: mixed.toHexString() };
    }
}

export const IPivotTableStyleService = createIdentifier<IPivotTableStyleService>('sheets-pivot-table-ui.pivot-table-style-service');
