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
import type { ICollaborationConfig } from './config.schema';
import {
    Disposable,
    ICommandService,
    IConfigService,
    isInternalEditorID,
    IUniverInstanceService,
    toDisposable,
} from '@univerjs/core';
import { ICollaborationService } from '../services/collaboration.service';
import { IPendingMutationSerivce } from '../services/offline-storage.service';
import { ISocketService } from '../services/socket.service';
import { COLLABORATION_PLUGIN_CONFIG_KEY } from './config.schema';

export class CollaborationController extends Disposable {
    constructor(
        @ISocketService private readonly _socketService: ISocketService,
        @IConfigService private readonly _configService: IConfigService,
        @ICommandService private readonly _commandService: ICommandService,
        @IUniverInstanceService private readonly _univerInstanceService: IUniverInstanceService,
        @ICollaborationService private readonly _collaborationService: ICollaborationService,
        @IPendingMutationSerivce private readonly _pendingMutationSerivce: IPendingMutationSerivce
    ) {
        super();
        this._init();
    }

    private _init(): void {
        this._initListener();
        this._initPendingMutationSerivceListener();
    }

    private _initPendingMutationSerivceListener(): void {
        this.disposeWithMe(
            this._pendingMutationSerivce.ready$.subscribe((ready) => {
                // TODO: 这里监听完之后可以complete了
                if (!ready) return;
                this._initSocket();
            })
        );
    }

    private _initSocket(): void {
        const config = this._configService.getConfig<ICollaborationConfig>(
            COLLABORATION_PLUGIN_CONFIG_KEY
        )!;
        const socket = this._socketService.createSocket(config.wsUrl);
        if (!socket) {
            throw new Error('Failed to create socket');
        }
        this.disposeWithMe(toDisposable(() => socket.disconnect()));
    }

    private _initListener(): void {
        this._initCommandListener();
    }

    private _initCommandListener(): void {
        this.disposeWithMe(
            this._commandService.onMutationExecutedForCollab((command, options) => {
                const commandParams = command.params as { unitId?: string } | undefined;
                const observedUnitId = commandParams?.unitId;
                if (options?.fromCollab) return;
                const unitId = (command.params as { unitId: string })?.unitId;
                if (!unitId) return;
                const unit = this._univerInstanceService.getUnit(unitId);
                if (!unit || isInternalEditorID(unitId)) return;
                const baseRev = this._collaborationService.getDocRev(unitId);
                this._collaborationService.sendChangeset({
                    unitId,
                    baseRev,
                    mutations: [command as IMutationInfo],
                });
            })
        );
    }
}
