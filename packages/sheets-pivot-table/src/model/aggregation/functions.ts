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
import type { IAggregator } from './aggregator';
import { AggregationType } from '../../types/enum';
import { BaseAggregator } from './aggregator';

/**
 * SUM aggregation function
 */
export class SumAggregator extends BaseAggregator {
    private _sum: number = 0;

    override init(): void {
        super.init();
        this._sum = 0;
    }

    override addValue(value: ICellData | null | undefined): void {
        const num = this._extractNumericValue(value);
        if (num !== null) {
            this._sum += num;
            this._count++;
        }
    }

    override getResult(): number | null {
        return this._count > 0 ? this._sum : null;
    }
}

/**
 * COUNT aggregation function
 */
export class CountAggregator extends BaseAggregator {
    override init(): void {
        super.init();
    }

    override addValue(value: ICellData | null | undefined): void {
        if (this._isNonEmpty(value)) {
            this._count++;
        }
    }

    override getResult(): number {
        return this._count;
    }
}

/**
 * AVERAGE aggregation function
 */
export class AverageAggregator extends BaseAggregator {
    private _sum: number = 0;
    private _numericCount: number = 0;

    override init(): void {
        super.init();
        this._sum = 0;
        this._numericCount = 0;
    }

    override addValue(value: ICellData | null | undefined): void {
        const num = this._extractNumericValue(value);
        if (num !== null) {
            this._sum += num;
            this._numericCount++;
        }
    }

    override getResult(): number | null {
        return this._numericCount > 0 ? this._sum / this._numericCount : null;
    }
}

/**
 * MIN aggregation function
 */
export class MinAggregator extends BaseAggregator {
    private _min: number = Number.POSITIVE_INFINITY;

    override init(): void {
        super.init();
        this._min = Number.POSITIVE_INFINITY;
    }

    override addValue(value: ICellData | null | undefined): void {
        const num = this._extractNumericValue(value);
        if (num !== null) {
            this._min = Math.min(this._min, num);
            this._count++;
        }
    }

    override getResult(): number | null {
        return this._count > 0 ? this._min : null;
    }
}

/**
 * MAX aggregation function
 */
export class MaxAggregator extends BaseAggregator {
    private _max: number = Number.NEGATIVE_INFINITY;

    override init(): void {
        super.init();
        this._max = Number.NEGATIVE_INFINITY;
    }

    override addValue(value: ICellData | null | undefined): void {
        const num = this._extractNumericValue(value);
        if (num !== null) {
            this._max = Math.max(this._max, num);
            this._count++;
        }
    }

    override getResult(): number | null {
        return this._count > 0 ? this._max : null;
    }
}

/**
 * Factory function to create aggregators
 */
export function createAggregator(type: AggregationType): IAggregator {
    switch (type) {
        case AggregationType.SUM:
            return new SumAggregator();
        case AggregationType.COUNT:
            return new CountAggregator();
        case AggregationType.AVERAGE:
            return new AverageAggregator();
        case AggregationType.MIN:
            return new MinAggregator();
        case AggregationType.MAX:
            return new MaxAggregator();
        default:
            throw new Error(`Unknown aggregation type: ${type}`);
    }
}
