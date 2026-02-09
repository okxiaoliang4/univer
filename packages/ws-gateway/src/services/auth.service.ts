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

import type { DocumentPermissions, GrpcClientService, UserInfo } from './grpc-client.service';
import { decodeJwt } from 'jose';
import { logger } from '../utils/logger';

/**
 * Authentication & authorization service.
 *
 * Mirrors the Rust AuthService:
 * - Decodes JWT to extract user info
 * - Optionally verifies token via gRPC User.CheckAccessToken
 * - Checks document permissions via gRPC Document.QueryUserPermissions
 * - Provides a debounced NotifyModifyDocument call
 */
export class AuthService {
    constructor(
        private readonly _grpcClient: GrpcClientService,
        private readonly _skipTokenVerification: boolean,
        private readonly _skipPermissionCheck: boolean
    ) {}

    /**
     * Verify an access token and extract user info.
     */
    async verifyToken(token: string): Promise<UserInfo> {
        if (!token) {
            throw new Error('Token is empty');
        }

        const payload = decodeJwt(token);
        const uid = (payload.id as string) || (payload.sub as string);
        if (!uid) {
            throw new Error('No user ID found in token');
        }
        const email = (payload.email as string) || '';

        if (!this._skipTokenVerification) {
            await this._grpcClient.verifyToken(uid, token);
            logger.info(`Token verified for user ${uid}`);
        } else {
            logger.info(`Skipping token verification for user ${uid}`);
        }

        return { uid, email };
    }

    /**
     * Check document permissions for a user.
     */
    async checkDocumentPermission(
        userId: string,
        docId: string,
        accessToken: string
    ): Promise<DocumentPermissions> {
        if (this._skipPermissionCheck) {
            logger.info(`Skipping permission check for user ${userId}, doc ${docId}`);
            return { readable: true, commentable: true, writable: true, owner: true };
        }

        return this._grpcClient.queryDocumentPermissions(userId, docId, accessToken);
    }
}
