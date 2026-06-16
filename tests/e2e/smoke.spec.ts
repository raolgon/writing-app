import { expect, test } from '@playwright/test';

test('shows the Phase 2 application shell', async ({ page }) => {
  await page.addInitScript(() => {
    const projectId = crypto.randomUUID();
    const documentId = crypto.randomUUID();
    const binderItemId = crypto.randomUUID();
    const createdAt = new Date().toISOString();
    const binderItem = {
      id: binderItemId,
      projectId,
      parentId: null,
      itemType: 'document',
      title: 'Sin título',
      synopsis: '',
      position: 0,
      icon: null,
      colorLabel: null,
      status: null,
      createdAt,
      updatedAt: createdAt,
      isExpanded: false,
      isArchived: false,
      trashedAt: null,
    };
    const document = {
      id: documentId,
      binderItemId,
      contentJson: { type: 'doc', content: [{ type: 'paragraph' }] },
      contentPlainText: '',
      wordCount: 0,
      characterCount: 0,
      revision: 0,
      createdAt,
      updatedAt: createdAt,
    };

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
      invoke: async (command: string) => {
        if (command === 'create_default_project') {
          return {
            sessionId: crypto.randomUUID(),
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

        if (command === 'list_binder_items') return [binderItem];
        if (command === 'get_document') return document;
        if (command === 'get_document_recovery') return null;
        if (command === 'get_inspector_data') {
          return {
            metadata: {
              documentId,
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

        throw new Error(`Unhandled command ${command}`);
      },
      transformCallback: () => 1,
      unregisterCallback: () => undefined,
    };
  });

  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'Borrador' })).toBeVisible();
  await expect(page.getByLabel('Binder')).toBeVisible();
  await expect(page.getByRole('region', { name: 'Editor' })).toBeVisible();
  await expect(page.getByLabel('Inspector')).toBeVisible();
  await expect(page.getByLabel('Editor de documento')).toBeFocused();
});
