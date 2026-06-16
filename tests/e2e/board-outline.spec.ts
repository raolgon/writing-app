import { expect, test } from '@playwright/test';

test('supports folder board quick edits, scene creation, and card reorder', async ({
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

    const projectId = '018fe3c4-5678-7abc-9def-0123456789ac';
    const sessionId = '018fe3c4-5678-7abc-9def-0123456789ad';
    const folderId = '018fe3c4-5678-7abc-9def-0123456789ae';
    const sceneAId = '018fe3c4-5678-7abc-9def-0123456789af';
    const sceneBId = '018fe3c4-5678-7abc-9def-0123456789b0';
    const createdAt = new Date().toISOString();
    const documents = new Map<string, DocumentRecord>();
    const binderItems: BinderItem[] = [
      {
        id: folderId,
        projectId,
        parentId: null,
        itemType: 'folder',
        title: 'Capítulo 1',
        synopsis: '',
        position: 0,
        icon: null,
        colorLabel: null,
        status: null,
        createdAt,
        updatedAt: createdAt,
        isExpanded: true,
        isArchived: false,
        trashedAt: null,
      },
      {
        id: sceneAId,
        projectId,
        parentId: folderId,
        itemType: 'document',
        title: 'Escena A',
        synopsis: 'Apertura',
        position: 0,
        icon: null,
        colorLabel: null,
        status: 'Borrador',
        createdAt,
        updatedAt: createdAt,
        isExpanded: false,
        isArchived: false,
        trashedAt: null,
      },
      {
        id: sceneBId,
        projectId,
        parentId: folderId,
        itemType: 'document',
        title: 'Escena B',
        synopsis: 'Giro',
        position: 1,
        icon: null,
        colorLabel: null,
        status: 'Pendiente',
        createdAt,
        updatedAt: createdAt,
        isExpanded: false,
        isArchived: false,
        trashedAt: null,
      },
    ];

    function now() {
      return new Date().toISOString();
    }

    function createDocument(binderItemId: string): DocumentRecord {
      const document = {
        id: crypto.randomUUID(),
        binderItemId,
        contentJson: { type: 'doc', content: [{ type: 'paragraph' }] },
        contentPlainText: '',
        wordCount: 0,
        characterCount: 0,
        revision: 0,
        createdAt: now(),
        updatedAt: now(),
      };
      documents.set(binderItemId, document);
      return document;
    }

    function sortSiblings(parentId: string | null) {
      return binderItems
        .filter((item) => item.parentId === parentId && item.trashedAt === null)
        .sort((left, right) => left.position - right.position);
    }

    function normalizePositions(parentId: string | null) {
      sortSiblings(parentId).forEach((item, index) => {
        item.position = index;
      });
    }

    function moveItem(
      itemId: string,
      parentId: string | null,
      targetPosition: number,
    ) {
      const item = binderItems.find((candidate) => candidate.id === itemId);
      if (!item) return;
      const oldParentId = item.parentId;
      item.parentId = parentId;
      item.position = targetPosition - 0.5;
      normalizePositions(oldParentId);
      normalizePositions(parentId);
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
        if (command === 'create_default_project') {
          return {
            sessionId,
            project: {
              id: projectId,
              title: 'Borrador',
              description: '',
              projectType: 'blank',
              path: '/tmp/draft',
              formatVersion: 1,
              databaseSchemaVersion: 1,
              lastOpenedAt: createdAt,
            },
          };
        }

        if (command === 'list_binder_items') {
          return binderItems
            .filter((item) => item.trashedAt === null)
            .sort((left, right) => left.position - right.position);
        }

        if (command === 'get_document') {
          const binderItemId = request.binderItemId as string;
          return documents.get(binderItemId) ?? createDocument(binderItemId);
        }

        if (command === 'get_document_recovery') return null;
        if (command === 'clear_document_recovery') return null;

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

        if (command === 'create_binder_item') {
          const parentId = (request.parentId as string | null) ?? null;
          const item: BinderItem = {
            id: crypto.randomUUID(),
            projectId,
            parentId,
            itemType: request.itemType as string,
            title: request.title as string,
            synopsis: '',
            position: sortSiblings(parentId).length,
            icon: null,
            colorLabel: null,
            status: null,
            createdAt: now(),
            updatedAt: now(),
            isExpanded: false,
            isArchived: false,
            trashedAt: null,
          };
          binderItems.push(item);
          if (item.itemType !== 'folder') createDocument(item.id);
          return item;
        }

        if (command === 'save_binder_synopsis') {
          const item = binderItems.find(
            (candidate) => candidate.id === request.itemId,
          );
          if (!item) throw new Error('missing item');
          item.synopsis = request.synopsis as string;
          item.updatedAt = now();
          return item;
        }

        if (command === 'move_binder_item') {
          moveItem(
            request.itemId as string,
            (request.parentId as string | null) ?? null,
            request.position as number,
          );
          return binderItems.filter((item) => item.trashedAt === null);
        }

        if (command === 'set_binder_item_expanded') {
          return binderItems.find((item) => item.id === request.itemId);
        }

        throw new Error(`Unhandled command ${command}`);
      },
      transformCallback: () => 1,
      unregisterCallback: () => undefined,
    };
  });

  await page.goto('/');
  await page.getByRole('button', { name: 'Capítulo 1' }).click();
  await expect(page.getByRole('tab', { name: 'Tablero' })).toHaveAttribute(
    'aria-selected',
    'true',
  );

  await page.getByText('Apertura').click();
  await page.getByLabel('Sinopsis de Escena A').fill('Apertura revisada');
  await page.getByRole('button', { name: '+ Escena' }).focus();
  await expect(page.getByText('Apertura revisada')).toBeVisible();

  await page.getByRole('button', { name: '+ Escena' }).click();
  await expect(page.getByRole('heading', { name: 'Sin título' })).toBeVisible();

  await page.getByRole('button', { name: 'Capítulo 1' }).click();
  await page
    .locator('article')
    .filter({ hasText: 'Escena B' })
    .dragTo(page.locator('article').filter({ hasText: 'Escena A' }));
  await expect(page.locator('article h3').first()).toContainText('Escena B');
});

test('sorts outline rows by clicked column', async ({ page }) => {
  await page.addInitScript(() => {
    const projectId = '018fe3c4-5678-7abc-9def-0123456789ac';
    const sessionId = '018fe3c4-5678-7abc-9def-0123456789ad';
    const folderId = '018fe3c4-5678-7abc-9def-0123456789ae';
    const createdAt = new Date().toISOString();
    const binderItems = [
      {
        id: folderId,
        projectId,
        parentId: null,
        itemType: 'folder',
        title: 'Capítulo 1',
        synopsis: '',
        position: 0,
        icon: null,
        colorLabel: null,
        status: null,
        createdAt,
        updatedAt: createdAt,
        isExpanded: true,
        isArchived: false,
        trashedAt: null,
      },
      {
        id: '018fe3c4-5678-7abc-9def-0123456789af',
        projectId,
        parentId: folderId,
        itemType: 'document',
        title: 'Beta',
        synopsis: 'Segunda',
        position: 0,
        icon: null,
        colorLabel: null,
        status: null,
        createdAt,
        updatedAt: createdAt,
        isExpanded: false,
        isArchived: false,
        trashedAt: null,
      },
      {
        id: '018fe3c4-5678-7abc-9def-0123456789b0',
        projectId,
        parentId: folderId,
        itemType: 'document',
        title: 'Alpha',
        synopsis: 'Primera',
        position: 1,
        icon: null,
        colorLabel: null,
        status: null,
        createdAt,
        updatedAt: createdAt,
        isExpanded: false,
        isArchived: false,
        trashedAt: null,
      },
    ];

    const tauriWindow = window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (command: string) => Promise<unknown>;
        transformCallback: () => number;
        unregisterCallback: () => undefined;
      };
    };

    tauriWindow.__TAURI_INTERNALS__ = {
      invoke: async (command: string) => {
        if (command === 'create_default_project') {
          return {
            sessionId,
            project: {
              id: projectId,
              title: 'Borrador',
              description: '',
              projectType: 'blank',
              path: '/tmp/draft',
              formatVersion: 1,
              databaseSchemaVersion: 1,
              lastOpenedAt: createdAt,
            },
          };
        }
        if (command === 'list_binder_items') return binderItems;
        if (command === 'get_document') {
          return {
            id: crypto.randomUUID(),
            binderItemId: '018fe3c4-5678-7abc-9def-0123456789af',
            contentJson: { type: 'doc', content: [{ type: 'paragraph' }] },
            contentPlainText: '',
            wordCount: 0,
            characterCount: 0,
            revision: 0,
            createdAt,
            updatedAt: createdAt,
          };
        }
        if (command === 'get_document_recovery') return null;
        if (command === 'get_inspector_data') {
          return { metadata: null, notes: [], snapshots: [] };
        }
        throw new Error(`Unhandled command ${command}`);
      },
      transformCallback: () => 1,
      unregisterCallback: () => undefined,
    };
  });

  await page.goto('/');
  await page.getByRole('button', { name: 'Capítulo 1' }).click();
  await page.getByRole('tab', { name: 'Esquema' }).click();
  await page.getByRole('button', { name: /Título/ }).click();

  await expect(
    page.locator('[aria-label="Esquema"] button').nth(4),
  ).toContainText('Alpha');
});
