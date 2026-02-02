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
import type {
    IMutationWithOpId,
    IOperationInfo,
    ITransformListResult,
} from '../common/types';
import { assign, fromPromise, setup } from 'xstate';

/**
 * Document sync context - managed by the state machine
 */
export interface IDocumentSyncContext {
    /** Document ID */
    docId: string;
    /** Current server revision number */
    serverRev: number;
    /** Mutations that have been sent to server, awaiting ACK */
    awaitingMutations: IMutationWithOpId[];
    /** Mutations pending to be sent */
    pendingMutations: IMutationWithOpId[];
    /** Last error message */
    lastError?: string;
    /** Whether currently fetching missed operations */
    isFetching: boolean;
    /**
     * Start revision for fetching missed operations
     *
     * When SEND_SUCCESS returns with a version gap (serverRev > expectedRev),
     * we need to fetch operations from expectedRev to serverRev-1.
     * This field stores the starting revision for the fetch.
     *
     * Example: baseRev=5, serverRev=8
     * - Our changeset was committed at rev 8
     * - We missed ops at rev 6, 7
     * - fetchFromRev = 6 (baseRev + 1)
     * - We update serverRev to 8 immediately
     * - fetchMiss fetches from 6, stops when reaching current serverRev
     */
    fetchFromRev?: number;
    /**
     * Maximum serverRev seen during fetchMiss state
     *
     * When in fetchMiss, we may receive RECEIVE_REMOTE events for new operations
     * that arrived after we started fetching. We track the maximum serverRev seen
     * so that after the current fetch completes, we can check if we need to
     * fetch more operations.
     */
    maxSeenServerRevDuringFetch?: number;
    /**
     * Number of mutations in the sent changeset (for version gap handling)
     *
     * When SEND_SUCCESS with version gap, we store the mutation count to calculate
     * the correct endRev for fetching. Our changeset spans revs
     * (serverRev - sentMutationCount + 1) to serverRev.
     */
    sentMutationCount?: number;
}

/**
 * Events that can be sent to the state machine
 */
export type DocumentSyncEvent =
    | { type: 'LOCAL_OPERATION'; mutations: IMutationWithOpId[] }
    | { type: 'SEND_CHANGESET' }
    | {
        type: 'SEND_SUCCESS';
        serverRev: number;
        /** True if server returned a revision higher than expected (missed ops between baseRev and serverRev) */
        hasVersionGap?: boolean;
        /** The expected revision (baseRev + 1), used with hasVersionGap to determine fetchFromRev */
        expectedRev?: number;
        /** Number of mutations in the changeset, used to calculate endRev for fetchMiss */
        mutationCount?: number;
    }
    | { type: 'SEND_FAILURE'; error: string }
    | { type: 'RECEIVE_REMOTE'; serverRev: number; mutations: IMutationInfo[]; userId: string }
    | { type: 'RECEIVE_ACK'; serverRev: number }
    | { type: 'FETCH_MISS' }
    | { type: 'FETCH_SUCCESS'; operations: IOperationInfo[] }
    | { type: 'FETCH_FAILURE'; error: string }
    | { type: 'TRANSFORM_COMPLETE'; m1Primes: IMutationInfo[]; m2Primes: IMutationInfo[]; serverRev: number; awaitingCount: number }
    | { type: 'TRANSFORM_ERROR'; error: string }
    | { type: 'NETWORK_CONNECTED' }
    | { type: 'NETWORK_DISCONNECTED' }
    | { type: 'RESOLVE_CONFLICT' }
    | { type: 'RESET' };

/**
 * Configuration for state machine callbacks
 */
export interface IDocumentSyncMachineConfig {
    /** Send changeset to server - should fire an event back with result */
    onSendChangeset: (mutations: IMutationWithOpId[], baseRev: number) => void;
    /** Fetch missed operations from server */
    onFetchOps: (startRev: number) => Promise<IOperationInfo[]>;
    /** Apply remote mutations to local state */
    onApplyRemote: (mutations: IMutationInfo[]) => Promise<void>;
    /** Transform local mutations against remote */
    onTransform: (local: IMutationInfo[], remote: IMutationInfo[]) => Promise<ITransformListResult>;
    /** Called when state changes */
    onStateChange?: (state: string, context: IDocumentSyncContext) => void;
    /** Persist pending mutations */
    onPersistPending?: (mutations: IMutationWithOpId[]) => Promise<void>;
}

/**
 * Create initial context for a document
 */
export function createInitialContext(docId: string, initialRev: number = 0): IDocumentSyncContext {
    return {
        docId,
        serverRev: initialRev,
        awaitingMutations: [],
        pendingMutations: [],
        lastError: undefined,
        isFetching: false,
        fetchFromRev: undefined,
        maxSeenServerRevDuringFetch: undefined,
        sentMutationCount: undefined,
    };
}

/**
 * Create the document sync state machine
 *
 * States:
 * - synced: Client and server are in sync, no pending operations
 * - pending: Client has unsent local operations
 * - awaiting: Operations sent, waiting for server ACK
 * - awaitingWithPending: Waiting for ACK + new pending operations
 * - fetchMiss: Fetching missed operations from server
 * - offline: Network disconnected
 * - conflict: Sync error that needs resolution
 */
// eslint-disable-next-line max-lines-per-function
export function createDocumentSyncMachine(config: IDocumentSyncMachineConfig) {
    return setup({
        types: {
            context: {} as IDocumentSyncContext,
            events: {} as DocumentSyncEvent,
            input: {} as { docId: string; initialRev?: number },
        },
        actions: {
            addToPending: assign({
                pendingMutations: ({ context, event }) => {
                    if (event.type !== 'LOCAL_OPERATION') return context.pendingMutations;
                    return [...context.pendingMutations, ...event.mutations];
                },
            }),
            movePendingToAwaiting: assign({
                awaitingMutations: ({ context }) => [
                    ...context.awaitingMutations,
                    ...context.pendingMutations,
                ],
                pendingMutations: () => [],
            }),
            /**
             * Merge awaiting mutations back to pending for retry
             *
             * This is used when reconnecting after disconnect during awaiting state.
             * Since the server may not have received the changeset, we need to
             * move awaiting mutations back to pending and resend them.
             */
            mergeAwaitingToPending: assign({
                pendingMutations: ({ context }) => {
                    const merged = [
                        ...context.awaitingMutations,
                        ...context.pendingMutations,
                    ];
                    if (context.awaitingMutations.length > 0) {
                        console.warn(`[StateMachine:${context.docId}] mergeAwaitingToPending: moving ${context.awaitingMutations.length} awaiting + ${context.pendingMutations.length} pending = ${merged.length} total pending`);
                    }
                    return merged;
                },
                awaitingMutations: () => [],
            }),
            clearAwaiting: assign({
                awaitingMutations: () => [],
            }),
            /**
             * Store the fetch start revision and mutation count for version gap handling
             *
             * When SEND_SUCCESS has a version gap (serverRev > expectedMaxRev):
             * - expectedRev = baseRev + 1 = the first revision we need to fetch
             * - serverRev = final revision of our changeset
             * - mutationCount = number of mutations in our changeset
             * - Our changeset spans revs (serverRev - mutationCount + 1) to serverRev
             * - We need to fetch ops from expectedRev to (serverRev - mutationCount)
             *
             * Example: baseRev=302, 7 mutations, serverRev=312 (other clients committed 303-305)
             * - expectedRev = 303
             * - Our changeset is at revs 306-312
             * - We need ops 303, 304, 305 (other clients' ops)
             * - fetchFromRev = 303, endRev = 306 (= serverRev - mutationCount + 1)
             */
            storeFetchFromRev: assign({
                fetchFromRev: ({ event }) => {
                    if (event.type !== 'SEND_SUCCESS') return undefined;
                    // expectedRev is baseRev + 1, which is where to start fetching
                    return event.expectedRev;
                },
                sentMutationCount: ({ event }) => {
                    if (event.type !== 'SEND_SUCCESS') return undefined;
                    return event.mutationCount;
                },
            }),
            clearFetchFromRev: assign({
                fetchFromRev: () => undefined,
                sentMutationCount: () => undefined,
            }),
            /**
             * Track the maximum serverRev seen during fetchMiss
             *
             * When RECEIVE_REMOTE arrives during fetchMiss, we don't apply it immediately
             * (the fetch will get those ops). Instead, we track the maximum serverRev so
             * that after fetch completes, we can check if we need to fetch more.
             */
            updateMaxSeenServerRev: assign({
                maxSeenServerRevDuringFetch: ({ context, event }) => {
                    if (event.type !== 'RECEIVE_REMOTE') return context.maxSeenServerRevDuringFetch;
                    const current = context.maxSeenServerRevDuringFetch ?? 0;
                    const newRev = event.serverRev;
                    if (newRev > current) {
                        console.warn(
                            `[StateMachine:${context.docId}] fetchMiss: new changeset_pushed arrived with rev ${newRev}, ` +
                            `updating maxSeenServerRev from ${current} to ${newRev}`
                        );
                        return newRev;
                    }
                    return current;
                },
            }),
            clearMaxSeenServerRev: assign({
                maxSeenServerRevDuringFetch: () => undefined,
            }),
            /**
             * Log version gap info from SEND_SUCCESS for debugging
             * This is called before transitioning to fetchMiss
             */
            logVersionGap: ({ context, event }) => {
                if (event.type !== 'SEND_SUCCESS' || !event.hasVersionGap) return;
                console.warn(
                    `[StateMachine:${context.docId}] SEND_SUCCESS with version gap detected: ` +
                    `expected rev ${event.expectedRev}, got ${event.serverRev}. ` +
                    `Will update serverRev to ${event.serverRev} and fetch ops from ${event.expectedRev} to ${event.serverRev - 1}.`
                );
            },
            updateServerRev: assign({
                serverRev: ({ context, event }) => {
                    if (event.type === 'SEND_SUCCESS' || event.type === 'RECEIVE_ACK') {
                        return event.serverRev;
                    }
                    if (event.type === 'TRANSFORM_COMPLETE') {
                        return event.serverRev;
                    }
                    if (event.type === 'RECEIVE_REMOTE') {
                        return event.serverRev;
                    }
                    return context.serverRev;
                },
            }),
            setError: assign({
                lastError: ({ event }) => {
                    if (event.type === 'SEND_FAILURE' || event.type === 'FETCH_FAILURE' || event.type === 'TRANSFORM_ERROR') {
                        return event.error;
                    }
                    return undefined;
                },
            }),
            clearError: assign({
                lastError: () => undefined,
            }),
            updateTransformedPending: assign({
                pendingMutations: ({ context, event }) => {
                    if (event.type !== 'TRANSFORM_COMPLETE') return context.pendingMutations;
                    // m1Primes contains [awaiting'..., pending'...], extract pending portion
                    const pendingPrimes = event.m1Primes.slice(event.awaitingCount);
                    return pendingPrimes.map((m, i) => ({
                        ...m,
                        opId: context.pendingMutations[i]?.opId || `transformed-pending-${i}`,
                    }));
                },
            }),
            updateTransformedAwaiting: assign({
                awaitingMutations: ({ context, event }) => {
                    if (event.type !== 'TRANSFORM_COMPLETE') return context.awaitingMutations;
                    // m1Primes contains [awaiting'..., pending'...], extract awaiting portion
                    const awaitingPrimes = event.m1Primes.slice(0, event.awaitingCount);
                    return awaitingPrimes.map((m, i) => ({
                        ...m,
                        opId: context.awaitingMutations[i]?.opId || `transformed-awaiting-${i}`,
                    }));
                },
            }),
            setFetching: assign({
                isFetching: () => true,
            }),
            clearFetching: assign({
                isFetching: () => false,
            }),
            /**
             * Trigger sending changeset via callback (not an invoke)
             *
             * This is important because the network call should NOT be cancelled
             * when the state changes. If a LOCAL_OPERATION comes in while awaiting,
             * we transition to awaitingWithPending, but the network call is already
             * in flight. The ACK will be handled by SEND_SUCCESS/SEND_FAILURE events.
             */
            triggerSendChangeset: ({ context }) => {
                config.onSendChangeset(context.awaitingMutations, context.serverRev);
            },
        },
        guards: {
            hasPending: ({ context }) => context.pendingMutations.length > 0,
            /**
             * Check if there are transformed pending mutations after fetchMiss completes.
             * Used to decide whether to go to 'pending' or 'synced' state.
             */
            hasTransformedPending: ({ event }) => {
                // Check if the event output has transformed local mutations
                if (event && typeof event === 'object' && 'output' in event) {
                    const output = (event as { output: { transformedLocal?: IMutationWithOpId[] } }).output;
                    return (output.transformedLocal?.length ?? 0) > 0;
                }
                return false;
            },
            hasAwaiting: ({ context }) => context.awaitingMutations.length > 0,
            hasPendingOrAwaiting: ({ context }) =>
                context.pendingMutations.length > 0 || context.awaitingMutations.length > 0,
            isVersionGap: ({ context, event }) => {
                if (event.type !== 'RECEIVE_REMOTE') return false;
                return event.serverRev > context.serverRev + 1;
            },
            isExpectedVersion: ({ context, event }) => {
                if (event.type !== 'RECEIVE_REMOTE') return false;
                return event.serverRev === context.serverRev + 1;
            },
            /**
             * Check if SEND_SUCCESS has a version gap
             *
             * This happens when the server returns a serverRev higher than expected,
             * meaning other clients committed operations between our baseRev and serverRev.
             * We need to fetch those missed operations after acknowledging our changeset.
             */
            hasVersionGapInSendSuccess: ({ event }) => {
                if (event.type !== 'SEND_SUCCESS') return false;
                return event.hasVersionGap === true;
            },
            /**
             * Check if we need to fetch more operations after the current fetch completes
             *
             * This happens when new changeset_pushed events arrived during fetchMiss.
             * We track the maximum serverRev seen, and if it's higher than our current
             * serverRev, we need to continue fetching.
             */
            needsMoreFetching: ({ context }) => {
                const maxSeen = context.maxSeenServerRevDuringFetch;
                if (maxSeen && maxSeen > context.serverRev) {
                    const isVersionGapMode = context.fetchFromRev !== undefined && context.fetchFromRev < context.serverRev;
                    console.warn(
                        `[StateMachine:${context.docId}] fetchMiss completed, serverRev=${context.serverRev}, ` +
                        `but maxSeenServerRev=${maxSeen}. Need to continue fetching. ` +
                        `Mode: ${isVersionGapMode ? `versionGap (will start from ${context.serverRev + 1})` : 'normal'}`
                    );
                    return true;
                }
                return false;
            },
        },
        actors: {
            fetchAndApplyMissedOps: fromPromise(async ({ input }: {
                input: {
                    startRev: number;
                    /**
                     * If set, stop applying operations at this revision (exclusive).
                     *
                     * This is used when we enter fetchMiss after SEND_SUCCESS with version gap.
                     * Our own changeset was committed at endRev, so we only need ops
                     * from startRev to endRev-1.
                     *
                     * Example: startRev=6, endRev=8
                     * - Apply ops 6, 7
                     * - Stop at 8 (our own changeset)
                     */
                    endRev?: number;
                    /**
                     * Local pending mutations that need to be transformed against fetched ops.
                     *
                     * When we have local pending ops and fetch remote ops, we need to:
                     * 1. Transform local against each remote op
                     * 2. Apply the transformed remote (m2Prime)
                     * 3. Update local to transformed (m1Prime)
                     *
                     * Without this, the local pending would be based on an old state.
                     */
                    localMutations: IMutationWithOpId[];
                };
            }): Promise<{ finalRev: number; appliedCount: number; transformedLocal: IMutationWithOpId[] }> => {
                const operations = await config.onFetchOps(input.startRev);
                let finalRev = input.startRev;
                let appliedCount = 0;
                let currentLocal: IMutationInfo[] = [...input.localMutations];

                // Apply each operation's mutations in order, transforming local against each
                for (const op of operations) {
                    // Stop at endRev (exclusive) - ops at endRev and above are either:
                    // 1. Our own changeset (after SEND_SUCCESS with gap)
                    // 2. New ops that arrived during fetch (will be handled in next iteration)
                    if (input.endRev !== undefined && op.rev >= input.endRev) {
                        console.warn(
                            `[fetchAndApplyMissedOps] Stopping at rev ${op.rev} (endRev=${input.endRev})`
                        );
                        break;
                    }

                    if (op.mutations && op.mutations.length > 0) {
                        if (currentLocal.length > 0) {
                            // Transform local against remote
                            // local' = transform(local, remote).m1Primes
                            // remote' = transform(local, remote).m2Primes (what we apply locally)
                            console.warn(
                                `[fetchAndApplyMissedOps] Transforming ${currentLocal.length} local ops against rev ${op.rev} (${op.mutations.length} remote ops)`
                            );
                            const result = await config.onTransform(currentLocal, op.mutations);
                            if (result.error) {
                                throw new Error(`Transform error at rev ${op.rev}: ${result.error}`);
                            }
                            // Apply transformed remote (m2Primes)
                            await config.onApplyRemote(result.m2Primes);
                            // Update local to transformed version (m1Primes)
                            currentLocal = result.m1Primes;
                            console.warn(
                                `[fetchAndApplyMissedOps] After transform: ${currentLocal.length} local ops remaining`
                            );
                        } else {
                            // No local ops, apply remote directly
                            await config.onApplyRemote(op.mutations);
                        }
                        appliedCount++;
                    }
                    finalRev = op.rev;
                }

                // Preserve opIds from original mutations
                const transformedLocal: IMutationWithOpId[] = currentLocal.map((m, i) => ({
                    ...m,
                    opId: input.localMutations[i]?.opId || `transformed-${i}`,
                }));

                return { finalRev, appliedCount, transformedLocal };
            }),
            transformAndApply: fromPromise(async ({ input }: {
                input: { local: IMutationInfo[]; remote: IMutationInfo[] };
            }) => {
                const result = await config.onTransform(input.local, input.remote);
                if (result.error) {
                    throw new Error(result.error);
                }
                // Apply transformed remote mutations (m2Primes)
                if (result.m2Primes.length > 0) {
                    await config.onApplyRemote(result.m2Primes);
                }
                return result;
            }),
            applyRemote: fromPromise(async ({ input }: {
                input: { mutations: IMutationInfo[] };
            }) => {
                await config.onApplyRemote(input.mutations);
            }),
        },
    }).createMachine({
        id: 'documentSync',
        initial: 'synced',
        context: ({ input }) => createInitialContext(input.docId, input.initialRev),
        states: {
            /**
             * Synced: Client and server are completely in sync
             */
            synced: {
                on: {
                    LOCAL_OPERATION: {
                        target: 'pending',
                        actions: ['addToPending'],
                    },
                    RECEIVE_REMOTE: [
                        {
                            guard: 'isVersionGap',
                            target: 'fetchMiss',
                        },
                        {
                            target: 'synced',
                            actions: ['updateServerRev'],
                            // Apply remote mutations directly (no local pending to transform)
                        },
                    ],
                    FETCH_MISS: {
                        target: 'fetchMiss',
                    },
                    NETWORK_DISCONNECTED: 'offline',
                },
            },

            /**
             * Pending: Client has local operations not yet sent to server
             */
            pending: {
                on: {
                    LOCAL_OPERATION: {
                        actions: ['addToPending'],
                    },
                    SEND_CHANGESET: {
                        target: 'awaiting',
                        actions: ['movePendingToAwaiting'],
                    },
                    RECEIVE_REMOTE: [
                        {
                            guard: 'isVersionGap',
                            target: 'fetchMiss',
                        },
                        {
                            // Need to transform pending against remote
                            target: 'pending',
                            // Transform will be handled by the collaboration service
                        },
                    ],
                    FETCH_MISS: {
                        target: 'fetchMiss',
                    },
                    TRANSFORM_COMPLETE: {
                        actions: ['updateTransformedPending', 'updateServerRev'],
                    },
                    TRANSFORM_ERROR: {
                        target: 'conflict',
                        actions: ['setError'],
                    },
                    NETWORK_DISCONNECTED: 'offline',
                },
            },

            /**
             * Awaiting: All operations sent, waiting for server ACK
             *
             * IMPORTANT: We use an entry action (not invoke) to send the changeset.
             * This is because the network call should NOT be cancelled when state changes.
             * If a LOCAL_OPERATION comes in while awaiting, we transition to awaitingWithPending,
             * but the network call is still in flight. The ACK will arrive and trigger
             * SEND_SUCCESS/SEND_FAILURE which are handled in BOTH states.
             */
            awaiting: {
                entry: ['triggerSendChangeset'],
                on: {
                    SEND_SUCCESS: [
                        {
                            // If there's a version gap, we need to fetch missed ops
                            // Our changeset was accepted at serverRev, but we missed operations
                            // between expectedRev and serverRev-1. We need to:
                            // 1. Clear awaiting (our changeset was accepted)
                            // 2. Update serverRev to ACK value (our commit is at this rev)
                            // 3. Store fetchFromRev = expectedRev (where to start fetching)
                            // 4. Go to fetchMiss to get ops from expectedRev to serverRev-1
                            guard: 'hasVersionGapInSendSuccess',
                            target: 'fetchMiss',
                            actions: ['clearAwaiting', 'updateServerRev', 'storeFetchFromRev', 'logVersionGap'],
                        },
                        {
                            // Normal case: no version gap, go to synced
                            target: 'synced',
                            actions: ['clearAwaiting', 'updateServerRev'],
                        },
                    ],
                    SEND_FAILURE: {
                        target: 'conflict',
                        actions: ['setError'],
                    },
                    LOCAL_OPERATION: {
                        target: 'awaitingWithPending',
                        actions: ['addToPending'],
                    },
                    RECEIVE_REMOTE: [
                        {
                            guard: 'isVersionGap',
                            target: 'fetchMiss',
                        },
                        {
                            // Transform awaiting against remote
                            target: 'awaiting',
                        },
                    ],
                    FETCH_MISS: {
                        target: 'fetchMiss',
                    },
                    TRANSFORM_COMPLETE: {
                        actions: ['updateTransformedAwaiting', 'updateServerRev'],
                    },
                    TRANSFORM_ERROR: {
                        target: 'conflict',
                        actions: ['setError'],
                    },
                    NETWORK_DISCONNECTED: 'offline',
                },
            },

            /**
             * AwaitingWithPending: Waiting for ACK + new pending operations
             */
            awaitingWithPending: {
                on: {
                    LOCAL_OPERATION: {
                        actions: ['addToPending'],
                    },
                    SEND_SUCCESS: [
                        {
                            // If there's a version gap, fetch missed ops first
                            // We still have pending ops, so after fetchMiss we'll go to pending
                            // Same as awaiting: update serverRev first, then fetch from expectedRev
                            guard: 'hasVersionGapInSendSuccess',
                            target: 'fetchMiss',
                            actions: ['clearAwaiting', 'updateServerRev', 'storeFetchFromRev', 'logVersionGap'],
                        },
                        {
                            // Normal case: no version gap, go to pending
                            target: 'pending',
                            actions: ['clearAwaiting', 'updateServerRev'],
                        },
                    ],
                    SEND_FAILURE: {
                        target: 'conflict',
                        actions: ['setError'],
                    },
                    RECEIVE_REMOTE: [
                        {
                            guard: 'isVersionGap',
                            target: 'fetchMiss',
                        },
                        {
                            // Transform both awaiting and pending against remote
                            target: 'awaitingWithPending',
                        },
                    ],
                    FETCH_MISS: {
                        target: 'fetchMiss',
                    },
                    TRANSFORM_COMPLETE: {
                        actions: ['updateTransformedPending', 'updateTransformedAwaiting', 'updateServerRev'],
                    },
                    TRANSFORM_ERROR: {
                        target: 'conflict',
                        actions: ['setError'],
                    },
                    NETWORK_DISCONNECTED: 'offline',
                },
            },

            /**
             * FetchMiss: Client missed operations, fetching from server
             *
             * This state fetches all missed operations and applies them in order.
             * After completion, transitions to pending (if local ops exist) or synced.
             *
             * IMPORTANT: When entering this state after reconnection, we must merge
             * any awaitingMutations back to pendingMutations, because the server may
             * not have received them (the ACK was lost due to disconnect).
             *
             * IMPORTANT: During fetchMiss, new changeset_pushed events may arrive.
             * We don't apply them immediately (the fetch will get those ops eventually).
             * Instead, we track the maximum serverRev seen. After fetch completes,
             * if there are newer ops, we stay in fetchMiss and continue fetching.
             */
            fetchMiss: {
                entry: ['setFetching', 'mergeAwaitingToPending', 'clearMaxSeenServerRev'],
                exit: ['clearFetching'],
                invoke: {
                    src: 'fetchAndApplyMissedOps',
                    input: ({ context }) => {
                        // Calculate endRev for version gap fetch:
                        // Our changeset spans revs (serverRev - sentMutationCount + 1) to serverRev
                        // We need to fetch ops BEFORE our changeset, so endRev = serverRev - sentMutationCount + 1
                        //
                        // Example: serverRev=312, sentMutationCount=7
                        // - Our changeset is at revs 306-312
                        // - endRev = 312 - 7 + 1 = 306 (stop before rev 306)
                        const isInitialGapFetch = context.fetchFromRev !== undefined &&
                            context.sentMutationCount !== undefined &&
                            context.fetchFromRev < context.serverRev;

                        const endRev = isInitialGapFetch
                            ? context.serverRev - (context.sentMutationCount ?? 1) + 1
                            : undefined;

                        return {
                            // If fetchFromRev is set (after SEND_SUCCESS with gap), start from there
                            // Otherwise (regular FETCH_MISS), start from current serverRev
                            startRev: context.fetchFromRev ?? context.serverRev,
                            // Set endRev only for initial version gap fetch
                            // - Initial gap fetch: stop before our changeset
                            // - Continuation fetch: no limit (fetch all new ops)
                            // - Normal fetch: no limit (fetch all)
                            endRev,
                            // Pass local pending mutations for transformation
                            // These will be transformed against each fetched remote op
                            localMutations: context.pendingMutations,
                        };
                    },
                    onDone: [
                        {
                            // If new changeset_pushed arrived during fetch with higher rev,
                            // stay in fetchMiss and continue fetching
                            guard: 'needsMoreFetching',
                            target: 'fetchMiss',
                            actions: [
                                assign({
                                    // If we were in version gap mode (fetchFromRev < serverRev),
                                    // set fetchFromRev = serverRev + 1 to skip our own changeset
                                    // and fetch new ops that arrived during the initial fetch.
                                    // For normal mode, keep fetchFromRev undefined.
                                    fetchFromRev: ({ context }) =>
                                        (context.fetchFromRev !== undefined && context.fetchFromRev < context.serverRev)
                                            ? context.serverRev + 1
                                            : undefined,
                                    // For version gap continuation, keep serverRev (our ACK value)
                                    // For normal mode, update to finalRev
                                    serverRev: ({ context, event }) =>
                                        (context.fetchFromRev !== undefined && context.fetchFromRev < context.serverRev)
                                            ? context.serverRev // Keep ACK value during version gap handling
                                            : event.output.finalRev,
                                    // Update pending with transformed local mutations
                                    pendingMutations: ({ event }) => event.output.transformedLocal,
                                }),
                                'clearMaxSeenServerRev',
                            ],
                        },
                        {
                            // Check if there are transformed pending mutations
                            guard: 'hasTransformedPending',
                            target: 'pending',
                            actions: [
                                assign({
                                    // Initial version gap fetch (fetchFromRev < serverRev): keep ACK value
                                    // Continuation fetch (fetchFromRev >= serverRev): use finalRev
                                    // Normal fetch (no fetchFromRev): use finalRev
                                    serverRev: ({ context, event }) =>
                                        (context.fetchFromRev !== undefined && context.fetchFromRev < context.serverRev)
                                            ? context.serverRev // Keep ACK value for initial gap fetch
                                            : event.output.finalRev,
                                    // Update pending with transformed local mutations
                                    pendingMutations: ({ event }) => event.output.transformedLocal,
                                }),
                                'clearFetchFromRev',
                                'clearMaxSeenServerRev',
                                'clearError',
                            ],
                        },
                        {
                            // No transformed pending, go to synced
                            target: 'synced',
                            actions: [
                                assign({
                                    // Initial version gap fetch (fetchFromRev < serverRev): keep ACK value
                                    // Continuation fetch (fetchFromRev >= serverRev): use finalRev
                                    // Normal fetch (no fetchFromRev): use finalRev
                                    serverRev: ({ context, event }) =>
                                        (context.fetchFromRev !== undefined && context.fetchFromRev < context.serverRev)
                                            ? context.serverRev // Keep ACK value for initial gap fetch
                                            : event.output.finalRev,
                                    // Clear pending
                                    pendingMutations: () => [],
                                }),
                                'clearFetchFromRev',
                                'clearMaxSeenServerRev',
                                'clearError',
                            ],
                        },
                    ],
                    onError: {
                        target: 'conflict',
                        actions: [
                            assign({ lastError: ({ event }) => String(event.error) }),
                            'clearFetchFromRev',
                            'clearMaxSeenServerRev',
                        ],
                    },
                },
                on: {
                    LOCAL_OPERATION: {
                        actions: ['addToPending'],
                    },
                    /**
                     * When RECEIVE_REMOTE arrives during fetchMiss:
                     * - Don't apply the mutations (the fetch will get them)
                     * - Track the serverRev so we know if more fetching is needed
                     */
                    RECEIVE_REMOTE: {
                        actions: ['updateMaxSeenServerRev'],
                    },
                    /**
                     * When another FETCH_MISS is triggered during fetchMiss:
                     * - Just stay in fetchMiss (we're already fetching)
                     * - The current fetch will continue, and if needed, we'll re-fetch
                     */
                    FETCH_MISS: {
                        // Stay in fetchMiss, no action needed
                    },
                    NETWORK_DISCONNECTED: 'offline',
                },
            },

            /**
             * Offline: Network disconnected
             */
            offline: {
                on: {
                    LOCAL_OPERATION: {
                        actions: ['addToPending'],
                    },
                    NETWORK_CONNECTED: [
                        {
                            guard: 'hasPendingOrAwaiting',
                            target: 'fetchMiss',
                            // Fetch missed ops then resync
                        },
                        {
                            target: 'synced',
                        },
                    ],
                },
            },

            /**
             * Conflict: Sync error that needs resolution
             *
             * IMPORTANT: We still accept LOCAL_OPERATION in conflict state to prevent
             * user edits from being lost. The user might not realize there's a conflict
             * and continue editing. These edits are buffered in pendingMutations and
             * will be synced once the conflict is resolved.
             */
            conflict: {
                on: {
                    LOCAL_OPERATION: {
                        // Buffer user edits even during conflict - don't lose user work!
                        actions: ['addToPending'],
                    },
                    RESOLVE_CONFLICT: 'fetchMiss',
                    RESET: {
                        target: 'synced',
                        actions: ['clearError', 'clearAwaiting', 'clearFetchFromRev', assign({ pendingMutations: () => [] })],
                    },
                    NETWORK_DISCONNECTED: 'offline',
                },
            },
        },
    });
}

export type DocumentSyncMachine = ReturnType<typeof createDocumentSyncMachine>;
