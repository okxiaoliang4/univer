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

import { ICommandService, LocaleService } from '@univerjs/core';
import { Button, FormLayout, Input } from '@univerjs/design';
import { SetWorksheetNameCommand } from '@univerjs/sheets';
import { ISidebarService, useDependency } from '@univerjs/ui';
import { useState } from 'react';
import { RENAME_INPUT_DIALOG_ID } from '../../../commands/operations/rename-sheet.operation';

export const RenameInput = (props: { subUnitId: string }) => {
    const { subUnitId } = props;
    const localeService = useDependency(LocaleService);
    const sidebarService = useDependency(ISidebarService);
    const commandService = useDependency(ICommandService);
    const [nameValue, setNameValue] = useState('');

    return (
        <FormLayout className="univer-flex univer-flex-col univer-gap-2">
            <Input autoFocus value={nameValue} onChange={(e) => setNameValue(e)} />
            <div className="univer-flex univer-items-center univer-justify-end univer-gap-1 univer-py-2">
                <Button
                    className={`
                      univer-flex-1
                      md:univer-flex-none
                    `}
                    onClick={() => sidebarService.close(RENAME_INPUT_DIALOG_ID)}
                >
                    {localeService.t('button.cancel')}
                </Button>
                <Button
                    className={`
                      univer-flex-1
                      md:univer-flex-none
                    `}
                    variant="primary"
                    onClick={() => {
                        commandService.executeCommand(SetWorksheetNameCommand.id, {
                            name: nameValue,
                            subUnitId,
                        });
                        sidebarService.close(RENAME_INPUT_DIALOG_ID);
                    }}
                >
                    {localeService.t('button.confirm')}
                </Button>
            </div>
        </FormLayout>
    );
};

RenameInput.componentKey = 'rename-input';
