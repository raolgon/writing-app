import { expect, test } from '@playwright/test';

test('searches project text, previews snippets, and opens a result', async ({
  page,
}) => {
  await page.addInitScript(() => {
    const projectId = '018fe3c4-5678-7abc-9def-0123456789ac';
    const sessionId = '018fe3c4-5678-7abc-9def-0123456789ad';
    const binderItemId = '018fe3c4-5678-7abc-9def-0123456789ae';
    const documentId = '018fe3c4-5678-7abc-9def-0123456789af';
    const createdAt = new Date().toISOString();
    const binderItem = {
      id: binderItemId,
      projectId,
      parentId: null,
      itemType: 'document',
      title: 'Capítulo 3',
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
      contentJson: {
        type: 'doc',
        content: [
          {
            type: 'paragraph',
            content: [
              {
                type: 'text',
                text: 'Entraron en la casa abandonada antes del amanecer.',
              },
            ],
          },
        ],
      },
      contentPlainText: 'Entraron en la casa abandonada antes del amanecer.',
      wordCount: 8,
      characterCount: 53,
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
            sessionId,
            project: {
              id: projectId,
              title: 'Borrador',
              description: '',
              projectType: 'blank',
              path: '/tmp/draft',
              formatVersion: 1,
              databaseSchemaVersion: 2,
              lastOpenedAt: createdAt,
            },
          };
        }
        if (command === 'list_binder_items') return [binderItem];
        if (command === 'get_document') return document;
        if (command === 'get_document_recovery') return null;
        if (command === 'get_inspector_data') {
          return { metadata: null, notes: [], snapshots: [] };
        }
        if (command === 'search_project') {
          return [
            {
              binderItemId,
              title: 'Capítulo 3',
              itemType: 'document',
              path: ['Manuscrito', 'Capítulo 3'],
              snippet: 'Entraron en la casa abandonada antes del amanecer.',
              updatedAt: createdAt,
            },
          ];
        }
        throw new Error(`Unhandled command ${command}`);
      },
      transformCallback: () => 1,
      unregisterCallback: () => undefined,
    };
  });

  await page.goto('/');
  await expect(page.getByLabel('Editor de documento')).toBeFocused();
  await page.getByRole('button', { name: 'Buscar' }).click();
  await page
    .getByLabel('Buscar')
    .getByPlaceholder('Buscar en el proyecto')
    .fill('casa');

  await expect(
    page
      .getByLabel('Resultados de búsqueda')
      .getByText('Manuscrito / Capítulo 3'),
  ).toBeVisible();
  await expect(page.locator('mark')).toContainText('casa');

  await page.getByRole('button', { name: 'Abrir resultado' }).click();
  await expect(page.getByRole('region', { name: 'Editor' })).toBeVisible();
  await expect(page.getByText('Resultado abierto: “casa”')).toBeVisible();
});

test('exports the project as a local file', async ({ page }) => {
  await page.addInitScript(() => {
    const projectId = '018fe3c4-5678-7abc-9def-0123456789ac';
    const sessionId = '018fe3c4-5678-7abc-9def-0123456789ad';
    const binderItemId = '018fe3c4-5678-7abc-9def-0123456789ae';
    const documentId = '018fe3c4-5678-7abc-9def-0123456789af';
    const createdAt = new Date().toISOString();
    const binderItem = {
      id: binderItemId,
      projectId,
      parentId: null,
      itemType: 'document',
      title: 'Capítulo 1',
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
      contentPlainText: 'Texto exportable',
      wordCount: 2,
      characterCount: 16,
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
              databaseSchemaVersion: 2,
              lastOpenedAt: createdAt,
            },
          };
        }
        if (command === 'list_binder_items') return [binderItem];
        if (command === 'get_document') return document;
        if (command === 'get_document_recovery') return null;
        if (command === 'get_inspector_data') {
          return { metadata: null, notes: [], snapshots: [] };
        }
        if (command === 'list_backups') return [];
        if (command === 'create_backup') {
          return {
            id: crypto.randomUUID(),
            createdAt,
            path: 'backups/test',
            kind: 'manual',
            formatVersion: 1,
            sizeBytes: 2048,
            status: 'complete',
          };
        }
        if (command === 'export_project') {
          return {
            fileName: request.format === 'txt' ? 'Borrador.txt' : 'Borrador.md',
            mimeType: request.format === 'txt' ? 'text/plain' : 'text/markdown',
            content: 'Capítulo 1\n\nTexto exportable',
          };
        }
        throw new Error(`Unhandled command ${command}`);
      },
      transformCallback: () => 1,
      unregisterCallback: () => undefined,
    };
  });

  await page.goto('/');
  await expect(page.getByLabel('Editor de documento')).toBeFocused();
  await page.getByRole('button', { name: 'Exportar' }).click();
  await page.getByRole('button', { name: 'Crear copia' }).click();
  await expect(page.getByText('Copia de seguridad creada')).toBeVisible();
  await page.getByText('Texto plano').click();

  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Exportar archivo' }).click();
  const download = await downloadPromise;

  expect(download.suggestedFilename()).toBe('Borrador.txt');
  await expect(page.getByText('Borrador.txt preparado')).toBeVisible();
});
