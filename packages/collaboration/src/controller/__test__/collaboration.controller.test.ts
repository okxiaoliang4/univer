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

import type { IConfigService } from '@univerjs/core';
import { describe, expect, it } from 'vitest';
import {
    MockConfigService,
    MockOfflineStorageService,
    MockSocketService,
} from '../../services/__test__/test-utils';
import { CollaborationController } from '../collaboration.controller';
import { COLLABORATION_PLUGIN_CONFIG_KEY } from '../config.schema';

/**
 * CollaborationController Tests
 *
 * In the isomorphic architecture:
 * - Controller is simplified to only manage socket lifecycle in main-only mode
 * - Mutation handling is done by CollaborationService (runs in worker or main)
 * - Controller waits for PendingMutationService to be ready before creating socket
 */
describe('CollaborationController', () => {
    it('creates socket in main-only mode (useRemote: false)', () => {
        const socketService = new MockSocketService();
        const configService = new MockConfigService({
            [COLLABORATION_PLUGIN_CONFIG_KEY]: { wsUrl: 'ws://test', userId: 'user-1', useRemote: false },
        });
        const pendingMutationService = new MockOfflineStorageService();

        // Trigger ready to allow socket creation
        pendingMutationService.setReady(true);

        const controller = new CollaborationController(
            socketService,
            configService as unknown as IConfigService,
            pendingMutationService
        );

        expect(socketService.createdUrl).toBe('ws://test');
        controller.dispose();
        expect(socketService.getSocket()?.disconnected).toBe(true);
    });

    it('skips socket creation in remote mode (useRemote: true, default)', () => {
        const socketService = new MockSocketService();
        const configService = new MockConfigService({
            // useRemote defaults to true, so socket creation should be skipped
            [COLLABORATION_PLUGIN_CONFIG_KEY]: { wsUrl: 'ws://test', userId: 'user-1' },
        });
        const pendingMutationService = new MockOfflineStorageService();

        // Trigger ready
        pendingMutationService.setReady(true);

        const controller = new CollaborationController(
            socketService,
            configService as unknown as IConfigService,
            pendingMutationService
        );

        // In remote mode, socket is managed by the worker context, not created here
        expect(socketService.createdUrl).toBeNull();
        controller.dispose();
    });

    it('waits for PendingMutationService to be ready before creating socket', () => {
        const socketService = new MockSocketService();
        const configService = new MockConfigService({
            [COLLABORATION_PLUGIN_CONFIG_KEY]: { wsUrl: 'ws://test', userId: 'user-1', useRemote: false },
        });
        const pendingMutationService = new MockOfflineStorageService();

        // Don't trigger ready yet
        const controller = new CollaborationController(
            socketService,
            configService as unknown as IConfigService,
            pendingMutationService
        );

        // Socket should not be created yet
        expect(socketService.createdUrl).toBeNull();

        // Now trigger ready
        pendingMutationService.setReady(true);

        // Socket should now be created
        expect(socketService.createdUrl).toBe('ws://test');

        controller.dispose();
    });

    it('does not create socket if PendingMutationService never becomes ready', () => {
        const socketService = new MockSocketService();
        const configService = new MockConfigService({
            [COLLABORATION_PLUGIN_CONFIG_KEY]: { wsUrl: 'ws://test', userId: 'user-1', useRemote: false },
        });
        const pendingMutationService = new MockOfflineStorageService();

        // Never trigger ready
        const controller = new CollaborationController(
            socketService,
            configService as unknown as IConfigService,
            pendingMutationService
        );

        // Socket should not be created
        expect(socketService.createdUrl).toBeNull();

        controller.dispose();
    });
});
