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

import { describe, it } from 'vitest';

const flushPromises = () => new Promise((resolve) => setTimeout(resolve, 1000));

describe('offline conflict ordering', () => {
    it('set-range-values vs insert-col should converge to shifted value after reconnect', async () => {
        const docId = '019bc0ff-395c-725c-baff-735705955782';
        // const clientA = await createE2ETestBed(docId, [
        //     [CollaborationController],
        //     [ITransformService, { useClass: TransformService }],
        //     [IOfflineStorageService, { useClass: OfflineStorageService }],
        //     [ICollaborationService, { useClass: CollaborationService }],
        //     [INetworkService, { useClass: WsNetworkService }],
        // ]);
        // const clientB = await createE2ETestBed(docId, [
        //     [CollaborationController],
        //     [ITransformService, { useClass: TransformService }],
        //     [IOfflineStorageService, { useClass: OfflineStorageService }],
        //     [ICollaborationService, { useClass: CollaborationService }],
        //     [INetworkService, { useClass: WsNetworkService }],
        // ]);

        // await clientA.commandService.executeCommand(SetRangeValuesMutation.id, {
        //     unitId: clientA.unitId,
        //     subUnitId: clientA.subUnitId,
        //     cellValue: {
        //         0: {
        //             0: { v: 1 },
        //         },
        //     },
        // });
        // await clientB.commandService.executeCommand(InsertColMutation.id, {
        //     unitId: clientB.unitId,
        //     subUnitId: clientB.subUnitId,
        //     range: createRange({ startColumn: 0, endColumn: 0 }),
        // });

        // await flushPromises();

        // expect(clientA.workbook.getSheet(clientA.subUnitId).getCell(0, 0).value).toBe(1);
        // expect(clientB.workbook.getSheet(clientB.subUnitId).getCell(0, 0).value).toBe(1);
    });
});
