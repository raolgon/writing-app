import { invoke as tauriInvoke } from '@tauri-apps/api/core';

type CommandArgs = { request?: Record<string, unknown> };

type BrowserProject = {
  id: string;
  title: string;
  description: string;
  projectType: string;
  path: string;
  formatVersion: number;
  databaseSchemaVersion: number;
  lastOpenedAt: string | null;
};

type BrowserBinderItem = {
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

type BrowserDocument = {
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

type BrowserMetadata = {
  documentId: string;
  label: string | null;
  status: string | null;
  targetWordCount: number | null;
  keywords: string[];
  customFields: unknown;
  includeInExport: boolean;
};

type BrowserNote = {
  id: string;
  projectId: string;
  binderItemId: string | null;
  title: string;
  content: string;
  createdAt: string;
  updatedAt: string;
};

type BrowserSnapshot = {
  id: string;
  documentId: string;
  name: string;
  contentJson: unknown;
  contentPlainText: string;
  createdAt: string;
};

type BrowserRecovery = {
  binderItemId: string;
  contentJson: unknown;
  contentPlainText: string;
  revision: number;
  updatedAt: string;
};

type BrowserBackupRecord = {
  id: string;
  createdAt: string;
  path: string;
  kind: 'automatic' | 'manual';
  formatVersion: number;
  sizeBytes: number | null;
  status: string;
};

type BrowserExportDocument = {
  binderItemId: string;
  title: string;
  path: string[];
  contentPlainText: string;
  wordCount: number;
};

type BrowserDatabase = {
  sessionId: string;
  project: BrowserProject;
  binderItems: BrowserBinderItem[];
  documents: Record<string, BrowserDocument>;
  metadata: Record<string, BrowserMetadata>;
  notes: BrowserNote[];
  snapshots: BrowserSnapshot[];
  recovery: Record<string, BrowserRecovery>;
  backups: BrowserBackupRecord[];
};

const STORAGE_KEY = 'local-writer.browser-project';

export async function invokeCommand<T>(
  command: string,
  args: Record<string, unknown>,
): Promise<T> {
  if (hasTauriRuntime()) {
    return tauriInvoke<T>(command, args);
  }

  return runBrowserCommand(command, args as CommandArgs) as T;
}

function hasTauriRuntime() {
  if (typeof window === 'undefined') return false;
  return Boolean(
    (
      window as unknown as {
        __TAURI_INTERNALS__?: { invoke?: unknown };
      }
    ).__TAURI_INTERNALS__?.invoke,
  );
}

async function runBrowserCommand(command: string, args: CommandArgs) {
  const request = args.request ?? {};
  const db = loadDatabase(request);

  switch (command) {
    case 'create_project':
    case 'create_default_project':
    case 'open_project':
      return createBrowserSession(command, request);
    case 'close_project':
      createBrowserBackup(db, 'automatic');
      return { closed: true };
    case 'create_backup':
      return createBrowserBackup(db, 'manual');
    case 'list_backups':
      return db.backups;
    case 'list_binder_items':
      return db.binderItems.filter((item) => item.trashedAt === null);
    case 'create_binder_item':
      return createBinderItem(db, request);
    case 'rename_binder_item':
      return updateBinderItem(db, request.itemId as string, {
        title: request.title as string,
      });
    case 'set_binder_item_expanded':
      return updateBinderItem(db, request.itemId as string, {
        isExpanded: Boolean(request.isExpanded),
      });
    case 'duplicate_binder_item':
      return duplicateBinderItem(db, request.itemId as string);
    case 'move_binder_item':
      return moveBinderItem(
        db,
        request.itemId as string,
        (request.parentId as string | null) ?? null,
        Number(request.position ?? 0),
      );
    case 'reorder_binder_items':
      return reorderBinderItems(
        db,
        (request.parentId as string | null) ?? null,
        (request.orderedIds as string[]) ?? [],
      );
    case 'trash_binder_item':
      updateBinderItem(db, request.itemId as string, { trashedAt: now() });
      return db.binderItems.filter((item) => item.trashedAt === null);
    case 'restore_binder_item':
      updateBinderItem(db, request.itemId as string, { trashedAt: null });
      return db.binderItems.filter((item) => item.trashedAt === null);
    case 'get_document':
      return getDocument(db, request.binderItemId as string);
    case 'save_document':
      return saveDocument(db, request);
    case 'record_document_recovery':
      return recordRecovery(db, request);
    case 'get_document_recovery':
      return db.recovery[request.binderItemId as string] ?? null;
    case 'clear_document_recovery':
      delete db.recovery[request.binderItemId as string];
      saveDatabase(db);
      return null;
    case 'get_inspector_data':
      return getInspectorData(db, request.binderItemId as string);
    case 'save_binder_synopsis':
      return updateBinderItem(db, request.itemId as string, {
        synopsis: request.synopsis as string,
      });
    case 'save_document_metadata':
      return saveMetadata(db, request);
    case 'save_project_note':
      return saveNote(db, request);
    case 'create_snapshot':
      return createSnapshot(db, request);
    case 'restore_snapshot':
      return restoreSnapshot(db, request.snapshotId as string);
    case 'search_project':
      return searchProject(db, String(request.query ?? ''));
    case 'export_project':
      return exportProject(db, request);
    default:
      throw new Error(`Comando no disponible en navegador: ${command}`);
  }
}

function createBrowserSession(
  command: string,
  request: Record<string, unknown>,
) {
  const fallbackTitle =
    command === 'open_project'
      ? titleFromPath(String(request.folderPath ?? 'Proyecto abierto'))
      : 'Borrador';
  const title = String(request.title ?? fallbackTitle).trim() || fallbackTitle;
  const existing = readDatabase();
  const db =
    command === 'open_project'
      ? createEmptyDatabase(
          title,
          String(request.folderPath ?? 'browser://open'),
        )
      : (existing ?? createEmptyDatabase(title));
  db.project.title = title;
  db.project.description = String(
    request.description ?? db.project.description,
  );
  db.project.projectType = String(
    request.projectType ?? db.project.projectType,
  );
  db.project.lastOpenedAt = now();
  saveDatabase(db);
  return { sessionId: db.sessionId, project: db.project };
}

function loadDatabase(request: Record<string, unknown>) {
  const db = readDatabase() ?? createEmptyDatabase('Borrador');
  if (!request.sessionId || request.sessionId === db.sessionId) {
    saveDatabase(db);
    return db;
  }
  return db;
}

function readDatabase(): BrowserDatabase | null {
  if (typeof localStorage === 'undefined') return null;
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) return null;
  try {
    const db = JSON.parse(raw) as BrowserDatabase;
    db.backups ??= [];
    db.recovery ??= {};
    db.snapshots ??= [];
    db.notes ??= [];
    return db;
  } catch {
    localStorage.removeItem(STORAGE_KEY);
    return null;
  }
}

function saveDatabase(db: BrowserDatabase) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(db));
}

function createEmptyDatabase(title: string, path?: string): BrowserDatabase {
  const projectId = uuid();
  const createdAt = now();
  return {
    sessionId: uuid(),
    project: {
      id: projectId,
      title,
      description: '',
      projectType: 'blank',
      path: path ?? `browser://${slugify(title)}`,
      formatVersion: 1,
      databaseSchemaVersion: 2,
      lastOpenedAt: createdAt,
    },
    binderItems: [],
    documents: {},
    metadata: {},
    notes: [],
    snapshots: [],
    recovery: {},
    backups: [],
  };
}

function createBinderItem(
  db: BrowserDatabase,
  request: Record<string, unknown>,
) {
  const parentId = (request.parentId as string | null) ?? null;
  const itemType = String(request.itemType ?? 'document');
  const createdAt = now();
  const item: BrowserBinderItem = {
    id: uuid(),
    projectId: db.project.id,
    parentId,
    itemType,
    title: String(request.title ?? 'Sin título'),
    synopsis: '',
    position: siblings(db, parentId).length,
    icon: null,
    colorLabel: null,
    status: null,
    createdAt,
    updatedAt: createdAt,
    isExpanded: itemType === 'folder',
    isArchived: false,
    trashedAt: null,
  };
  db.binderItems.push(item);
  if (itemType !== 'folder') {
    getDocument(db, item.id);
  }
  saveDatabase(db);
  return item;
}

function updateBinderItem(
  db: BrowserDatabase,
  itemId: string,
  patch: Partial<BrowserBinderItem>,
) {
  const item = db.binderItems.find((candidate) => candidate.id === itemId);
  if (!item) throw new Error('Elemento no encontrado');
  Object.assign(item, patch, { updatedAt: now() });
  saveDatabase(db);
  return item;
}

function duplicateBinderItem(db: BrowserDatabase, itemId: string) {
  const source = db.binderItems.find((item) => item.id === itemId);
  if (!source) throw new Error('Elemento no encontrado');
  const item = createBinderItem(db, {
    parentId: source.parentId,
    itemType: source.itemType,
    title: `${source.title} copia`,
  });
  item.synopsis = source.synopsis;
  if (source.itemType !== 'folder') {
    const sourceDocument = getDocument(db, source.id);
    const document = getDocument(db, item.id);
    document.contentJson = sourceDocument.contentJson;
    document.contentPlainText = sourceDocument.contentPlainText;
    document.wordCount = sourceDocument.wordCount;
    document.characterCount = sourceDocument.characterCount;
  }
  saveDatabase(db);
  return item;
}

function moveBinderItem(
  db: BrowserDatabase,
  itemId: string,
  parentId: string | null,
  position: number,
) {
  const item = db.binderItems.find((candidate) => candidate.id === itemId);
  if (!item) throw new Error('Elemento no encontrado');
  item.parentId = parentId;
  item.position = position - 0.5;
  normalizePositions(db, item.parentId);
  saveDatabase(db);
  return db.binderItems.filter((candidate) => candidate.trashedAt === null);
}

function reorderBinderItems(
  db: BrowserDatabase,
  parentId: string | null,
  orderedIds: string[],
) {
  orderedIds.forEach((id, position) => {
    const item = db.binderItems.find((candidate) => candidate.id === id);
    if (item && item.parentId === parentId) item.position = position;
  });
  saveDatabase(db);
  return db.binderItems.filter((item) => item.trashedAt === null);
}

function getDocument(db: BrowserDatabase, binderItemId: string) {
  const existing = db.documents[binderItemId];
  if (existing) return existing;

  const createdAt = now();
  const document: BrowserDocument = {
    id: uuid(),
    binderItemId,
    contentJson: { type: 'doc', content: [{ type: 'paragraph' }] },
    contentPlainText: '',
    wordCount: 0,
    characterCount: 0,
    revision: 0,
    createdAt,
    updatedAt: createdAt,
  };
  db.documents[binderItemId] = document;
  db.metadata[binderItemId] = defaultMetadata(document.id);
  saveDatabase(db);
  return document;
}

function saveDocument(db: BrowserDatabase, request: Record<string, unknown>) {
  const binderItemId = request.binderItemId as string;
  const document = getDocument(db, binderItemId);
  const contentPlainText = String(request.contentPlainText ?? '');
  Object.assign(document, {
    contentJson: request.contentJson,
    contentPlainText,
    wordCount: countWords(contentPlainText),
    characterCount: contentPlainText.length,
    revision: document.revision + 1,
    updatedAt: now(),
  });
  delete db.recovery[binderItemId];
  saveDatabase(db);
  return document;
}

function recordRecovery(db: BrowserDatabase, request: Record<string, unknown>) {
  const recovery = {
    binderItemId: request.binderItemId as string,
    contentJson: request.contentJson,
    contentPlainText: String(request.contentPlainText ?? ''),
    revision: Number(request.revision ?? 0),
    updatedAt: now(),
  };
  db.recovery[recovery.binderItemId] = recovery;
  saveDatabase(db);
  return recovery;
}

function getInspectorData(db: BrowserDatabase, binderItemId: string) {
  const document = getDocument(db, binderItemId);
  return {
    metadata: db.metadata[binderItemId] ?? defaultMetadata(document.id),
    notes: db.notes.filter((note) => note.binderItemId === binderItemId),
    snapshots: db.snapshots.filter(
      (snapshot) => snapshot.documentId === document.id,
    ),
  };
}

function saveMetadata(db: BrowserDatabase, request: Record<string, unknown>) {
  const binderItemId = request.binderItemId as string;
  const document = getDocument(db, binderItemId);
  const metadata: BrowserMetadata = {
    documentId: document.id,
    label: (request.label as string | null) ?? null,
    status: (request.status as string | null) ?? null,
    targetWordCount: (request.targetWordCount as number | null) ?? null,
    keywords: (request.keywords as string[]) ?? [],
    customFields: request.customFields ?? {},
    includeInExport: Boolean(request.includeInExport),
  };
  db.metadata[binderItemId] = metadata;
  saveDatabase(db);
  return metadata;
}

function saveNote(db: BrowserDatabase, request: Record<string, unknown>) {
  const noteId = (request.id as string | null) ?? uuid();
  const existing = db.notes.find((note) => note.id === noteId);
  const updatedAt = now();
  if (existing) {
    existing.title = String(request.title ?? existing.title);
    existing.content = String(request.content ?? existing.content);
    existing.updatedAt = updatedAt;
    saveDatabase(db);
    return existing;
  }

  const note: BrowserNote = {
    id: noteId,
    projectId: db.project.id,
    binderItemId: (request.binderItemId as string | null) ?? null,
    title: String(request.title ?? 'Notas'),
    content: String(request.content ?? ''),
    createdAt: updatedAt,
    updatedAt,
  };
  db.notes.push(note);
  saveDatabase(db);
  return note;
}

function createSnapshot(db: BrowserDatabase, request: Record<string, unknown>) {
  const document = getDocument(db, request.binderItemId as string);
  const snapshot: BrowserSnapshot = {
    id: uuid(),
    documentId: document.id,
    name: String(request.name ?? 'Snapshot manual'),
    contentJson: document.contentJson,
    contentPlainText: document.contentPlainText,
    createdAt: now(),
  };
  db.snapshots.unshift(snapshot);
  saveDatabase(db);
  return snapshot;
}

function restoreSnapshot(db: BrowserDatabase, snapshotId: string) {
  const snapshot = db.snapshots.find(
    (candidate) => candidate.id === snapshotId,
  );
  if (!snapshot) throw new Error('Snapshot no encontrado');
  const document = Object.values(db.documents).find(
    (candidate) => candidate.id === snapshot.documentId,
  );
  if (!document) throw new Error('Documento no encontrado');
  document.contentJson = snapshot.contentJson;
  document.contentPlainText = snapshot.contentPlainText;
  document.wordCount = countWords(snapshot.contentPlainText);
  document.characterCount = snapshot.contentPlainText.length;
  document.revision += 1;
  document.updatedAt = now();
  saveDatabase(db);
  return document;
}

function searchProject(db: BrowserDatabase, query: string) {
  const normalized = query.trim().toLocaleLowerCase();
  if (!normalized) return [];

  return db.binderItems
    .filter((item) => item.trashedAt === null)
    .map((item) => {
      const document = db.documents[item.id];
      const metadata = db.metadata[item.id];
      const notes = db.notes
        .filter((note) => note.binderItemId === item.id)
        .map((note) => `${note.title} ${note.content}`)
        .join(' ');
      const haystack = [
        item.title,
        item.synopsis,
        document?.contentPlainText ?? '',
        notes,
        metadata?.keywords.join(' ') ?? '',
      ].join(' ');
      return { item, document, haystack };
    })
    .filter(({ haystack }) => haystack.toLocaleLowerCase().includes(normalized))
    .map(({ item, document, haystack }) => ({
      binderItemId: item.id,
      title: item.title,
      itemType: item.itemType,
      path: binderPath(db, item.id),
      snippet: snippet(haystack, normalized),
      updatedAt: document?.updatedAt ?? item.updatedAt,
    }));
}

function exportProject(db: BrowserDatabase, request: Record<string, unknown>) {
  const format = String(request.format ?? 'markdown');
  const documents = exportDocuments(db, request);
  const includeTitles = Boolean(request.includeTitles);
  const separateScenes = Boolean(request.separateScenes);
  const content =
    format === 'json'
      ? JSON.stringify({ projectTitle: db.project.title, documents }, null, 2)
      : renderDocuments(documents, format, includeTitles, separateScenes);
  const extension = format === 'markdown' ? 'md' : format;
  const mimeType =
    format === 'json'
      ? 'application/json'
      : format === 'html'
        ? 'text/html'
        : format === 'markdown'
          ? 'text/markdown'
          : 'text/plain';
  return {
    fileName: `${slugify(db.project.title)}.${extension}`,
    mimeType,
    content,
  };
}

function createBrowserBackup(
  db: BrowserDatabase,
  kind: BrowserBackupRecord['kind'],
) {
  const createdAt = now();
  const record: BrowserBackupRecord = {
    id: uuid(),
    createdAt,
    path: `backups/${createdAt.replace(/[^a-z0-9]/gi, '')}`,
    kind,
    formatVersion: 1,
    sizeBytes: JSON.stringify(db).length,
    status: 'complete',
  };
  db.backups = [record, ...(db.backups ?? [])].slice(0, 10);
  saveDatabase(db);
  return record;
}

function exportDocuments(
  db: BrowserDatabase,
  request: Record<string, unknown>,
): BrowserExportDocument[] {
  const scope = String(request.scope ?? 'included');
  const binderItemId = (request.binderItemId as string | null) ?? null;
  const ids =
    scope === 'document' && binderItemId
      ? [binderItemId]
      : scope === 'folder' && binderItemId
        ? descendants(db, binderItemId)
        : db.binderItems
            .filter((item) => item.trashedAt === null && db.documents[item.id])
            .filter((item) => db.metadata[item.id]?.includeInExport ?? true)
            .sort((left, right) => left.position - right.position)
            .map((item) => item.id);

  return ids
    .map((id) => {
      const item = db.binderItems.find((candidate) => candidate.id === id);
      const document = db.documents[id];
      if (!item || !document) return null;
      return {
        binderItemId: id,
        title: item.title,
        path: binderPath(db, id),
        contentPlainText: document.contentPlainText,
        wordCount: document.wordCount,
      };
    })
    .filter((document): document is BrowserExportDocument => document !== null);
}

function renderDocuments(
  documents: BrowserExportDocument[],
  format: string,
  includeTitles: boolean,
  separateScenes: boolean,
) {
  const separator = separateScenes ? '\n\n---\n\n' : '\n\n';
  if (format === 'html') {
    return `<!doctype html>\n<html><body>\n${documents
      .map((document) => {
        const title = includeTitles
          ? `<h1>${escapeHtml(String(document.title))}</h1>`
          : '';
        const body = String(document.contentPlainText)
          .split('\n\n')
          .filter(Boolean)
          .map((paragraph) => `<p>${escapeHtml(paragraph)}</p>`)
          .join('\n');
        return `<section>\n${title}\n${body}\n</section>`;
      })
      .join('\n<hr />\n')}\n</body></html>`;
  }

  return documents
    .map((document) => {
      const title =
        includeTitles && format === 'markdown'
          ? `# ${document.title}`
          : includeTitles
            ? String(document.title)
            : '';
      return [title, String(document.contentPlainText)]
        .filter(Boolean)
        .join('\n\n');
    })
    .join(separator);
}

function defaultMetadata(documentId: string): BrowserMetadata {
  return {
    documentId,
    label: null,
    status: null,
    targetWordCount: null,
    keywords: [],
    customFields: {},
    includeInExport: true,
  };
}

function siblings(db: BrowserDatabase, parentId: string | null) {
  return db.binderItems
    .filter((item) => item.parentId === parentId && item.trashedAt === null)
    .sort((left, right) => left.position - right.position);
}

function normalizePositions(db: BrowserDatabase, parentId: string | null) {
  siblings(db, parentId).forEach((item, index) => {
    item.position = index;
  });
}

function descendants(db: BrowserDatabase, parentId: string): string[] {
  return siblings(db, parentId).flatMap((item) =>
    item.itemType === 'folder' ? descendants(db, item.id) : [item.id],
  );
}

function binderPath(db: BrowserDatabase, itemId: string) {
  const path: string[] = [];
  let current = db.binderItems.find((item) => item.id === itemId) ?? null;
  while (current) {
    path.unshift(current.title);
    current = current.parentId
      ? (db.binderItems.find((item) => item.id === current?.parentId) ?? null)
      : null;
  }
  return path;
}

function snippet(value: string, query: string) {
  const index = value.toLocaleLowerCase().indexOf(query);
  if (index < 0) return value.slice(0, 160);
  return value.slice(Math.max(0, index - 60), index + query.length + 100);
}

function countWords(text: string) {
  return text.trim().split(/\s+/).filter(Boolean).length;
}

function uuid() {
  return crypto.randomUUID();
}

function now() {
  return new Date().toISOString();
}

function slugify(value: string) {
  return (
    value
      .trim()
      .replace(/[^a-z0-9_-]+/gi, '-')
      .replace(/^-+|-+$/g, '') || 'export'
  );
}

function titleFromPath(value: string) {
  const normalized = value.replaceAll('\\', '/').replace(/\/+$/g, '');
  return normalized.split('/').filter(Boolean).at(-1) ?? 'Proyecto abierto';
}

function escapeHtml(value: string) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}
