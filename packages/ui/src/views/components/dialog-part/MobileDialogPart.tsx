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

import type { ReactNode } from 'react';
import type { IDialogPartMethodOptions } from './interface';
import { Drawer, DrawerContent, DrawerFooter, DrawerHeader, DrawerTitle } from '@univerjs/design';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { CustomLabel } from '../../../components/custom-label/CustomLabel';
import { IDialogService } from '../../../services/dialog/dialog.service';
import { useDependency } from '../../../utils/di';

export function MobileDialogPart() {
    const dialogService = useDependency(IDialogService);

    const [dialogOptions, setDialogOptions] = useState<IDialogPartMethodOptions[]>([]);

    useEffect(() => {
        const dialog$ = dialogService.getDialogs$();
        const subscription = dialog$.subscribe((options: IDialogPartMethodOptions[]) => {
            setDialogOptions(options);
        });

        return () => subscription.unsubscribe();
    }, [dialogService]);

    const handleOpenChange = useCallback((id: string, open: boolean) => {
        if (!open) {
            dialogService.close(id);
        }
    }, [dialogService]);

    const processedDialogs = useMemo(() => dialogOptions.map((options) => {
        const { children, title, footer, ...restProps } = options;

        const processed: {
            id: string;
            open: boolean;
            modal: boolean;
            maskClosable?: boolean;
            keyboard?: boolean;
            width?: number | string;
            title?: ReactNode;
            children?: ReactNode;
            footer?: ReactNode;
            onClose?: () => void;
        } = {
            id: options.id,
            open: options.open ?? false,
            modal: options.mask !== false,
            maskClosable: options.maskClosable,
            keyboard: options.keyboard,
            width: options.width,
            onClose: options.onClose,
        };

        // Process children, title, footer from ICustomLabelProps to ReactNode
        for (const key of ['children', 'title', 'footer']) {
            const k = key as keyof IDialogPartMethodOptions;
            const props = options[k] as any;

            if (props) {
                (processed as any)[k] = <CustomLabel {...props} />;
            }
        }

        return processed;
    }), [dialogOptions]);

    return (
        <>
            {processedDialogs.map((options) => (
                <Drawer
                    key={options.id}
                    open={options.open}
                    onOpenChange={(open) => handleOpenChange(options.id, open)}
                    direction="bottom"
                    modal={options.modal}
                >
                    <DrawerContent
                        style={{
                            width: options.width ? (typeof options.width === 'number' ? `${options.width}px` : options.width) : undefined,
                        }}
                        onEscapeKeyDown={(e) => {
                            if (options.keyboard === false) {
                                e.preventDefault();
                                return;
                            }
                            handleOpenChange(options.id, false);
                            options.onClose?.();
                        }}
                        onPointerDownOutside={(e) => {
                            if (options.maskClosable === false) {
                                e.preventDefault();
                                return;
                            }
                            handleOpenChange(options.id, false);
                            options.onClose?.();
                        }}
                    >
                        {options.title && (
                            <DrawerHeader>
                                <DrawerTitle>
                                    {options.title}
                                </DrawerTitle>
                            </DrawerHeader>
                        )}

                        {options.children}

                        {options.footer && (
                            <DrawerFooter>
                                {options.footer}
                            </DrawerFooter>
                        )}
                    </DrawerContent>
                </Drawer>
            ))}
        </>
    );
}
