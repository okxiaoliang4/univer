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

/**
 * Mutation info extended with operation ID for tracking
 */
export interface IMutationWithOpId<T extends object = object> extends IMutationInfo<T> {
    opId: string;
}

/**
 * Document synchronization state
 */
export type DocumentSyncStateValue =
    | 'synced'
    | 'pending'
    | 'awaiting'
    | 'awaitingWithPending'
    | 'fetchMiss'
    | 'offline'
    | 'conflict';

/**
 * Document sync state with metadata
 */
export interface IDocumentSyncState {
    state: DocumentSyncStateValue;
    serverRev: number;
    pendingCount: number;
    awaitingCount: number;
    lastError?: string;
}

/**
 * Network connection status
 */
export type NetworkConnectionStatus = 'connected' | 'disconnected' | 'connecting';

/**
 * Changeset pushed from server (broadcast from other clients)
 */
export interface IChangesetPushed {
    docId: string;
    serverRev: number;
    userId: string;
}

/**
 * Request to send a changeset to server
 */
export interface IChangesetRequest {
    baseRev: number;
    mutations: IMutationInfo[];
    docId: string;
    clientId?: string;
}

/**
 * Server acknowledgment for a changeset
 */
export interface IChangesetAck {
    status: 'ok' | 'error';
    serverRev?: number;
    opIds?: string[];
    message?: string;
}

/**
 * Request to join a document
 */
export interface IJoinDocRequest {
    docId: string;
}

/**
 * Server acknowledgment for joining a document
 */
export interface IJoinDocAck {
    status: 'ok' | 'error';
    version?: number;
    message?: string;
}

/**
 * Request to fetch operations from server
 */
export interface IFetchOpsRequest {
    docId: string;
    startRev: number;
}

/**
 * Operation info from server
 */
export interface IOperationInfo {
    rev: number;
    userId: string;
    mutations: IMutationInfo[];
}

/**
 * Server response for fetch ops request
 */
export interface IFetchOpsResult {
    status: 'ok' | 'error';
    operations?: IOperationInfo[];
    message?: string;
}

/**
 * Transform result from OT engine
 */
export interface ITransformResult {
    m1Prime?: IMutationInfo;
    m2Prime?: IMutationInfo;
    error?: string;
}

/**
 * Transform list result from OT engine
 */
export interface ITransformListResult {
    m1Primes: IMutationInfo[];
    m2Primes: IMutationInfo[];
    error?: string;
}

/**
 * User awareness state (selection, cursor, etc.)
 */
export interface IUserAwareness {
    docId: string;
    userId: string;
    userName?: string;
    color?: string;
    selection?: {
        sheetId?: string;
        ranges?: Array<{ startRow: number; startColumn: number; endRow: number; endColumn: number }>;
        cursor?: { row: number; col: number };
    };
    timestamp: number;
}

/**
 * RPC channel names for cross-thread communication
 *
 * In the isomorphic architecture, services use unified names.
 * The same service interface works in both main thread and worker.
 */
export const COLLABORATION_SERVICE_NAME = 'univer.collaboration.service';
export const UNDO_REDO_TRANSFORM_SERVICE_NAME = 'univer.collaboration.undo-redo-transform.service';
export const AWARENESS_REMOTE_SERVICE_NAME = 'univer.awareness-remote.service';
/**
 * RPC channel name for awareness callback (main thread service called by worker)
 */
export const AWARENESS_CALLBACK_SERVICE_NAME = 'univer.awareness-callback.service';
/**
 * RPC channel name for collaboration callback (main thread service called by worker)
 */
export const COLLABORATION_CALLBACK_SERVICE_NAME = 'univer.collaboration-callback.service';

/**
 * @deprecated Use COLLABORATION_SERVICE_NAME instead
 */
export const COLLABORATION_REMOTE_SERVICE_NAME = COLLABORATION_SERVICE_NAME;
