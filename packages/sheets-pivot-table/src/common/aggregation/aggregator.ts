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

import type { ICellData } from '@univerjs/core';

/**
 * Base interface for aggregation functions
 */
export interface IAggregator {
    /**
     * Initialize the aggregator
     */
    init(): void;

    /**
     * Add a value to the aggregation
     * @param value - Cell data to aggregate
     */
    addValue(value: ICellData | null | undefined): void;

    /**
     * Get the aggregated result
     * @returns The aggregated value
     */
    getResult(): number | string | null;

    /**
     * Reset the aggregator to initial state
     */
    reset(): void;
}

/**
 * Base abstract class for aggregators
 */
export abstract class BaseAggregator implements IAggregator {
    protected _count: number = 0;

    init(): void {
        this._count = 0;
    }

    abstract addValue(value: ICellData | null | undefined): void;

    abstract getResult(): number | string | null;

    reset(): void {
        this.init();
    }

    /**
     * Helper to extract numeric value from cell data
     */
    protected _extractNumericValue(value: ICellData | null | undefined): number | null {
        if (!value) return null;

        // Handle numeric values
        if (typeof value.v === 'number') {
            return value.v;
        }

        // Handle string numeric values
        if (typeof value.v === 'string') {
            const num = Number(value.v);
            return Number.isNaN(num) ? null : num;
        }

        // Handle boolean as number
        if (typeof value.v === 'boolean') {
            return value.v ? 1 : 0;
        }

        return null;
    }

    /**
     * Helper to check if cell is non-empty
     */
    protected _isNonEmpty(value: ICellData | null | undefined): boolean {
        if (!value) return false;
        return value.v !== null && value.v !== undefined && value.v !== '';
    }
}
