<script lang="ts">
  import { onMount } from 'svelte';
  import { Editor, type JSONContent } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import Underline from '@tiptap/extension-underline';
  import Link from '@tiptap/extension-link';

  interface Props {
    content: unknown;
    editable?: boolean;
    autoFocus?: boolean;
    showToolbar?: boolean;
    fontSize: number;
    lineHeight: number;
    onChange: (contentJson: JSONContent, contentPlainText: string) => void;
  }

  let {
    content,
    editable = true,
    autoFocus = false,
    showToolbar = false,
    fontSize,
    lineHeight,
    onChange,
  }: Props = $props();

  let editorElement = $state<HTMLDivElement | null>(null);
  let editor = $state<Editor | null>(null);
  let updateToken = $state(0);

  onMount(() => {
    if (!editorElement) return;

    const instance = new Editor({
      element: editorElement,
      extensions: [
        StarterKit,
        Underline,
        Link.configure({
          openOnClick: false,
          autolink: true,
        }),
      ],
      content: normalizeContent(content),
      editable,
      editorProps: {
        attributes: {
          class:
            'min-h-full max-w-none font-serif outline-none prose prose-neutral prose-p:text-ink-900 prose-p:leading-relaxed prose-headings:text-ink-900',
          'aria-label': 'Editor de documento',
        },
      },
      onUpdate: ({ editor }) => {
        updateToken += 1;
        onChange(editor.getJSON(), editor.getText());
      },
      onSelectionUpdate: () => {
        updateToken += 1;
      },
      onTransaction: () => {
        updateToken += 1;
      },
    });

    editor = instance;

    if (autoFocus) {
      requestAnimationFrame(() => {
        instance.commands.focus('end');
      });
    }

    return () => {
      instance.destroy();
      editor = null;
    };
  });

  function normalizeContent(value: unknown): JSONContent {
    if (isJsonObject(value) && value.type === 'doc') {
      return value as JSONContent;
    }

    return {
      type: 'doc',
      content: [{ type: 'paragraph' }],
    };
  }

  function isJsonObject(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null && !Array.isArray(value);
  }

  function runCommand(command: () => boolean) {
    command();
    updateToken += 1;
  }

  function setLink() {
    if (!editor) return;
    const previousUrl = editor.getAttributes('link').href as string | undefined;
    const url = window.prompt('URL', previousUrl ?? '');
    if (url === null) return;

    if (url.trim() === '') {
      runCommand(() => editor?.chain().focus().unsetLink().run() ?? false);
      return;
    }

    runCommand(
      () =>
        editor
          ?.chain()
          .focus()
          .extendMarkRange('link')
          .setLink({ href: url })
          .run() ?? false,
    );
  }

  function handleEditorKeydown(event: KeyboardEvent) {
    if (!editor?.isFocused) return;
    const modifier = event.metaKey || event.ctrlKey;
    if (modifier && event.key.toLowerCase() === 'k') {
      event.preventDefault();
      setLink();
    }
  }
</script>

<svelte:window onkeydown={handleEditorKeydown} />

{#if showToolbar}
  <div
    class="mb-4 flex flex-wrap gap-1 border-b border-ink-100 pb-3"
    aria-label="Formato"
  >
    <button
      type="button"
      class:bg-ink-100={editor?.isActive('bold')}
      class="h-8 min-w-8 rounded px-2 font-semibold hover:bg-ink-100"
      title="Negrita"
      aria-label="Negrita"
      onclick={() =>
        runCommand(() => editor?.chain().focus().toggleBold().run() ?? false)}
    >
      B
    </button>
    <button
      type="button"
      class:bg-ink-100={editor?.isActive('italic')}
      class="h-8 min-w-8 rounded px-2 italic hover:bg-ink-100"
      title="Cursiva"
      aria-label="Cursiva"
      onclick={() =>
        runCommand(() => editor?.chain().focus().toggleItalic().run() ?? false)}
    >
      I
    </button>
    <button
      type="button"
      class:bg-ink-100={editor?.isActive('underline')}
      class="h-8 min-w-8 rounded px-2 underline hover:bg-ink-100"
      title="Subrayado"
      aria-label="Subrayado"
      onclick={() =>
        runCommand(
          () => editor?.chain().focus().toggleUnderline().run() ?? false,
        )}
    >
      U
    </button>
    <button
      type="button"
      class:bg-ink-100={editor?.isActive('strike')}
      class="h-8 min-w-8 rounded px-2 line-through hover:bg-ink-100"
      title="Tachado"
      aria-label="Tachado"
      onclick={() =>
        runCommand(() => editor?.chain().focus().toggleStrike().run() ?? false)}
    >
      S
    </button>
    <button
      type="button"
      class:bg-ink-100={editor?.isActive('heading', { level: 2 })}
      class="h-8 min-w-8 rounded px-2 hover:bg-ink-100"
      title="Encabezado"
      aria-label="Encabezado"
      onclick={() =>
        runCommand(
          () =>
            editor?.chain().focus().toggleHeading({ level: 2 }).run() ?? false,
        )}
    >
      H
    </button>
    <button
      type="button"
      class:bg-ink-100={editor?.isActive('bulletList')}
      class="h-8 min-w-8 rounded px-2 hover:bg-ink-100"
      title="Lista"
      aria-label="Lista"
      onclick={() =>
        runCommand(
          () => editor?.chain().focus().toggleBulletList().run() ?? false,
        )}
    >
      •
    </button>
    <button
      type="button"
      class:bg-ink-100={editor?.isActive('orderedList')}
      class="h-8 min-w-8 rounded px-2 hover:bg-ink-100"
      title="Lista numerada"
      aria-label="Lista numerada"
      onclick={() =>
        runCommand(
          () => editor?.chain().focus().toggleOrderedList().run() ?? false,
        )}
    >
      1.
    </button>
    <button
      type="button"
      class:bg-ink-100={editor?.isActive('blockquote')}
      class="h-8 min-w-8 rounded px-2 hover:bg-ink-100"
      title="Cita"
      aria-label="Cita"
      onclick={() =>
        runCommand(
          () => editor?.chain().focus().toggleBlockquote().run() ?? false,
        )}
    >
      ”
    </button>
    <button
      type="button"
      class="h-8 min-w-8 rounded px-2 hover:bg-ink-100"
      title="Separador"
      aria-label="Separador"
      onclick={() =>
        runCommand(
          () => editor?.chain().focus().setHorizontalRule().run() ?? false,
        )}
    >
      —
    </button>
    <button
      type="button"
      class:bg-ink-100={editor?.isActive('link')}
      class="h-8 min-w-8 rounded px-2 hover:bg-ink-100"
      title="Enlace"
      aria-label="Enlace"
      onclick={setLink}
    >
      ↗
    </button>
    <button
      type="button"
      class="h-8 min-w-8 rounded px-2 hover:bg-ink-100"
      title="Deshacer"
      aria-label="Deshacer"
      onclick={() =>
        runCommand(() => editor?.chain().focus().undo().run() ?? false)}
    >
      ↶
    </button>
    <button
      type="button"
      class="h-8 min-w-8 rounded px-2 hover:bg-ink-100"
      title="Rehacer"
      aria-label="Rehacer"
      onclick={() =>
        runCommand(() => editor?.chain().focus().redo().run() ?? false)}
    >
      ↷
    </button>
  </div>
{/if}

{#key updateToken}
  <div class="sr-only" aria-live="polite">Editor actualizado {updateToken}</div>
{/key}

<div
  class="h-full min-h-[calc(100vh-12rem)]"
  role="group"
  aria-label="Editor enriquecido"
>
  <div
    bind:this={editorElement}
    class="h-full min-h-[inherit]"
    style={`font-size: ${fontSize}px; line-height: ${lineHeight}`}
  ></div>
</div>
