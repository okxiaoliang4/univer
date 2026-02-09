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

import process from 'node:process';

export interface IConfig {
    port: number;
    metricsPort: number;
    redisUrl: string;
    etcdEndpoints: string[];
    userRpcPrefix: string;
    documentRpcPrefix: string;
    skipTokenVerification: boolean;
    skipPermissionCheck: boolean;
    awarenessTtlSeconds: number;
    pingIntervalMs: number;
    pingTimeoutMs: number;
}

export function loadConfig(): IConfig {
    return {
        port: Number.parseInt(process.env.PORT || '8080', 10),
        metricsPort: Number.parseInt(process.env.METRICS_PORT || '9464', 10),
        redisUrl: process.env.REDIS_URL || 'redis://localhost:6379',
        etcdEndpoints: (process.env.ETCD_ENDPOINTS || 'http://localhost:2379')
            .split(',')
            .map((s) => s.trim())
            .filter(Boolean),
        userRpcPrefix: process.env.USER_RPC_PREFIX || 'user.rpc',
        documentRpcPrefix: process.env.DOCUMENT_RPC_PREFIX || 'document.rpc',
        skipTokenVerification:
            process.env.SKIP_TOKEN_VERIFICATION === 'true' ||
            process.env.SKIP_TOKEN_VERIFICATION === '1',
        skipPermissionCheck:
            process.env.SKIP_PERMISSION_CHECK === 'true' ||
            process.env.SKIP_PERMISSION_CHECK === '1',
        awarenessTtlSeconds: Number.parseInt(process.env.AWARENESS_TTL_SECONDS || '120', 10),
        pingIntervalMs: Number.parseInt(process.env.PING_INTERVAL_MS || '15000', 10),
        pingTimeoutMs: Number.parseInt(process.env.PING_TIMEOUT_MS || '30000', 10),
    };
}
