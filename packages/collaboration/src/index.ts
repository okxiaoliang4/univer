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

export type { IDocumentSyncState, IUserAwareness, NetworkConnectionStatus } from './common/types';
export type { ICollaborationConfig } from './controller/config.schema';
export { CollaborationPlugin } from './plugin';
export {
    AwarenessRemoteProxyService,
    AwarenessRemoteService,
    IAwarenessRemoteService,
} from './services/awareness-remote.service';
export { IAwarenessService } from './services/awareness.service';
export {
    CollaborationProxyService,
    CollaborationService,
    ICollaborationService,
} from './services/collaboration.service';
export { CollaborationUndoRedoService } from './services/undo-redo.service';
