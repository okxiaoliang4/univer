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
import { createIdentifier, Disposable, ILogService } from '@univerjs/core';

export interface IPendingMutations {
    unitId: string;
    mutations: IMutationInfo[];
    baseRev: number;
    userId: string;
    timestamp: number;
}

export interface IOfflineStorageService {
    savePendingMutations(unitId: string, mutations: IMutationInfo[], baseRev: number, userId: string): Promise<void>;
    loadPendingMutations(unitId: string): Promise<IPendingMutations | null>;
    loadAllPendingMutations(): Promise<IPendingMutations[]>;
    clearPendingMutations(unitId: string): Promise<void>;
    clearAllPendingMutations(): Promise<void>;
}

export const IOfflineStorageService = createIdentifier<IOfflineStorageService>('univer.collaboration.offline-storage.service');

const DB_NAME = 'univer-collaboration';
const STORE_NAME = 'pending-mutations';
const DB_VERSION = 1;

export class OfflineStorageService extends Disposable implements IOfflineStorageService {
    private _db: IDBDatabase | null = null;
    private _initPromise: Promise<void> | null = null;

    constructor(
        @ILogService private readonly _logger: ILogService
    ) {
        super();
        this._initPromise = this._initDB();
    }

    private async _initDB(): Promise<void> {
        return new Promise((resolve, reject) => {
            const request = indexedDB.open(DB_NAME, DB_VERSION);

            request.onerror = () => {
                this._logger.error('Failed to open IndexedDB:', request.error);
                reject(request.error);
            };

            request.onsuccess = () => {
                this._db = request.result;
                this._logger.log('IndexedDB opened successfully');
                resolve();
            };

            request.onupgradeneeded = (event) => {
                const db = (event.target as IDBOpenDBRequest).result;
                if (!db.objectStoreNames.contains(STORE_NAME)) {
                    const objectStore = db.createObjectStore(STORE_NAME, { keyPath: 'unitId' });
                    objectStore.createIndex('timestamp', 'timestamp', { unique: false });
                }
            };
        });
    }

    private async _ensureDB(): Promise<IDBDatabase> {
        if (this._initPromise) {
            await this._initPromise;
            this._initPromise = null;
        }
        if (!this._db) {
            throw new Error('IndexedDB not initialized');
        }
        return this._db;
    }

    async savePendingMutations(unitId: string, mutations: IMutationInfo[], baseRev: number, userId: string): Promise<void> {
        const db = await this._ensureDB();

        // First, load existing mutations to accumulate
        const existing = await this.loadPendingMutations(unitId);

        return new Promise((resolve, reject) => {
            const transaction = db.transaction([STORE_NAME], 'readwrite');
            const store = transaction.objectStore(STORE_NAME);

            // Accumulate mutations instead of replacing
            const accumulatedMutations = existing ? [...existing.mutations, ...mutations] : mutations;
            // Keep the original baseRev if it exists (from the first mutation)
            const effectiveBaseRev = existing ? existing.baseRev : baseRev;

            const data: IPendingMutations = {
                unitId,
                mutations: accumulatedMutations,
                baseRev: effectiveBaseRev,
                userId,
                timestamp: Date.now(),
            };
            const request = store.put(data);

            request.onerror = () => {
                this._logger.error('Failed to save pending mutations:', request.error);
                reject(request.error);
            };

            request.onsuccess = () => {
                this._logger.log(`Saved pending mutations for unitId: ${unitId}, count: ${accumulatedMutations.length} (added ${mutations.length})`);
                resolve();
            };
        });
    }

    async loadPendingMutations(unitId: string): Promise<IPendingMutations | null> {
        const db = await this._ensureDB();
        return new Promise((resolve, reject) => {
            const transaction = db.transaction([STORE_NAME], 'readonly');
            const store = transaction.objectStore(STORE_NAME);
            const request = store.get(unitId);

            request.onerror = () => {
                this._logger.error('Failed to load pending mutations:', request.error);
                reject(request.error);
            };

            request.onsuccess = () => {
                const result = request.result as IPendingMutations | undefined;
                resolve(result || null);
            };
        });
    }

    async loadAllPendingMutations(): Promise<IPendingMutations[]> {
        const db = await this._ensureDB();
        return new Promise((resolve, reject) => {
            const transaction = db.transaction([STORE_NAME], 'readonly');
            const store = transaction.objectStore(STORE_NAME);
            const request = store.getAll();

            request.onerror = () => {
                this._logger.error('Failed to load all pending mutations:', request.error);
                reject(request.error);
            };

            request.onsuccess = () => {
                const results = request.result as IPendingMutations[];
                resolve(results || []);
            };
        });
    }

    async clearPendingMutations(unitId: string): Promise<void> {
        const db = await this._ensureDB();
        return new Promise((resolve, reject) => {
            const transaction = db.transaction([STORE_NAME], 'readwrite');
            const store = transaction.objectStore(STORE_NAME);
            const request = store.delete(unitId);

            request.onerror = () => {
                this._logger.error('Failed to clear pending mutations:', request.error);
                reject(request.error);
            };

            request.onsuccess = () => {
                this._logger.log(`Cleared pending mutations for unitId: ${unitId}`);
                resolve();
            };
        });
    }

    async clearAllPendingMutations(): Promise<void> {
        const db = await this._ensureDB();
        return new Promise((resolve, reject) => {
            const transaction = db.transaction([STORE_NAME], 'readwrite');
            const store = transaction.objectStore(STORE_NAME);
            const request = store.clear();

            request.onerror = () => {
                this._logger.error('Failed to clear all pending mutations:', request.error);
                reject(request.error);
            };

            request.onsuccess = () => {
                this._logger.log('Cleared all pending mutations');
                resolve();
            };
        });
    }

    override dispose(): void {
        if (this._db) {
            this._db.close();
            this._db = null;
        }
        super.dispose();
    }
}
