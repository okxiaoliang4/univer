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
import type { TransformResult } from '@univerjs/univer-ot-wasm';
import { createIdentifier, Disposable } from '@univerjs/core';
import { MutationInfo, TransformService as UniverOTWasmTransformService } from '@univerjs/univer-ot-wasm';

export interface ITransformResult {
    m1Prime: IMutationInfo;
    m2Prime: IMutationInfo;
    error?: string;
}

export interface ITransformListResult {
    m1Primes: IMutationInfo[];
    m2Primes: IMutationInfo[];
    error?: string;
}

export interface ITransformService {
    compose(m1: IMutationInfo, m2: IMutationInfo): IMutationInfo[];
    transformList(m1List: IMutationInfo[], m2List: IMutationInfo[]): ITransformListResult;
    transform(m1: IMutationInfo, m2: IMutationInfo): ITransformResult;
}

export const ITransformService = createIdentifier<ITransformService>('univer.collaboration.transform.service');

export class TransformService extends Disposable implements ITransformService {
    private _transformService: UniverOTWasmTransformService;

    constructor(
    ) {
        super();
        this._transformService = new UniverOTWasmTransformService();
    }

    private _logMutation(prefix: string, m: IMutationInfo): void {
        // Using console.warn for debugging - can be changed to ILogService if needed
        console.warn(`[TransformService] ${prefix}: id=${m.id}, params=${JSON.stringify(m.params)?.substring(0, 200)}`);
    }

    compose(_m1: IMutationInfo, _m2: IMutationInfo): IMutationInfo[] {
        // TODO: 实现compose
        return [];
    }

    transformList(m1List: IMutationInfo[], m2List: IMutationInfo[]): ITransformListResult {
        // Standard OT transformation for two lists of operations
        // Given:
        //   - m1List: local pending operations (already applied locally)
        //   - m2List: server operations (need to be applied locally)
        // Returns:
        //   - m1Primes: transformed local ops to send to server
        //   - m2Primes: transformed server ops to apply locally
        //
        // The key insight is:
        //   - Local state = base + m1List (already applied)
        //   - Server state = base + m2List
        //   - To sync: apply m2Primes locally, send m1Primes to server
        //   - Both sides end up at: base + m1List + m2Primes = base + m2List + m1Primes

        if (m1List.length === 0) {
            return { m1Primes: [], m2Primes: m2List };
        }

        if (m2List.length === 0) {
            return { m1Primes: m1List, m2Primes: [] };
        }

        // Clone the lists to avoid modifying originals
        let currentM1List = [...m1List];
        const m2Primes: IMutationInfo[] = [];

        // For each server operation, transform it against all local operations
        // and update the local operations list
        for (const m2 of m2List) {
            let currentM2 = m2;
            const newM1List: IMutationInfo[] = [];

            // Transform m2 against each m1, updating both
            for (const m1 of currentM1List) {
                const transformResult = this.transform(m1, currentM2);

                if (transformResult.error) {
                    return {
                        m1Primes: [],
                        m2Primes: [],
                        error: transformResult.error,
                    };
                }

                // m1' goes to the new list
                newM1List.push(transformResult.m1Prime);
                // m2' is used for next transformation
                currentM2 = transformResult.m2Prime;
            }

            // After transforming against all m1s, add the final m2'
            m2Primes.push(currentM2);
            // Update m1 list for next m2
            currentM1List = newM1List;
        }

        return {
            m1Primes: currentM1List,
            m2Primes,
        };
    }

    transform(m1: IMutationInfo, m2: IMutationInfo): ITransformResult {
        let m1Info: MutationInfo | undefined;
        let m2Info: MutationInfo | undefined;
        let transformResult: TransformResult | undefined;
        try {
            this._logMutation('transform input m1', m1);
            this._logMutation('transform input m2', m2);

            m1Info = new MutationInfo(m1.id, m1.params);
            m2Info = new MutationInfo(m2.id, m2.params);
            transformResult = this._transformService.transform(m1Info, m2Info);

            // params is JsValue, Rust automatically handles memory lifecycle
            // No need to call free() or deepClone - params is directly mapped to JS object
            const result = {
                m1Prime: {
                    id: transformResult.m1_prime.id,
                    type: m1.type,
                    params: transformResult.m1_prime.params,
                },
                m2Prime: {
                    id: transformResult.m2_prime.id,
                    type: m2.type,
                    params: transformResult.m2_prime.params,
                },
                error: transformResult.error,
            };

            this._logMutation('transform output m1Prime', result.m1Prime);
            this._logMutation('transform output m2Prime', result.m2Prime);
            if (result.error) {
                console.error(`[TransformService] transform error: ${result.error}`);
            }

            return result;
        } finally {
            transformResult?.free();
            // Free the input MutationInfo instances
            m1Info?.free();
            m2Info?.free();
        }
    }

    override dispose(): void {
        super.dispose();
        this._transformService.free();
    }
}
