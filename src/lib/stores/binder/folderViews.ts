import type { BinderItem } from '$lib/schemas/binder';

export type OutlineSortKey =
  | 'position'
  | 'title'
  | 'itemType'
  | 'status'
  | 'updatedAt';
export type SortDirection = 'asc' | 'desc';

export function directChildren(
  items: BinderItem[],
  parentId: string,
): BinderItem[] {
  return items
    .filter((item) => item.parentId === parentId && item.trashedAt === null)
    .toSorted((left, right) => left.position - right.position);
}

export function documentChildren(
  items: BinderItem[],
  parentId: string,
): BinderItem[] {
  return directChildren(items, parentId).filter(itemHasDocument);
}

export function itemHasDocument(item: BinderItem): boolean {
  return item.itemType !== 'folder' && item.itemType !== 'trash';
}

export function sortOutlineItems(
  items: BinderItem[],
  sortKey: OutlineSortKey,
  direction: SortDirection,
): BinderItem[] {
  const multiplier = direction === 'asc' ? 1 : -1;
  return [...items].sort((left, right) => {
    const primary = compareByKey(left, right, sortKey);
    if (primary !== 0) return primary * multiplier;
    return left.position - right.position;
  });
}

function compareByKey(
  left: BinderItem,
  right: BinderItem,
  sortKey: OutlineSortKey,
) {
  switch (sortKey) {
    case 'position':
      return left.position - right.position;
    case 'title':
      return compareText(left.title, right.title);
    case 'itemType':
      return compareText(left.itemType, right.itemType);
    case 'status':
      return compareText(left.status ?? '', right.status ?? '');
    case 'updatedAt':
      return Date.parse(left.updatedAt) - Date.parse(right.updatedAt);
  }
}

function compareText(left: string, right: string) {
  return left.localeCompare(right, 'es', { sensitivity: 'base' });
}
