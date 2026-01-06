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

import type { Plugin, PluginCtor } from '@univerjs/core';
import { UniverDocsDrawingMobileUIPlugin } from '@univerjs/docs-drawing-ui';
import { UniverDocsMentionUIPlugin } from '@univerjs/docs-mention-ui';
import { UniverMobileKeyboardUIPlugin } from '@univerjs/mobile-keyboard-ui';
import { UniverSheetsConditionalFormattingMobileUIPlugin } from '@univerjs/sheets-conditional-formatting-ui';
import { UniverSheetsDataValidationMobileUIPlugin } from '@univerjs/sheets-data-validation-ui';
import { UniverSheetsDrawingUIPlugin } from '@univerjs/sheets-drawing-ui';
import { UniverSheetsFilterMobileUIPlugin } from '@univerjs/sheets-filter-ui';
import { UniverSheetsFormulaUIPlugin } from '@univerjs/sheets-formula-ui';
import { UniverSheetsNoteUIPlugin } from '@univerjs/sheets-note-ui';
import { UniverSheetsNumfmtMobileUIPlugin } from '@univerjs/sheets-numfmt-ui';
import { UniverSheetsTableUIPlugin } from '@univerjs/sheets-table-ui';
import { UniverSheetsThreadCommentMobileUIPlugin } from '@univerjs/sheets-thread-comment-ui';
import { UniverThreadCommentUIPlugin } from '@univerjs/thread-comment-ui';

export default function getLazyPlugins(): Array<[PluginCtor<Plugin>] | [PluginCtor<Plugin>, unknown]> {
    return [
        [UniverDocsDrawingMobileUIPlugin],
        [UniverDocsMentionUIPlugin],
        [UniverSheetsNumfmtMobileUIPlugin],
        [UniverThreadCommentUIPlugin],
        [UniverSheetsThreadCommentMobileUIPlugin],
        [UniverSheetsNoteUIPlugin],
        [UniverSheetsTableUIPlugin],
        [UniverSheetsFormulaUIPlugin],
        [UniverSheetsDataValidationMobileUIPlugin],
        [UniverSheetsConditionalFormattingMobileUIPlugin],
        [UniverSheetsFilterMobileUIPlugin, { useRemoteFilterValuesGenerator: false }],
        [UniverSheetsDrawingUIPlugin],
        [UniverMobileKeyboardUIPlugin],
    ];
}
