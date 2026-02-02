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

import type { IUser } from '@univerjs/core';
import type { ISetSelectionsOperationParams } from '@univerjs/sheets';
import type { Observable, Subscription } from 'rxjs';
import type { IUserAwareness } from '../common/types';
import {
    createIdentifier,
    Disposable,
    ILogService,
    Inject,
    Injector,
    isInternalEditorID,
    IUniverInstanceService,
    UserManagerService,
} from '@univerjs/core';
import { BehaviorSubject, Subject } from 'rxjs';
import { IAwarenessRemoteService } from './awareness-remote.service';
import { ICollaborationService } from './collaboration.service';

export const IAwarenessService =
    createIdentifier<IAwarenessService>('IAwarenessService');

export interface IAwarenessService extends Disposable {
    init$: Observable<Map<string, boolean>>;
    userIds$: Observable<Set<string>>;
    clientId$: Observable<Map<string, number>>;
    state$: Observable<Map<string, Map<number, IAwarenessState>>>;
    add$: Observable<{ unitId: string; clientId: number }>;
    remove$: Observable<{ unitId: string; clientId: number }>;
    update$: Observable<{ unitId: string; clientId: number }>;
    getClientId(unitId: string): number | undefined;
    getState(unitId: string): Map<number, IAwarenessState> | undefined;
    getUnitIds(): string[];
    setLocalStateField(unitId: string, field: string, value: unknown): void;
}

export interface IAwarenessState {
    clientID: number;
    id: string;
    name: string;
    selectionParams: ISetSelectionsOperationParams;
    // color: string
    // avatar: string
}

/**
 * AwarenessService manages user presence and selection awareness.
 *
 * In isomorphic architecture, this service runs in the main thread and
 * communicates with the worker via ICollaborationService RPC proxy.
 * It uses IUniverInstanceService to detect document loading/unloading,
 * and handles connection/reconnection scenarios.
 */
export class AwarenessService extends Disposable implements IAwarenessService {
    private _joinedUnits: Set<string> = new Set();

    private _init$ = new BehaviorSubject<Map<string, boolean>>(new Map());
    private _userIds$: BehaviorSubject<Set<string>> = new BehaviorSubject(
        new Set()
    );

    private _state$ = new BehaviorSubject<
        Map<string, Map<number, IAwarenessState>>
    >(new Map());

    private _clientId$ = new BehaviorSubject<Map<string, number>>(new Map());
    private _add$ = new Subject<{ unitId: string; clientId: number }>();
    private _remove$ = new Subject<{ unitId: string; clientId: number }>();
    private _update$ = new Subject<{ unitId: string; clientId: number }>();
    private _localState = new Map<string, IAwarenessState>();

    // Track document-specific subscriptions for proper cleanup
    private _documentSubscriptions = new Map<
        string,
        Array<{ unsubscribe: () => void }>
    >();

    // Track awareness update subscription
    private _awarenessSubscription: Subscription | null = null;

    init$ = this._init$.asObservable();
    userIds$ = this._userIds$.asObservable();
    clientId$ = this._clientId$.asObservable();
    state$ = this._state$.asObservable();
    add$ = this._add$.asObservable();
    remove$ = this._remove$.asObservable();
    update$ = this._update$.asObservable();

    constructor(
        @Inject(Injector) readonly _injector: Injector,
        @Inject(IAwarenessRemoteService)
        private readonly _awarenessRemoteService: IAwarenessRemoteService,
        @Inject(ICollaborationService)
        private readonly _collaborationService: ICollaborationService,
        @Inject(UserManagerService)
        private readonly _userManagerService: UserManagerService,
        @Inject(IUniverInstanceService)
        private readonly _univerInstanceService: IUniverInstanceService,
        @Inject(ILogService)
        private readonly _logger: ILogService
    ) {
        super();

        this._initInstanceListeners(); // 监听文档加载/卸载
        this._initConnectionListener(); // 监听连接状态，连接时重新初始化
        this._initAwarenessSubscription(); // 订阅 awarenessUpdate$
    }

    /**
     * Listen to document loading/unloading from IUniverInstanceService
     */
    private _initInstanceListeners(): void {
        this._logger.log('AwarenessService: initInstanceListeners');
        // Listen to document added
        this.disposeWithMe(
            this._univerInstanceService.unitAdded$.subscribe((unit) => {
                this._logger.log('AwarenessService: unitAdded', unit.getUnitId());
                const unitId = unit.getUnitId();
                if (isInternalEditorID(unitId)) return;
                this._joinedUnits.add(unitId);
                this._initAwareness(unitId);
            })
        );

        // Listen to document disposed
        this.disposeWithMe(
            this._univerInstanceService.unitDisposed$.subscribe((unit) => {
                this._logger.log('AwarenessService: unitDisposed', unit.getUnitId());
                const unitId = unit.getUnitId();
                if (isInternalEditorID(unitId)) return;
                this._cleanupUnit(unitId);
            })
        );
    }

    /**
     * Listen to connection status changes for reconnection handling
     */
    private _initConnectionListener(): void {
        this.disposeWithMe(
            this._collaborationService.connectionStatus$.subscribe((status) => {
                if (status === 'connected') {
                    // Connection established (initial or reconnect)
                    this._onConnectionEstablished();
                } else if (status === 'disconnected') {
                    // Mark all units as not initialized (will re-init on reconnect)
                }
            })
        );
    }

    /**
     * Subscribe to awareness updates from remote (via RPC)
     */
    private _initAwarenessSubscription(): void {
        this._awarenessSubscription =
            this._awarenessRemoteService.awarenessUpdate$.subscribe((awareness) => {
                this._handleAwarenessUpdate(awareness);
            });
        this.disposeWithMe({
            dispose: () => this._awarenessSubscription?.unsubscribe(),
        });
    }

    /**
     * Handle connection established (initial or reconnect)
     */
    private _onConnectionEstablished(): void {
        // Re-init awareness for all joined units
        // This handles both:
        // 1. Initial connection (units loaded before socket ready)
        // 2. Reconnection (re-fetch states, re-broadcast local state)
        this._joinedUnits.forEach((unitId) => {
            this._reinitAwareness(unitId);
        });
    }

    /**
     * Re-initialize awareness (used for reconnection or late socket connection)
     */
    private async _reinitAwareness(unitId: string): Promise<void> {
        try {
            // 1. Fetch current awareness states from server
            const states = await this._awarenessRemoteService.initAwareness(unitId);

            // 2. Update local state map (this will clear stale entries)
            const stateMap = new Map<number, IAwarenessState>();
            for (const awareness of states) {
                const stateClientId = this._getClientIdFromUserId(awareness.userId);
                // Convert IUserAwareness.selection to ISetSelectionsOperationParams format
                const selection = awareness.selection;
                const selectionParams: ISetSelectionsOperationParams = {
                    unitId,
                    subUnitId: selection?.sheetId || '',
                    selections:
                        selection?.ranges?.map((range) => ({
                            range: {
                                startRow: range.startRow,
                                startColumn: range.startColumn,
                                endRow: range.endRow,
                                endColumn: range.endColumn,
                            },
                            primary: null,
                            style: null,
                        })) || [],
                };
                const state: IAwarenessState = {
                    clientID: stateClientId,
                    id: awareness.userId,
                    name: awareness.userName || '',
                    selectionParams,
                };

                stateMap.set(state.clientID, state);
                this._addClient(state.id);
            }

            this._state$.value.set(unitId, stateMap);
            this._state$.next(this._state$.value);

            // 3. Re-broadcast local state
            const localState = this._localState.get(unitId);
            if (localState) {
                this._broadcastAwareness(unitId, localState);
            }
        } catch (error) {
            console.error('AwarenessService: Failed to reinit awareness', error);
        }
    }

    /**
     * Clean up unit when it's disposed
     */
    private _cleanupUnit(unitId: string): void {
        this._joinedUnits.delete(unitId);
        this._localState.delete(unitId);
        this._state$.value.delete(unitId);
        this._clientId$.value.delete(unitId);
        this._init$.value.delete(unitId);
        this._state$.next(this._state$.value);
        this._clientId$.next(this._clientId$.value);
        this._init$.next(this._init$.value);

        // Clean up document-specific subscriptions
        const subscriptions = this._documentSubscriptions.get(unitId);
        if (subscriptions) {
            subscriptions.forEach((sub) => sub.unsubscribe());
            this._documentSubscriptions.delete(unitId);
        }
    }

    /**
     * Handle awareness update received from remote
     */
    private _handleAwarenessUpdate(awareness: IUserAwareness): void {
        const targetUnitId = awareness.docId;
        if (!targetUnitId || !this._joinedUnits.has(targetUnitId)) {
            return;
        }

        // Convert IUserAwareness.selection to ISetSelectionsOperationParams format
        const selection = awareness.selection;
        const selectionParams: ISetSelectionsOperationParams = {
            unitId: targetUnitId,
            subUnitId: selection?.sheetId || '',
            selections:
                selection?.ranges?.map((range) => ({
                    range: {
                        startRow: range.startRow,
                        startColumn: range.startColumn,
                        endRow: range.endRow,
                        endColumn: range.endColumn,
                    },
                    primary: null,
                    style: null,
                })) || [],
        };

        // Convert IUserAwareness to IAwarenessState
        const clientId = this._getClientIdFromUserId(awareness.userId);
        const state: IAwarenessState = {
            clientID: clientId,
            id: awareness.userId,
            name: awareness.userName || '',
            selectionParams,
        };

        const stateMap = this._state$.value.get(targetUnitId) ?? new Map();

        // Remove old entries for the same user with different clientId
        stateMap.forEach((existing, existingClientId) => {
            if (existing.id === state.id && existingClientId !== state.clientID) {
                stateMap.delete(existingClientId);
                this._remove$.next({
                    unitId: targetUnitId,
                    clientId: existingClientId,
                });
            }
        });

        stateMap.set(state.clientID, state);
        this._state$.value.set(targetUnitId, stateMap);
        this._state$.next(this._state$.value);
        this._update$.next({ unitId: targetUnitId, clientId: state.clientID });
    }

    /**
     * Get or create a client ID for a user
     */
    private _getClientIdFromUserId(userId: string): number {
        // Simple hash function to convert userId to a numeric clientId
        let hash = 0;
        for (let i = 0; i < userId.length; i++) {
            const char = userId.charCodeAt(i);
            hash = (hash << 5) - hash + char;
            hash = hash & hash; // Convert to 32bit integer
        }
        return Math.abs(hash);
    }

    private async _initAwareness(unitId: string): Promise<void> {
        this._logger.log('AwarenessService: initAwareness', unitId);
        const clientId =
            this._clientId$.value.get(unitId) ?? this._createClientId();
        this._clientId$.value.set(unitId, clientId);
        this._clientId$.next(this._clientId$.value);

        // Fetch existing awareness states via IAwarenessRemoteService
        try {
            const states = await this._awarenessRemoteService.initAwareness(unitId);
            const stateMap = new Map<number, IAwarenessState>();

            for (const awareness of states) {
                const stateClientId = this._getClientIdFromUserId(awareness.userId);
                // Convert IUserAwareness.selection to ISetSelectionsOperationParams format
                const selection = awareness.selection;
                const selectionParams: ISetSelectionsOperationParams = {
                    unitId,
                    subUnitId: selection?.sheetId || '',
                    selections:
                        selection?.ranges?.map((range) => ({
                            range: {
                                startRow: range.startRow,
                                startColumn: range.startColumn,
                                endRow: range.endRow,
                                endColumn: range.endColumn,
                            },
                            primary: null,
                            style: null,
                        })) || [],
                };
                const state: IAwarenessState = {
                    clientID: stateClientId,
                    id: awareness.userId,
                    name: awareness.userName || '',
                    selectionParams,
                };

                // Remove duplicates for same user
                const existingEntries = Array.from(stateMap.entries()).filter(
                    ([, existing]) => existing.id === state.id
                );
                existingEntries.forEach(([existingClientId]) => {
                    stateMap.delete(existingClientId);
                    this._remove$.next({
                        unitId,
                        clientId: existingClientId,
                    });
                });

                stateMap.set(state.clientID, state);
                this._add$.next({ unitId, clientId: state.clientID });
                this._addClient(state.id);
            }

            this._state$.value.set(unitId, stateMap);
            this._state$.next(this._state$.value);

            // Mark as initialized
            this._init$.value.set(unitId, true);
            this._init$.next(this._init$.value);
        } catch (error) {
            console.error('AwarenessService: Failed to init awareness', error);
        }

        // Set up local state broadcasting
        const setCurrentState = (currentUser: IUser) => {
            if (!currentUser) return;
            const state: IAwarenessState = {
                clientID: clientId,
                id: currentUser.userID,
                name: currentUser.name,
                selectionParams: {
                    unitId: '',
                    subUnitId: '',
                    selections: [],
                },
            };
            this._localState.set(unitId, state);
            this._broadcastAwareness(unitId, state);
        };

        const user = this._userManagerService.getCurrentUser();
        setCurrentState(user);

        // Create document-specific subscription that will be cleaned up on unitDisposed
        const subscription = this._userManagerService.currentUser$.subscribe(
            (currentUser) => {
                setCurrentState(currentUser);
            }
        );

        // Track subscription for cleanup
        if (!this._documentSubscriptions.has(unitId)) {
            this._documentSubscriptions.set(unitId, []);
        }
        this._documentSubscriptions.get(unitId)?.push(subscription);
    }

    /**
     * Broadcast local awareness state via IAwarenessRemoteService (RPC to worker)
     */
    private _broadcastAwareness(unitId: string, state: IAwarenessState): void {
        // Convert ISetSelectionsOperationParams to IUserAwareness.selection format
        const selectionParams = state.selectionParams;
        const awareness: IUserAwareness = {
            docId: unitId,
            userId: state.id,
            userName: state.name,
            selection: {
                sheetId: selectionParams?.subUnitId,
                ranges: selectionParams?.selections?.map((sel) => ({
                    startRow: sel.range.startRow,
                    startColumn: sel.range.startColumn,
                    endRow: sel.range.endRow,
                    endColumn: sel.range.endColumn,
                })),
            },
            timestamp: Date.now(),
        };

        this._awarenessRemoteService.setLocalAwareness(unitId, awareness);
    }

    private _addClient(clientID: string): void {
        this._userIds$.next(new Set(this._userIds$.value.add(clientID)));
    }

    getClientId(unitId: string): number | undefined {
        return this._clientId$.value.get(unitId);
    }

    getState(unitId: string): Map<number, IAwarenessState> | undefined {
        return this._state$.value.get(unitId);
    }

    getUnitIds(): string[] {
        return Array.from(this._joinedUnits);
    }

    setLocalStateField(unitId: string, field: string, value: unknown): void {
        const currentState = this._localState.get(unitId);
        if (!currentState) {
            return;
        }
        const nextState = {
            ...currentState,
            [field]: value,
        } as IAwarenessState;
        this._localState.set(unitId, nextState);
        this._broadcastAwareness(unitId, nextState);
    }

    private _createClientId(): number {
        return Math.floor(Math.random() * 1_000_000_000_000);
    }

    override dispose(): void {
        super.dispose();
        this._awarenessSubscription?.unsubscribe();
        this._documentSubscriptions.forEach((subs) => {
            subs.forEach((s) => {
                s.unsubscribe();
            });
        });
        this._documentSubscriptions.clear();
    }
}
