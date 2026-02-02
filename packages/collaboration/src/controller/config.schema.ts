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

import type { DependencyOverride } from '@univerjs/core';

export const COLLABORATION_PLUGIN_CONFIG_KEY = 'collaboration.config';

export const configSymbol = Symbol(COLLABORATION_PLUGIN_CONFIG_KEY);

export interface ICollaborationConfig {
    wsUrl: string;
    accessToken: string;
    params?: Record<string, string | number | boolean>;
    override?: DependencyOverride;

    /**
     * Whether this plugin instance is the remote side (worker/server).
     * - true: Register actual implementations and expose via RPC
     * - false: Client side (default)
     * @default false
     */
    isRemoteSide?: boolean;

    /**
     * Whether to use a remote context for collaboration (only when isRemoteSide=false).
     * - true: Use RPC proxies to communicate with remote context
     * - false: Run all services locally in main thread (main-only mode)
     * @default true
     */
    useRemote?: boolean;
}

export const defaultPluginConfig: ICollaborationConfig = {
    wsUrl: 'ws://localhost:8080/ws',
    accessToken: '',
    isRemoteSide: false,
    useRemote: true,
};
