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
} from '../../services/__test__/test-utils';
import { CollaborationController } from '../collaboration.controller';
import { COLLABORATION_PLUGIN_CONFIG_KEY } from '../config.schema';

/**
 * CollaborationController Tests
 *
 * In the new architecture:
 * - Controller no longer manages socket lifecycle (socket.service removed)
 * - Connection is managed by CollaborationService via INetworkService
 * - Controller only initializes PendingMutationService listener
 */
describe('CollaborationController', () => {
    it('initializes when PendingMutationService is ready', () => {
        const configService = new MockConfigService({
            [COLLABORATION_PLUGIN_CONFIG_KEY]: { wsUrl: 'ws://test', accessToken: 'test-token', useRemote: false },
        });
        const pendingMutationService = new MockOfflineStorageService();

        const controller = new CollaborationController(
            configService as unknown as IConfigService,
            pendingMutationService
        );

        // Trigger ready
        pendingMutationService.setReady(true);

        // Controller should be created successfully
        expect(controller).toBeDefined();
        controller.dispose();
    });

    it('can be created without PendingMutationService being ready', () => {
        const configService = new MockConfigService({
            [COLLABORATION_PLUGIN_CONFIG_KEY]: { wsUrl: 'ws://test', accessToken: 'test-token' },
        });
        const pendingMutationService = new MockOfflineStorageService();

        // Never trigger ready
        const controller = new CollaborationController(
            configService as unknown as IConfigService,
            pendingMutationService
        );

        expect(controller).toBeDefined();
        controller.dispose();
    });
});
