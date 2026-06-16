<script lang="ts">
  import BinderTree from './BinderTree.svelte';

  import type { BinderItemType } from '$lib/schemas/binder';
  import type { BinderNode } from '$lib/stores/binder/binder';

  interface Props {
    nodes: BinderNode[];
    selectedId: string | null;
    depth?: number;
    onSelect: (id: string) => void;
    onToggle: (item: BinderNode) => void;
    onRename: (item: BinderNode, title: string) => void;
    onCreateChild: (parentId: string, itemType: BinderItemType) => void;
    onDuplicate: (item: BinderNode) => void;
    onTrash: (item: BinderNode) => void;
    onMove: (itemId: string, parentId: string | null, position: number) => void;
  }

  let {
    nodes,
    selectedId,
    depth = 0,
    onSelect,
    onToggle,
    onRename,
    onCreateChild,
    onDuplicate,
    onTrash,
    onMove,
  }: Props = $props();

  let editingId = $state<string | null>(null);
  let draftTitle = $state('');
  let menuId = $state<string | null>(null);

  function startRename(item: BinderNode) {
    menuId = null;
    editingId = item.id;
    draftTitle = item.title;
  }

  function commitRename(item: BinderNode) {
    const title = draftTitle.trim();
    editingId = null;
    if (title && title !== item.title) {
      onRename(item, title);
    }
  }

  function submitRename(event: SubmitEvent, item: BinderNode) {
    event.preventDefault();
    commitRename(item);
  }

  function iconFor(itemType: BinderItemType): string {
    switch (itemType) {
      case 'folder':
        return '▸';
      case 'document':
        return '□';
      case 'research':
        return '◇';
      case 'character':
        return '◎';
      case 'location':
        return '⌂';
      case 'note':
        return '✎';
      case 'trash':
        return '×';
    }
  }

  function dragStart(event: DragEvent, item: BinderNode) {
    event.dataTransfer?.setData('text/plain', item.id);
    event.dataTransfer?.setDragImage?.(event.currentTarget as Element, 8, 8);
  }

  function allowDrop(event: DragEvent) {
    event.preventDefault();
  }

  function dropOnItem(event: DragEvent, item: BinderNode) {
    event.preventDefault();
    const itemId = event.dataTransfer?.getData('text/plain');
    if (!itemId || itemId === item.id) return;

    if (item.itemType === 'folder') {
      onMove(itemId, item.id, item.children.length);
      return;
    }

    onMove(itemId, item.parentId, item.position);
  }

  function openContextMenu(event: MouseEvent, item: BinderNode) {
    event.preventDefault();
    onSelect(item.id);
    menuId = menuId === item.id ? null : item.id;
  }

  function closeMenu() {
    menuId = null;
  }
</script>

<ul
  class={depth === 0 ? 'space-y-2' : 'mt-2 space-y-2'}
  role={depth === 0 ? 'tree' : 'group'}
>
  {#each nodes as node (node.id)}
    <li>
      <div
        class="group grid grid-cols-[auto_1fr_auto] items-center gap-2 rounded-lg px-3 py-2 text-base"
        class:bg-accent-50={selectedId === node.id}
        class:text-ink-900={selectedId === node.id}
        class:text-ink-500={selectedId !== node.id}
        style={`padding-left: ${0.75 + depth * 0.75}rem`}
        draggable="true"
        role="treeitem"
        tabindex="-1"
        aria-selected={selectedId === node.id}
        aria-expanded={node.children.length > 0 ? node.isExpanded : undefined}
        ondragstart={(event) => dragStart(event, node)}
        ondragover={allowDrop}
        ondrop={(event) => dropOnItem(event, node)}
        oncontextmenu={(event) => openContextMenu(event, node)}
      >
        <button
          type="button"
          class="h-6 w-6 rounded text-sm text-ink-500 hover:bg-white"
          aria-label={node.children.length > 0
            ? node.isExpanded
              ? 'Contraer'
              : 'Expandir'
            : node.itemType}
          onclick={() =>
            node.children.length > 0 ? onToggle(node) : onSelect(node.id)}
        >
          {node.children.length > 0
            ? node.isExpanded
              ? '▾'
              : '▸'
            : iconFor(node.itemType)}
        </button>

        {#if editingId === node.id}
          <form class="min-w-0" onsubmit={(event) => submitRename(event, node)}>
            <input
              class="h-8 w-full rounded border border-accent-500 bg-white px-2 text-base"
              bind:value={draftTitle}
              onblur={() => commitRename(node)}
              aria-label="Nuevo título"
            />
          </form>
        {:else}
          <button
            type="button"
            class="min-w-0 truncate rounded px-1 py-1 text-left font-medium hover:bg-white"
            aria-current={selectedId === node.id ? 'true' : undefined}
            ondblclick={() => startRename(node)}
            onclick={() => onSelect(node.id)}
          >
            {node.title}
          </button>
        {/if}

        <div
          class="flex opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100"
        >
          {#if node.itemType === 'folder'}
            <button
              type="button"
              class="h-7 w-7 rounded hover:bg-white"
              title="Nuevo documento"
              aria-label="Nuevo documento"
              onclick={() => onCreateChild(node.id, 'document')}
            >
              +
            </button>
          {/if}
          <button
            type="button"
            class="h-7 w-7 rounded hover:bg-white"
            title="Duplicar"
            aria-label="Duplicar"
            onclick={() => onDuplicate(node)}
          >
            ⧉
          </button>
          <button
            type="button"
            class="h-7 w-7 rounded hover:bg-white"
            title="Mover a papelera"
            aria-label="Mover a papelera"
            onclick={() => onTrash(node)}
          >
            ×
          </button>
        </div>
      </div>

      {#if menuId === node.id}
        <div
          class="ml-8 mt-1 grid w-40 gap-1 rounded-lg border border-ink-100 bg-white p-1 text-sm shadow"
          role="menu"
          aria-label={`Acciones de ${node.title}`}
        >
          {#if node.itemType === 'folder'}
            <button
              type="button"
              class="rounded px-2 py-1 text-left hover:bg-accent-50"
              role="menuitem"
              onclick={() => {
                closeMenu();
                onCreateChild(node.id, 'document');
              }}
            >
              Nuevo documento
            </button>
          {/if}
          <button
            type="button"
            class="rounded px-2 py-1 text-left hover:bg-accent-50"
            role="menuitem"
            onclick={() => startRename(node)}
          >
            Renombrar
          </button>
          <button
            type="button"
            class="rounded px-2 py-1 text-left hover:bg-accent-50"
            role="menuitem"
            onclick={() => {
              closeMenu();
              onDuplicate(node);
            }}
          >
            Duplicar
          </button>
          <button
            type="button"
            class="rounded px-2 py-1 text-left hover:bg-accent-50"
            role="menuitem"
            onclick={() => {
              closeMenu();
              onTrash(node);
            }}
          >
            Mover a papelera
          </button>
        </div>
      {/if}

      {#if node.isExpanded && node.children.length > 0}
        <BinderTree
          nodes={node.children}
          {selectedId}
          depth={depth + 1}
          {onSelect}
          {onToggle}
          {onRename}
          {onCreateChild}
          {onDuplicate}
          {onTrash}
          {onMove}
        />
      {/if}
    </li>
  {/each}
</ul>
