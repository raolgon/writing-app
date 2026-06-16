import { derived, writable } from 'svelte/store';

import type { BinderItem } from '$lib/schemas/binder';

export interface BinderNode extends BinderItem {
  children: BinderNode[];
}

export const binderItemsStore = writable<BinderItem[]>([]);

export const binderTreeStore = derived(binderItemsStore, ($items) =>
  buildBinderTree($items),
);

export function buildBinderTree(items: BinderItem[]): BinderNode[] {
  const nodes = new Map<string, BinderNode>();
  const roots: BinderNode[] = [];

  for (const item of items) {
    nodes.set(item.id, { ...item, children: [] });
  }

  for (const item of items) {
    const node = nodes.get(item.id);
    if (!node) continue;

    if (item.parentId) {
      const parent = nodes.get(item.parentId);
      if (parent) {
        parent.children.push(node);
        continue;
      }
    }

    roots.push(node);
  }

  const sortNodes = (nodeList: BinderNode[]) => {
    nodeList.sort((left, right) => left.position - right.position);
    for (const node of nodeList) {
      sortNodes(node.children);
    }
  };

  sortNodes(roots);
  return roots;
}
