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

import type {
    CommandListener,
    ICommand,
    ICommandInfo,
    ICommandService,
    ICreateUnitOptions,
    IDisposable,
    IExecutionOptions,
    ILogService,
    IMutationInfo,
    UnitModel,
    UnitType,
} from '@univerjs/core';
import type { Socket } from 'socket.io-client';
import type { IMutationWithOpId } from '../collaboration.types';
import type { IPendingMutations, IPendingMutationSerivce } from '../offline-storage.service';
import type { IChangesetAck, IChangesetPushed, IFetchOpsAck, IJoinDocAck, ISocketService } from '../socket.service';
import { CommandType, UniverInstanceType } from '@univerjs/core';
import { InsertColMutation, InsertRowMutation, RemoveColMutation, RemoveRowMutation, SetRangeValuesMutation } from '@univerjs/sheets';
import { BehaviorSubject, Subject } from 'rxjs';

export const TEST_UNIT_ID = 'unit-1';
export const TEST_SUB_UNIT_ID = 'sheet-1';

export interface IMockSocket {
    id: string;
    disconnected: boolean;
    disconnect: () => void;
}

export class MockSocketService implements ISocketService {
    private _socket: IMockSocket | null = null;
    private _connected$ = new Subject<void>();
    private _disconnected$ = new Subject<void>();
    private _changesetPushed$ = new Subject<IChangesetPushed>();
    connected$ = this._connected$.asObservable();
    connected = false;
    disconnected$ = this._disconnected$.asObservable();
    changesetPushed$ = this._changesetPushed$.asObservable();

    createdUrl: string | null = null;
    emitted: Array<{ event: string; args: unknown[] }> = [];
    nextJoinAck: IJoinDocAck = { status: 'ok', version: 1 };
    nextChangesetAck: IChangesetAck = { status: 'ok', serverRev: 1 };
    nextFetchOpsAck: IFetchOpsAck = { status: 'ok', operations: [] };

    createSocket(url: string): Socket | null {
        this.createdUrl = url;
        this._socket = {
            id: 'socket-1',
            disconnected: false,
            disconnect: () => {
                if (this._socket) {
                    this._socket.disconnected = true;
                }
            },
        };
        return this._socket as unknown as Socket;
    }

    getSocket(): Socket | null {
        return this._socket as unknown as Socket;
    }

    setSocketState(connected: boolean): void {
        this.connected = connected;
        if (!this._socket) {
            this._socket = {
                id: 'socket-1',
                disconnected: !connected,
                disconnect: () => {
                    if (this._socket) {
                        this._socket.disconnected = true;
                    }
                },
            };
        } else {
            this._socket.disconnected = !connected;
        }
    }

    emitConnected(): void {
        this.connected = true;
        this._connected$.next();
    }

    emitDisconnected(): void {
        this.connected = false;
        this._disconnected$.next();
    }

    emitChangesetPushed(payload: IChangesetPushed): void {
        this._changesetPushed$.next(payload);
    }

    emit(event: string, ...args: unknown[]): void {
        this.emitted.push({ event, args });
        const maybeAck = args[args.length - 1] as ((payload: unknown) => void) | undefined;
        if (typeof maybeAck !== 'function') {
            return;
        }

        if (event === 'join_doc') {
            maybeAck(this.nextJoinAck);
            return;
        }

        if (event === 'changeset') {
            maybeAck(this.nextChangesetAck);
            return;
        }

        if (event === 'fetch_ops') {
            maybeAck(this.nextFetchOpsAck);
        }
    }
}

export class MockOfflineStorageService implements IPendingMutationSerivce {
    private _store = new Map<string, IPendingMutations>();
    private _ready$ = new BehaviorSubject<boolean>(true);
    clearCalls: string[] = [];
    saveCalls: string[] = [];
    removeCalls: Array<{ unitId: string; opIds: string[] }> = [];

    readonly ready$ = this._ready$.asObservable();

    isReady(): boolean {
        return this._ready$.value;
    }

    has(unitId: string): boolean {
        return this._store.has(unitId);
    }

    get(unitId: string): IMutationWithOpId[] | null {
        return this._store.get(unitId)?.mutations ?? null;
    }

    async add(unitId: string, mutations: IMutationWithOpId[], baseRev: number): Promise<IPendingMutations> {
        const existing = this._store.get(unitId);
        const accumulated = existing ? [...existing.mutations, ...mutations] : mutations;
        const next = {
            unitId,
            mutations: accumulated,
            baseRev: existing?.baseRev ?? baseRev,
            userId: existing?.userId ?? 'test-user',
        };
        this._store.set(unitId, next);
        this.saveCalls.push(unitId);
        return next;
    }

    async update(unitId: string, mutations: IMutationWithOpId[], baseRev: number): Promise<IPendingMutations> {
        const existing = this._store.get(unitId);
        const next = {
            unitId,
            mutations,
            baseRev,
            userId: existing?.userId ?? 'test-user',
        };
        this._store.set(unitId, next);
        this.saveCalls.push(unitId);
        return next;
    }

    async getBaseRev(unitId: string): Promise<number> {
        return this._store.get(unitId)?.baseRev ?? 0;
    }

    async removeByOpIds(unitId: string, opIds: string[]): Promise<IPendingMutations> {
        const current = this._store.get(unitId);
        const toRemove = new Set(opIds);
        const remaining = current?.mutations.filter((mutation) => !toRemove.has(mutation.opId)) ?? [];
        const next = {
            unitId,
            mutations: remaining,
            baseRev: current?.baseRev ?? 0,
            userId: current?.userId ?? 'test-user',
        };
        this._store.set(unitId, next);
        this.removeCalls.push({ unitId, opIds });
        return next;
    }

    async clear(unitId: string): Promise<void> {
        this._store.delete(unitId);
        this.clearCalls.push(unitId);
    }

    async loadPendingMutations(unitId: string): Promise<IPendingMutations | null> {
        return this._store.get(unitId) ?? null;
    }

    async loadAllPendingMutations(): Promise<IPendingMutations[]> {
        return [...this._store.values()];
    }

    async savePendingMutations(unitId: string, mutations: IMutationWithOpId[], baseRev: number, userId: string): Promise<void> {
        const existing = this._store.get(unitId);
        const accumulated = existing ? [...existing.mutations, ...mutations] : mutations;
        this._store.set(unitId, {
            unitId,
            mutations: accumulated,
            baseRev: existing?.baseRev ?? baseRev,
            userId,
        });
        this.saveCalls.push(unitId);
    }

    async clearPendingMutations(unitId: string): Promise<void> {
        await this.clear(unitId);
    }

    async clearAllPendingMutations(): Promise<void> {
        this._store.clear();
    }
}

export class MockLogService implements ILogService {
    debug(): void {
        // no-op
    }

    log(): void {
        // no-op
    }

    warn(): void {
        // no-op
    }

    error(): void {
        // no-op
    }

    deprecate(): void {
        // no-op
    }

    setLogLevel(): void {
        // no-op
    }
}

export class MockConfigService {
    private _config = new Map<string | symbol, unknown>();
    private _configChanged$ = new Subject<{ [key: string]: unknown }>();
    readonly configChanged$ = this._configChanged$.asObservable();

    constructor(initial?: Record<string, unknown>) {
        if (initial) {
            Object.entries(initial).forEach(([key, value]) => {
                this._config.set(key, value);
            });
        }
    }

    getConfig<T>(id: string | symbol): T | null | undefined {
        return this._config.get(id) as T;
    }

    setConfig(id: string | symbol, value: unknown): void {
        this._config.set(id, value);
        this._configChanged$.next({ [id.toString()]: value });
    }

    deleteConfig(id: string | symbol): boolean {
        return this._config.delete(id);
    }

    subscribeConfigValue$<T = unknown>(_key: string) {
        return new Subject<T>().asObservable();
    }
}

export class MockContextService {
    private _contextChanged$ = new Subject<{ [key: string]: boolean }>();
    readonly contextChanged$ = this._contextChanged$.asObservable();
    private _contextMap = new Map<string, boolean>();

    getContextValue(key: string): boolean {
        return this._contextMap.get(key) ?? false;
    }

    setContextValue(key: string, value: boolean): void {
        this._contextMap.set(key, value);
        this._contextChanged$.next({ [key]: value });
    }

    subscribeContextValue$(_key: string) {
        return new Subject<boolean>().asObservable();
    }
}

export class MockCommandService implements ICommandService {
    private _collabListeners: CommandListener[] = [];
    private _beforeListeners: CommandListener[] = [];
    private _afterListeners: CommandListener[] = [];
    private _registered = new Set<string>();

    executed: Array<{ id: string; params?: object; options?: IExecutionOptions }> = [];

    emitMutationExecutedForCollab(commandInfo: Readonly<ICommandInfo>, options?: IExecutionOptions): void {
        this._collabListeners.forEach((listener) => {
            listener(commandInfo, options);
        });
    }

    hasCommand(commandId: string): boolean {
        return this._registered.has(commandId);
    }

    registerCommand(command: ICommand<object, unknown>): IDisposable {
        this._registered.add(command.id);
        return { dispose: () => this._registered.delete(command.id) };
    }

    unregisterCommand(commandId: string): void {
        this._registered.delete(commandId);
    }

    registerMultipleCommand(command: ICommand<object, unknown>): IDisposable {
        return this.registerCommand(command);
    }

    async executeCommand<P extends object = object, R = boolean>(
        id: string,
        params?: P,
        options?: IExecutionOptions
    ): Promise<R> {
        return this.syncExecuteCommand(id, params, options);
    }

    syncExecuteCommand<P extends object = object, R = boolean>(
        id: string,
        params?: P,
        options?: IExecutionOptions
    ): R {
        this.executed.push({ id, params: params as object, options });
        return true as R;
    }

    onCommandExecuted(listener: CommandListener): IDisposable {
        this._afterListeners.push(listener);
        return {
            dispose: () => {
                const index = this._afterListeners.indexOf(listener);
                if (index >= 0) {
                    this._afterListeners.splice(index, 1);
                }
            },
        };
    }

    beforeCommandExecuted(listener: CommandListener): IDisposable {
        this._beforeListeners.push(listener);
        return {
            dispose: () => {
                const index = this._beforeListeners.indexOf(listener);
                if (index >= 0) {
                    this._beforeListeners.splice(index, 1);
                }
            },
        };
    }

    onMutationExecutedForCollab(listener: CommandListener): IDisposable {
        this._collabListeners.push(listener);
        return {
            dispose: () => {
                const index = this._collabListeners.indexOf(listener);
                if (index >= 0) {
                    this._collabListeners.splice(index, 1);
                }
            },
        };
    }
}

export class MockUnit {
    type: UnitType;
    private _unitId: string;
    private _rev: number;

    constructor(unitId: string, type: UnitType = UniverInstanceType.UNIVER_SHEET, rev = 0) {
        this._unitId = unitId;
        this.type = type;
        this._rev = rev;
    }

    getUnitId(): string {
        return this._unitId;
    }

    getRev(): number {
        return this._rev;
    }

    setRev(rev: number): void {
        this._rev = rev;
    }
}

export class MockUniverInstanceService {
    private _units = new Map<string, MockUnit>();
    private _unitsByType = new Map<UnitType, MockUnit[]>();
    private _focusedUnitId: string | null = null;
    private _unitAdded$ = new Subject<UnitModel>();
    private _unitDisposed$ = new Subject<UnitModel>();
    private _focused$ = new BehaviorSubject<string | null>(null);

    unitAdded$ = this._unitAdded$.asObservable();
    unitDisposed$ = this._unitDisposed$.asObservable();
    focused$ = this._focused$.asObservable();

    getTypeOfUnitAdded$(_type: UnitType) {
        return this.unitAdded$;
    }

    getTypeOfUnitDisposed$(_type: UnitType) {
        return this.unitDisposed$;
    }

    __addUnit(unit: UnitModel): void {
        const mockUnit = unit as unknown as MockUnit;
        this._units.set(mockUnit.getUnitId(), mockUnit);
        const list = this._unitsByType.get(mockUnit.type) ?? [];
        list.push(mockUnit);
        this._unitsByType.set(mockUnit.type, list);
        this._unitAdded$.next(unit);
    }

    addUnit(unit: MockUnit): void {
        this.__addUnit(unit as unknown as UnitModel);
    }

    focusUnit(unitId: string | null): void {
        this._focusedUnitId = unitId;
        this._focused$.next(unitId);
    }

    getFocusedUnit(): UnitModel | null {
        if (!this._focusedUnitId) {
            return null;
        }
        return (this._units.get(this._focusedUnitId) as unknown as UnitModel) ?? null;
    }

    getCurrentUnitForType(_type: UnitType): UnitModel | null {
        return null;
    }

    getCurrentUnitOfType(_type: UnitType): UnitModel | null {
        return null;
    }

    setCurrentUnitForType(_unitId: string): void {
        // no-op
    }

    getCurrentTypeOfUnit$(_type: UnitType) {
        return new BehaviorSubject<UnitModel | null>(null).asObservable();
    }

    createUnit<T, U extends UnitModel>(_type: UnitType, _data: Partial<T>, _options?: ICreateUnitOptions): U {
        throw new Error('Not implemented in mock');
    }

    disposeUnit(_unitId: string): boolean {
        return false;
    }

    registerCtorForType<T extends UnitModel>(_type: UnitType, _ctor: new (...args: unknown[]) => T): IDisposable {
        return { dispose: () => {} };
    }

    changeDoc(_unitId: string): void {
        // no-op
    }

    getUnit(id: string): UnitModel | null {
        return (this._units.get(id) as unknown as UnitModel) ?? null;
    }

    getAllUnitsForType(type: UnitType): UnitModel[] {
        return (this._unitsByType.get(type) as unknown as UnitModel[]) ?? [];
    }

    getUnitType(_unitId: string): UnitType {
        return UniverInstanceType.UNIVER_SHEET;
    }

    getUniverSheetInstance(_unitId: string): UnitModel | null {
        return null;
    }

    getUniverDocInstance(_unitId: string): UnitModel | null {
        return null;
    }

    getCurrentUniverDocInstance(): UnitModel | null {
        return null;
    }
}

export const createRange = (overrides?: Partial<{ startRow: number; endRow: number; startColumn: number; endColumn: number }>) => ({
    startRow: 0,
    endRow: 0,
    startColumn: 0,
    endColumn: 0,
    ...overrides,
});

export const createSetRangeValuesMutation = (
    overrides?: Partial<IMutationInfo>
): IMutationInfo => ({
    id: SetRangeValuesMutation.id,
    type: CommandType.MUTATION,
    params: {
        unitId: TEST_UNIT_ID,
        subUnitId: TEST_SUB_UNIT_ID,
        cellValue: {
            0: {
                0: { v: '1' },
            },
        },
    },
    ...overrides,
});

export const createInsertRowMutation = (
    overrides?: Partial<IMutationInfo>
): IMutationInfo => ({
    id: InsertRowMutation.id,
    type: CommandType.MUTATION,
    params: {
        unitId: TEST_UNIT_ID,
        subUnitId: TEST_SUB_UNIT_ID,
        range: createRange(),
    },
    ...overrides,
});

export const createInsertColMutation = (
    overrides?: Partial<IMutationInfo>
): IMutationInfo => ({
    id: InsertColMutation.id,
    type: CommandType.MUTATION,
    params: {
        unitId: TEST_UNIT_ID,
        subUnitId: TEST_SUB_UNIT_ID,
        range: createRange(),
    },
    ...overrides,
});

export const createRemoveRowMutation = (
    overrides?: Partial<IMutationInfo>
): IMutationInfo => ({
    id: RemoveRowMutation.id,
    type: CommandType.MUTATION,
    params: {
        unitId: TEST_UNIT_ID,
        subUnitId: TEST_SUB_UNIT_ID,
        range: createRange(),
    },
    ...overrides,
});

export const createRemoveColMutation = (
    overrides?: Partial<IMutationInfo>
): IMutationInfo => ({
    id: RemoveColMutation.id,
    type: CommandType.MUTATION,
    params: {
        unitId: TEST_UNIT_ID,
        subUnitId: TEST_SUB_UNIT_ID,
        range: createRange(),
    },
    ...overrides,
});
