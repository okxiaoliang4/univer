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
import type { TransformListResult, TransformResult } from '@univerjs/univer-ot-wasm';
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

const NOOP_MUTATION_ID = '__noop__';

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
        const m1InfoList: MutationInfo[] = [];
        const m2InfoList: MutationInfo[] = [];
        let transformListResult: TransformListResult | undefined;
        const resultM1InfoList: MutationInfo[] = [];
        const resultM2InfoList: MutationInfo[] = [];

        try {
            // Convert IMutationInfo[] to MutationInfo[] for wasm
            for (const m of m1List) {
                m1InfoList.push(new MutationInfo(m.id, m.params));
            }
            for (const m of m2List) {
                m2InfoList.push(new MutationInfo(m.id, m.params));
            }

            // Create JS arrays - MutationInfo objects can be directly used in JS arrays
            const m1JsArray = m1InfoList;
            const m2JsArray = m2InfoList;

            // Call wasm transform_list
            transformListResult = this._transformService.transform_list(m1JsArray, m2JsArray);

            // Extract results and convert back to IMutationInfo[]
            // Note: getter_with_clone returns cloned Vec, so we need to keep references for cleanup
            const m1Primes: IMutationInfo[] = transformListResult.m1_prime_list.map((m, index) => {
                resultM1InfoList.push(m); // Keep reference for cleanup
                return {
                    id: m.id,
                    type: m1List[index]?.type,
                    params: m.params,
                };
            });

            const m2Primes: IMutationInfo[] = transformListResult.m2_prime_list.map((m, index) => {
                resultM2InfoList.push(m); // Keep reference for cleanup
                return {
                    id: m.id,
                    type: m2List[index]?.type,
                    params: m.params,
                };
            });

            const result: ITransformListResult = {
                m1Primes: m1Primes.filter((m) => m.id !== NOOP_MUTATION_ID),
                m2Primes: m2Primes.filter((m) => m.id !== NOOP_MUTATION_ID),
                error: transformListResult.error || undefined,
            };

            if (result.error) {
                console.error(`[TransformService] transformList error: ${result.error}`);
            }

            return result;
        } finally {
            // Free all MutationInfo instances in the result arrays
            // These are cloned from TransformListResult via getter_with_clone
            for (const m of resultM1InfoList) {
                m.free();
            }
            for (const m of resultM2InfoList) {
                m.free();
            }

            // Free the TransformListResult (this will free the original Vec<MutationInfo> containers)
            transformListResult?.free();

            // Free all MutationInfo instances in the input arrays
            for (const m of m1InfoList) {
                m.free();
            }
            for (const m of m2InfoList) {
                m.free();
            }
        }
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
