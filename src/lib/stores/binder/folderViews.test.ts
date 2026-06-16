import { describe, expect, it } from 'vitest';

import type { BinderItem } from '$lib/schemas/binder';
import {
  directChildren,
  documentChildren,
  sortOutlineItems,
} from './folderViews';

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

function item(
  overrides: Partial<BinderItem> & Pick<BinderItem, 'id'>,
): BinderItem {
  return {
    ...baseItem,
    parentId: null,
    itemType: 'document',
    title: 'Untitled',
    position: 0,
    ...overrides,
  };
}

describe('folder view helpers', () => {
  it('returns direct children in binder order', () => {
    const parentId = '018fe3c4-5678-7abc-9def-0123456789ad';
    const children = directChildren(
      [
        item({
          id: '018fe3c4-5678-7abc-9def-0123456789ae',
          parentId,
          title: 'Second',
          position: 1,
        }),
        item({
          id: '018fe3c4-5678-7abc-9def-0123456789af',
          parentId,
          title: 'First',
          position: 0,
        }),
        item({
          id: '018fe3c4-5678-7abc-9def-0123456789b0',
          parentId: null,
          title: 'Root',
          position: 0,
        }),
      ],
      parentId,
    );

    expect(children.map((child) => child.title)).toEqual(['First', 'Second']);
  });

  it('uses only document-backed items for board cards', () => {
    const parentId = '018fe3c4-5678-7abc-9def-0123456789ad';
    const cards = documentChildren(
      [
        item({
          id: '018fe3c4-5678-7abc-9def-0123456789ae',
          parentId,
          itemType: 'folder',
          title: 'Folder',
        }),
        item({
          id: '018fe3c4-5678-7abc-9def-0123456789af',
          parentId,
          itemType: 'document',
          title: 'Scene',
        }),
      ],
      parentId,
    );

    expect(cards.map((card) => card.title)).toEqual(['Scene']);
  });

  it('sorts outline rows by title and keeps position as a tie breaker', () => {
    const rows = sortOutlineItems(
      [
        item({
          id: '018fe3c4-5678-7abc-9def-0123456789ae',
          title: 'Beta',
          position: 0,
        }),
        item({
          id: '018fe3c4-5678-7abc-9def-0123456789af',
          title: 'Alpha',
          position: 1,
        }),
      ],
      'title',
      'asc',
    );

    expect(rows.map((row) => row.title)).toEqual(['Alpha', 'Beta']);
  });
});
