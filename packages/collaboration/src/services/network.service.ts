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

import type { Observable } from 'rxjs';
import type {
    IChangesetAck,
    IChangesetPushed,
    IChangesetRequest,
    IFetchOpsResult,
    IJoinDocAck,
    IUserAwareness,
    NetworkConnectionStatus,
} from '../common/types';
import { createIdentifier } from '@univerjs/core';

/**
 * Network service interface for collaboration
 *
 * This abstraction allows for different network implementations:
 * - Socket.io (real-time WebSocket)
 * - HTTP polling (fallback)
 * - Custom transport (for testing or special cases)
 */
export interface INetworkService {
    /**
     * Observable for connection status changes
     */
    connectionStatus$: Observable<NetworkConnectionStatus>;

    /**
     * Set the connection status
     * @param status Connection status
     */
    setConnectionStatus(status: NetworkConnectionStatus): void;

    /**
     * Observable for incoming changeset pushes (broadcasts from other clients)
     */
    changesetPushed$: Observable<IChangesetPushed>;

    /**
     * Observable for awareness updates from other users
     */
    awarenessUpdate$: Observable<IUserAwareness>;

    /**
     * Connect to the collaboration server
     */
    connect(): Promise<void>;

    /**
     * Disconnect from the collaboration server
     */
    disconnect(): void;

    /**
     * Join a document room for collaboration
     * @param docId Document identifier
     */
    joinDoc(docId: string): Promise<IJoinDocAck>;

    /**
     * Leave a document room
     * @param docId Document identifier
     */
    leaveDoc(docId: string): void;

    /**
     * Send a changeset to the server
     * @param request Changeset request with mutations
     */
    sendChangeset(request: IChangesetRequest): Promise<IChangesetAck>;

    /**
     * Fetch operations from server starting from a revision
     * @param docId Document identifier
     * @param startRev Starting revision number
     */
    fetchOps(docId: string, startRev: number): Promise<IFetchOpsResult>;

    /**
     * Get the client identifier
     */
    getClientId(): string | undefined;

    /**
     * Check if currently connected to the server
     */
    isConnected(): boolean;

    /**
     * Broadcast user awareness state (selection, cursor, etc.)
     * @param awareness User awareness state
     */
    broadcastAwareness(awareness: IUserAwareness): Promise<void>;

    /**
     * Initialize awareness for a document and get current states
     * @param docId Document identifier
     * @returns Promise with current awareness states for the document
     */
    initAwareness(docId: string): Promise<IAwarenessInitResult>;
}

/**
 * Result of awareness initialization
 */
export interface IAwarenessInitResult {
    status: string;
    states: IUserAwareness[];
}

export const INetworkService = createIdentifier<INetworkService>(
    'univer.collaboration.network.service'
);
