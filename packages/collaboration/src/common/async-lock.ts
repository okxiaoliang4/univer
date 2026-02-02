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

/**
 * A simple async lock for ensuring sequential execution of async operations.
 * Uses FIFO (first-in-first-out) ordering for queued operations.
 *
 * @example
 * ```typescript
 * const lock = new AsyncLock();
 *
 * async function criticalSection() {
 *     const release = await lock.acquire('operation-1');
 *     try {
 *         // Critical section code here
 *         await someAsyncOperation();
 *     } finally {
 *         release();
 *     }
 * }
 * ```
 */
export class AsyncLock {
    private _queue: Array<{
        resolve: (release: () => void) => void;
        id: string;
    }> = [];

    private _locked = false;

    /**
     * Acquire the lock. If the lock is already held, the caller will be queued
     * and the promise will resolve when it's their turn.
     *
     * @param operationId Optional identifier for debugging purposes
     * @returns A function to release the lock. Must be called when done.
     */
    async acquire(operationId?: string): Promise<() => void> {
        return new Promise((resolve) => {
            const release = () => {
                this._locked = false;
                const next = this._queue.shift();
                if (next) {
                    this._locked = true;
                    next.resolve(this._createRelease());
                }
            };

            if (!this._locked) {
                this._locked = true;
                resolve(release);
            } else {
                this._queue.push({
                    resolve: (releaseFunc) => resolve(releaseFunc),
                    id: operationId ?? `op-${Date.now()}`,
                });
            }
        });
    }

    /**
     * Create a release function for the next queued operation.
     */
    private _createRelease(): () => void {
        return () => {
            this._locked = false;
            const next = this._queue.shift();
            if (next) {
                this._locked = true;
                next.resolve(this._createRelease());
            }
        };
    }

    /**
     * Execute a function while holding the lock.
     * This is a convenience method that handles acquire/release automatically.
     *
     * @param operationId Optional identifier for debugging purposes
     * @param fn The async function to execute
     * @returns The result of the function
     */
    async withLock<T>(operationId: string | undefined, fn: () => Promise<T>): Promise<T> {
        const release = await this.acquire(operationId);
        try {
            return await fn();
        } finally {
            release();
        }
    }

    /**
     * Check if the lock is currently held.
     */
    isLocked(): boolean {
        return this._locked;
    }

    /**
     * Get the number of operations waiting in the queue.
     */
    getQueueLength(): number {
        return this._queue.length;
    }
}
