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

import type { Direction, IOperation } from '@univerjs/core';
import type { KeyboardMode } from '../../services/keyboard.service';
import { CommandType } from '@univerjs/core';
import { IKeyboardService } from '../../services/keyboard.service';

export interface IKeyboardInsertTextOperationParams {
    text?: string;
}

export const KeyboardInsertTextOperation: IOperation<IKeyboardInsertTextOperationParams> = {
    id: 'keyboard-ui.operation.insert-text',
    type: CommandType.OPERATION,
    handler: (accessor, params) => {
        const { text } = params as IKeyboardInsertTextOperationParams;
        if (!text) {
            return false;
        }
        accessor.get(IKeyboardService).insertText(text);
        return true;
    },
};

export interface IKeyboardInsertFunctionOperationParams {
    funcName: string;
}
export const KeyboardInsertFunctionOperation: IOperation<IKeyboardInsertFunctionOperationParams> = {
    id: 'keyboard-ui.operation.insert-function',
    type: CommandType.OPERATION,
    handler: (accessor, params) => {
        const { funcName } = params as IKeyboardInsertFunctionOperationParams;
        if (!funcName) {
            return false;
        }
        accessor.get(IKeyboardService).insertFunction(funcName);
        return true;
    },
};

export interface IKeyboardInsertQuotesOperationParams {
    quotes: string;
}
export const KeyboardInsertQuotesOperation: IOperation<IKeyboardInsertQuotesOperationParams> = {
    id: 'keyboard-ui.operation.insert-quotes',
    type: CommandType.OPERATION,
    handler: (accessor, params) => {
        const { quotes } = params as IKeyboardInsertQuotesOperationParams;
        if (!quotes) {
            return false;
        }
        accessor.get(IKeyboardService).insertQuotes(quotes);
        return true;
    },
};

export const KeyboardDeleteBackwardOperation: IOperation = {
    id: 'keyboard-ui.operation.delete-backward',
    type: CommandType.OPERATION,
    handler: (accessor) => {
        accessor.get(IKeyboardService).deleteBackward();
        return true;
    },
};

export interface IKeyboardConfirmAndMoveOperationParams {
    direction: Direction.DOWN | Direction.RIGHT;
}
export const KeyboardConfirmAndMoveOperation: IOperation<IKeyboardConfirmAndMoveOperationParams> = {
    id: 'keyboard-ui.operation.confirm-and-move',
    type: CommandType.OPERATION,
    handler: (accessor, params) => {
        const { direction } = params as IKeyboardConfirmAndMoveOperationParams;
        accessor.get(IKeyboardService).confirmAndMove(direction);
        return true;
    },
};

export interface IKeyboardSetModeOperationParams {
    mode: KeyboardMode;
}
export const KeyboardSetModeOperation: IOperation<IKeyboardSetModeOperationParams> = {
    id: 'keyboard-ui.operation.set-mode',
    type: CommandType.OPERATION,
    handler: (accessor, params) => {
        const { mode } = params as IKeyboardSetModeOperationParams;
        accessor.get(IKeyboardService).setKeyboardMode(mode);
        return true;
    },
};

export interface IToggleKeyboardOperationParams {
    visible?: boolean;
}
export const ToggleKeyboardOperation: IOperation<IToggleKeyboardOperationParams> = {
    id: 'keyboard-ui.operation.toggle-keyboard',
    type: CommandType.OPERATION,
    handler: (accessor, params = {}) => {
        const { visible } = params as IToggleKeyboardOperationParams;
        accessor.get(IKeyboardService).toggleKeyboard(visible);
        return true;
    },
};
