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
import type { WasmComposeResult, WasmTransformListResult, WasmTransformResult } from '@univerjs/univer-ot-wasm';
import { createIdentifier, Disposable } from '@univerjs/core';
import {
    TransformService as UniverOTWasmTransformService,
    WasmMutationInfo,
} from '@univerjs/univer-ot-wasm';

export interface ITransformResult {
    m1Prime?: IMutationInfo;
    m2Prime?: IMutationInfo;
    error?: string;
}

export interface ITransformListResult {
    m1Primes: IMutationInfo[];
    m2Primes: IMutationInfo[];
    error?: string;
}

export interface ITransformService {
    compose(m1: IMutationInfo, m2: IMutationInfo): IMutationInfo[];
    transformList(
        m1List: IMutationInfo[],
        m2List: IMutationInfo[],
    ): ITransformListResult;
    transform(m1: IMutationInfo, m2: IMutationInfo): ITransformResult;
    composeList(mutations: IMutationInfo[]): IMutationInfo[];
}

export const ITransformService = createIdentifier<ITransformService>(
    'univer.collaboration.transform.service'
);

export class TransformService extends Disposable implements ITransformService {
    private _transformService: UniverOTWasmTransformService;

    constructor() {
        super();
        this._transformService = new UniverOTWasmTransformService();
    }

    private _logMutation(prefix: string, m: IMutationInfo): void {
        // Using console.warn for debugging - can be changed to ILogService if needed
        console.warn(
            `[TransformService] ${prefix}: id=${m.id}, params=${JSON.stringify(m.params)?.substring(0, 200)}`
        );
    }

    compose(m1: IMutationInfo, m2: IMutationInfo): IMutationInfo[] {
        let m1Info: WasmMutationInfo | undefined;
        let m2Info: WasmMutationInfo | undefined;
        let composeResult: WasmComposeResult | undefined;
        const composed: IMutationInfo[] = [];
        const resultInfoList: WasmMutationInfo[] = [];
        try {
            m1Info = new WasmMutationInfo(m1.id, m1.params);
            m2Info = new WasmMutationInfo(m2.id, m2.params);
            composeResult = this._transformService.compose([m1Info, m2Info]);
            const mutations = composeResult.mutations;
            for (const mutation of mutations) {
                resultInfoList.push(mutation);
                composed.push({
                    id: mutation.id,
                    type: m1.type ?? m2.type,
                    params: mutation.params,
                });
            }
            return composed;
        } finally {
            for (const mutation of resultInfoList) {
                mutation.free();
            }
            if (composeResult?.free) {
                composeResult.free();
            }
            m1Info?.free();
            m2Info?.free();
        }
    }

    composeList(mutations: IMutationInfo[]): IMutationInfo[] {
        if (mutations.length <= 1) {
            return mutations;
        }

        const input: WasmMutationInfo[] = [];
        const output: IMutationInfo[] = [];
        let composeResult: WasmComposeResult | undefined;
        const resultInfoList: WasmMutationInfo[] = [];
        try {
            for (const mutation of mutations) {
                input.push(new WasmMutationInfo(mutation.id, mutation.params));
            }
            composeResult = this._transformService.compose(input);
            const resultMutations = composeResult.mutations;
            const typeHint = mutations[0]?.type;
            for (const mutation of resultMutations) {
                resultInfoList.push(mutation);
                output.push({
                    id: mutation.id,
                    type: typeHint,
                    params: mutation.params,
                });
            }
            return output;
        } finally {
            for (const mutation of resultInfoList) {
                mutation.free();
            }
            if (composeResult?.free) {
                composeResult.free();
            }
            for (const mutation of input) {
                mutation.free();
            }
        }
    }

    transformList(
        m1List: IMutationInfo[],
        m2List: IMutationInfo[]
    ): ITransformListResult {
        const m1InfoList: WasmMutationInfo[] = [];
        const m2InfoList: WasmMutationInfo[] = [];
        let transformListResult: WasmTransformListResult | undefined;
        const resultM1InfoList: WasmMutationInfo[] = [];
        const resultM2InfoList: WasmMutationInfo[] = [];

        try {
            for (const m of m1List) {
                m1InfoList.push(new WasmMutationInfo(m.id, m.params));
            }
            for (const m of m2List) {
                m2InfoList.push(new WasmMutationInfo(m.id, m.params));
            }

            transformListResult = this._transformService.transformList(
                m1InfoList,
                m2InfoList
            );

            const m1Primes: IMutationInfo[] = transformListResult.m1_prime_list.map(
                (m, index) => {
                    resultM1InfoList.push(m);
                    return {
                        id: m.id,
                        type: m1List[index]?.type,
                        params: m.params,
                    };
                }
            );

            const m2Primes: IMutationInfo[] = transformListResult.m2_prime_list.map(
                (m, index) => {
                    resultM2InfoList.push(m);
                    return {
                        id: m.id,
                        type: m2List[index]?.type,
                        params: m.params,
                    };
                }
            );

            const result: ITransformListResult = {
                m1Primes,
                m2Primes,
                error: transformListResult.error || undefined,
            };

            if (result.error) {
                console.error(
                    `[TransformService] transformList error: ${result.error}`
                );
            }

            return result;
        } finally {
            for (const m of resultM1InfoList) {
                m.free();
            }
            for (const m of resultM2InfoList) {
                m.free();
            }

            if (transformListResult?.free) {
                transformListResult.free();
            }

            for (const m of m1InfoList) {
                m.free();
            }
            for (const m of m2InfoList) {
                m.free();
            }
        }
    }

    transform(m1: IMutationInfo, m2: IMutationInfo): ITransformResult {
        let m1Info: WasmMutationInfo | undefined;
        let m2Info: WasmMutationInfo | undefined;
        let transformResult: WasmTransformResult | undefined;
        const resultM1InfoList: WasmMutationInfo[] = [];
        const resultM2InfoList: WasmMutationInfo[] = [];
        try {
            this._logMutation('transform input m1', m1);
            this._logMutation('transform input m2', m2);

            m1Info = new WasmMutationInfo(m1.id, m1.params);
            m2Info = new WasmMutationInfo(m2.id, m2.params);
            transformResult = this._transformService.transform(m1Info, m2Info);

            const m1Prime = transformResult.m1_prime;
            const m2Prime = transformResult.m2_prime;

            if (m1Prime) {
                resultM1InfoList.push(m1Prime);
            }
            if (m2Prime) {
                resultM2InfoList.push(m2Prime);
            }

            const result: ITransformResult = {
                m1Prime: m1Prime
                    ? {
                        id: m1Prime.id,
                        type: m1.type,
                        params: m1Prime.params,
                    }
                    : undefined,
                m2Prime: m2Prime
                    ? {
                        id: m2Prime.id,
                        type: m2.type,
                        params: m2Prime.params,
                    }
                    : undefined,
                error: transformResult.error || undefined,
            };

            if (result.m1Prime) {
                this._logMutation('transform output m1Prime', result.m1Prime);
            }
            if (result.m2Prime) {
                this._logMutation('transform output m2Prime', result.m2Prime);
            }
            if (result.error) {
                console.error(`[TransformService] transform error: ${result.error}`);
            }

            return result;
        } finally {
            for (const mutation of resultM1InfoList) {
                mutation.free();
            }
            for (const mutation of resultM2InfoList) {
                mutation.free();
            }
            transformResult?.free();
            m1Info?.free();
            m2Info?.free();
        }
    }

    override dispose(): void {
        super.dispose();
        this._transformService.free();
    }
}
