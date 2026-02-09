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

import {
    Disposable,
    IConfigService,
} from '@univerjs/core';
import { IPendingMutationSerivce } from '../services/offline-storage.service';

/**
 * Collaboration Controller (Main Thread)
 *
 * In the isomorphic architecture, this controller is simplified:
 * - useRemote=true (default): Network communication happens in worker context.
 *   CollaborationService manages connection and document sync.
 * - useRemote=false: All services run locally in main thread.
 *   CollaborationService handles connection via INetworkService.connect().
 *
 * The heavy lifting (OT transform, network sync, state machine) is handled by
 * CollaborationService, which runs either in worker (useRemote=true) or
 * main thread (useRemote=false).
 */
export class CollaborationController extends Disposable {
    constructor(
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
                // if (!ready) return;
                // PendingMutationService is ready. Connection is now managed by
                // CollaborationService._initNetworkListeners() → _networkService.connect().
            })
        );
    }
}
