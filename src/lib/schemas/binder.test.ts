import { describe, expect, it } from 'vitest';

import {
  binderItemSchema,
  createBinderItemRequestSchema,
  reorderBinderItemsRequestSchema,
} from './binder';

describe('binder schemas', () => {
  it('validates binder items returned by Tauri', () => {
    const result = binderItemSchema.safeParse({
      id: '018fe3c4-5678-7abc-9def-0123456789ab',
      projectId: '018fe3c4-5678-7abc-9def-0123456789ac',
      parentId: null,
      itemType: 'document',
      title: 'Scene',
      synopsis: '',
      position: 0,
      icon: null,
      colorLabel: null,
      status: null,
      createdAt: '2026-06-15T00:00:00.000Z',
      updatedAt: '2026-06-15T00:00:00.000Z',
      isExpanded: false,
      isArchived: false,
      trashedAt: null,
    });

    expect(result.success).toBe(true);
  });

  it('validates create requests', () => {
    const result = createBinderItemRequestSchema.safeParse({
      sessionId: '018fe3c4-5678-7abc-9def-0123456789ab',
      parentId: null,
      itemType: 'folder',
      title: 'Chapter',
    });

    expect(result.success).toBe(true);
  });

  it('rejects negative reorder gaps at request boundary', () => {
    const result = reorderBinderItemsRequestSchema.safeParse({
      sessionId: '018fe3c4-5678-7abc-9def-0123456789ab',
      parentId: null,
      orderedIds: ['not-a-uuid'],
    });

    expect(result.success).toBe(false);
  });
});
