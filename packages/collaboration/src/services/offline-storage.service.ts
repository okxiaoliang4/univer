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

import type { Nullable } from '@univerjs/core';
import type { Observable } from 'rxjs';
import type {
    ICollaborationConfig,
} from '../controller/config.schema';
import type { IMutationWithOpId } from './collaboration.types';
import { createIdentifier, Disposable, IConfigService } from '@univerjs/core';
import localforage from 'localforage';
import { BehaviorSubject } from 'rxjs';
import { COLLABORATION_PLUGIN_CONFIG_KEY } from '../controller/config.schema';

/**
 * Extract userId from JWT token without verification.
 * This is safe for client-side usage since the server will verify the token.
 * The userId is used only for local storage namespacing.
 */
function extractUserIdFromToken(token: string): string {
    if (!token) {
        return '';
    }
    try {
        const parts = token.split('.');
        if (parts.length !== 3) {
            return '';
        }
        const payload = parts[1];
        // Handle base64url encoding (replace - with + and _ with /)
        const base64 = payload.replace(/-/g, '+').replace(/_/g, '/');
        const decoded = JSON.parse(atob(base64));
        // Try common JWT claim names for user ID
        return decoded.uid || decoded.sub || decoded.user_id || decoded.userId || '';
    } catch {
        return '';
    }
}

export interface IPendingMutations {
    unitId: string;
    mutations: IMutationWithOpId[];
    baseRev: number;
    userId: string;
}

export interface IPendingMutationSerivce {
    ready$: Observable<boolean>;
    isReady(): boolean;
    add(unitId: string, mutations: IMutationWithOpId[], baseRev: number): Promise<IPendingMutations>;
    has(unitId: string): boolean;
    update(unitId: string, mutations: IMutationWithOpId[], baseRev: number): Promise<IPendingMutations>;
    get(unitId: string): IMutationWithOpId[];
    getBaseRev(unitId: string): Promise<number>;
    removeByOpIds(unitId: string, opIds: string[]): Promise<IPendingMutations>;
    clear(unitId: string): Promise<void>;
}

export const IPendingMutationSerivce = createIdentifier<IPendingMutationSerivce>('univer.collaboration.pending-mutation-serivce');

const DB_NAME = 'univer-collaboration';
const STORE_NAME = 'pending-mutations';
const DB_VERSION = 1;

export class PendingMutationSerivce extends Disposable implements IPendingMutationSerivce {
    private readonly _readySubject = new BehaviorSubject<boolean>(false);
    readonly ready$ = this._readySubject.asObservable();
    private _storage: Nullable<LocalForage> = null;
    private _userId: string;
    private _pendingMutations: Map<string, IMutationWithOpId[]> = new Map();

    constructor(
        @IConfigService private readonly _configService: IConfigService
    ) {
        super();

        const config = this._configService.getConfig<ICollaborationConfig>(COLLABORATION_PLUGIN_CONFIG_KEY)!;
        this._userId = extractUserIdFromToken(config.accessToken);

        this._init();
    }

    private async _init() {
        this._storage = localforage.createInstance({
            driver: localforage.INDEXEDDB,
            name: DB_NAME,
            version: DB_VERSION,
            storeName: STORE_NAME,
        });
        await this._storage.ready();
        await this._storage.iterate<IPendingMutations, void>((item) => {
            // 只加载当前用户的 pending mutations
            if (item.userId !== this._userId) {
                return;
            }
            this._pendingMutations.set(item.unitId, item.mutations);
        });

        this._readySubject.next(true);
    }

    isReady(): boolean {
        return this._readySubject.value;
    }

    private _getKey(unitId: string) {
        return `${this._userId}-${unitId}`;
    }

    has(unitId: string): boolean {
        return this._pendingMutations.has(unitId);
    }

    async add(unitId: string, mutations: IMutationWithOpId[], baseRev: number): Promise<IPendingMutations> {
        const existing = this._pendingMutations.get(unitId);
        const accumulated = existing ? [...existing, ...mutations] : mutations;
        this._pendingMutations.set(unitId, accumulated);
        await this._storage?.setItem<IPendingMutations>(this._getKey(unitId), {
            unitId,
            mutations: accumulated,
            baseRev,
            userId: this._userId,
        });

        return {
            unitId,
            mutations: accumulated,
            baseRev,
            userId: this._userId,
        };
    }

    get(unitId: string): IMutationWithOpId[] {
        return this._pendingMutations.get(unitId) ?? [];
    }

    async getBaseRev(unitId: string): Promise<number> {
        const stored = await this._storage?.getItem<IPendingMutations>(
            this._getKey(unitId)
        );
        return stored?.baseRev ?? 0;
    }

    async update(unitId: string, mutations: IMutationWithOpId[], baseRev: number): Promise<IPendingMutations> {
        this._pendingMutations.set(unitId, mutations);
        await this._storage?.setItem<IPendingMutations>(this._getKey(unitId), {
            unitId,
            mutations,
            baseRev,
            userId: this._userId,
        });
        return {
            unitId,
            mutations,
            baseRev,
            userId: this._userId,
        };
    }

    async removeByOpIds(unitId: string, opIds: string[]): Promise<IPendingMutations> {
        const current = this._pendingMutations.get(unitId) ?? [];
        const stored = await this._storage?.getItem<IPendingMutations>(
            this._getKey(unitId)
        );
        const currentBaseRev = stored?.baseRev ?? 0;
        if (opIds.length === 0) {
            return {
                unitId,
                mutations: current,
                baseRev: currentBaseRev,
                userId: this._userId,
            };
        }
        const toRemove = new Set(opIds);
        const remaining = current.filter((mutation) => !toRemove.has(mutation.opId));
        this._pendingMutations.set(unitId, remaining);
        await this._storage?.setItem<IPendingMutations>(this._getKey(unitId), {
            unitId,
            mutations: remaining,
            baseRev: currentBaseRev,
            userId: this._userId,
        });
        return {
            unitId,
            mutations: remaining,
            baseRev: currentBaseRev,
            userId: this._userId,
        };
    }

    async clear(unitId: string): Promise<void> {
        this._pendingMutations.delete(unitId);
        await this._storage?.removeItem(this._getKey(unitId));
    }

    override dispose(): void {
        super.dispose();

        this._storage = null;
        this._pendingMutations.clear();
    }
}
