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

import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { SheetsPivotTablePanelService } from '../pivot-table-panel.service';

describe('SheetsPivotTablePanelService', () => {
    let service: SheetsPivotTablePanelService;

    beforeEach(() => {
        service = new SheetsPivotTablePanelService();
    });

    afterEach(() => {
        service.dispose();
    });

    describe('openCreationDialog', () => {
        it('should set creationDialogOpen to true', () => {
            service.openCreationDialog();

            const state = service.panelState$.value;
            expect(state.creationDialogOpen).toBe(true);
        });
    });

    describe('closeCreationDialog', () => {
        it('should set creationDialogOpen to false', () => {
            service.openCreationDialog();
            service.closeCreationDialog();

            const state = service.panelState$.value;
            expect(state.creationDialogOpen).toBe(false);
        });
    });

    describe('openPanel', () => {
        it('should set panelOpen to true and set active pivot table', () => {
            const unitId = 'unit1';
            const subUnitId = 'subunit1';
            const pivotTableId = 'pivot1';

            service.openPanel(unitId, subUnitId, pivotTableId);

            const state = service.panelState$.value;
            expect(state.panelOpen).toBe(true);
            expect(state.activePivotTable).toEqual({
                unitId,
                subUnitId,
                pivotTableId,
            });
        });
    });

    describe('closePanel', () => {
        it('should set panelOpen to false and clear active pivot table', () => {
            service.openPanel('unit1', 'subunit1', 'pivot1');
            service.closePanel();

            const state = service.panelState$.value;
            expect(state.panelOpen).toBe(false);
            expect(state.activePivotTable).toBeUndefined();
        });
    });

    describe('isCreationDialogOpen', () => {
        it('should return false initially', () => {
            expect(service.isCreationDialogOpen()).toBe(false);
        });

        it('should return true after opening creation dialog', () => {
            service.openCreationDialog();
            expect(service.isCreationDialogOpen()).toBe(true);
        });

        it('should return false after closing creation dialog', () => {
            service.openCreationDialog();
            service.closeCreationDialog();
            expect(service.isCreationDialogOpen()).toBe(false);
        });
    });

    describe('isPanelOpen', () => {
        it('should return false initially', () => {
            expect(service.isPanelOpen()).toBe(false);
        });

        it('should return true after opening panel', () => {
            service.openPanel('unit1', 'subunit1', 'pivot1');
            expect(service.isPanelOpen()).toBe(true);
        });

        it('should return false after closing panel', () => {
            service.openPanel('unit1', 'subunit1', 'pivot1');
            service.closePanel();
            expect(service.isPanelOpen()).toBe(false);
        });
    });

    describe('getActivePivotTable', () => {
        it('should return undefined initially', () => {
            expect(service.getActivePivotTable()).toBeUndefined();
        });

        it('should return active pivot table info after opening panel', () => {
            const unitId = 'unit1';
            const subUnitId = 'subunit1';
            const pivotTableId = 'pivot1';

            service.openPanel(unitId, subUnitId, pivotTableId);

            const activePivotTable = service.getActivePivotTable();
            expect(activePivotTable).toEqual({
                unitId,
                subUnitId,
                pivotTableId,
            });
        });

        it('should return undefined after closing panel', () => {
            service.openPanel('unit1', 'subunit1', 'pivot1');
            service.closePanel();

            expect(service.getActivePivotTable()).toBeUndefined();
        });
    });

    describe('panelState$ observable', () => {
        it('should emit state changes', async () => {
            const states: any[] = [];

            const subscription = service.panelState$.subscribe((state) => {
                states.push(state);
            });

            service.openCreationDialog();

            // Wait for state to propagate
            await new Promise((resolve) => setTimeout(resolve, 10));

            expect(states.length).toBeGreaterThanOrEqual(2);
            expect(states[0].creationDialogOpen).toBe(false);
            expect(states[0].panelOpen).toBe(false);
            expect(states[1].creationDialogOpen).toBe(true);

            subscription.unsubscribe();
        });

        it('should emit when panel is opened with pivot table info', async () => {
            const states: any[] = [];

            const subscription = service.panelState$.subscribe((state) => {
                states.push(state);
            });

            service.openPanel('unit1', 'subunit1', 'pivot1');

            // Wait for state to propagate
            await new Promise((resolve) => setTimeout(resolve, 10));

            expect(states.length).toBeGreaterThanOrEqual(2);
            const lastState = states[states.length - 1];
            expect(lastState.panelOpen).toBe(true);
            expect(lastState.activePivotTable).toEqual({
                unitId: 'unit1',
                subUnitId: 'subunit1',
                pivotTableId: 'pivot1',
            });

            subscription.unsubscribe();
        });
    });
});
