import { describe, expect, it } from 'vitest';

import type { BinderItem } from '$lib/schemas/binder';
import { buildBinderTree } from './binder';

const baseItem = {
  projectId: '018fe3c4-5678-7abc-9def-0123456789ac',
  synopsis: '',
  icon: null,
  colorLabel: null,
  status: null,
  createdAt: '2026-06-15T00:00:00.000Z',
  updatedAt: '2026-06-15T00:00:00.000Z',
  isExpanded: true,
  isArchived: false,
  trashedAt: null,
} satisfies Omit<
  BinderItem,
  'id' | 'parentId' | 'itemType' | 'title' | 'position'
>;

describe('buildBinderTree', () => {
  it('nests children and preserves sibling order', () => {
    const parentId = '018fe3c4-5678-7abc-9def-0123456789ad';
    const tree = buildBinderTree([
      {
        ...baseItem,
        id: '018fe3c4-5678-7abc-9def-0123456789ae',
        parentId,
        itemType: 'document',
        title: 'Second',
        position: 1,
      },
      {
        ...baseItem,
        id: parentId,
        parentId: null,
        itemType: 'folder',
        title: 'Folder',
        position: 0,
      },
      {
        ...baseItem,
        id: '018fe3c4-5678-7abc-9def-0123456789af',
        parentId,
        itemType: 'document',
        title: 'First',
        position: 0,
      },
    ]);

    expect(tree).toHaveLength(1);
    expect(tree[0]?.children.map((item) => item.title)).toEqual([
      'First',
      'Second',
    ]);
  });
});
