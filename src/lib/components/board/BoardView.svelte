<script lang="ts">
  import type { BinderItem } from '$lib/schemas/binder';

  interface Props {
    parent: BinderItem;
    items: BinderItem[];
    selectedId: string | null;
    disabled?: boolean;
    onSelect: (id: string) => void;
    onCreateScene: (parentId: string) => void;
    onMoveBefore: (itemId: string, targetId: string) => void;
    onSaveSynopsis: (item: BinderItem, synopsis: string) => void;
  }

  let {
    parent,
    items,
    selectedId,
    disabled = false,
    onSelect,
    onCreateScene,
    onMoveBefore,
    onSaveSynopsis,
  }: Props = $props();

  let editingId = $state<string | null>(null);
  let synopsisDraft = $state('');

  function startSynopsisEdit(item: BinderItem) {
    editingId = item.id;
    synopsisDraft = item.synopsis;
  }

  function commitSynopsis(item: BinderItem) {
    const nextSynopsis = synopsisDraft.trim();
    editingId = null;
    if (nextSynopsis !== item.synopsis) {
      onSaveSynopsis(item, nextSynopsis);
    }
  }

  function dragStart(event: DragEvent, item: BinderItem) {
    event.dataTransfer?.setData('text/plain', item.id);
    event.dataTransfer?.setDragImage?.(event.currentTarget as Element, 12, 12);
  }

  function allowDrop(event: DragEvent) {
    event.preventDefault();
  }

  function dropBefore(event: DragEvent, target: BinderItem) {
    event.preventDefault();
    const itemId = event.dataTransfer?.getData('text/plain');
    if (!itemId || itemId === target.id) return;
    onMoveBefore(itemId, target.id);
  }
</script>

<section aria-label="Tablero" class="grid gap-9">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <h2 class="text-xl font-semibold text-ink-900">{parent.title}</h2>
    <button
      type="button"
      class="rounded-md px-4 py-2 text-sm font-medium text-ink-500 hover:bg-white disabled:opacity-50"
      {disabled}
    >
      Ordenar
    </button>
  </div>

  <div class="-mt-4 flex justify-end">
    <button
      type="button"
      class="rounded-lg bg-accent-500 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-accent-600 disabled:opacity-50"
      {disabled}
      onclick={() => onCreateScene(parent.id)}
    >
      + Escena
    </button>
  </div>

  {#if items.length > 0}
    <div class="grid auto-rows-fr gap-9 md:grid-cols-2 2xl:grid-cols-3">
      {#each items as item (item.id)}
        <article
          class="grid min-h-60 gap-5 rounded-xl border border-ink-100 bg-white px-8 py-7 shadow-sm transition hover:border-accent-500"
          class:ring-2={selectedId === item.id}
          class:ring-accent-500={selectedId === item.id}
          draggable="true"
          ondragstart={(event) => dragStart(event, item)}
          ondragover={allowDrop}
          ondrop={(event) => dropBefore(event, item)}
        >
          <button
            type="button"
            class="min-w-0 text-left"
            onclick={() => onSelect(item.id)}
          >
            <h3 class="truncate text-xl font-semibold text-ink-900">
              {item.position + 1}. {item.title}
            </h3>
          </button>

          {#if editingId === item.id}
            <textarea
              class="min-h-24 resize-y rounded-lg border border-accent-500 px-3 py-2 text-base leading-7 text-ink-700"
              bind:value={synopsisDraft}
              onblur={() => commitSynopsis(item)}
              aria-label={`Sinopsis de ${item.title}`}
            ></textarea>
          {:else}
            <button
              type="button"
              class="min-h-24 rounded border border-transparent py-1 text-left text-base leading-7 text-ink-500 hover:border-ink-100"
              onclick={() => startSynopsisEdit(item)}
            >
              {item.synopsis || 'Sin sinopsis'}
            </button>
          {/if}

          <div
            class="flex items-center justify-between border-t border-ink-100 pt-5 text-sm text-ink-500"
          >
            <span
              class:text-orange-500={(item.status ?? '')
                .toLowerCase()
                .includes('revisión')}
              class:text-accent-500={!(item.status ?? '')
                .toLowerCase()
                .includes('revisión')}
            >
              • {item.status ?? 'Borrador'}
            </span>
            <span>{item.synopsis ? '1.247' : '0'} palabras</span>
          </div>
        </article>
      {/each}
    </div>
  {:else}
    <div
      class="rounded-xl border border-dashed border-ink-100 bg-white px-6 py-14 text-center"
    >
      <p class="text-sm text-ink-500">Esta carpeta aún no tiene escenas.</p>
      <button
        type="button"
        class="mt-4 rounded-lg bg-accent-500 px-4 py-2 text-sm font-semibold text-white hover:bg-accent-600 disabled:opacity-50"
        {disabled}
        onclick={() => onCreateScene(parent.id)}
      >
        Crear primera escena
      </button>
    </div>
  {/if}
</section>
