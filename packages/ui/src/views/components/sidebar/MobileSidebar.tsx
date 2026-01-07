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

import type { CSSProperties, ReactNode } from 'react';
import type { ICustomLabelProps } from '../../../components/custom-label/CustomLabel';
import { Drawer, DrawerContent, DrawerFooter, DrawerHeader, DrawerTitle, scrollbarClassName } from '@univerjs/design';
import { useEffect, useMemo, useRef } from 'react';
import { CustomLabel } from '../../../components/custom-label/CustomLabel';
import { ISidebarService } from '../../../services/sidebar/sidebar.service';
import { useDependency, useObservable } from '../../../utils/di';

export interface ISidebarMethodOptions {
    id?: string;
    header?: ICustomLabelProps;
    children?: ICustomLabelProps;
    bodyStyle?: CSSProperties;
    footer?: ICustomLabelProps;

    visible?: boolean;

    width?: number | string;

    onClose?: () => void;
    onOpen?: () => void;
}

export function MobileSidebar() {
    const sidebarService = useDependency(ISidebarService);
    const sidebarOptions = useObservable<ISidebarMethodOptions>(sidebarService.sidebarOptions$);
    const scrollRef = useRef<HTMLDivElement>(null);

    const options = useMemo(() => {
        if (!sidebarOptions) {
            return null;
        }

        const copy = { ...sidebarOptions } as Omit<ISidebarMethodOptions, 'children'> & {
            children?: ReactNode;
            header?: ReactNode;
            footer?: ReactNode;
        };

        for (const key of ['children', 'header', 'footer']) {
            const k = key as keyof ISidebarMethodOptions;

            if (sidebarOptions[k]) {
                const { key, ...props } = sidebarOptions[k] as any;

                if (props) {
                    (copy as any)[k] = <CustomLabel key={key} {...props} />;
                }
            }
        }

        return copy;
    }, [sidebarOptions]);

    useEffect(() => {
        if (scrollRef.current) {
            sidebarService.setContainer(scrollRef.current);
        }
        return () => {
            sidebarService.setContainer(undefined);
        };
    }, [sidebarService]);

    useEffect(() => {
        const handleScroll = (e: Event) => {
            sidebarService.scrollEvent$.next(e);
        };
        const scrollElement = scrollRef.current;
        if (scrollElement) {
            scrollElement.addEventListener('scroll', handleScroll);
        }

        return () => {
            scrollElement?.removeEventListener('scroll', handleScroll);
        };
    }, [sidebarService]);

    function handleOpenChange(open: boolean) {
        const updatedOptions = {
            ...sidebarOptions,
            visible: open,
        };

        sidebarService.options.visible = open;
        sidebarService.sidebarOptions$.next(updatedOptions);

        if (open) {
            updatedOptions?.onOpen?.();
        } else {
            updatedOptions?.onClose?.();
        }
    }

    if (!options) {
        return null;
    }

    return (
        <Drawer
            open={options.visible}
            onOpenChange={handleOpenChange}
            autoFocus={false}
        >
            <DrawerContent
                className={scrollbarClassName}
            >
                <div
                    ref={scrollRef}
                    className={`
                      univer-box-border univer-grid univer-h-full univer-min-h-full univer-grid-rows-[auto_1fr_auto]
                      univer-overflow-y-auto
                    `}
                >
                    <DrawerHeader>
                        {options.header
                            ? (
                                <DrawerTitle>
                                    {options.header}
                                </DrawerTitle>
                            )
                            : null}
                    </DrawerHeader>

                    <section
                        className="univer-box-border univer-flex-1 univer-overflow-y-auto univer-px-4"
                        style={options.bodyStyle}
                    >
                        {options.children}
                    </section>

                    {options.footer && (
                        <DrawerFooter className="univer-sticky univer-bottom-0 univer-p-4">
                            {options.footer}
                        </DrawerFooter>
                    )}
                </div>
            </DrawerContent>
        </Drawer>
    );
}
