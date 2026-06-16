import { expect, test } from '@playwright/test';

test('opens an existing project from the project title control', async ({
  page,
}) => {
  await page.addInitScript(() => {
    const defaultProjectId = crypto.randomUUID();
    const reopenedProjectId = crypto.randomUUID();
    const defaultSessionId = crypto.randomUUID();
    const reopenedSessionId = crypto.randomUUID();
    const defaultBinderItemId = crypto.randomUUID();
    const reopenedBinderItemId = crypto.randomUUID();
    const createdAt = new Date().toISOString();
    let activeProject: 'default' | 'reopened' = 'default';

    window.prompt = () => '/tmp/reopened-project';

    function binderItem(projectId: string, id: string, title: string) {
      return {
        id,
        projectId,
        parentId: null,
        itemType: 'document',
        title,
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
    }

    function documentRecord(binderItemId: string, text: string) {
      return {
        id: crypto.randomUUID(),
        binderItemId,
        contentJson: {
          type: 'doc',
          content: [
            {
              type: 'paragraph',
              content: [{ type: 'text', text }],
            },
          ],
        },
        contentPlainText: text,
        wordCount: text.trim().split(/\s+/).filter(Boolean).length,
        characterCount: text.length,
        revision: 0,
        createdAt,
        updatedAt: createdAt,
      };
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
          activeProject = 'default';
          return {
            sessionId: defaultSessionId,
            project: {
              id: defaultProjectId,
              title: 'Borrador',
              description: '',
              projectType: 'blank',
              path: '/tmp/default',
              formatVersion: 1,
              databaseSchemaVersion: 2,
              lastOpenedAt: createdAt,
            },
          };
        }

        if (command === 'open_project') {
          if (request.folderPath !== '/tmp/reopened-project') {
            throw new Error(`unexpected folder path ${request.folderPath}`);
          }
          activeProject = 'reopened';
          return {
            sessionId: reopenedSessionId,
            project: {
              id: reopenedProjectId,
              title: 'Proyecto abierto',
              description: '',
              projectType: 'blank',
              path: '/tmp/reopened-project',
              formatVersion: 1,
              databaseSchemaVersion: 2,
              lastOpenedAt: createdAt,
            },
          };
        }

        if (command === 'close_project') return { closed: true };

        if (command === 'list_binder_items') {
          return activeProject === 'reopened'
            ? [
                binderItem(
                  reopenedProjectId,
                  reopenedBinderItemId,
                  'Capítulo abierto',
                ),
              ]
            : [binderItem(defaultProjectId, defaultBinderItemId, 'Sin título')];
        }

        if (command === 'get_document') {
          const binderItemId = request.binderItemId as string;
          return binderItemId === reopenedBinderItemId
            ? documentRecord(reopenedBinderItemId, 'Texto reabierto')
            : documentRecord(defaultBinderItemId, '');
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
  await expect(page.getByLabel('Editor de documento')).toBeFocused();

  await page.getByRole('button', { name: 'Abrir proyecto' }).click();

  await expect(
    page.getByRole('heading', { name: 'Proyecto abierto' }),
  ).toBeVisible();
  await expect(
    page.getByRole('heading', { name: 'Capítulo abierto' }),
  ).toBeVisible();
  await expect(page.getByText('Texto reabierto')).toBeVisible();
});
