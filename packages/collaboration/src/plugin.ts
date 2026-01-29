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

import type { ICollaborationConfig } from './controller/config.schema';
import {
    IConfigService,
    Inject,
    Injector,
    merge,
    mergeOverrideWithDependencies,
    Plugin,
    registerDependencies,
    touchDependencies,
} from '@univerjs/core';
import { CollaborationController } from './controller/collaboration.controller';
import { COLLABORATION_PLUGIN_CONFIG_KEY, defaultPluginConfig } from './controller/config.schema';
import { AwarenessService, IAwarenessService } from './services/awareness.service';
import { CollaborationService, ICollaborationService } from './services/collaboration.service';
import { IPendingMutationSerivce, PendingMutationSerivce } from './services/offline-storage.service';
import { ISocketService, SocketService } from './services/socket.service';
import { ITransformService, TransformService } from './services/transform.service';

export class CollaborationPlugin extends Plugin {
    static override pluginName = 'COLLABORATION_PLUGIN';

    constructor(
        private readonly _config: Partial<ICollaborationConfig> = defaultPluginConfig,
        @Inject(Injector) protected override _injector: Injector,
        @IConfigService private readonly _configService: IConfigService
    ) {
        super();
        const { ...rest } = merge(
            {},
            defaultPluginConfig,
            this._config
        );
        this._configService.setConfig(COLLABORATION_PLUGIN_CONFIG_KEY, rest);
    }

    override onStarting(): void {
        const config = this._configService.getConfig<ICollaborationConfig>(COLLABORATION_PLUGIN_CONFIG_KEY);
        registerDependencies(this._injector, mergeOverrideWithDependencies([
            [ISocketService, { useClass: SocketService }],
            [CollaborationController],
            [ITransformService, { useClass: TransformService }],
            [IPendingMutationSerivce, { useClass: PendingMutationSerivce }],
            [ICollaborationService, { useClass: CollaborationService }],
            [IAwarenessService, { useClass: AwarenessService }],
        ], config?.override ?? []));

        touchDependencies(this._injector, [
            [CollaborationController],
            [IAwarenessService],
        ]);
    }
}
