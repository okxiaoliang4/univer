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
} from './test-utils';

describe('TransformService', () => {
    it('returns identity when one list is empty', () => {
        const service = new TransformService();
        const m1 = createSetRangeValuesMutation();
        const m2 = createInsertRowMutation();

        const resultA = service.transformList([], [m2]);
        expect(resultA.error).toBeUndefined();
        expect(resultA.m1Primes).toEqual([]);
        expect(resultA.m2Primes).toEqual([m2]);

        const resultB = service.transformList([m1], []);
        expect(resultB.error).toBeUndefined();
        expect(resultB.m1Primes).toEqual([m1]);
        expect(resultB.m2Primes).toEqual([]);
        service.dispose();
    });

    it('transforms multiple server mutations sequentially', () => {
        const service = new TransformService();
        const m1List = [createSetRangeValuesMutation(), createInsertColMutation()];
        const m2List = [createInsertRowMutation(), createRemoveRowMutation()];

        const result = service.transformList(m1List, m2List);
        expect(result.error).toBeUndefined();
        expect(result.m1Primes).toHaveLength(m1List.length);
        expect(result.m2Primes).toHaveLength(m2List.length);
        expect(result.m1Primes.map((m) => m.id)).toEqual(m1List.map((m) => m.id));
        expect(result.m2Primes.map((m) => m.id)).toEqual(m2List.map((m) => m.id));
        service.dispose();
    });

    it('transforms supported mutation pairs without errors', () => {
        const service = new TransformService();
        const m1 = createSetRangeValuesMutation();
        const m2 = createInsertRowMutation();

        const result = service.transform(m1, m2);
        expect(result.error).toBeUndefined();
        expect(result.m1Prime.id).toBe(m1.id);
        expect(result.m2Prime.id).toBe(m2.id);
        service.dispose();
    });
});
