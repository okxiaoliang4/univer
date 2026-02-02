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

import type { IRPCChannelService } from '@univerjs/rpc';
import type { ICollaborationCallbackService } from '../collaboration.service';
import { firstValueFrom } from 'rxjs';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { COLLABORATION_CALLBACK_SERVICE_NAME, COLLABORATION_SERVICE_NAME } from '../../common/types';
import { CollaborationProxyService } from '../collaboration.service';
import { MockLogService } from './test-utils';

/**
 * Mock RPC channel that simulates the channel interface expected by toModule()
 *
 * When toModule() creates a Proxy, it:
 * - Calls channel.subscribe(propName, args) for observable properties (ending with $)
 * - Calls channel.call(propName, args) for regular methods
 */
class MockRPCChannel {
    constructor(private readonly _service: Record<string, unknown>) {}

    /**
     * Called by toModule() for observable properties (properties ending with $)
     * This includes both plain observable properties and methods that return observables.
     */
    subscribe(propName: string, args: unknown[]): unknown {
        const prop = this._service[propName];

        // If it's a function (method that returns observable), call it with args
        if (typeof prop === 'function') {
            return prop.apply(this._service, args);
        }

        // If it's a BehaviorSubject or similar, return its observable
        if (
            prop &&
            typeof (prop as { asObservable?: () => unknown }).asObservable ===
                'function'
        ) {
            return (prop as { asObservable: () => unknown }).asObservable();
        }

        return prop;
    }

    /**
     * Called by toModule() for regular method calls
     */
    call(propName: string, args: unknown[]): unknown {
        const method = this._service[propName];
        if (typeof method === 'function') {
            return method.apply(this._service, args);
        }
        return method;
    }
}

/**
 * Mock RPC channel service for testing
 */
class MockRPCChannelService implements IRPCChannelService {
    private _channels = new Map<string, MockRPCChannel>();
    private _registeredCallbacks = new Map<string, unknown>();

    requestChannel<T>(name: string): T {
        const channel = this._channels.get(name);
        if (!channel) {
            throw new Error(`Channel ${name} not found`);
        }
        return channel as T;
    }

    registerChannel(name: string, channel: unknown): { dispose: () => void } {
        this._registeredCallbacks.set(name, channel);
        return { dispose: () => this._registeredCallbacks.delete(name) };
    }

    /**
     * Get a registered callback service (for testing)
     */
    getRegisteredCallback<T>(name: string): T | undefined {
        return this._registeredCallbacks.get(name) as T | undefined;
    }

    /**
     * Set a service as a channel (wraps it in MockRPCChannel)
     */
    setService(name: string, service: Record<string, unknown>): void {
        this._channels.set(name, new MockRPCChannel(service));
    }
}

/**
 * Mock remote service for testing CollaborationProxyService
 *
 * This simulates the CollaborationService running in a remote context (worker)
 * Updated to use the new interface without Observable properties
 */
function createMockRemoteService() {
    return {
        getConnectionStatus: vi.fn().mockReturnValue('connected'),
        getDocumentState: vi.fn().mockReturnValue({
            state: 'synced',
            serverRev: 0,
            pendingCount: 0,
            awaitingCount: 0,
        }),
        getSavedStatus: vi.fn().mockReturnValue(true),
        isDocumentSynced: vi.fn().mockReturnValue(true),
        getServerRev: vi.fn().mockReturnValue(5),
        flush: vi.fn(),
        reset: vi.fn(),
    };
}

describe('CollaborationProxyService', () => {
    let rpcService: MockRPCChannelService;
    let logService: MockLogService;
    let mockRemoteService: ReturnType<typeof createMockRemoteService>;
    let service: CollaborationProxyService;

    beforeEach(() => {
        rpcService = new MockRPCChannelService();
        logService = new MockLogService();
        mockRemoteService = createMockRemoteService();

        // Register the mock remote service using the correct channel name
        rpcService.setService(COLLABORATION_SERVICE_NAME, mockRemoteService);

        service = new CollaborationProxyService(rpcService, logService);
    });

    afterEach(() => {
        service.dispose();
    });

    describe('callback service registration', () => {
        it('should register callback service on construction', () => {
            const callback = rpcService.getRegisteredCallback<ICollaborationCallbackService>(
                COLLABORATION_CALLBACK_SERVICE_NAME
            );
            expect(callback).toBeDefined();
        });
    });

    describe('connection status', () => {
        it('should expose connection status observable', () => {
            expect(service.connectionStatus$).toBeDefined();
        });

        it('should start with disconnected status before callback', async () => {
            const status = await firstValueFrom(service.connectionStatus$);
            expect(status).toBe('disconnected');
        });

        it('should update connection status when callback is invoked', async () => {
            // Invoke the callback to simulate worker notification
            service.onConnectionStatusChange('connected');

            const status = await firstValueFrom(service.connectionStatus$);
            expect(status).toBe('connected');
        });

        it('should return current connection status via getConnectionStatus', () => {
            service.onConnectionStatusChange('connecting');
            expect(service.getConnectionStatus()).toBe('connecting');
        });
    });

    describe('document state', () => {
        it('should return document state observable', () => {
            const state$ = service.getDocumentState$('test-doc');
            expect(state$).toBeDefined();
        });

        it('should return default state before callback', async () => {
            const state$ = service.getDocumentState$('test-doc');
            const state = await firstValueFrom(state$);
            expect(state.state).toBe('synced');
            expect(state.serverRev).toBe(0);
        });

        it('should update document state when callback is invoked', async () => {
            const newState = {
                state: 'pending' as const,
                serverRev: 5,
                pendingCount: 2,
                awaitingCount: 0,
            };

            service.onDocumentStateChange('test-doc', newState);

            const state = service.getDocumentState('test-doc');
            expect(state.state).toBe('pending');
            expect(state.serverRev).toBe(5);
            expect(state.pendingCount).toBe(2);
        });

        it('should emit to observable when state changes', async () => {
            const state$ = service.getDocumentState$('test-doc');

            // Subscribe and collect values
            const states: unknown[] = [];
            const subscription = state$.subscribe((state) => states.push(state));

            service.onDocumentStateChange('test-doc', {
                state: 'awaiting',
                serverRev: 3,
                pendingCount: 0,
                awaitingCount: 1,
            });

            expect(states.length).toBe(2); // default + updated
            expect((states[1] as { state: string }).state).toBe('awaiting');

            subscription.unsubscribe();
        });
    });

    describe('saved status', () => {
        it('should return observable for saved status', () => {
            const savedStatus$ = service.getSavedStatus$('test-doc');
            expect(savedStatus$).toBeDefined();
        });

        it('should return true by default', () => {
            expect(service.getSavedStatus('test-doc')).toBe(true);
        });

        it('should update saved status when callback is invoked', () => {
            service.onSavedStatusChange('test-doc', false);
            expect(service.getSavedStatus('test-doc')).toBe(false);

            service.onSavedStatusChange('test-doc', true);
            expect(service.getSavedStatus('test-doc')).toBe(true);
        });
    });

    describe('document sync status', () => {
        it('should return true when document is synced', async () => {
            // Wait a tick for async initialization
            await new Promise((resolve) => setTimeout(resolve, 10));

            mockRemoteService.isDocumentSynced.mockReturnValue(true);
            expect(service.isDocumentSynced('test-doc')).toBe(true);
        });

        it('should return false when document is not synced', async () => {
            // Wait a tick for async initialization
            await new Promise((resolve) => setTimeout(resolve, 10));

            mockRemoteService.isDocumentSynced.mockReturnValue(false);
            expect(service.isDocumentSynced('test-doc')).toBe(false);
        });

        it('should return true when remote not available', () => {
            const noRemoteRpc = new MockRPCChannelService();
            const noRemoteService = new CollaborationProxyService(
                noRemoteRpc,
                logService
            );
            expect(noRemoteService.isDocumentSynced('test-doc')).toBe(true);
            noRemoteService.dispose();
        });
    });

    describe('document revision', () => {
        it('should return server revision from remote', async () => {
            // Wait a tick for async initialization
            await new Promise((resolve) => setTimeout(resolve, 10));

            mockRemoteService.getServerRev.mockReturnValue(10);
            expect(service.getServerRev('test-doc')).toBe(10);
        });

        it('should return 0 when remote not available', () => {
            const noRemoteRpc = new MockRPCChannelService();
            const noRemoteService = new CollaborationProxyService(
                noRemoteRpc,
                logService
            );
            expect(noRemoteService.getServerRev('test-doc')).toBe(0);
            noRemoteService.dispose();
        });
    });

    describe('flush', () => {
        it('should have flush method', () => {
            expect(typeof service.flush).toBe('function');
        });

        it('should call remote flush when remote is ready', async () => {
            // Wait a tick for async initialization
            await new Promise((resolve) => setTimeout(resolve, 10));

            service.flush('test-doc');
            expect(mockRemoteService.flush).toHaveBeenCalledWith('test-doc');
        });
    });

    describe('reset', () => {
        it('should call remote reset', async () => {
            // Wait a tick for async initialization
            await new Promise((resolve) => setTimeout(resolve, 10));

            service.reset('test-doc');
            expect(mockRemoteService.reset).toHaveBeenCalledWith('test-doc');
        });
    });

    describe('dispose', () => {
        it('should complete all subjects on dispose', () => {
            let completed = false;
            service.connectionStatus$.subscribe({
                complete: () => {
                    completed = true;
                },
            });

            service.dispose();
            expect(completed).toBe(true);
        });
    });
});
