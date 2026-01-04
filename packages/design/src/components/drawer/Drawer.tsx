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

import * as React from 'react';
import { Drawer as DrawerPrimitive } from 'vaul';

import { clsx } from '../../helper/clsx';

export type IDrawerProps = React.ComponentProps<typeof DrawerPrimitive.Root>;

function Drawer({
    ...props
}: IDrawerProps) {
    return <DrawerPrimitive.Root data-slot="drawer" {...props} />;
}

export type IDrawerTriggerProps = React.ComponentProps<typeof DrawerPrimitive.Trigger>;
function DrawerTrigger({
    ...props
}: IDrawerTriggerProps) {
    return <DrawerPrimitive.Trigger data-slot="drawer-trigger" {...props} />;
}

export type IDrawerPortalProps = React.ComponentProps<typeof DrawerPrimitive.Portal>;
function DrawerPortal({
    ...props
}: IDrawerPortalProps) {
    return <DrawerPrimitive.Portal data-slot="drawer-portal" {...props} />;
}

export type IDrawerCloseProps = React.ComponentProps<typeof DrawerPrimitive.Close>;
function DrawerClose({
    ...props
}: IDrawerCloseProps) {
    return <DrawerPrimitive.Close data-slot="drawer-close" {...props} />;
}

export type IDrawerOverlayProps = React.ComponentProps<typeof DrawerPrimitive.Overlay>;
function DrawerOverlay({
    className,
    ...props
}: IDrawerOverlayProps) {
    return (
        <DrawerPrimitive.Overlay
            data-slot="drawer-overlay"
            className={clsx(
                `
                  data-[state=open]:univer-animate-in data-[state=open]:univer-fade-in-0
                  data-[state=closed]:univer-animate-out data-[state=closed]:univer-fade-out-0
                  univer-fixed univer-inset-0 univer-z-50 univer-bg-[rgba(0,0,0,0.8)]
                `,
                className
            )}
            {...props}
        />
    );
}

export type IDrawerContentProps = React.ComponentProps<typeof DrawerPrimitive.Content> & {
};
function DrawerContent({
    className,
    children,
    ...props
}: IDrawerContentProps) {
    return (
        <DrawerPortal data-slot="drawer-portal">
            <DrawerOverlay />
            <DrawerPrimitive.Content
                data-slot="drawer-content"
                className={clsx(
                    `
                      univer-group/drawer-content univer-fixed univer-z-50 univer-flex univer-h-auto univer-flex-col
                      univer-bg-white
                    `,
                    `
                      data-[vaul-drawer-direction=top]:univer-inset-x-0 data-[vaul-drawer-direction=top]:univer-top-0
                      data-[vaul-drawer-direction=top]:univer-mb-24 data-[vaul-drawer-direction=top]:univer-max-h-[80vh]
                      data-[vaul-drawer-direction=top]:univer-rounded-b-lg
                      data-[vaul-drawer-direction=top]:univer-border-b
                    `,
                    `
                      data-[vaul-drawer-direction=bottom]:univer-inset-x-0
                      data-[vaul-drawer-direction=bottom]:univer-bottom-0
                      data-[vaul-drawer-direction=bottom]:univer-mt-24
                      data-[vaul-drawer-direction=bottom]:univer-max-h-[80vh]
                      data-[vaul-drawer-direction=bottom]:univer-rounded-t-lg
                      data-[vaul-drawer-direction=bottom]:univer-border-t
                    `,
                    `
                      data-[vaul-drawer-direction=right]:univer-inset-y-0
                      data-[vaul-drawer-direction=right]:univer-right-0 data-[vaul-drawer-direction=right]:univer-w-3/4
                      data-[vaul-drawer-direction=right]:univer-border-l
                      data-[vaul-drawer-direction=right]:sm:univer-max-w-sm
                    `,
                    `
                      data-[vaul-drawer-direction=left]:univer-inset-y-0 data-[vaul-drawer-direction=left]:univer-left-0
                      data-[vaul-drawer-direction=left]:univer-w-3/4 data-[vaul-drawer-direction=left]:univer-border-r
                      data-[vaul-drawer-direction=left]:sm:univer-max-w-sm
                    `,
                    className
                )}
                {...props}
            >
                <div
                    className={`
                      univer-group-data-[vaul-drawer-direction=bottom]/drawer-content:block
                      univer-mx-auto univer-mt-4 univer-h-2 univer-w-[100px] univer-shrink-0 univer-rounded-full
                      univer-bg-gray-200
                    `}
                />
                {children}
            </DrawerPrimitive.Content>
        </DrawerPortal>
    );
}

export type IDrawerHeaderProps = React.ComponentProps<'div'>;
function DrawerHeader({ className, ...props }: IDrawerHeaderProps) {
    return (
        <div
            data-slot="drawer-header"
            className={clsx(
                `
                  univer-group-data-[vaul-drawer-direction=bottom]/drawer-content:text-center
                  univer-group-data-[vaul-drawer-direction=top]/drawer-content:text-center
                  univer-sticky univer-top-0 univer-z-10 univer-flex univer-flex-col univer-items-center
                  univer-justify-between univer-gap-0.5 univer-bg-white univer-p-2
                  md:univer-gap-1.5 md:univer-text-left
                `,
                className
            )}
            {...props}
        />
    );
}

export type IDrawerFooterProps = React.ComponentProps<'div'>;
function DrawerFooter({ className, ...props }: IDrawerFooterProps) {
    return (
        <div
            data-slot="drawer-footer"
            className={clsx('univer-mt-auto univer-flex univer-flex-col univer-gap-2 univer-p-4', className)}
            {...props}
        />
    );
}

export type IDrawerTitleProps = React.ComponentProps<typeof DrawerPrimitive.Title>;
function DrawerTitle({
    className,
    ...props
}: IDrawerTitleProps) {
    return (
        <DrawerPrimitive.Title
            data-slot="drawer-title"
            className={clsx(`
              univer-text-base univer-font-medium univer-text-gray-800
              dark:!univer-text-white
            `, className)}
            {...props}
        />
    );
}

export type IDrawerDescriptionProps = React.ComponentProps<typeof DrawerPrimitive.Description>;
function DrawerDescription({
    className,
    ...props
}: IDrawerDescriptionProps) {
    return (
        <DrawerPrimitive.Description
            data-slot="drawer-description"
            className={clsx('univer-text-sm univer-text-gray-500', className)}
            {...props}
        />
    );
}

export {
    Drawer,
    DrawerClose,
    DrawerContent,
    DrawerDescription,
    DrawerFooter,
    DrawerHeader,
    DrawerOverlay,
    DrawerPortal,
    DrawerTitle,
    DrawerTrigger,
};
