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
import type { Observable } from 'rxjs';
import {
    createIdentifier,
    Disposable,
    Inject,
    Injector,

    UserManagerService,
} from '@univerjs/core';
import { BehaviorSubject, Subject } from 'rxjs';
import { ICollaborationService } from './collaboration.service';
import { ISocketService } from './socket.service';

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
    getClientId(unitId: string): Promise<number | undefined>;
    getState(unitId: string): Promise<Map<number, IAwarenessState> | undefined>;
    getUnitIds(): Promise<string[]>;
    getInitUnitIds(): Promise<string[]>;
    setLocalStateField(
        unitId: string,
        field: string,
        value: unknown,
    ): Promise<void>;
}

export interface IAwarenessState {
    clientID: number;
    id: string;
    name: string;
    selectionParams: ISetSelectionsOperationParams;
  // color: string
  // avatar: string
}

export class AwarenessService extends Disposable implements IAwarenessService {
    private _joinedUnits: Set<string> = new Set();
    private _init$: BehaviorSubject<Map<string, boolean>> = new BehaviorSubject(
        new Map()
    );

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

    init$ = this._init$.asObservable();
    userIds$ = this._userIds$.asObservable();
    clientId$ = this._clientId$.asObservable();
    state$ = this._state$.asObservable();
    add$ = this._add$.asObservable();
    remove$ = this._remove$.asObservable();
    update$ = this._update$.asObservable();

    constructor(
        @Inject(Injector) readonly _injector: Injector,
        @Inject(ICollaborationService)
        private readonly _collaborationService: ICollaborationService,
        @Inject(ISocketService)
        private readonly _socketService: ISocketService,
        @Inject(UserManagerService)
        private readonly _userManagerService: UserManagerService
    ) {
        super();

        this._initWs();
    }

    _initWs() {
        this.disposeWithMe(
            this._collaborationService.docJoined$.subscribe((unitId) => {
                this._joinedUnits.add(unitId);
                this._initAwareness(unitId);
            })
        );

        this.disposeWithMe(
            this._collaborationService.docLeft$.subscribe((unitId) => {
                this._joinedUnits.delete(unitId);
                this._localState.delete(unitId);
                this._init$.value.delete(unitId);
                this._state$.value.delete(unitId);
                this._clientId$.value.delete(unitId);
                this._init$.next(this._init$.value);
                this._state$.next(this._state$.value);
                this._clientId$.next(this._clientId$.value);
            })
        );
    }

    private presenceUpdateListener(payload: IAwarenessState) {
        const targetUnitId = payload.selectionParams?.unitId || unitId;
        if (!this._joinedUnits.has(targetUnitId)) {
            return;
        }
        const stateMap = this._state$.value.get(targetUnitId) ?? new Map();
        stateMap.forEach((existing, existingClientId) => {
            if (existing.id === payload.id && existingClientId !== payload.clientID) {
                stateMap.delete(existingClientId);
                this._remove$.next({ unitId: targetUnitId, clientId: existingClientId });
            }
        });
        stateMap.set(payload.clientID, payload);
        this._state$.value.set(targetUnitId, stateMap);
        this._state$.next(this._state$.value);
        this._update$.next({ unitId: targetUnitId, clientId: payload.clientID });
    }

    private _initAwareness(unitId: string) {
        const clientId = this._clientId$.value.get(unitId) ?? this._createClientId();
        this._clientId$.value.set(unitId, clientId);
        this._clientId$.next(this._clientId$.value);

        const socket = this._socketService.getSocket();
        if (socket) {
            const presenceUpdateListener = this.presenceUpdateListener.bind(this);
            socket.off('presence_update', presenceUpdateListener);
            socket.on('presence_update', presenceUpdateListener);

            socket.emit(
                'awareness_init',
                { docId: unitId },
                (ack: { status: string; states: IAwarenessState[] }) => {
                    if (ack.status !== 'ok') return;
                    const stateMap = new Map<number, IAwarenessState>();
                    ack.states.forEach((state) => {
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
                    });
                    this._state$.value.set(unitId, stateMap);
                    this._state$.next(this._state$.value);
                    this._init$.value.set(unitId, true);
                    this._init$.next(this._init$.value);
                }
            );
        }

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
            this._emitPresenceUpdate(unitId, state);
        };
        const user = this._userManagerService.getCurrentUser();

        setCurrentState(user);
        this.disposeWithMe(
            this._userManagerService.currentUser$.subscribe((currentUser) => {
                setCurrentState(currentUser);
            })
        );
    }

    private _addClient(clientID: string) {
        this._userIds$.next(new Set(this._userIds$.value.add(clientID)));
    }

    private _removeClient(clientID: string) {
        const newSet = new Set(this._userIds$.value);
        newSet.delete(clientID);
        this._userIds$.next(newSet);
    }

    async getClientId(unitId: string) {
        return this._clientId$.value.get(unitId);
    }

    async getState(unitId: string) {
        return this._state$.value.get(unitId);
    }

    async getUnitIds() {
        return Array.from(this._joinedUnits);
    }

    async getInitUnitIds() {
        return (await this.getUnitIds()).filter((id) => this._init$.value.has(id));
    }

    async setLocalStateField(unitId: string, field: string, value: unknown) {
        const currentState = this._localState.get(unitId);
        if (!currentState) {
            return;
        }
        const nextState = {
            ...currentState,
            [field]: value,
        } as IAwarenessState;
        this._localState.set(unitId, nextState);
        this._emitPresenceUpdate(unitId, nextState);
    }

    private _emitPresenceUpdate(unitId: string, state: IAwarenessState) {
        const socket = this._socketService.getSocket();
        if (!socket || socket.disconnected) {
            return;
        }
        socket.emit('presence_update', {
            docId: unitId,
            clientId: state.clientID,
            user: { id: state.id, name: state.name },
            selectionParams: state.selectionParams,
        });
    }

    private _createClientId(): number {
        return Math.floor(Math.random() * 1_000_000_000_000);
    }
}
