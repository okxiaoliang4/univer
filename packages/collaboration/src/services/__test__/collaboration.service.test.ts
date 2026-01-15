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

import type { IMutationInfo } from '@univerjs/core';
import { CommandType } from '@univerjs/core';
import { SetRangeValuesMutation } from '@univerjs/sheets';
import { describe, expect, it } from 'vitest';
import { CollaborationService } from '../collaboration.service';

describe('CollaborationService', () => {
    it('should be defined', () => {
        const collaborationService = new CollaborationService();
        expect(collaborationService).toBeDefined();
    });

    it('should transform two mutations', () => {
        const collaborationService = new CollaborationService();
        const m1: IMutationInfo = {
            id: SetRangeValuesMutation.id,
            type: CommandType.MUTATION,
            params: {
                0: {
                    0: {
                        v: '1',
                    },
                },
            },
        };
        const m2: IMutationInfo = {
            id: SetRangeValuesMutation.id,
            type: CommandType.MUTATION,
            params: {
                0: {
                    0: {
                        v: '2',
                    },
                },
            },
        };
        const result = collaborationService.transform(m1, m2);

        expect(result.m1Prime).toBeDefined();
        expect(result.m2Prime).toBeDefined();
        expect(result.error).toBeUndefined();
    });
});
