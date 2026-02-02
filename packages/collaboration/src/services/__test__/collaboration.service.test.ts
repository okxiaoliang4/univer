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
import { BehaviorSubject, firstValueFrom, of } from 'rxjs';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { COLLABORATION_SERVICE_NAME } from '../../common/types';
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

    requestChannel<T>(name: string): T {
        const channel = this._channels.get(name);
        if (!channel) {
            throw new Error(`Channel ${name} not found`);
        }
        return channel as T;
    }

    registerChannel(name: string, channel: unknown): void {
        this._channels.set(name, channel as MockRPCChannel);
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
 */
function createMockRemoteService() {
    return {
        connectionStatus$: new BehaviorSubject<
            'connected' | 'disconnected' | 'connecting'
        >('connected'),
        docJoined$: new BehaviorSubject<string>(''),
        docLeft$: new BehaviorSubject<string>(''),
        getDocumentState$: vi.fn().mockReturnValue(
            new BehaviorSubject({
                state: 'synced',
                serverRev: 0,
                pendingCount: 0,
                awaitingCount: 0,
            }).asObservable()
        ),
        getSavedStatus$: vi.fn().mockReturnValue(of(true)),
        isDocumentSynced: vi.fn().mockReturnValue(true),
        getServerRev: vi.fn().mockReturnValue(5),
        flush: vi.fn(),
        reset: vi.fn(),
        setAwareness: vi.fn().mockResolvedValue(undefined),
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

    describe('connection status', () => {
        it('should expose connection status observable', () => {
            expect(service.connectionStatus$).toBeDefined();
        });

        it('should start with disconnected status before remote connects', async () => {
            // Initial state should be disconnected before remote initialization completes
            // Note: This tests the fallback behavior, not the RPC forwarding
            const status = await firstValueFrom(service.connectionStatus$);
            expect(status).toBe('disconnected');
        });
    });

    describe('document state', () => {
        it('should return document state observable', () => {
            const state$ = service.getDocumentState$('test-doc');
            expect(state$).toBeDefined();
        });

        it('should return default state when remote not ready', () => {
            // Create a new service without remote
            const noRemoteRpc = new MockRPCChannelService();
            const noRemoteService = new CollaborationProxyService(
                noRemoteRpc,
                logService
            );

            const state$ = noRemoteService.getDocumentState$('test-doc');
            state$.subscribe((state) => {
                expect(state.state).toBe('synced');
                expect(state.serverRev).toBe(0);
            });

            noRemoteService.dispose();
        });
    });

    describe('saved status', () => {
        it('should return observable for saved status', async () => {
            // Wait a tick for async initialization
            await new Promise((resolve) => setTimeout(resolve, 10));

            const savedStatus$ = service.getSavedStatus$('test-doc');
            expect(savedStatus$).toBeDefined();
        });

        it('should return true when remote not available', async () => {
            const noRemoteRpc = new MockRPCChannelService();
            const noRemoteService = new CollaborationProxyService(
                noRemoteRpc,
                logService
            );

            const savedStatus$ = noRemoteService.getSavedStatus$('test-doc');
            const saved = await firstValueFrom(savedStatus$);
            expect(saved).toBe(true);

            noRemoteService.dispose();
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
