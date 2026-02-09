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

import type { DescService } from '@bufbuild/protobuf';
import type { Client, Transport } from '@connectrpc/connect';
import { createClient } from '@connectrpc/connect';
import { createGrpcTransport } from '@connectrpc/connect-node';
import { Document } from '@pagepeek/rpcs/document_pb';
import { User } from '@pagepeek/rpcs/user_pb';
import { Etcd3 } from 'etcd3';
import { logger } from '../utils/logger';

/**
 * gRPC Service schemas.
 *
 * Since we can't run buf generate in this context, we define minimal
 * hand-written service descriptors compatible with ConnectRPC.
 * These match the proto definitions in user.proto and document.proto.
 *
 * NOTE: When proto code generation is set up (via buf generate), replace
 * these with the generated `User` and `Document` service definitions.
 */

// ─── Proto-like type definitions ─────────────────────────────

interface CheckAccessTokenReq {
    uid: string;
    accessToken: string;
    epId: string;
    invoker: string;
}

interface QueryUserPermissionsReq {
    documentId: string;
    userId?: string;
    accessToken?: string;
}

interface QueryUserPermissionsResp {
    permissions?: {
        readable: boolean;
        commentable: boolean;
        writable: boolean;
        owner: boolean;
    };
}

interface NotifyModifyDocumentReq {
    userId: string;
    documentId: string;
    timestamp: number;
}

/** Permissions returned from gRPC */
export interface DocumentPermissions {
    readable: boolean;
    commentable: boolean;
    writable: boolean;
    owner: boolean;
}

export interface UserInfo {
    uid: string;
    email: string;
}

// ─── Channel refresh config ──────────────────────────────────

const CHANNEL_REFRESH_INTERVAL_MS = 5 * 60 * 1000; // 5 minutes

interface CachedTransport {
    transport: Transport;
    endpoint: string;
    createdAt: number;
}

/**
 * gRPC client service using raw HTTP/2 calls through ConnectRPC transports.
 *
 * Since we use hand-rolled proto definitions (not buf-generated), we make
 * direct unary calls using the transport rather than typed service clients.
 */
export class GrpcClientService {
    private _etcd: Etcd3;
    private _userRpcPrefix: string;
    private _documentRpcPrefix: string;
    private _userTransport: CachedTransport | null = null;
    private _documentTransport: CachedTransport | null = null;
    private _grpcClients: Map<Transport, Client<DescService>> = new Map();

    constructor(
        etcdEndpoints: string[],
        userRpcPrefix: string,
        documentRpcPrefix: string
    ) {
        this._etcd = new Etcd3({ hosts: etcdEndpoints });
        this._userRpcPrefix = userRpcPrefix;
        this._documentRpcPrefix = documentRpcPrefix;
    }

    private async _getServiceEndpoint(prefix: string): Promise<string> {
        const entries = await this._etcd.getAll().prefix(prefix).strings();
        const endpoints = Object.values(entries);
        if (endpoints.length === 0) {
            throw new Error(`No available endpoints for service: ${prefix}`);
        }
        // Random selection for load balancing
        const endpoint = endpoints[Math.floor(Math.random() * endpoints.length)];
        const url = endpoint.startsWith('http://') || endpoint.startsWith('https://')
            ? endpoint
            : `http://${endpoint}`;
        return url;
    }

    private async _getUserTransport(): Promise<Transport> {
        const now = Date.now();
        if (
            this._userTransport &&
            now - this._userTransport.createdAt < CHANNEL_REFRESH_INTERVAL_MS
        ) {
            return this._userTransport.transport;
        }

        const endpoint = await this._getServiceEndpoint(this._userRpcPrefix);
        const transport = createGrpcTransport({
            baseUrl: endpoint,
        });
        this._userTransport = { transport, endpoint, createdAt: now };
        logger.info(`Created new user service transport: ${endpoint}`);
        return transport;
    }

    private async _getDocumentTransport(): Promise<Transport> {
        const now = Date.now();
        if (
            this._documentTransport &&
            now - this._documentTransport.createdAt < CHANNEL_REFRESH_INTERVAL_MS
        ) {
            return this._documentTransport.transport;
        }

        const endpoint = await this._getServiceEndpoint(this._documentRpcPrefix);
        const transport = createGrpcTransport({
            baseUrl: endpoint,
        });
        this._documentTransport = { transport, endpoint, createdAt: now };
        logger.info(`Created new document service transport: ${endpoint}`);
        return transport;
    }

    _getClient<T extends DescService>(desc: T, transport: Transport): Client<T> {
        const client = this._grpcClients.get(transport);
        if (client) return client as Client<T>;

        const newClient = createClient(desc, transport);
        this._grpcClients.set(transport, newClient);
        return newClient;
    }

    /**
     * Verify an access token by calling User.CheckAccessToken.
     *
     * Uses raw HTTP/2 unary call matching the tonic-generated proto service.
     */
    async verifyToken(uid: string, token: string): Promise<void> {
        const transport = await this._getUserTransport();
        const userClient = this._getClient(User, transport) as Client<typeof User>;

        const body = {
            uid,
            access_token: token,
            ep_id: '',
            invoker: 'ws-gateway',
        };

        await userClient.checkAccessToken(body);
    }

    /**
     * Query document permissions via Document.QueryUserPermissions
     */
    async queryDocumentPermissions(
        userId: string,
        docId: string,
        accessToken: string
    ): Promise<DocumentPermissions> {
        const transport = await this._getDocumentTransport();
        const client = this._getClient(Document, transport) as Client<typeof Document>;
        const res = await client.getDocument({
            documentId: docId,
            userId,
            accessToken,
        });
        const perms = res.doc?.permissions;
        return {
            readable: perms?.readable ?? false,
            commentable: perms?.commentable ?? false,
            writable: perms?.writable ?? false,
            owner: perms?.owner ?? false,
        };
    }

    /**
     * Notify document modification via Document.NotifyModifyDocument
     */
    async notifyModifyDocument(userId: string, docId: string): Promise<void> {
        const transport = await this._getDocumentTransport();
        const client = this._getClient(Document, transport) as Client<typeof Document>;
        await client.notifyModifyDocument({
            userId,
            documentId: docId,
        });
    }

    async close(): Promise<void> {
        this._etcd.close();
    }
}
