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

import type { IPivotField, IPivotFilterCriteria } from '../../types/type';

import { afterEach, describe, expect, it, vi } from 'vitest';
import { AggregationType } from '../../types/enum';
import { PivotEngine, PivotValuePosition } from '../pivot-engine';

// Helper functions to create IPivotField objects
function createValueField(name: string, sourceColumnIndex: number, aggregation: AggregationType = AggregationType.SUM): IPivotField {
    return {
        id: `value-${name}-${aggregation}`,
        sourceColumnIndex,
        name,
        aggregation,
    };
}

function createRowField(name: string, sourceColumnIndex: number): IPivotField {
    return {
        id: `row-${name}`,
        sourceColumnIndex,
        name,
    };
}

function createColumnField(name: string, sourceColumnIndex: number): IPivotField {
    return {
        id: `col-${name}`,
        sourceColumnIndex,
        name,
    };
}

function createFilterField(name: string, sourceColumnIndex: number, filter: IPivotFilterCriteria): IPivotField {
    return {
        id: `filter-${name}`,
        sourceColumnIndex,
        name,
        filter,
    };
}

describe('PivotEngine', () => {
    afterEach(() => {
        vi.clearAllMocks();
    });

    it('should be defined', () => {
        expect(PivotEngine).toBeDefined();
    });

    it('should create an instance of PivotEngine', () => {
        const pivotEngine = new PivotEngine({
            valueFields: [],
            rowFields: [],
            columnFields: [],
            filterFields: [],
            valuePosition: PivotValuePosition.Column,
            sourceData: {},
        });
    });
});
