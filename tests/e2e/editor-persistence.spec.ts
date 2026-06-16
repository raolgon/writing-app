import { expect, test } from '@playwright/test';

test('creates a document, writes text, switches away, and reloads saved content', async ({
  page,
}) => {
  await page.addInitScript(() => {
    type BinderItem = {
      id: string;
      projectId: string;
      parentId: string | null;
      itemType: string;
      title: string;
      synopsis: string;
      position: number;
      icon: string | null;
      colorLabel: string | null;
      status: string | null;
      createdAt: string;
      updatedAt: string;
      isExpanded: boolean;
      isArchived: boolean;
      trashedAt: string | null;
    };

    type DocumentRecord = {
      id: string;
      binderItemId: string;
      contentJson: unknown;
      contentPlainText: string;
      wordCount: number;
      characterCount: number;
      revision: number;
      createdAt: string;
      updatedAt: string;
    };

    const projectId = crypto.randomUUID();
    let sessionId = crypto.randomUUID();
    const binderItems: BinderItem[] = [];
    const documents = new Map<string, DocumentRecord>();
    const recovery = new Map<string, unknown>();

    function now() {
      return new Date().toISOString();
    }

    function countWords(text: string) {
      return text.trim().split(/\s+/).filter(Boolean).length;
    }

    function createDocument(binderItemId: string): DocumentRecord {
      const createdAt = now();
      const document = {
        id: crypto.randomUUID(),
        binderItemId,
        contentJson: {
          type: 'doc',
          content: [{ type: 'paragraph' }],
        },
        contentPlainText: '',
        wordCount: 0,
        characterCount: 0,
        revision: 0,
        createdAt,
        updatedAt: createdAt,
      };
      documents.set(binderItemId, document);
      return document;
    }

    const tauriWindow = window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (
          command: string,
          args: { request?: Record<string, unknown> },
        ) => Promise<unknown>;
        transformCallback: () => number;
        unregisterCallback: () => undefined;
      };
    };

    tauriWindow.__TAURI_INTERNALS__ = {
      invoke: async (
        command: string,
        args: { request?: Record<string, unknown> },
      ) => {
        const request = args.request ?? {};
        if (
          command === 'create_project' ||
          command === 'create_default_project' ||
          command === 'open_project'
        ) {
          sessionId = crypto.randomUUID();
          return {
            sessionId,
            project: {
              id: projectId,
              title: request.title ?? 'Draft',
              description: '',
              projectType: request.projectType ?? 'blank',
              path: request.folderPath ?? '/tmp/draft',
              formatVersion: 1,
              databaseSchemaVersion: 1,
              lastOpenedAt: now(),
            },
          };
        }

        if (command === 'list_binder_items') {
          return binderItems.filter((item) => item.trashedAt === null);
        }

        if (command === 'create_binder_item') {
          const createdAt = now();
          const item: BinderItem = {
            id: crypto.randomUUID(),
            projectId,
            parentId: (request.parentId as string | null) ?? null,
            itemType: request.itemType as string,
            title: request.title as string,
            synopsis: '',
            position: binderItems.length,
            icon: null,
            colorLabel: null,
            status: null,
            createdAt,
            updatedAt: createdAt,
            isExpanded: false,
            isArchived: false,
            trashedAt: null,
          };
          binderItems.push(item);
          if (item.itemType !== 'folder') {
            createDocument(item.id);
          }
          return item;
        }

        if (command === 'get_document') {
          const binderItemId = request.binderItemId as string;
          return documents.get(binderItemId) ?? createDocument(binderItemId);
        }

        if (command === 'save_document') {
          const binderItemId = request.binderItemId as string;
          const existing =
            documents.get(binderItemId) ?? createDocument(binderItemId);
          const contentPlainText = request.contentPlainText as string;
          const saved: DocumentRecord = {
            ...existing,
            contentJson: request.contentJson,
            contentPlainText,
            wordCount: countWords(contentPlainText),
            characterCount: contentPlainText.length,
            revision: existing.revision + 1,
            updatedAt: now(),
          };
          documents.set(binderItemId, saved);
          recovery.delete(binderItemId);
          return saved;
        }

        if (command === 'record_document_recovery') {
          recovery.set(request.binderItemId as string, request);
          return {
            binderItemId: request.binderItemId,
            contentJson: request.contentJson,
            contentPlainText: request.contentPlainText,
            revision: request.revision,
            updatedAt: now(),
          };
        }

        if (command === 'get_document_recovery') {
          return recovery.get(request.binderItemId as string) ?? null;
        }

        if (command === 'get_inspector_data') {
          const binderItemId = request.binderItemId as string;
          const document =
            documents.get(binderItemId) ?? createDocument(binderItemId);
          return {
            metadata: {
              documentId: document.id,
              label: null,
              status: null,
              targetWordCount: null,
              keywords: [],
              customFields: {},
              includeInExport: true,
            },
            notes: [],
            snapshots: [],
          };
        }

        if (command === 'clear_document_recovery') {
          recovery.delete(request.binderItemId as string);
          return null;
        }

        if (command === 'set_binder_item_expanded') {
          return binderItems.find((item) => item.id === request.itemId);
        }

        if (command === 'trash_binder_item') {
          return binderItems.filter((item) => item.id !== request.itemId);
        }

        if (command === 'duplicate_binder_item') {
          return binderItems.find((item) => item.id === request.itemId);
        }

        throw new Error(`Unhandled command ${command}`);
      },
      transformCallback: () => 1,
      unregisterCallback: () => undefined,
    };
  });

  await page.goto('/');

  const editor = page.getByLabel('Editor de documento');
  await expect(editor).toBeFocused();
  await page.keyboard.type('Texto persistente de prueba');
  await expect(page.getByText('4 palabras')).toBeVisible();
  await expect(
    page.locator('footer').getByText('Guardado localmente'),
  ).toBeVisible({
    timeout: 5000,
  });

  await page.getByRole('button', { name: 'Nuevo documento' }).click();
  await page.getByRole('button', { name: 'Sin título', exact: true }).click();

  await expect(page.getByText('Texto persistente de prueba')).toBeVisible();
});
