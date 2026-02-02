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

import type { ICollaborationConfig } from './config.schema';
import {
    Disposable,
    IConfigService,
    toDisposable,
} from '@univerjs/core';
import { IPendingMutationSerivce } from '../services/offline-storage.service';
import { ISocketService } from '../services/socket.service';
import { COLLABORATION_PLUGIN_CONFIG_KEY } from './config.schema';

/**
 * Collaboration Controller (Main Thread)
 *
 * In the isomorphic architecture, this controller is simplified:
 * - useRemote=true (default): Socket and mutation handling happen in worker context.
 *   The controller only initializes socket for main-only mode.
 * - useRemote=false: Socket is created in main thread, but mutation handling
 *   is done by CollaborationService which listens to onMutationExecutedForCollab.
 *
 * The heavy lifting (OT transform, network sync, state machine) is handled by
 * CollaborationService, which runs either in worker (useRemote=true) or
 * main thread (useRemote=false).
 */
export class CollaborationController extends Disposable {
    constructor(
        @ISocketService private readonly _socketService: ISocketService,
        @IConfigService private readonly _configService: IConfigService,
        @IPendingMutationSerivce private readonly _pendingMutationSerivce: IPendingMutationSerivce
    ) {
        super();
        this._init();
    }

    private _init(): void {
        this._initPendingMutationServiceListener();
    }

    private _initPendingMutationServiceListener(): void {
        this.disposeWithMe(
            this._pendingMutationSerivce.ready$.subscribe((ready) => {
                if (!ready) return;
                this._initSocketIfNeeded();
            })
        );
    }

    private _initSocketIfNeeded(): void {
        const config = this._configService.getConfig<ICollaborationConfig>(
            COLLABORATION_PLUGIN_CONFIG_KEY
        )!;

        // In remote mode (useRemote=true), the socket is managed in the worker context.
        // NoopSocketService returns null, which is expected - skip socket setup.
        const useRemote = config.useRemote ?? true;
        if (useRemote) {
            // Socket is handled by CollaborationService in the worker context
            return;
        }

        // Main-only mode (useRemote=false): create socket in main thread
        const socket = this._socketService.createSocket(config);
        if (!socket) {
            throw new Error('Failed to create socket');
        }
        this.disposeWithMe(toDisposable(() => socket.disconnect()));
    }
}
