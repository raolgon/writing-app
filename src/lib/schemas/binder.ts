import { z } from 'zod';

export const binderItemTypeSchema = z.enum([
  'folder',
  'document',
  'research',
  'character',
  'location',
  'note',
  'trash',
]);

export const binderItemSchema = z.object({
  id: z.string().uuid(),
  projectId: z.string().uuid(),
  parentId: z.string().uuid().nullable(),
  itemType: binderItemTypeSchema,
  title: z.string().min(1),
  synopsis: z.string(),
  position: z.number().int(),
  icon: z.string().nullable(),
  colorLabel: z.string().nullable(),
  status: z.string().nullable(),
  createdAt: z.string().datetime(),
  updatedAt: z.string().datetime(),
  isExpanded: z.boolean(),
  isArchived: z.boolean(),
  trashedAt: z.string().datetime().nullable(),
});

export const binderItemsSchema = z.array(binderItemSchema);

export const listBinderItemsRequestSchema = z.object({
  sessionId: z.string().uuid(),
  includeTrashed: z.boolean(),
});

export const createBinderItemRequestSchema = z.object({
  sessionId: z.string().uuid(),
  parentId: z.string().uuid().nullable(),
  itemType: binderItemTypeSchema,
  title: z.string().min(1),
});

export const renameBinderItemRequestSchema = z.object({
  sessionId: z.string().uuid(),
  itemId: z.string().uuid(),
  title: z.string().min(1),
});

export const setBinderItemExpandedRequestSchema = z.object({
  sessionId: z.string().uuid(),
  itemId: z.string().uuid(),
  isExpanded: z.boolean(),
});

export const duplicateBinderItemRequestSchema = z.object({
  sessionId: z.string().uuid(),
  itemId: z.string().uuid(),
});

export const moveBinderItemRequestSchema = z.object({
  sessionId: z.string().uuid(),
  itemId: z.string().uuid(),
  parentId: z.string().uuid().nullable(),
  position: z.number().int().nonnegative(),
});

export const reorderBinderItemsRequestSchema = z.object({
  sessionId: z.string().uuid(),
  parentId: z.string().uuid().nullable(),
  orderedIds: z.array(z.string().uuid()),
});

export const trashBinderItemRequestSchema = z.object({
  sessionId: z.string().uuid(),
  itemId: z.string().uuid(),
});

export const restoreBinderItemRequestSchema = trashBinderItemRequestSchema;

export type BinderItemType = z.infer<typeof binderItemTypeSchema>;
export type BinderItem = z.infer<typeof binderItemSchema>;
export type ListBinderItemsRequest = z.infer<
  typeof listBinderItemsRequestSchema
>;
export type CreateBinderItemRequest = z.infer<
  typeof createBinderItemRequestSchema
>;
export type RenameBinderItemRequest = z.infer<
  typeof renameBinderItemRequestSchema
>;
export type SetBinderItemExpandedRequest = z.infer<
  typeof setBinderItemExpandedRequestSchema
>;
export type DuplicateBinderItemRequest = z.infer<
  typeof duplicateBinderItemRequestSchema
>;
export type MoveBinderItemRequest = z.infer<typeof moveBinderItemRequestSchema>;
export type ReorderBinderItemsRequest = z.infer<
  typeof reorderBinderItemsRequestSchema
>;
export type TrashBinderItemRequest = z.infer<
  typeof trashBinderItemRequestSchema
>;
export type RestoreBinderItemRequest = z.infer<
  typeof restoreBinderItemRequestSchema
>;
