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

import { describe, expect, it } from 'vitest';
import { TransformService } from '../transform.service';
import {
    createInsertColMutation,
    createInsertRowMutation,
    createRemoveRowMutation,
    createSetRangeValuesMutation,
    MockLogService,
} from './test-utils';

describe('TransformService', () => {
    it('transforms SetRangeValues × InsertCol', async () => {
        const service = new TransformService(new MockLogService());
        const m1 = createSetRangeValuesMutation();
        const m2 = createInsertColMutation();

        const resultA = await service.transform(m1, m2);
        expect(resultA.error).toBeUndefined();
        expect(resultA.m1Prime).toBeDefined();
        const cellValue = (
            resultA.m1Prime as unknown as {
                params: { cellValue: Array<Record<number, { v: string }>> };
            }
        ).params.cellValue;
        expect(cellValue[0]?.[0]).toBeUndefined();
        expect(cellValue[0]?.[1].v).toBe('1');
        service.dispose();
    });

    it('transforms SetRangeValues × InsertRow', async () => {
        const service = new TransformService(new MockLogService());
        const m1 = createSetRangeValuesMutation();
        const m2 = createInsertRowMutation();

        const resultA = await service.transform(m1, m2);
        expect(resultA.error).toBeUndefined();
        expect(resultA.m1Prime).toBeDefined();
        const cellValue = (
            resultA.m1Prime as unknown as {
                params: { cellValue: Array<Record<number, { v: string }>> };
            }
        ).params.cellValue;
        expect(cellValue[0]?.[0]).toBeUndefined();
        expect(cellValue[1]?.[0].v).toBe('1');
        service.dispose();
    });

    it('transform list SetRangeValues × InsertRow', async () => {
        const service = new TransformService(new MockLogService());
        const m1 = createSetRangeValuesMutation();
        const m2 = createInsertRowMutation();

        const resultA = await service.transformList([m1], [m2]);
        expect(resultA.error).toBeUndefined();
        expect(resultA.m1Primes).toHaveLength(1);
        expect(resultA.m2Primes).toHaveLength(1);
        service.dispose();
    });

    it('returns identity when one list is empty', async () => {
        const service = new TransformService(new MockLogService());
        const m1 = createSetRangeValuesMutation();
        const m2 = createInsertRowMutation();

        const resultA = await service.transformList([], [m2]);
        expect(resultA.error).toBeUndefined();
        expect(resultA.m1Primes).toEqual([]);
        expect(resultA.m2Primes).toEqual([m2]);

        const resultB = await service.transformList([m1], []);
        expect(resultB.error).toBeUndefined();
        expect(resultB.m1Primes).toEqual([m1]);
        expect(resultB.m2Primes).toEqual([]);
        service.dispose();
    });

    it('transforms multiple server mutations sequentially', async () => {
        const service = new TransformService(new MockLogService());
        const m1List = [createSetRangeValuesMutation(), createInsertColMutation()];
        const m2List = [createInsertRowMutation(), createRemoveRowMutation()];

        const result = await service.transformList(m1List, m2List);
        expect(result.error).toBeUndefined();
        expect(result.m1Primes).toHaveLength(m1List.length);
        expect(result.m2Primes).toHaveLength(m2List.length);
        expect(result.m1Primes.map((m) => m.id)).toEqual(m1List.map((m) => m.id));
        expect(result.m2Primes.map((m) => m.id)).toEqual(m2List.map((m) => m.id));
        service.dispose();
    });

    it('transforms supported mutation pairs without errors', async () => {
        const service = new TransformService(new MockLogService());
        const m1 = createSetRangeValuesMutation();
        const m2 = createInsertRowMutation();

        const result = await service.transform(m1, m2);
        expect(result.error).toBeUndefined();
        expect(result.m1Prime).toBeDefined();
        expect(result.m2Prime).toBeDefined();
        expect((result.m1Prime as { id: string }).id).toBe(m1.id);
        expect((result.m2Prime as { id: string }).id).toBe(m2.id);
        service.dispose();
    });

    it('composes two mutations', async () => {
        const service = new TransformService(new MockLogService());
        const m1 = createSetRangeValuesMutation();
        const m2 = createSetRangeValuesMutation();

        const result = await service.compose(m1, m2);
        expect(result).toBeDefined();
        expect(Array.isArray(result)).toBe(true);
        service.dispose();
    });

    it('composes a list of mutations', async () => {
        const service = new TransformService(new MockLogService());
        const mutations = [
            createSetRangeValuesMutation(),
            createSetRangeValuesMutation(),
        ];

        const result = await service.composeList(mutations);
        expect(result).toBeDefined();
        expect(Array.isArray(result)).toBe(true);
        service.dispose();
    });

    it('returns same mutation for single-element composeList', async () => {
        const service = new TransformService(new MockLogService());
        const m1 = createSetRangeValuesMutation();

        const result = await service.composeList([m1]);
        expect(result).toEqual([m1]);
        service.dispose();
    });

    it('returns empty array for empty composeList', async () => {
        const service = new TransformService(new MockLogService());

        const result = await service.composeList([]);
        expect(result).toEqual([]);
        service.dispose();
    });
});
