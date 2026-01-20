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

import type { ICanvasPopup } from '@univerjs/sheets-ui';

export const CollaborationSelectionPopup = (props: {
    popup: ICanvasPopup & {
        extraProps: {
            name: string;
            color: string;
        };
    };
}) => {
    return (
        <div
            style={{
                background: props.popup.extraProps.color,
                borderRadius: '0.25rem',
                padding: '0 0.25rem',
                color: 'white',
                fontSize: '0.75rem',
                fontWeight: 'bold',
                opacity: 0.8,
            }}
        >
            {props.popup.extraProps.name}
        </div>
    );
};

CollaborationSelectionPopup.componentKey = 'CollaborationSelectionPopup';
