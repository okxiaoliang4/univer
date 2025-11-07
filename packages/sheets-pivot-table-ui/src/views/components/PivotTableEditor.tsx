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

import type {
    CancelDrop,
    CollisionDetection,
    DropAnimation,
    KeyboardCoordinateGetter,
    Modifiers,
    UniqueIdentifier,
} from '@dnd-kit/core';
import type {
    AnimateLayoutChanges,
    SortingStrategy,
} from '@dnd-kit/sortable';
import type { IPivotField, PivotTable } from '@univerjs/sheets-pivot-table';
import type React from 'react';
import type { IFieldItemsContainerProps } from './FieldItemsContainer';
import {
    closestCenter,
    defaultDropAnimationSideEffects,
    DndContext,
    DragOverlay,
    getFirstCollision,
    KeyboardSensor,
    MeasuringStrategy,
    MouseSensor,
    pointerWithin,
    rectIntersection,
    TouchSensor,
    useSensor,
    useSensors,
} from '@dnd-kit/core';
import {
    arrayMove,
    defaultAnimateLayoutChanges,
    SortableContext,
    useSortable,
    verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { generateRandomId } from '@univerjs/core';
import { useObservable } from '@univerjs/ui';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { coordinateGetter as multipleContainersCoordinateGetter } from '../../common/multipleContainersKeyboardCoordinates';
import { FieldItem } from './FieldItem';
import { FieldItemsContainer } from './FieldItemsContainer';

const animateLayoutChanges: AnimateLayoutChanges = (args) =>
    defaultAnimateLayoutChanges({ ...args, wasDragging: true });

function DroppableContainer({
    children,
    columns = 1,
    disabled,
    id,
    items,
    style,
    ...props
}: IFieldItemsContainerProps & {
    disabled?: boolean;
    id: UniqueIdentifier;
    items: UniqueIdentifier[];
    style?: React.CSSProperties;
}) {
    const {
        active,
        attributes,
        isDragging,
        listeners,
        over,
        setNodeRef,
        transition,
        transform,
    } = useSortable({
        id,
        data: {
            type: 'container',
            children: items,
        },
        animateLayoutChanges,
    });
    const isOverContainer = over
        ? (id === over.id && active?.data.current?.type !== 'container') ||
    items.includes(over.id)
        : false;

    return (
        <FieldItemsContainer
            className="flex-1"
            ref={disabled ? undefined : setNodeRef}
            style={{
                ...style,
                transition,
                transform: CSS.Translate.toString(transform),
                opacity: isDragging ? 0.5 : undefined,
            }}
            hover={isOverContainer}
            handleProps={{
                ...attributes,
                ...listeners,
            }}
            columns={columns}
            {...props}
        >
            {children}
        </FieldItemsContainer>
    );
}

const dropAnimation: DropAnimation = {
    sideEffects: defaultDropAnimationSideEffects({
        styles: {
            active: {
                opacity: '0.5',
            },
        },
    }),
};

const defaultGetItemStyles = () => ({});
const defaultWrapperStyle = () => ({});

interface IPivotTableEditorProps {
    pivotTable: PivotTable;
    adjustScale?: boolean;
    cancelDrop?: CancelDrop;
    columns?: number;
    containerStyle?: React.CSSProperties;
    coordinateGetter?: KeyboardCoordinateGetter;
    getItemStyles?(args: {
        value: UniqueIdentifier;
        index: number;
        overIndex: number;
        isDragging: boolean;
        containerId: UniqueIdentifier;
        isSorting: boolean;
        isDragOverlay: boolean;
    }): React.CSSProperties;
    wrapperStyle?(args: { index: number }): React.CSSProperties;
    handle?: boolean;
    renderItem?(): React.ReactElement;
    strategy?: SortingStrategy;
    modifiers?: Modifiers;
    minimal?: boolean;
    trashable?: boolean;
    scrollable?: boolean;
    vertical?: boolean;
}

export function PivotTableEditor({
    pivotTable,
    adjustScale = false,
    cancelDrop,
    columns,
    handle = true,
    containerStyle,
    coordinateGetter = multipleContainersCoordinateGetter,
    getItemStyles = defaultGetItemStyles,
    wrapperStyle = defaultWrapperStyle,
    minimal = false,
    modifiers,
    renderItem,
    strategy = verticalListSortingStrategy,
    scrollable,
}: IPivotTableEditorProps) {
    // Source fields - constant, never modified
    const sourceFields: IPivotField[] = [
        {
            id: 'Product',
            name: 'Product',
            sourceColumnIndex: 0,
        },
        {
            id: 'East',
            name: 'East',
            sourceColumnIndex: 1,
        },
        {
            id: 'West',
            name: 'West',
            sourceColumnIndex: 2,
        },
    ];

    // Get current fields from pivotTable observables
    const valueFields = useObservable(pivotTable.valueFields$) || [];
    const rowFields = useObservable(pivotTable.rowFields$) || [];
    const columnFields = useObservable(pivotTable.columnFields$) || [];
    const filterFields = useObservable(pivotTable.filterFields$) || [];

    // Memoized containers structure directly from pivotTable data
    const items: Record<string, IPivotField[]> = useMemo(() => ({
        sourceFields,
        filterFields,
        columnFields,
        rowFields,
        valueFields,
    }), [filterFields, columnFields, rowFields, valueFields]);

    const containers = useMemo(() =>
        Object.keys(items) as UniqueIdentifier[], [items]);

    const [activeId, setActiveId] = useState<UniqueIdentifier | null>(null);
    const lastOverId = useRef<UniqueIdentifier | null>(null);
    const recentlyMovedToNewContainer = useRef(false);
    const isSortingContainer =
        activeId != null ? containers.includes(activeId) : false;

    // Track active container for drag operations
    const activeContainerRef = useRef<UniqueIdentifier | null>(null);

    // Helper: Get base field from sourceFields by ID
    const getSourceField = useCallback((sourceColumnIndex: number): IPivotField | null => {
        // Extract base ID (remove _suffix if exists)
        return sourceFields.find((f) => f.sourceColumnIndex === sourceColumnIndex) || null;
    }, []);

    // Helper: Check if a field (by base ID) exists in exclusive containers
    const findFieldInExclusiveContainers = useCallback((sourceColumnIndex: number): {
        container: 'filterFields' | 'columnFields' | 'rowFields' | null;
        field: IPivotField | null;
    } => {
        // Check filterFields
        const filterField = filterFields.find((f) => f.sourceColumnIndex === sourceColumnIndex);
        if (filterField) {
            return { container: 'filterFields', field: filterField };
        }

        // Check columnFields
        const columnField = columnFields.find((f) => f.sourceColumnIndex === sourceColumnIndex);
        if (columnField) {
            return { container: 'columnFields', field: columnField };
        }

        // Check rowFields
        const rowField = rowFields.find((f) => f.sourceColumnIndex === sourceColumnIndex);
        if (rowField) {
            return { container: 'rowFields', field: rowField };
        }

        return { container: null, field: null };
    }, [filterFields, columnFields, rowFields]);

  /**
   * Custom collision detection strategy optimized for multiple containers
   *
   * - First, find any droppable containers intersecting with the pointer.
   * - If there are none, find intersecting containers with the active draggable.
   * - If there are no intersecting containers, return the last matched intersection
   *
   */
    const collisionDetectionStrategy: CollisionDetection = useCallback(
        (args) => {
            if (activeId && activeId in items) {
                return closestCenter({
                    ...args,
                    droppableContainers: args.droppableContainers.filter(
                        (container) => container.id in items
                    ),
                });
            }

            // Start by finding any intersecting droppable
            const pointerIntersections = pointerWithin(args);
            const intersections =
                pointerIntersections.length > 0
                    ? // If there are droppables intersecting with the pointer, return those
                    pointerIntersections
                    : rectIntersection(args);
            let overId = getFirstCollision(intersections, 'id');

            if (overId != null) {
                if (overId in items) {
                    const containerItems = items[overId];

                // If a container is matched and it contains items (columns 'A', 'B', 'C')
                    if (containerItems.length > 0) {
                  // Return the closest droppable within that container
                        overId = closestCenter({
                            ...args,
                            droppableContainers: args.droppableContainers.filter(
                                (container) =>
                                    container.id !== overId &&
                      containerItems.some((field) => field.id === container.id)
                            ),
                        })[0]?.id;
                    }
                }

                lastOverId.current = overId;

                return [{ id: overId }];
            }

            // When a draggable item moves to a new container, the layout may shift
            // and the `overId` may become `null`. We manually set the cached `lastOverId`
            // to the id of the draggable item that was moved to the new container, otherwise
            // the previous `overId` will be returned which can cause items to incorrectly shift positions
            if (recentlyMovedToNewContainer.current) {
                lastOverId.current = activeId;
            }

            // If no droppable is matched, return the last match
            return lastOverId.current ? [{ id: lastOverId.current }] : [];
        },
        [activeId, items]
    );
    const sensors = useSensors(
        useSensor(MouseSensor),
        useSensor(TouchSensor),
        useSensor(KeyboardSensor, {
            coordinateGetter,
        })
    );
    const findContainer = (id: UniqueIdentifier) => {
        if (id in items) {
            return id;
        }

        // Check if it's in one of the droppable containers
        return Object.keys(items).find((key) => items[key].some((field) => field.id === id));
    };

    const getIndex = (id: UniqueIdentifier) => {
        const container = findContainer(id);

        if (!container) {
            return -1;
        }

        const index = (items[container as keyof typeof items] as IPivotField[]).findIndex((field) => field.id === id);

        return index;
    };

    const onDragCancel = () => {
        setActiveId(null);
        activeContainerRef.current = null;
    };

    return (
        <DndContext
            sensors={sensors}
            collisionDetection={collisionDetectionStrategy}
            measuring={{
                droppable: {
                    strategy: MeasuringStrategy.Always,
                },
            }}
            onDragStart={(params) => {
                const { active } = params;
                const container = findContainer(active.id);
                setActiveId(active.id);
                activeContainerRef.current = container || null;
            }}
            onDragOver={({ over }) => {
                // Track the current over target for visual feedback
                if (over?.id) {
                    lastOverId.current = over.id;
                }
            }}
            onDragEnd={({ active, over }) => {
                const activeContainer = findContainer(active.id);

                if (!activeContainer) {
                    setActiveId(null);
                    return;
                }

                const overId = over?.id;

                if (overId == null) {
                    setActiveId(null);
                    return;
                }

                const overContainer = findContainer(overId);

                // Don't allow dropping back to source
                if (!overContainer || overContainer === 'sourceFields') {
                    setActiveId(null);
                    return;
                }

                const activeFieldId = String(active.id);

                const activeSourceColumnIndex = sourceFields.find((f) => f.id === activeFieldId)?.sourceColumnIndex ?? -1;

                // Handle dragging from source fields (copy operation)
                if (activeContainer === 'sourceFields') {
                    const sourceField = getSourceField(activeSourceColumnIndex);
                    if (!sourceField) {
                        setActiveId(null);
                        return;
                    }

                    const newField: IPivotField = {
                        ...sourceField,
                        id: generateRandomId(6),
                    };
                    // Apply Excel pivot table rules
                    if (overContainer === 'valueFields') {
                        // valueFields can have duplicates - create new field with unique ID
                        pivotTable.setValueFields([...valueFields, newField]);
                    } else {
                        // filterFields, columnFields, rowFields - exclusive rule
                        const existingField = findFieldInExclusiveContainers(activeSourceColumnIndex);

                        // Remove from other exclusive containers if exists
                        if (existingField.container && existingField.container !== overContainer) {
                            if (existingField.container === 'filterFields') {
                                pivotTable.setFilterFields(filterFields.filter((f) => f.sourceColumnIndex !== sourceField.sourceColumnIndex));
                            } else if (existingField.container === 'columnFields') {
                                pivotTable.setColumnFields(columnFields.filter((f) => f.sourceColumnIndex !== sourceField.sourceColumnIndex));
                            } else if (existingField.container === 'rowFields') {
                                pivotTable.setRowFields(rowFields.filter((f) => f.sourceColumnIndex !== sourceField.sourceColumnIndex));
                            }
                        }

                        // Add to target container
                        if (overContainer === 'filterFields' && !filterFields.some((f) => f.sourceColumnIndex === sourceField.sourceColumnIndex)) {
                            pivotTable.setFilterFields([...filterFields, existingField.field || newField]);
                        } else if (overContainer === 'columnFields' && !columnFields.some((f) => f.sourceColumnIndex === sourceField.sourceColumnIndex)) {
                            pivotTable.setColumnFields([...columnFields, existingField.field || newField]);
                        } else if (overContainer === 'rowFields' && !rowFields.some((f) => f.sourceColumnIndex === sourceField.sourceColumnIndex)) {
                            pivotTable.setRowFields([...rowFields, existingField.field || newField]);
                        }
                    }
                } else {
                    // Handle dragging between containers (not from source)
                    const draggedField = [...valueFields, ...rowFields, ...columnFields, ...filterFields].find((f) => f.id === activeFieldId);

                    if (!draggedField) {
                        setActiveId(null);
                        return;
                    }

                    if (activeContainer === overContainer) {
                        // Reordering within same container
                        const currentFields = activeContainer === 'valueFields'
                            ? valueFields :
                            activeContainer === 'rowFields'
                                ? rowFields :
                                activeContainer === 'columnFields'
                                    ? columnFields :
                                    filterFields;

                        const activeIndex = currentFields.findIndex((f) => f.id === activeFieldId);
                        const overIndex = currentFields.findIndex((f) => f.id === overId);

                        if (activeIndex !== -1 && overIndex !== -1 && activeIndex !== overIndex) {
                            const reorderedFields = arrayMove(currentFields, activeIndex, overIndex);

                            if (activeContainer === 'valueFields') {
                                pivotTable.setValueFields(reorderedFields);
                            } else if (activeContainer === 'rowFields') {
                                pivotTable.setRowFields(reorderedFields);
                            } else if (activeContainer === 'columnFields') {
                                pivotTable.setColumnFields(reorderedFields);
                            } else if (activeContainer === 'filterFields') {
                                pivotTable.setFilterFields(reorderedFields);
                            }
                        }
                    } else {
                        // Moving between different containers
                        const sourceField = getSourceField(draggedField.sourceColumnIndex);

                        if (!sourceField) {
                            setActiveId(null);
                            return;
                        }

                        // Remove from source container first
                        if (activeContainer === 'valueFields') {
                            pivotTable.setValueFields(valueFields.filter((f) => f.id !== activeFieldId));
                        } else if (activeContainer === 'rowFields') {
                            pivotTable.setRowFields(rowFields.filter((f) => f.id !== activeFieldId));
                        } else if (activeContainer === 'columnFields') {
                            pivotTable.setColumnFields(columnFields.filter((f) => f.id !== activeFieldId));
                        } else if (activeContainer === 'filterFields') {
                            pivotTable.setFilterFields(filterFields.filter((f) => f.id !== activeFieldId));
                        }

                        const newField: IPivotField = {
                            ...sourceField,
                            id: generateRandomId(6),
                        };
                        // Add to target container
                        if (overContainer === 'valueFields') {
                            // Moving to valueFields - create new field with unique ID
                            pivotTable.setValueFields([...valueFields, newField]);
                        } else {
                            // Moving to exclusive container - remove from others first
                            if (overContainer !== 'filterFields') {
                                pivotTable.setFilterFields(filterFields.filter((f) => f.sourceColumnIndex !== sourceField.sourceColumnIndex));
                            }
                            if (overContainer !== 'columnFields') {
                                pivotTable.setColumnFields(columnFields.filter((f) => f.sourceColumnIndex !== sourceField.sourceColumnIndex));
                            }
                            if (overContainer !== 'rowFields') {
                                pivotTable.setRowFields(rowFields.filter((f) => f.sourceColumnIndex !== sourceField.sourceColumnIndex));
                            }

                            // Add to target
                            if (overContainer === 'filterFields') {
                                pivotTable.setFilterFields([...filterFields, newField]);
                            } else if (overContainer === 'columnFields') {
                                pivotTable.setColumnFields([...columnFields, newField]);
                            } else if (overContainer === 'rowFields') {
                                pivotTable.setRowFields([...rowFields, newField]);
                            }
                        }
                    }
                }

                setActiveId(null);
                activeContainerRef.current = null;
            }}
            cancelDrop={cancelDrop}
            onDragCancel={onDragCancel}
            modifiers={modifiers}
        >
            <div className="univer-grid univer-grid-cols-2 univer-gap-2">
                {containers.map((containerId) => (
                    <DroppableContainer
                        key={containerId}
                        id={containerId}
                        label={minimal ? undefined : `${containerId}`}
                        columns={columns}
                        items={items[containerId].map((field) => field.id)}
                        scrollable={scrollable}
                        style={containerStyle}
                        unstyled={minimal}
                    >
                        <SortableContext items={items[containerId].map((field) => field.id)} strategy={strategy}>
                            {items[containerId]?.map((field: IPivotField, index: number) => {
                                return (
                                    <SortableItem
                                        disabled={isSortingContainer}
                                        key={field.id}
                                        id={field.id}
                                        index={index}
                                        handle={handle}
                                        style={getItemStyles}
                                        wrapperStyle={wrapperStyle}
                                        renderItem={renderItem}
                                        containerId={containerId}
                                        getIndex={getIndex}
                                        value={field.name}
                                    />
                                );
                            })}
                        </SortableContext>
                    </DroppableContainer>
                ))}
            </div>
            {createPortal(
                <DragOverlay adjustScale={adjustScale} dropAnimation={dropAnimation}>
                    {activeId
                        ? containers.includes(activeId)
                            ? renderContainerDragOverlay(activeId)
                            : renderSortableItemDragOverlay(activeId)
                        : null}
                </DragOverlay>,
                document.body
            )}
        </DndContext>
    );

    function renderSortableItemDragOverlay(id: UniqueIdentifier) {
        // Find the field to display its name
        const container = findContainer(id);
        const field = container
            ? items[container]?.find((f: IPivotField) => f.id === id)
            : null;

        return (
            <FieldItem
                id={String(id)}
                value={field?.name || id}
                handle={handle}
                style={getItemStyles({
                    containerId: findContainer(id) as UniqueIdentifier,
                    overIndex: -1,
                    index: getIndex(id),
                    value: id,
                    isSorting: true,
                    isDragging: true,
                    isDragOverlay: true,
                })}
                wrapperStyle={wrapperStyle({ index: 0 })}
                renderItem={renderItem}
                dragOverlay
            />
        );
    }

    function renderContainerDragOverlay(containerId: UniqueIdentifier) {
        return (
            <FieldItemsContainer
                label={`Column ${containerId}`}
                columns={columns}
                style={{
                    height: '100%',
                }}
                shadow
                unstyled={false}
            >
                {items[String(containerId)].map((item: IPivotField, index: number) => (
                    <FieldItem
                        id={item.id}
                        key={item.id}
                        value={item.name}
                        handle={handle}
                        style={getItemStyles({
                            containerId,
                            overIndex: -1,
                            index: getIndex(item.id),
                            value: item.id,
                            isDragging: false,
                            isSorting: false,
                            isDragOverlay: false,
                        })}
                        wrapperStyle={wrapperStyle({ index })}
                        renderItem={renderItem}
                    />
                ))}
            </FieldItemsContainer>
        );
    }
}

interface ISortableItemProps {
    containerId: UniqueIdentifier;
    id: UniqueIdentifier;
    index: number;
    handle: boolean;
    disabled?: boolean;
    value: string;
    style(args: {
        value: UniqueIdentifier;
        index: number;
        overIndex: number;
        isDragging: boolean;
        containerId: UniqueIdentifier;
        isSorting: boolean;
    }): React.CSSProperties;
    getIndex(id: UniqueIdentifier): number;
    renderItem?(): React.ReactElement;
    wrapperStyle({ index }: { index: number }): React.CSSProperties;
}

function SortableItem({
    disabled,
    id,
    index,
    handle,
    renderItem,
    style,
    containerId,
    getIndex,
    wrapperStyle,
    value,
}: ISortableItemProps) {
    const {
        setNodeRef,
        setActivatorNodeRef,
        listeners,
        isDragging,
        isSorting,
        over,
        overIndex,
        transform,
        transition,
    } = useSortable({
        id,
    });
    const mounted = useMountStatus();
    const mountedWhileDragging = isDragging && !mounted;

    return (
        <FieldItem
            ref={disabled ? undefined : setNodeRef}
            id={String(id)}
            value={value}
            dragging={isDragging}
            sorting={isSorting}
            handle={handle}
            handleProps={handle ? { ref: setActivatorNodeRef } : undefined}
            index={index}
            wrapperStyle={wrapperStyle({ index })}
            style={style({
                index,
                value: id,
                isDragging,
                isSorting,
                overIndex: over ? getIndex(over.id) : overIndex,
                containerId,
            })}
            transition={transition}
            transform={transform}
            fadeIn={mountedWhileDragging}
            listeners={listeners}
            renderItem={renderItem}
        />
    );
}

function useMountStatus() {
    const [isMounted, setIsMounted] = useState(false);

    useEffect(() => {
        const timeout = setTimeout(() => setIsMounted(true), 500);

        return () => clearTimeout(timeout);
    }, []);

    return isMounted;
}
