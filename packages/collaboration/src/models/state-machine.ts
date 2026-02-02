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
}

/**
 * Events that can be sent to the state machine
 */
export type DocumentSyncEvent =
    | { type: 'LOCAL_OPERATION'; mutations: IMutationWithOpId[] }
    | { type: 'SEND_CHANGESET' }
    | { type: 'SEND_SUCCESS'; serverRev: number; ackMutations?: IMutationInfo[] }
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
            clearAwaiting: assign({
                awaitingMutations: () => [],
            }),
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
        },
        actors: {
            fetchAndApplyMissedOps: fromPromise(async ({ input }: {
                input: { startRev: number };
            }): Promise<{ finalRev: number; appliedCount: number }> => {
                const operations = await config.onFetchOps(input.startRev);
                let finalRev = input.startRev;

                // Apply each operation's mutations in order
                for (const op of operations) {
                    if (op.mutations && op.mutations.length > 0) {
                        await config.onApplyRemote(op.mutations);
                    }
                    finalRev = op.rev;
                }

                return { finalRev, appliedCount: operations.length };
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
                    SEND_SUCCESS: {
                        target: 'synced',
                        actions: ['clearAwaiting', 'updateServerRev'],
                    },
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
                    SEND_SUCCESS: {
                        target: 'pending',
                        actions: ['clearAwaiting', 'updateServerRev'],
                    },
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
             */
            fetchMiss: {
                entry: ['setFetching'],
                exit: ['clearFetching'],
                invoke: {
                    src: 'fetchAndApplyMissedOps',
                    input: ({ context }) => ({
                        startRev: context.serverRev,
                    }),
                    onDone: [
                        {
                            guard: 'hasPendingOrAwaiting',
                            target: 'pending',
                            actions: [
                                assign({
                                    serverRev: ({ event }) => event.output.finalRev,
                                }),
                                'clearError',
                            ],
                            // Note: pending/awaiting ops should already be transformed
                            // against each fetched op as they were applied
                        },
                        {
                            target: 'synced',
                            actions: [
                                assign({
                                    serverRev: ({ event }) => event.output.finalRev,
                                }),
                                'clearError',
                            ],
                        },
                    ],
                    onError: {
                        target: 'conflict',
                        actions: [assign({ lastError: ({ event }) => String(event.error) })],
                    },
                },
                on: {
                    LOCAL_OPERATION: {
                        actions: ['addToPending'],
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
             */
            conflict: {
                on: {
                    RESOLVE_CONFLICT: 'fetchMiss',
                    RESET: {
                        target: 'synced',
                        actions: ['clearError', 'clearAwaiting', assign({ pendingMutations: () => [] })],
                    },
                    NETWORK_DISCONNECTED: 'offline',
                },
            },
        },
    });
}

export type DocumentSyncMachine = ReturnType<typeof createDocumentSyncMachine>;
