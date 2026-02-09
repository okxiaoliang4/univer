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

import type { Dependency } from '@univerjs/core';
import type { ICollaborationConfig } from './controller/config.schema';
import {
    IConfigService,
    Inject,
    Injector,
    IUndoRedoService,
    merge,
    Plugin,
    registerDependencies,
    touchDependencies,
} from '@univerjs/core';
import { fromModule, IRPCChannelService } from '@univerjs/rpc';
import {
    AWARENESS_REMOTE_SERVICE_NAME,
    COLLABORATION_SERVICE_NAME,
    UNDO_REDO_TRANSFORM_SERVICE_NAME,
} from './common/types';
import { CollaborationController } from './controller/collaboration.controller';
import {
    COLLABORATION_PLUGIN_CONFIG_KEY,
    defaultPluginConfig,
} from './controller/config.schema';
import {
    AwarenessRemoteProxyService,
    AwarenessRemoteService,
    IAwarenessRemoteService,
} from './services/awareness-remote.service';
import {
    AwarenessService,
    IAwarenessService,
} from './services/awareness.service';
import {
    CollaborationProxyService,
    CollaborationService,
    ICollaborationService,
} from './services/collaboration.service';
import { INetworkService } from './services/network.service';
import { NoopNetworkService } from './services/noop-services';
import {
    IPendingMutationSerivce,
    PendingMutationSerivce,
} from './services/offline-storage.service';
import {
    ITransformService,
    TransformService,
} from './services/transform.service';
import {
    IUndoRedoTransformService,
    UndoRedoTransformService,
} from './services/undo-redo-transform.service';
import { CollaborationUndoRedoService } from './services/undo-redo.service';
import { WsNetworkService } from './services/ws-network.service';

/**
 * Unified Collaboration Plugin
 *
 * This plugin provides real-time collaboration capabilities with three operational modes:
 *
 * ## Mode 1: Remote Side (isRemoteSide: true)
 * For worker/server contexts. Registers actual implementations:
 * - TransformService (WASM OT)
 * - WsNetworkService (native WebSocket + HTTP)
 * - CollaborationService (orchestrator)
 * - UndoRedoTransformService
 * Exposes services via RPC for client to consume.
 *
 * ## Mode 2: Client with Remote (isRemoteSide: false, useRemote: true) [DEFAULT]
 * For main thread with remote context. Uses RPC proxies:
 * - CollaborationProxyService (proxy to remote)
 * - CollaborationUndoRedoService (lazy transform via RPC)
 * Heavy operations run in remote context for better performance.
 *
 * ## Mode 3: Main-Only (isRemoteSide: false, useRemote: false)
 * All operations run in main thread:
 * - Useful for testing, SSR, or simple documents
 * - No remote context required
 *
 * Following the Univer remote pattern (similar to sheets-formula):
 * - Remote side uses fromModule() to expose services
 * - Client side uses toModule() to consume remote services
 */
export class CollaborationPlugin extends Plugin {
    static override pluginName = 'COLLABORATION_PLUGIN';

    constructor(
        private readonly _config: Partial<ICollaborationConfig> = defaultPluginConfig,
        @Inject(Injector) protected override _injector: Injector,
        @IConfigService private readonly _configService: IConfigService
    ) {
        super();

        const mergedConfig = merge({}, defaultPluginConfig, this._config);
        this._configService.setConfig(COLLABORATION_PLUGIN_CONFIG_KEY, mergedConfig);
    }

    override onStarting(): void {
        const config =
            this._configService.getConfig<ICollaborationConfig>(
                COLLABORATION_PLUGIN_CONFIG_KEY
            ) ?? {};
        const isRemoteSide = (config as ICollaborationConfig).isRemoteSide ?? false;
        const useRemote = (config as ICollaborationConfig).useRemote ?? true;

        if (isRemoteSide) {
            // Mode 1: Remote side - register actual implementations
            this._registerRemoteSideServices();
        } else if (useRemote) {
            // Mode 2: Client with remote - use RPC proxies (default)
            this._registerClientWithRemoteServices();
            touchDependencies(this._injector, [
                [IAwarenessService],
            ]);
        } else {
            // Mode 3: Main-only - all services run locally
            this._registerMainOnlyServices();
        }
    }

    /**
     * Register services for remote side (worker/server)
     */
    private _registerRemoteSideServices(): void {
        registerDependencies(this._injector, [
            // Core transform and network services
            [ITransformService, { useClass: TransformService }],
            [INetworkService, { useClass: WsNetworkService }],
            [IUndoRedoTransformService, { useClass: UndoRedoTransformService }],

            // Offline storage for pending mutations persistence
            [IPendingMutationSerivce, { useClass: PendingMutationSerivce }],

            // Awareness service (actual implementation in worker)
            [IAwarenessRemoteService, { useClass: AwarenessRemoteService }],

            // Main orchestrator service (actual implementation)
            [ICollaborationService, { useClass: CollaborationService }],
        ]);
    }

    /**
     * Register services for client with remote context (default mode)
     *
     * Note: In this mode, network communication happens in the remote context.
     * The main thread uses NoopNetworkService to satisfy dependency requirements.
     * Actual network operations are handled in the remote context.
     *
     * AwarenessService runs in main thread (to access UI state) and calls
     * AwarenessRemoteProxyService which forwards to worker via RPC.
     */
    private _registerClientWithRemoteServices(): void {
        const dependencies: Dependency[] = [
            // Offline storage for pending mutations
            [IPendingMutationSerivce, { useClass: PendingMutationSerivce }],

            // Noop network service for main thread - actual implementation runs in remote
            [INetworkService, { useClass: NoopNetworkService }],

            // Awareness remote proxy - forwards to worker via RPC
            [IAwarenessRemoteService, { useClass: AwarenessRemoteProxyService }],

            // Awareness service runs in main thread, calls remote via RPC
            [IAwarenessService, { useClass: AwarenessService }],

            // Controller for mutation handling
            [CollaborationController],

            // Proxy to remote service via RPC
            [ICollaborationService, { useClass: CollaborationProxyService }],
        ];
        registerDependencies(this._injector, dependencies);

        this._injector.replace([IUndoRedoService, { useClass: CollaborationUndoRedoService, lazy: true }]);
    }

    /**
     * Register services for main-only mode (no remote context)
     */
    private _registerMainOnlyServices(): void {
        const dependencies: Dependency[] = [
            // Core services
            [IPendingMutationSerivce, { useClass: PendingMutationSerivce }],
            [INetworkService, { useClass: WsNetworkService }],

            // Awareness remote service (runs locally in main thread)
            [IAwarenessRemoteService, { useClass: AwarenessRemoteService }],

            // Awareness service (manages UI state)
            [IAwarenessService, { useClass: AwarenessService }],

            // Controller
            [CollaborationController],

            // Local transform service (runs in main thread)
            [ITransformService, { useClass: TransformService }],
            [IUndoRedoTransformService, { useClass: UndoRedoTransformService }],

            // Full collaboration service (runs locally)
            [ICollaborationService, { useClass: CollaborationService }],
        ];

        registerDependencies(this._injector, dependencies);

        this._injector.replace([IUndoRedoService, { useClass: CollaborationUndoRedoService, lazy: true }]);
    }

    override onReady(): void {
        const config = this._configService.getConfig<ICollaborationConfig>(
            COLLABORATION_PLUGIN_CONFIG_KEY
        ) ?? {};
        const isRemoteSide = (config as ICollaborationConfig).isRemoteSide ?? false;
        const useRemote = (config as ICollaborationConfig).useRemote ?? true;

        if (isRemoteSide) {
            // Remote side: expose services via RPC
            this._registerRPCChannels();

            // Touch to instantiate
            touchDependencies(this._injector, [[ICollaborationService]]);
        } else if (useRemote) {
            // Client with remote: touch controller and awareness
            touchDependencies(this._injector, [
                [CollaborationController],
                [IAwarenessService],
            ]);
        } else {
            // Main-only: touch all client services including awareness
            touchDependencies(this._injector, [
                [CollaborationController],
                [IAwarenessService],
            ]);
        }
    }

    /**
     * Register RPC channels for remote side
     */
    private _registerRPCChannels(): void {
        const rpcService = this._injector.get(IRPCChannelService);

        // Expose CollaborationService
        rpcService.registerChannel(
            COLLABORATION_SERVICE_NAME,
            fromModule(this._injector.get(ICollaborationService))
        );

        // Expose AwarenessRemoteService
        rpcService.registerChannel(
            AWARENESS_REMOTE_SERVICE_NAME,
            fromModule(this._injector.get(IAwarenessRemoteService))
        );

        // Expose UndoRedoTransformService
        rpcService.registerChannel(
            UNDO_REDO_TRANSFORM_SERVICE_NAME,
            fromModule(this._injector.get(IUndoRedoTransformService))
        );
    }
}
