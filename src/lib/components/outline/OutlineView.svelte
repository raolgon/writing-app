<script lang="ts">
  import type { BinderItem } from '$lib/schemas/binder';
  import {
    sortOutlineItems,
    type OutlineSortKey,
    type SortDirection,
  } from '$lib/stores/binder/folderViews';

  interface Props {
    parent: BinderItem;
    items: BinderItem[];
    selectedId: string | null;
    onSelect: (id: string) => void;
  }

  let { parent, items, selectedId, onSelect }: Props = $props();

  let sortKey = $state<OutlineSortKey>('position');
  let sortDirection = $state<SortDirection>('asc');
  let titleWidth = $state(28);
  let synopsisWidth = $state(44);
  let statusWidth = $state(18);

  const sortedItems = $derived(sortOutlineItems(items, sortKey, sortDirection));
  const gridTemplate = $derived(
    `${titleWidth}rem minmax(${synopsisWidth}rem, 1fr) ${statusWidth}rem 10rem`,
  );

  function toggleSort(nextKey: OutlineSortKey) {
    if (sortKey === nextKey) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
      return;
    }
    sortKey = nextKey;
    sortDirection = 'asc';
  }

  function sortLabel(key: OutlineSortKey) {
    if (sortKey !== key) return '';
    return sortDirection === 'asc' ? '↑' : '↓';
  }
</script>

<section aria-label="Esquema" class="grid gap-6">
  <div class="flex flex-wrap items-end justify-between gap-3">
    <h2 class="text-xl font-semibold text-ink-900">{parent.title}</h2>
    <div class="grid gap-2 text-xs text-ink-500 sm:grid-cols-3">
      <label class="grid gap-1">
        <span>Título</span>
        <input type="range" min="18" max="40" bind:value={titleWidth} />
      </label>
      <label class="grid gap-1">
        <span>Sinopsis</span>
        <input type="range" min="28" max="64" bind:value={synopsisWidth} />
      </label>
      <label class="grid gap-1">
        <span>Estado</span>
        <input type="range" min="10" max="28" bind:value={statusWidth} />
      </label>
    </div>
  </div>

  <div class="overflow-auto rounded-xl border border-ink-100 bg-white">
    <div class="min-w-[56rem]">
      <div
        class="grid border-b border-ink-100 bg-white text-xs font-semibold uppercase text-ink-500"
        style={`grid-template-columns: ${gridTemplate}`}
      >
        <button
          type="button"
          class="px-3 py-2 text-left hover:bg-white"
          onclick={() => toggleSort('title')}
        >
          Título {sortLabel('title')}
        </button>
        <button
          type="button"
          class="px-3 py-2 text-left hover:bg-white"
          onclick={() => toggleSort('position')}
        >
          Sinopsis {sortLabel('position')}
        </button>
        <button
          type="button"
          class="px-3 py-2 text-left hover:bg-white"
          onclick={() => toggleSort('status')}
        >
          Estado {sortLabel('status')}
        </button>
        <button
          type="button"
          class="px-3 py-2 text-left hover:bg-white"
          onclick={() => toggleSort('updatedAt')}
        >
          Actualizado {sortLabel('updatedAt')}
        </button>
      </div>

      {#if sortedItems.length > 0}
        {#each sortedItems as item (item.id)}
          <button
            type="button"
            class="grid w-full border-b border-ink-100 text-left text-sm last:border-b-0 hover:bg-accent-50"
            class:bg-accent-50={selectedId === item.id}
            style={`grid-template-columns: ${gridTemplate}`}
            onclick={() => onSelect(item.id)}
          >
            <span class="truncate px-3 py-3 font-medium">{item.title}</span>
            <span class="line-clamp-2 px-3 py-3 leading-6 text-ink-700">
              {item.synopsis || 'Sin sinopsis'}
            </span>
            <span class="truncate px-3 py-3 text-ink-500">
              {item.status ?? 'Sin estado'}
            </span>
            <span class="truncate px-3 py-3 text-ink-500">
              {new Date(item.updatedAt).toLocaleDateString()}
            </span>
          </button>
        {/each}
      {:else}
        <p class="px-4 py-8 text-sm text-ink-500">
          Esta carpeta aún no tiene elementos.
        </p>
      {/if}
    </div>
  </div>
</section>
