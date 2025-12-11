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

import type { ICellData, IObjectMatrixPrimitiveType, Nullable } from '@univerjs/core';

export type FieldType = 'numeric' | 'text';

/**
 * Service for detecting field data types
 */
export class FieldTypeDetectionService {
    /**
     * Detect if a field (column) contains numeric or text data
     * @param sourceData - The source data matrix
     * @param columnIndex - The column index to analyze (0-based, relative to source range)
     * @returns 'numeric' if all non-empty values are numeric, 'text' otherwise
     */
    detectFieldType(
        sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>,
        columnIndex: number
    ): FieldType {
        const values: (string | number | null | undefined)[] = [];

        // Extract all values from the column (skip header row, which is typically row 0)
        for (const [rowKey, rowData] of Object.entries(sourceData)) {
            const row = Number.parseInt(rowKey, 10);
            // Skip header row (row 0)
            if (row === 0) {
                continue;
            }

            const cellData = rowData?.[columnIndex];
            if (cellData == null) {
                continue;
            }

            const value = cellData.v;
            if (value == null || value === '') {
                continue;
            }

            values.push(value);
        }

        // If no values found, default to text
        if (values.length === 0) {
            return 'text';
        }

        // Check if all values are numeric
        const allNumeric = values.every((val) => {
            if (typeof val === 'number') {
                return true;
            }
            if (typeof val === 'string') {
                // Try to parse as number
                const trimmed = val.trim();
                if (trimmed === '') {
                    return false;
                }
                // Check if it's a valid number (including decimals, negative numbers, scientific notation)
                return !Number.isNaN(Number(trimmed)) && trimmed !== '';
            }
            return false;
        });

        return allNumeric ? 'numeric' : 'text';
    }
}
