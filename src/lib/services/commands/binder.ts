import { z } from 'zod';

import {
  binderItemSchema,
  binderItemsSchema,
  createBinderItemRequestSchema,
  duplicateBinderItemRequestSchema,
  listBinderItemsRequestSchema,
  moveBinderItemRequestSchema,
  renameBinderItemRequestSchema,
  reorderBinderItemsRequestSchema,
  restoreBinderItemRequestSchema,
  setBinderItemExpandedRequestSchema,
  trashBinderItemRequestSchema,
  type BinderItem,
  type CreateBinderItemRequest,
  type DuplicateBinderItemRequest,
  type ListBinderItemsRequest,
  type MoveBinderItemRequest,
  type RenameBinderItemRequest,
  type ReorderBinderItemsRequest,
  type RestoreBinderItemRequest,
  type SetBinderItemExpandedRequest,
  type TrashBinderItemRequest,
} from '$lib/schemas/binder';
import { commandErrorSchema } from '$lib/schemas/project';
import { TauriCommandError } from './project';
import { invokeCommand } from './runtime';

export async function listBinderItems(
  request: ListBinderItemsRequest,
): Promise<BinderItem[]> {
  return invokeBinderCommand(
    'list_binder_items',
    { request: listBinderItemsRequestSchema.parse(request) },
    binderItemsSchema,
  );
}

export async function createBinderItem(
  request: CreateBinderItemRequest,
): Promise<BinderItem> {
  return invokeBinderCommand(
    'create_binder_item',
    { request: createBinderItemRequestSchema.parse(request) },
    binderItemSchema,
  );
}

export async function renameBinderItem(
  request: RenameBinderItemRequest,
): Promise<BinderItem> {
  return invokeBinderCommand(
    'rename_binder_item',
    { request: renameBinderItemRequestSchema.parse(request) },
    binderItemSchema,
  );
}

export async function setBinderItemExpanded(
  request: SetBinderItemExpandedRequest,
): Promise<BinderItem> {
  return invokeBinderCommand(
    'set_binder_item_expanded',
    { request: setBinderItemExpandedRequestSchema.parse(request) },
    binderItemSchema,
  );
}

export async function duplicateBinderItem(
  request: DuplicateBinderItemRequest,
): Promise<BinderItem> {
  return invokeBinderCommand(
    'duplicate_binder_item',
    { request: duplicateBinderItemRequestSchema.parse(request) },
    binderItemSchema,
  );
}

export async function moveBinderItem(
  request: MoveBinderItemRequest,
): Promise<BinderItem[]> {
  return invokeBinderCommand(
    'move_binder_item',
    { request: moveBinderItemRequestSchema.parse(request) },
    binderItemsSchema,
  );
}

export async function reorderBinderItems(
  request: ReorderBinderItemsRequest,
): Promise<BinderItem[]> {
  return invokeBinderCommand(
    'reorder_binder_items',
    { request: reorderBinderItemsRequestSchema.parse(request) },
    binderItemsSchema,
  );
}

export async function trashBinderItem(
  request: TrashBinderItemRequest,
): Promise<BinderItem[]> {
  return invokeBinderCommand(
    'trash_binder_item',
    { request: trashBinderItemRequestSchema.parse(request) },
    binderItemsSchema,
  );
}

export async function restoreBinderItem(
  request: RestoreBinderItemRequest,
): Promise<BinderItem[]> {
  return invokeBinderCommand(
    'restore_binder_item',
    { request: restoreBinderItemRequestSchema.parse(request) },
    binderItemsSchema,
  );
}

async function invokeBinderCommand<T>(
  command: string,
  args: Record<string, unknown>,
  schema: z.ZodType<T>,
): Promise<T> {
  try {
    const result = await invokeCommand<unknown>(command, args);
    return schema.parse(result);
  } catch (error) {
    const parsed = commandErrorSchema.safeParse(error);
    if (parsed.success) {
      throw new TauriCommandError(parsed.data);
    }

    throw error;
  }
}
