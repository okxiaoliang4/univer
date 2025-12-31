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

import './global.css';

// Controllers
export { MobileKeyboardController } from './controllers/mobile-keyboard.controller';
export { MobileViewportController } from './controllers/mobile-viewport.controller';

// Plugin
export { UniverMobileKeyboardUIPlugin } from './plugin';

// Services
export { IMobileKeyboardService, type KeyboardMode, MobileKeyboardService } from './services/mobile-keyboard.service';

export { KeyboardContainer, KeyboardItem } from './views/keyboard/common/KeyboardContainer';
export { MobileFormulaBar } from './views/keyboard/common/MobileFormulaBar';
export { ModeSwitcher } from './views/keyboard/common/ModeSwitcher';
export { OperationToolbar } from './views/keyboard/common/OperationToolbar';
export { FormulaKeyboard } from './views/keyboard/formula-keyboard/FormulaKeyboard';
export { FunctionBrowser } from './views/keyboard/formula-keyboard/FunctionBrowser';
export { useKeyboardInput } from './views/keyboard/hooks/use-keyboard-input';
export { NumberKeyboard } from './views/keyboard/number-keyboard/NumberKeyboard';
export { TextKeyboard } from './views/keyboard/text-keyboard/TextKeyboard';
