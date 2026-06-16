<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';

  import BoardView from '$lib/components/board/BoardView.svelte';
  import BinderTree from '$lib/components/binder/BinderTree.svelte';
  import RichTextEditor from '$lib/components/editor/RichTextEditor.svelte';
  import OutlineView from '$lib/components/outline/OutlineView.svelte';
  import { APP_NAME } from '$lib/config/app';
  import type { BackupRecord } from '$lib/schemas/backup';
  import type { BinderItem, BinderItemType } from '$lib/schemas/binder';
  import {
    clearDocumentRecovery,
    getDocument,
    getDocumentRecovery,
    recordDocumentRecovery,
    saveDocument,
  } from '$lib/services/commands/document';
  import {
    createBinderItem,
    duplicateBinderItem,
    listBinderItems,
    moveBinderItem,
    renameBinderItem,
    setBinderItemExpanded,
    trashBinderItem,
  } from '$lib/services/commands/binder';
  import { createBackup, listBackups } from '$lib/services/commands/backup';
  import {
    closeProject,
    createDefaultProject,
    openProject,
  } from '$lib/services/commands/project';
  import type {
    ExportFormat,
    ExportScope,
    SearchResult,
  } from '$lib/schemas/searchExport';
  import {
    exportProject,
    searchProject,
  } from '$lib/services/commands/searchExport';
  import type { Snapshot } from '$lib/schemas/inspector';
  import {
    createSnapshot,
    getInspectorData,
    restoreSnapshot,
    saveBinderSynopsis,
    saveDocumentMetadata,
    saveProjectNote,
  } from '$lib/services/commands/inspector';
  import { binderItemsStore, binderTreeStore } from '$lib/stores/binder/binder';
  import {
    directChildren,
    documentChildren,
    itemHasDocument,
  } from '$lib/stores/binder/folderViews';
  import {
    currentDocumentStore,
    editorDirtyStore,
  } from '$lib/stores/editor/editor';
  import { projectSessionStore } from '$lib/stores/project/session';
  import { saveErrorStore, saveStatusStore } from '$lib/stores/saving/saving';
  import { selectedBinderItemIdStore } from '$lib/stores/selection/selection';

  const DEFAULT_PROJECT_TITLE = 'Borrador';
  const DEFAULT_DOCUMENT_TITLE = 'Sin título';
  const DEFAULT_FOLDER_TITLE = 'Nueva carpeta';
  const EXPORT_FORMAT_OPTIONS: Array<{
    format: ExportFormat;
    label: string;
    extension: string;
  }> = [
    { format: 'markdown', label: 'Markdown', extension: '.md' },
    { format: 'txt', label: 'Texto plano', extension: '.txt' },
    { format: 'html', label: 'HTML', extension: '.html' },
    { format: 'json', label: 'Proyecto completo', extension: '.json' },
  ];

  let errorMessage = $state<string | null>(null);
  let isBusy = $state(false);
  let hasBootstrapped = false;
  let editorContentJson = $state<unknown>(null);
  let editorPlainText = $state('');
  let focusMode = $state(false);
  let editorFontSize = $state(18);
  let editorLineHeight = $state(1.7);
  let editorRenderNonce = $state(0);
  let inspectorError = $state<string | null>(null);
  let synopsisDraft = $state('');
  let labelDraft = $state('');
  let statusDraft = $state('');
  let targetWordCountDraft = $state('');
  let keywordsDraft = $state('');
  let includeInExportDraft = $state(true);
  let noteId = $state<string | null>(null);
  let noteDraft = $state('');
  let snapshots = $state<Snapshot[]>([]);
  let snapshotName = $state('Snapshot manual');
  let recoveryTimer: ReturnType<typeof setTimeout> | null = null;
  let autosaveTimer: ReturnType<typeof setTimeout> | null = null;
  let synopsisTimer: ReturnType<typeof setTimeout> | null = null;
  let metadataTimer: ReturnType<typeof setTimeout> | null = null;
  let noteTimer: ReturnType<typeof setTimeout> | null = null;
  let folderView = $state<'board' | 'outline'>('board');
  let workspaceView = $state<'write' | 'search' | 'export'>('write');
  let searchQuery = $state('');
  let searchResults = $state<SearchResult[]>([]);
  let selectedSearchResult = $state<SearchResult | null>(null);
  let searchError = $state<string | null>(null);
  let isSearching = $state(false);
  let searchTimer: ReturnType<typeof setTimeout> | null = null;
  let activeSearchHighlight = $state('');
  let exportScope = $state<ExportScope>('included');
  let exportFormat = $state<ExportFormat>('markdown');
  let exportIncludeTitles = $state(true);
  let exportSeparateScenes = $state(true);
  let exportError = $state<string | null>(null);
  let exportMessage = $state('');
  let isExporting = $state(false);
  let backupRecords = $state<BackupRecord[]>([]);
  let backupError = $state<string | null>(null);
  let backupMessage = $state('');
  let isBackingUp = $state(false);

  const selectedItem = $derived(
    $binderItemsStore.find((item) => item.id === $selectedBinderItemIdStore) ??
      null,
  );
  const liveWordCount = $derived(countWords(editorPlainText));
  const selectedFolderChildren = $derived(
    selectedItem?.itemType === 'folder'
      ? directChildren($binderItemsStore, selectedItem.id)
      : [],
  );
  const selectedFolderDocumentChildren = $derived(
    selectedItem?.itemType === 'folder'
      ? documentChildren($binderItemsStore, selectedItem.id)
      : [],
  );
  const isSimpleMode = $derived(
    $projectSessionStore?.project.projectType === 'blank',
  );
  const hasBinderStructure = $derived(
    $binderItemsStore.length > 1 ||
      $binderItemsStore.some((item) => item.itemType === 'folder'),
  );
  const showBinderPanel = $derived(
    !focusMode &&
      workspaceView === 'write' &&
      selectedItem?.itemType !== 'folder' &&
      (!isSimpleMode || hasBinderStructure),
  );
  const showInspectorPanel = $derived(
    !focusMode &&
      workspaceView === 'write' &&
      selectedItem?.itemType !== 'folder',
  );
  const workspaceGridClass = $derived(
    focusMode
      ? 'grid min-h-0 grid-cols-[4.5rem_1fr] bg-white'
      : workspaceView !== 'write' || selectedItem?.itemType === 'folder'
        ? 'grid min-h-0 grid-cols-[1fr] bg-ink-50'
        : showBinderPanel
          ? 'grid min-h-0 grid-cols-[19rem_1fr_25rem] bg-white'
          : 'grid min-h-0 grid-cols-[1fr_25rem] bg-white',
  );
  const activeModeLabel = $derived(
    workspaceView === 'search'
      ? 'Buscar'
      : workspaceView === 'export'
        ? 'Exportar'
        : selectedItem?.itemType === 'folder'
          ? 'Organizar'
          : $projectSessionStore?.project.projectType === 'screenplay'
            ? 'Guion'
            : isSimpleMode
              ? 'Sencillo'
              : 'Novela',
  );
  const savedLabel = $derived(
    $saveStatusStore === 'dirty'
      ? 'Cambios sin guardar'
      : $saveStatusStore === 'saving'
        ? 'Guardando...'
        : $saveStatusStore === 'error'
          ? 'Error al guardar'
          : 'Guardado localmente',
  );
  const targetWordCount = $derived(
    parseOptionalPositiveInteger(targetWordCountDraft),
  );
  const targetProgress = $derived(
    Math.min(
      100,
      targetWordCount ? (liveWordCount / targetWordCount) * 100 : 0,
    ),
  );
  const bottomLeftStatus = $derived(
    workspaceView === 'search'
      ? 'Buscar en todo el proyecto'
      : workspaceView === 'export'
        ? 'Los archivos se guardan localmente'
        : selectedItem?.itemType === 'folder'
          ? `${selectedFolderDocumentChildren.length} escenas`
          : `${liveWordCount} palabras`,
  );
  const bottomRightStatus = $derived(
    workspaceView === 'search'
      ? selectedSearchResult
        ? 'Enter para abrir'
        : `${searchResults.length} resultados`
      : workspaceView === 'export'
        ? 'Sin conexión a Internet'
        : `✓ ${savedLabel}`,
  );

  onMount(() => {
    void bootstrapWritingSpace();
  });

  async function bootstrapWritingSpace() {
    if (hasBootstrapped) return;
    hasBootstrapped = true;

    await run(async () => {
      let session = get(projectSessionStore);
      if (!session) {
        session = await createDefaultProject({
          title: DEFAULT_PROJECT_TITLE,
          description: '',
          projectType: 'blank',
        });
        projectSessionStore.set(session);
        selectedBinderItemIdStore.set(null);
        currentDocumentStore.set(null);
      }

      await loadSessionWorkspace(session.sessionId, 'preserve');
    });
  }

  async function loadSessionWorkspace(
    sessionId: string,
    viewMode: 'write' | 'preserve' = 'write',
  ) {
    const items = await listBinderItems({
      sessionId,
      includeTrashed: false,
    });
    binderItemsStore.set(items);

    const initialDocument =
      items.find((item) => itemHasDocument(item)) ??
      (await createBinderItem({
        sessionId,
        parentId: null,
        itemType: 'document',
        title: DEFAULT_DOCUMENT_TITLE,
      }));

    if (!items.some((item) => item.id === initialDocument.id)) {
      binderItemsStore.update((currentItems) => [
        ...currentItems,
        initialDocument,
      ]);
    }

    await selectBinderItem(initialDocument.id, viewMode);
  }

  async function openProjectFromPrompt() {
    const currentSession = get(projectSessionStore);
    const folderPath = window.prompt(
      'Ruta de la carpeta del proyecto',
      currentSession?.project.path ?? '',
    );
    const trimmedPath = folderPath?.trim();
    if (!trimmedPath || trimmedPath === currentSession?.project.path) return;

    await run(async () => {
      await saveCurrentDocument();
      const nextSession = await openProject({ folderPath: trimmedPath });
      const previousSession = get(projectSessionStore);
      projectSessionStore.set(nextSession);
      selectedBinderItemIdStore.set(null);
      currentDocumentStore.set(null);
      editorContentJson = null;
      editorPlainText = '';
      resetInspector();
      workspaceView = 'write';
      await loadSessionWorkspace(nextSession.sessionId);

      if (
        previousSession &&
        previousSession.sessionId !== nextSession.sessionId
      ) {
        try {
          await closeProject({ sessionId: previousSession.sessionId });
        } catch {
          // Opening the requested project succeeded; a failed close should not discard it.
        }
      }
    });
  }

  async function createQuickDocument() {
    await createItem(null, 'document', nextUntitledTitle());
  }

  async function createQuickFolder() {
    await createItem(null, 'folder', nextFolderTitle());
  }

  function nextUntitledTitle() {
    const count = get(binderItemsStore).filter(
      (item) =>
        item.itemType === 'document' &&
        item.title.startsWith(DEFAULT_DOCUMENT_TITLE),
    ).length;

    return count === 0
      ? DEFAULT_DOCUMENT_TITLE
      : `${DEFAULT_DOCUMENT_TITLE} ${count + 1}`;
  }

  function nextFolderTitle() {
    const count = get(binderItemsStore).filter(
      (item) =>
        item.itemType === 'folder' &&
        item.title.startsWith(DEFAULT_FOLDER_TITLE),
    ).length;

    return count === 0
      ? DEFAULT_FOLDER_TITLE
      : `${DEFAULT_FOLDER_TITLE} ${count + 1}`;
  }

  async function refreshBinder() {
    const session = get(projectSessionStore);
    if (!session) return;
    const items = await listBinderItems({
      sessionId: session.sessionId,
      includeTrashed: false,
    });
    binderItemsStore.set(items);
  }

  async function createItem(
    parentId: string | null,
    itemType: BinderItemType,
    title: string,
  ) {
    const session = get(projectSessionStore);
    if (!session) return;

    await run(async () => {
      const item = await createBinderItem({
        sessionId: session.sessionId,
        parentId,
        itemType,
        title,
      });
      binderItemsStore.update((items) => [...items, item]);
      await selectBinderItem(item.id);
    });
  }

  async function renameItem(itemId: string, title: string) {
    const session = get(projectSessionStore);
    if (!session) return;

    await run(async () => {
      const item = await renameBinderItem({
        sessionId: session.sessionId,
        itemId,
        title,
      });
      binderItemsStore.update((items) =>
        items.map((existing) => (existing.id === item.id ? item : existing)),
      );
    });
  }

  async function toggleItem(itemId: string, isExpanded: boolean) {
    const session = get(projectSessionStore);
    if (!session) return;

    await run(async () => {
      const item = await setBinderItemExpanded({
        sessionId: session.sessionId,
        itemId,
        isExpanded,
      });
      binderItemsStore.update((items) =>
        items.map((existing) => (existing.id === item.id ? item : existing)),
      );
    });
  }

  async function duplicateItem(itemId: string) {
    const session = get(projectSessionStore);
    if (!session) return;

    await run(async () => {
      await duplicateBinderItem({ sessionId: session.sessionId, itemId });
      await refreshBinder();
    });
  }

  async function moveItem(
    itemId: string,
    parentId: string | null,
    position: number,
  ) {
    const session = get(projectSessionStore);
    if (!session) return;

    await run(async () => {
      const items = await moveBinderItem({
        sessionId: session.sessionId,
        itemId,
        parentId,
        position,
      });
      binderItemsStore.set(items);
    });
  }

  async function moveBoardCardBefore(itemId: string, targetId: string) {
    const target = get(binderItemsStore).find((item) => item.id === targetId);
    if (!target) return;
    await moveItem(itemId, target.parentId, target.position);
  }

  async function createSceneFromBoard(parentId: string) {
    await createItem(parentId, 'document', nextUntitledTitle());
  }

  async function trashItem(itemId: string) {
    const session = get(projectSessionStore);
    if (!session) return;

    await run(async () => {
      const items = await trashBinderItem({
        sessionId: session.sessionId,
        itemId,
      });
      binderItemsStore.set(items);
      if (get(selectedBinderItemIdStore) === itemId) {
        selectedBinderItemIdStore.set(null);
        currentDocumentStore.set(null);
      }
    });
  }

  async function selectBinderItem(
    itemId: string,
    viewMode: 'write' | 'preserve' = 'write',
  ) {
    const item = get(binderItemsStore).find(
      (candidate) => candidate.id === itemId,
    );
    if (!item) return;

    if (viewMode === 'write') {
      workspaceView = 'write';
    }
    await saveCurrentDocument();
    selectedBinderItemIdStore.set(itemId);

    if (!itemHasDocument(item)) {
      currentDocumentStore.set(null);
      editorDirtyStore.set(false);
      saveStatusStore.set('idle');
      resetInspector();
      return;
    }

    await loadDocument(item.id);
  }

  async function loadDocument(binderItemId: string) {
    const session = get(projectSessionStore);
    if (!session) return;

    await run(async () => {
      const [document, recovery] = await Promise.all([
        getDocument({ sessionId: session.sessionId, binderItemId }),
        getDocumentRecovery({ sessionId: session.sessionId, binderItemId }),
      ]);

      currentDocumentStore.set(document);
      if (recovery && recovery.revision === document.revision) {
        const shouldRecover = window.confirm(
          'Hay contenido recuperado para este documento. ¿Quieres restaurarlo?',
        );
        if (shouldRecover) {
          editorContentJson = recovery.contentJson;
          editorPlainText = recovery.contentPlainText;
          editorDirtyStore.set(true);
          saveStatusStore.set('dirty');
          scheduleAutosave();
          return;
        }
        await clearDocumentRecovery({
          sessionId: session.sessionId,
          binderItemId,
        });
      }

      editorContentJson = document.contentJson;
      editorPlainText = document.contentPlainText;
      editorRenderNonce += 1;
      editorDirtyStore.set(false);
      saveStatusStore.set('saved');
      saveErrorStore.set(null);
      await loadInspector(binderItemId);
    });
  }

  async function loadInspector(binderItemId: string) {
    const session = get(projectSessionStore);
    const item = get(binderItemsStore).find(
      (candidate) => candidate.id === binderItemId,
    );
    if (!session || !item) {
      resetInspector();
      return;
    }

    inspectorError = null;
    synopsisDraft = item.synopsis;
    try {
      const data = await getInspectorData({
        sessionId: session.sessionId,
        binderItemId,
      });
      labelDraft = data.metadata?.label ?? '';
      statusDraft = data.metadata?.status ?? '';
      targetWordCountDraft =
        data.metadata?.targetWordCount === null ||
        data.metadata?.targetWordCount === undefined
          ? ''
          : String(data.metadata.targetWordCount);
      keywordsDraft = data.metadata?.keywords.join(', ') ?? '';
      includeInExportDraft = data.metadata?.includeInExport ?? true;
      const firstNote = data.notes[0] ?? null;
      noteId = firstNote?.id ?? null;
      noteDraft = firstNote?.content ?? '';
      snapshots = data.snapshots;
    } catch (error) {
      inspectorError =
        error instanceof Error
          ? error.message
          : 'No se pudo cargar el inspector';
    }
  }

  function resetInspector() {
    inspectorError = null;
    synopsisDraft = '';
    labelDraft = '';
    statusDraft = '';
    targetWordCountDraft = '';
    keywordsDraft = '';
    includeInExportDraft = true;
    noteId = null;
    noteDraft = '';
    snapshots = [];
  }

  function handleEditorChange(contentJson: unknown, contentPlainText: string) {
    editorContentJson = contentJson;
    editorPlainText = contentPlainText;
    editorDirtyStore.set(true);
    saveStatusStore.set('dirty');
    saveErrorStore.set(null);
    scheduleRecoveryRecord();
    scheduleAutosave();
  }

  function scheduleRecoveryRecord() {
    if (recoveryTimer) clearTimeout(recoveryTimer);
    recoveryTimer = setTimeout(() => {
      void recordCurrentRecovery();
    }, 300);
  }

  function scheduleAutosave() {
    if (autosaveTimer) clearTimeout(autosaveTimer);
    autosaveTimer = setTimeout(() => {
      void saveCurrentDocument();
    }, 1200);
  }

  function scheduleSynopsisSave() {
    if (synopsisTimer) clearTimeout(synopsisTimer);
    synopsisTimer = setTimeout(() => {
      void saveSynopsisNow();
    }, 600);
  }

  function scheduleMetadataSave() {
    if (metadataTimer) clearTimeout(metadataTimer);
    metadataTimer = setTimeout(() => {
      void saveMetadataNow();
    }, 600);
  }

  function scheduleNoteSave() {
    if (noteTimer) clearTimeout(noteTimer);
    noteTimer = setTimeout(() => {
      void saveNoteNow();
    }, 700);
  }

  async function saveSynopsisNow() {
    const session = get(projectSessionStore);
    const item = selectedItem;
    if (!session || !item) return;

    try {
      const saved = await saveBinderSynopsis({
        sessionId: session.sessionId,
        itemId: item.id,
        synopsis: synopsisDraft,
      });
      binderItemsStore.update((items) =>
        items.map((existing) => (existing.id === saved.id ? saved : existing)),
      );
      inspectorError = null;
    } catch (error) {
      inspectorError =
        error instanceof Error ? error.message : 'No se pudo guardar sinopsis';
    }
  }

  async function saveItemSynopsisNow(item: BinderItem, synopsis: string) {
    const session = get(projectSessionStore);
    if (!session) return;

    try {
      const saved = await saveBinderSynopsis({
        sessionId: session.sessionId,
        itemId: item.id,
        synopsis,
      });
      binderItemsStore.update((items) =>
        items.map((existing) => (existing.id === saved.id ? saved : existing)),
      );
      if (selectedItem?.id === saved.id) {
        synopsisDraft = saved.synopsis;
      }
      inspectorError = null;
    } catch (error) {
      inspectorError =
        error instanceof Error ? error.message : 'No se pudo guardar sinopsis';
    }
  }

  async function saveMetadataNow() {
    const session = get(projectSessionStore);
    const item = selectedItem;
    if (!session || !item || !itemHasDocument(item)) return;

    try {
      await saveDocumentMetadata({
        sessionId: session.sessionId,
        binderItemId: item.id,
        label: emptyToNull(labelDraft),
        status: emptyToNull(statusDraft),
        targetWordCount: parseOptionalPositiveInteger(targetWordCountDraft),
        keywords: parseKeywords(keywordsDraft),
        customFields: {},
        includeInExport: includeInExportDraft,
      });
      inspectorError = null;
    } catch (error) {
      inspectorError =
        error instanceof Error
          ? error.message
          : 'No se pudieron guardar metadatos';
    }
  }

  async function saveNoteNow() {
    const session = get(projectSessionStore);
    const item = selectedItem;
    if (!session || !item || (!noteId && !noteDraft.trim())) return;

    try {
      const saved = await saveProjectNote({
        sessionId: session.sessionId,
        id: noteId,
        binderItemId: item.id,
        title: 'Notas',
        content: noteDraft,
      });
      noteId = saved.id;
      inspectorError = null;
    } catch (error) {
      inspectorError =
        error instanceof Error ? error.message : 'No se pudo guardar la nota';
    }
  }

  async function createManualSnapshot() {
    const session = get(projectSessionStore);
    const item = selectedItem;
    if (!session || !item || !itemHasDocument(item)) return;

    await saveCurrentDocument();
    try {
      const snapshot = await createSnapshot({
        sessionId: session.sessionId,
        binderItemId: item.id,
        name: snapshotName.trim() || 'Snapshot manual',
      });
      snapshots = [snapshot, ...snapshots];
      snapshotName = 'Snapshot manual';
      inspectorError = null;
    } catch (error) {
      inspectorError =
        error instanceof Error ? error.message : 'No se pudo crear snapshot';
    }
  }

  async function restoreSnapshotById(snapshotId: string) {
    const session = get(projectSessionStore);
    if (!session) return;

    const shouldRestore = window.confirm(
      'Esto reemplazará el contenido actual con el snapshot seleccionado. ¿Continuar?',
    );
    if (!shouldRestore) return;

    try {
      const restored = await restoreSnapshot({
        sessionId: session.sessionId,
        snapshotId,
      });
      currentDocumentStore.set(restored);
      editorContentJson = restored.contentJson;
      editorPlainText = restored.contentPlainText;
      editorRenderNonce += 1;
      editorDirtyStore.set(false);
      saveStatusStore.set('saved');
      saveErrorStore.set(null);
      inspectorError = null;
    } catch (error) {
      inspectorError =
        error instanceof Error
          ? error.message
          : 'No se pudo restaurar snapshot';
    }
  }

  async function recordCurrentRecovery() {
    const session = get(projectSessionStore);
    const document = get(currentDocumentStore);
    if (!session || !document || !get(editorDirtyStore)) return;

    try {
      await recordDocumentRecovery({
        sessionId: session.sessionId,
        binderItemId: document.binderItemId,
        contentJson: editorContentJson,
        contentPlainText: editorPlainText,
        revision: document.revision,
      });
    } catch (error) {
      saveErrorStore.set(
        error instanceof Error
          ? error.message
          : 'No se pudo registrar recuperación',
      );
    }
  }

  async function saveCurrentDocument() {
    const session = get(projectSessionStore);
    const document = get(currentDocumentStore);
    if (!session || !document || !get(editorDirtyStore)) return;

    if (autosaveTimer) {
      clearTimeout(autosaveTimer);
      autosaveTimer = null;
    }

    saveStatusStore.set('saving');
    saveErrorStore.set(null);
    try {
      const saved = await saveDocument({
        sessionId: session.sessionId,
        binderItemId: document.binderItemId,
        contentJson: editorContentJson,
        contentPlainText: editorPlainText,
        expectedRevision: document.revision,
      });
      currentDocumentStore.set(saved);
      editorDirtyStore.set(false);
      saveStatusStore.set('saved');
    } catch (error) {
      saveStatusStore.set('error');
      saveErrorStore.set(
        error instanceof Error ? error.message : 'No se pudo guardar',
      );
    }
  }

  function countWords(text: string) {
    return text.trim().split(/\s+/).filter(Boolean).length;
  }

  function emptyToNull(value: string) {
    const trimmed = value.trim();
    return trimmed ? trimmed : null;
  }

  function parseOptionalPositiveInteger(value: string) {
    const trimmed = value.trim();
    if (!trimmed) return null;
    const parsed = Number.parseInt(trimmed, 10);
    return Number.isFinite(parsed) && parsed >= 0 ? parsed : null;
  }

  function parseKeywords(value: string) {
    return value
      .split(',')
      .map((keyword) => keyword.trim())
      .filter(Boolean);
  }

  function showSearchView() {
    workspaceView = 'search';
    activeSearchHighlight = '';
    requestAnimationFrame(() => {
      document.getElementById('project-search-input')?.focus();
    });
  }

  function showExportView() {
    workspaceView = 'export';
    activeSearchHighlight = '';
    void refreshBackups();
  }

  function scheduleSearch() {
    if (searchTimer) clearTimeout(searchTimer);
    searchTimer = setTimeout(() => {
      void runSearch();
    }, 250);
  }

  async function runSearch() {
    const session = get(projectSessionStore);
    const query = searchQuery.trim();
    if (!session || !query) {
      searchResults = [];
      selectedSearchResult = null;
      searchError = null;
      return;
    }

    isSearching = true;
    searchError = null;
    try {
      const results = await searchProject({
        sessionId: session.sessionId,
        query,
      });
      searchResults = results;
      selectedSearchResult = results[0] ?? null;
    } catch (error) {
      searchError =
        error instanceof Error ? error.message : 'No se pudo buscar';
    } finally {
      isSearching = false;
    }
  }

  async function openSearchResult(result: SearchResult) {
    selectedSearchResult = result;
    activeSearchHighlight = searchQuery.trim();
    await selectBinderItem(result.binderItemId);
    setTimeout(() => {
      activeSearchHighlight = '';
    }, 4000);
  }

  async function exportCurrentProject() {
    const session = get(projectSessionStore);
    if (!session) return;

    const binderItemId =
      exportScope === 'included' ? null : (selectedItem?.id ?? null);
    if (exportScope !== 'included' && !binderItemId) {
      exportError = 'Selecciona un documento o carpeta para exportar.';
      return;
    }

    isExporting = true;
    exportError = null;
    exportMessage = '';
    try {
      const file = await exportProject({
        sessionId: session.sessionId,
        scope: exportScope,
        format: exportFormat,
        binderItemId,
        includeTitles: exportIncludeTitles,
        separateScenes: exportSeparateScenes,
      });
      downloadExport(file.fileName, file.mimeType, file.content);
      exportMessage = `${file.fileName} preparado`;
    } catch (error) {
      exportError =
        error instanceof Error ? error.message : 'No se pudo exportar';
    } finally {
      isExporting = false;
    }
  }

  async function refreshBackups() {
    const session = get(projectSessionStore);
    if (!session) return;
    try {
      backupRecords = await listBackups({ sessionId: session.sessionId });
      backupError = null;
    } catch (error) {
      backupError =
        error instanceof Error
          ? error.message
          : 'No se pudieron cargar las copias';
    }
  }

  async function createManualBackup() {
    const session = get(projectSessionStore);
    if (!session) return;

    isBackingUp = true;
    backupError = null;
    backupMessage = '';
    try {
      const record = await createBackup({ sessionId: session.sessionId });
      backupRecords = [
        record,
        ...backupRecords.filter((item) => item.id !== record.id),
      ];
      backupMessage = 'Copia de seguridad creada';
    } catch (error) {
      backupError =
        error instanceof Error
          ? error.message
          : 'No se pudo crear la copia de seguridad';
    } finally {
      isBackingUp = false;
    }
  }

  function downloadExport(fileName: string, mimeType: string, content: string) {
    const url = URL.createObjectURL(new Blob([content], { type: mimeType }));
    const link = document.createElement('a');
    link.href = url;
    link.download = fileName;
    link.click();
    URL.revokeObjectURL(url);
  }

  function highlightedSnippetParts(result: SearchResult | null) {
    const snippet = result?.snippet || result?.title || '';
    const query = searchQuery.trim();
    if (!query) return [{ text: snippet, highlighted: false }];

    const tokens = query.split(/\s+/).filter(Boolean).map(escapeRegExp);
    if (tokens.length === 0) return [{ text: snippet, highlighted: false }];

    const pattern = new RegExp(`(${tokens.join('|')})`, 'gi');
    const matchPattern = new RegExp(`^(${tokens.join('|')})$`, 'i');
    return snippet
      .split(pattern)
      .filter((part) => part.length > 0)
      .map((part) => ({
        text: part,
        highlighted: matchPattern.test(part),
      }));
  }

  function escapeRegExp(value: string) {
    return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }

  function handleGlobalKeydown(event: KeyboardEvent) {
    if (workspaceView === 'search' && event.key === 'Enter') {
      const result = selectedSearchResult ?? searchResults[0] ?? null;
      if (result) {
        event.preventDefault();
        void openSearchResult(result);
      }
    }

    const modifier = event.metaKey || event.ctrlKey;
    if (modifier && event.key.toLowerCase() === 'f') {
      event.preventDefault();
      showSearchView();
      return;
    }

    if (modifier && event.shiftKey && event.key.toLowerCase() === 'n') {
      event.preventDefault();
      void createQuickFolder();
      return;
    }

    if (modifier && event.key.toLowerCase() === 'n') {
      event.preventDefault();
      void createQuickDocument();
      return;
    }

    if (modifier && event.key.toLowerCase() === 's') {
      event.preventDefault();
      void saveCurrentDocument();
      return;
    }

    if (event.key === 'F11') {
      event.preventDefault();
      focusMode = !focusMode;
    }
  }

  async function run(action: () => Promise<void>) {
    isBusy = true;
    errorMessage = null;
    try {
      await action();
    } catch (error) {
      errorMessage =
        error instanceof Error ? error.message : 'Error desconocido';
    } finally {
      isBusy = false;
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<svelte:head>
  <title>{APP_NAME}</title>
  <meta
    name="description"
    content="Aplicación de escritorio local-first para proyectos de escritura."
  />
</svelte:head>

<main
  class="grid h-screen grid-rows-[4.5rem_1fr_3.5rem] bg-ink-50 text-ink-900"
>
  <header
    class="grid grid-cols-[1fr_auto_1fr] items-center border-b border-ink-100 bg-white px-9"
  >
    <div class="flex min-w-0 items-center gap-6">
      <div class="flex items-center gap-5 text-3xl leading-none text-ink-500">
        <button type="button" class="hover:text-ink-900" aria-label="Anterior"
          >‹</button
        >
        <button type="button" class="hover:text-ink-900" aria-label="Siguiente"
          >›</button
        >
      </div>
      <div class="flex min-w-0 items-center gap-3">
        <input
          class="min-w-0 max-w-[22rem] truncate bg-transparent text-xl font-semibold outline-none focus:text-accent-600"
          aria-label="Título del documento"
          value={selectedItem?.title ??
            $projectSessionStore?.project.title ??
            'Sin título'}
          onblur={(event) => {
            const title = event.currentTarget.value.trim();
            if (selectedItem && title && title !== selectedItem.title) {
              void renameItem(selectedItem.id, title);
            }
          }}
          onkeydown={(event) => {
            if (event.key === 'Enter') {
              event.currentTarget.blur();
            }
          }}
        />
        <button
          type="button"
          class="text-sm text-ink-500 hover:text-ink-900"
          aria-label="Abrir proyecto"
          title="Abrir proyecto"
          onclick={openProjectFromPrompt}
        >
          ⌄
        </button>
      </div>
      {#if workspaceView === 'write' && selectedItem?.itemType !== 'folder'}
        <div class="flex shrink-0 items-center gap-2">
          <button
            type="button"
            class="rounded-md px-2 py-1 text-sm font-semibold text-ink-500 hover:bg-ink-50"
            disabled={isBusy || !$projectSessionStore}
            title="Nuevo documento"
            aria-label="Nuevo documento"
            onclick={createQuickDocument}
          >
            + Doc
          </button>
          <button
            type="button"
            class="rounded-md px-2 py-1 text-sm font-semibold text-ink-500 hover:bg-ink-50"
            disabled={isBusy || !$projectSessionStore}
            title="Nueva carpeta"
            aria-label="Nueva carpeta"
            onclick={createQuickFolder}
          >
            + Carpeta
          </button>
        </div>
      {/if}
    </div>

    <div
      class="rounded-xl bg-accent-100 px-8 py-2.5 text-base font-semibold text-accent-500"
    >
      {activeModeLabel}
    </div>

    <div class="flex items-center justify-end gap-10 text-lg font-semibold">
      <button
        type="button"
        class="hover:text-accent-500"
        onclick={showSearchView}>Buscar</button
      >
      <button
        type="button"
        class="hover:text-accent-500"
        onclick={() => {
          focusMode = !focusMode;
        }}
      >
        Modo enfoque
      </button>
      <button
        type="button"
        class="hover:text-accent-500"
        onclick={showExportView}>Exportar</button
      >
      <button type="button" class="text-2xl" aria-label="Preferencias"
        >⚙</button
      >
    </div>
  </header>

  <div class={workspaceGridClass}>
    {#if focusMode}
      <aside
        class="border-r border-ink-100 bg-ink-50 px-8 py-9"
        aria-label="Binder compacto"
      >
        <button
          type="button"
          class="text-2xl text-ink-500"
          aria-label="Abrir documentos">≡</button
        >
      </aside>
    {:else if showBinderPanel}
      <aside
        aria-label="Binder"
        class="overflow-auto border-r border-ink-100 bg-ink-50 px-6 py-8"
      >
        <div class="mb-7 flex items-center justify-between">
          <h2 class="text-sm font-bold uppercase text-ink-500">Proyecto</h2>
          <div class="flex gap-2">
            <button
              type="button"
              class="rounded-md px-2 py-1 text-sm font-semibold text-ink-500 hover:bg-white"
              disabled={isBusy || !$projectSessionStore}
              title="Nuevo documento"
              aria-label="Nuevo documento"
              onclick={createQuickDocument}
            >
              + Doc
            </button>
            <button
              type="button"
              class="rounded-md px-2 py-1 text-sm font-semibold text-ink-500 hover:bg-white"
              disabled={isBusy || !$projectSessionStore}
              title="Nueva carpeta"
              aria-label="Nueva carpeta"
              onclick={createQuickFolder}
            >
              + Carpeta
            </button>
          </div>
        </div>

        {#if errorMessage}
          <p
            class="mb-4 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700"
          >
            {errorMessage}
          </p>
        {/if}

        {#if $projectSessionStore && $binderTreeStore.length > 0}
          <BinderTree
            nodes={$binderTreeStore}
            selectedId={$selectedBinderItemIdStore}
            onSelect={selectBinderItem}
            onToggle={(item) => toggleItem(item.id, !item.isExpanded)}
            onRename={(item, title) => renameItem(item.id, title)}
            onCreateChild={(parentId, itemType) =>
              createItem(parentId, itemType, 'Nuevo documento')}
            onDuplicate={(item) => duplicateItem(item.id)}
            onTrash={(item) => trashItem(item.id)}
            onMove={moveItem}
          />
        {:else}
          <p class="text-sm leading-6 text-ink-500">
            Preparando un proyecto local...
          </p>
        {/if}
      </aside>
    {/if}

    <section
      aria-label={workspaceView === 'search'
        ? 'Buscar'
        : workspaceView === 'export'
          ? 'Exportar'
          : 'Editor'}
      class={workspaceView !== 'write' || selectedItem?.itemType === 'folder'
        ? 'min-h-0 overflow-auto bg-ink-50 px-14 py-10'
        : 'min-h-0 overflow-auto bg-white px-10 py-10'}
    >
      {#if workspaceView === 'search'}
        <div class="mx-auto grid max-w-[112rem] gap-8">
          <form
            class="rounded-xl border border-ink-100 bg-white px-7 py-5"
            onsubmit={(event) => {
              event.preventDefault();
              void runSearch();
            }}
          >
            <label class="grid grid-cols-[auto_1fr_auto] items-center gap-6">
              <span class="text-2xl text-ink-500">⌕</span>
              <input
                id="project-search-input"
                class="bg-transparent text-xl font-medium text-ink-900 outline-none"
                placeholder="Buscar en el proyecto"
                bind:value={searchQuery}
                oninput={scheduleSearch}
              />
              <span class="text-base font-medium text-ink-500">
                {isSearching
                  ? 'Buscando...'
                  : `${searchResults.length} resultados`}
              </span>
            </label>
          </form>

          {#if searchError}
            <p
              class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700"
            >
              {searchError}
            </p>
          {/if}

          <div class="grid min-h-[42rem] grid-cols-[29rem_1fr] gap-8">
            <aside
              class="overflow-auto rounded-xl border border-ink-100 bg-white p-5"
              aria-label="Resultados de búsqueda"
            >
              <h2 class="mb-4 text-lg font-semibold text-ink-900">
                Resultados
              </h2>
              {#if searchResults.length > 0}
                <div class="grid gap-3">
                  {#each searchResults as result (result.binderItemId)}
                    <button
                      type="button"
                      class="rounded-lg px-4 py-3 text-left hover:bg-accent-100"
                      class:bg-accent-100={selectedSearchResult?.binderItemId ===
                        result.binderItemId}
                      onclick={() => {
                        selectedSearchResult = result;
                      }}
                      ondblclick={() => openSearchResult(result)}
                    >
                      <span class="block font-semibold text-ink-900">
                        {result.title}
                      </span>
                      <span class="mt-1 block text-sm leading-5 text-ink-500">
                        {result.path.join(' / ')}
                      </span>
                    </button>
                  {/each}
                </div>
              {:else}
                <p class="text-base leading-7 text-ink-500">
                  Escribe una búsqueda para encontrar capítulos, notas, sinopsis
                  y palabras clave.
                </p>
              {/if}
            </aside>

            <article
              class="rounded-xl border border-ink-100 bg-white px-16 py-14"
              aria-label="Vista previa de búsqueda"
            >
              {#if selectedSearchResult}
                <div class="max-w-5xl">
                  <p class="mb-4 text-sm font-semibold uppercase text-ink-500">
                    {selectedSearchResult.path.join(' / ')}
                  </p>
                  <h1 class="text-4xl font-bold text-ink-900">
                    {selectedSearchResult.title}
                  </h1>
                  <p class="mt-10 text-2xl leading-10 text-ink-900">
                    {#each highlightedSnippetParts(selectedSearchResult) as part, index (index)}
                      {#if part.highlighted}
                        <mark class="rounded bg-yellow-100 px-0.5"
                          >{part.text}</mark
                        >
                      {:else}
                        {part.text}
                      {/if}
                    {/each}
                  </p>
                  <button
                    type="button"
                    class="mt-12 rounded-lg bg-accent-500 px-6 py-3 text-base font-semibold text-white hover:bg-accent-600"
                    onclick={() => {
                      if (selectedSearchResult) {
                        void openSearchResult(selectedSearchResult);
                      }
                    }}
                  >
                    Abrir resultado
                  </button>
                </div>
              {:else}
                <p class="text-xl leading-8 text-ink-500">
                  Selecciona un resultado para ver el fragmento resaltado.
                </p>
              {/if}
            </article>
          </div>
        </div>
      {:else if workspaceView === 'export'}
        <div class="mx-auto max-w-[100rem]">
          <div class="mb-10">
            <h1 class="text-4xl font-bold text-ink-900">Exportar proyecto</h1>
            <p class="mt-3 text-xl text-ink-500">
              Prepara una copia local en el formato que prefieras.
            </p>
          </div>

          <div class="grid grid-cols-2 gap-8">
            <section class="rounded-xl border border-ink-100 bg-white p-9">
              <h2 class="mb-8 text-lg font-semibold text-ink-900">Contenido</h2>
              <div class="grid gap-7">
                <label class="text-ink-800 flex items-center gap-4 text-lg">
                  <input
                    type="radio"
                    name="export-scope"
                    value="included"
                    bind:group={exportScope}
                  />
                  Manuscrito completo
                </label>
                <label class="text-ink-800 flex items-center gap-4 text-lg">
                  <input
                    type="radio"
                    name="export-scope"
                    value="folder"
                    bind:group={exportScope}
                  />
                  Carpeta seleccionada
                </label>
                <label class="text-ink-800 flex items-center gap-4 text-lg">
                  <input
                    type="radio"
                    name="export-scope"
                    value="document"
                    bind:group={exportScope}
                  />
                  Documento actual
                </label>
              </div>

              <h2 class="mb-6 mt-16 text-lg font-semibold text-ink-900">
                Opciones
              </h2>
              <div class="grid gap-6">
                <label class="text-ink-800 flex items-center gap-4 text-lg">
                  <input type="checkbox" bind:checked={exportIncludeTitles} />
                  Incluir títulos
                </label>
                <label class="text-ink-800 flex items-center gap-4 text-lg">
                  <input type="checkbox" bind:checked={exportSeparateScenes} />
                  Separar escenas
                </label>
              </div>

              <div class="mt-16 border-t border-ink-100 pt-8">
                <div class="flex items-start justify-between gap-6">
                  <div>
                    <h2 class="text-lg font-semibold text-ink-900">
                      Copia de seguridad
                    </h2>
                    <p class="mt-2 text-base leading-7 text-ink-500">
                      Se crea una copia automática al cerrar el proyecto.
                    </p>
                  </div>
                  <button
                    type="button"
                    class="rounded-lg border border-ink-100 px-4 py-2 text-sm font-semibold text-ink-700 hover:bg-ink-50 disabled:opacity-60"
                    disabled={isBackingUp}
                    onclick={createManualBackup}
                  >
                    {isBackingUp ? 'Creando...' : 'Crear copia'}
                  </button>
                </div>
                {#if backupRecords[0]}
                  <p class="mt-4 text-sm text-ink-500">
                    Última copia:
                    {new Date(backupRecords[0].createdAt).toLocaleString()}
                  </p>
                {/if}
                {#if backupMessage}
                  <p class="mt-4 text-sm font-medium text-emerald-700">
                    {backupMessage}
                  </p>
                {/if}
                {#if backupError}
                  <p class="mt-4 text-sm font-medium text-red-700">
                    {backupError}
                  </p>
                {/if}
              </div>
            </section>

            <section class="rounded-xl border border-ink-100 bg-white p-9">
              <h2 class="mb-8 text-lg font-semibold text-ink-900">Formato</h2>
              <div class="grid gap-5">
                {#each EXPORT_FORMAT_OPTIONS as { format, label, extension } (format)}
                  <label
                    class="grid cursor-pointer grid-cols-[1fr_auto] rounded-lg border border-ink-100 px-6 py-5 text-lg font-semibold"
                    class:bg-accent-100={exportFormat === format}
                  >
                    <span>{label}</span>
                    <span class="text-ink-500">{extension}</span>
                    <input
                      class="sr-only"
                      type="radio"
                      name="export-format"
                      value={format}
                      bind:group={exportFormat}
                    />
                  </label>
                {/each}
              </div>
            </section>
          </div>

          {#if exportError}
            <p
              class="mt-6 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700"
            >
              {exportError}
            </p>
          {/if}
          {#if exportMessage}
            <p
              class="mt-6 rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700"
            >
              {exportMessage}
            </p>
          {/if}

          <div class="mt-10 flex justify-end">
            <button
              type="button"
              class="rounded-xl bg-accent-500 px-16 py-4 text-lg font-semibold text-white hover:bg-accent-600 disabled:opacity-60"
              disabled={isExporting}
              onclick={exportCurrentProject}
            >
              {isExporting ? 'Exportando...' : 'Exportar archivo'}
            </button>
          </div>
        </div>
      {:else if selectedItem?.itemType === 'folder'}
        <div class="mx-auto max-w-[112rem]">
          <div class="mb-10 flex items-center justify-between">
            <h1 class="text-xl font-semibold text-ink-900">
              {selectedItem.title}
            </h1>
            <div
              class="rounded-xl bg-accent-100 p-1 text-base font-semibold text-accent-500"
              role="tablist"
              aria-label="Vista de carpeta"
            >
              <button
                type="button"
                role="tab"
                aria-selected={folderView === 'board'}
                class="rounded-lg px-5 py-2"
                class:bg-white={folderView === 'board'}
                onclick={() => {
                  folderView = 'board';
                }}
              >
                Tablero
              </button>
              <button
                type="button"
                role="tab"
                aria-selected={folderView === 'outline'}
                class="rounded-lg px-5 py-2"
                class:bg-white={folderView === 'outline'}
                onclick={() => {
                  folderView = 'outline';
                }}
              >
                Esquema
              </button>
            </div>
          </div>

          {#if folderView === 'board'}
            <BoardView
              parent={selectedItem}
              items={selectedFolderDocumentChildren}
              selectedId={$selectedBinderItemIdStore}
              disabled={isBusy}
              onSelect={selectBinderItem}
              onCreateScene={createSceneFromBoard}
              onMoveBefore={moveBoardCardBefore}
              onSaveSynopsis={saveItemSynopsisNow}
            />
          {:else}
            <OutlineView
              parent={selectedItem}
              items={selectedFolderChildren}
              selectedId={$selectedBinderItemIdStore}
              onSelect={selectBinderItem}
            />
          {/if}
        </div>
      {:else if $currentDocumentStore && selectedItem && itemHasDocument(selectedItem)}
        <div class="mx-auto min-h-full w-full max-w-none">
          {#if activeSearchHighlight}
            <div
              class="mb-8 rounded-lg border border-yellow-200 bg-yellow-50 px-4 py-3 text-sm font-medium text-yellow-900"
            >
              Resultado abierto: “{activeSearchHighlight}”
            </div>
          {/if}

          {#key `${$currentDocumentStore.id}:${editorRenderNonce}`}
            <RichTextEditor
              content={editorContentJson}
              autoFocus={true}
              fontSize={editorFontSize}
              lineHeight={editorLineHeight}
              onChange={handleEditorChange}
            />
          {/key}
        </div>
      {:else}
        <div class="mx-auto mt-20 max-w-2xl">
          <p class="font-serif text-3xl leading-10 text-ink-900">
            Preparando hoja en blanco...
          </p>
        </div>
      {/if}
    </section>

    {#if showInspectorPanel}
      <aside
        aria-label="Inspector"
        class="overflow-auto border-l border-ink-100 bg-white px-9 py-9"
      >
        <div class="space-y-9">
          <div>
            <h2 class="text-lg font-semibold text-ink-900">Detalles</h2>
            {#if inspectorError}
              <p
                class="mt-3 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700"
              >
                {inspectorError}
              </p>
            {/if}
          </div>

          {#if $currentDocumentStore && selectedItem && itemHasDocument(selectedItem)}
            <label class="grid gap-3">
              <span class="text-base font-semibold text-ink-500">Sinopsis</span>
              <textarea
                class="min-h-36 resize-y rounded-lg border border-ink-100 bg-white px-5 py-4 text-base leading-7 text-ink-700"
                bind:value={synopsisDraft}
                oninput={scheduleSynopsisSave}
              ></textarea>
            </label>

            <label class="grid gap-3">
              <span class="text-base font-semibold text-ink-500">Estado</span>
              <input
                class="rounded-lg border border-ink-100 bg-white px-5 py-4 text-base text-ink-700"
                bind:value={statusDraft}
                oninput={scheduleMetadataSave}
                placeholder="• Borrador"
              />
            </label>

            <div class="grid gap-2">
              <span class="text-base font-semibold text-ink-500">Palabras</span>
              <p class="text-3xl font-semibold text-ink-900">
                {liveWordCount.toLocaleString()}
              </p>
              {#if targetWordCount}
                <div class="mt-2 h-1 rounded-full bg-ink-100">
                  <div
                    class="h-1 rounded-full bg-emerald-500"
                    style={`width: ${targetProgress}%`}
                  ></div>
                </div>
                <p class="text-sm font-medium text-ink-500">
                  Meta opcional: {targetWordCount.toLocaleString()}
                </p>
              {/if}
            </div>

            <details class="rounded-lg border border-ink-100 p-4">
              <summary class="cursor-pointer text-sm font-semibold text-ink-700"
                >Más detalles</summary
              >
              <div class="mt-4 grid gap-4">
                <label class="grid gap-2">
                  <span class="text-sm font-medium text-ink-500">Etiqueta</span>
                  <input
                    class="rounded-lg border border-ink-100 bg-white px-3 py-2 text-sm"
                    bind:value={labelDraft}
                    oninput={scheduleMetadataSave}
                  />
                </label>
                <label class="grid gap-2">
                  <span class="text-sm font-medium text-ink-500"
                    >Palabras clave</span
                  >
                  <input
                    class="rounded-lg border border-ink-100 bg-white px-3 py-2 text-sm"
                    bind:value={keywordsDraft}
                    oninput={scheduleMetadataSave}
                  />
                </label>
                <label class="grid gap-2">
                  <span class="text-sm font-medium text-ink-500"
                    >Meta de palabras opcional</span
                  >
                  <input
                    inputmode="numeric"
                    class="rounded-lg border border-ink-100 bg-white px-3 py-2 text-sm"
                    bind:value={targetWordCountDraft}
                    oninput={scheduleMetadataSave}
                  />
                </label>
                <label class="flex items-center gap-2 text-sm text-ink-700">
                  <input
                    type="checkbox"
                    class="h-4 w-4"
                    bind:checked={includeInExportDraft}
                    onchange={scheduleMetadataSave}
                  />
                  Incluir en exportación
                </label>
                <label class="grid gap-2">
                  <span class="text-sm font-medium text-ink-500">Notas</span>
                  <textarea
                    class="min-h-24 resize-y rounded-lg border border-ink-100 bg-white px-3 py-2 text-sm leading-6"
                    bind:value={noteDraft}
                    oninput={scheduleNoteSave}
                  ></textarea>
                </label>
                <div class="grid gap-3 border-t border-ink-100 pt-4">
                  <span class="text-sm font-semibold text-ink-700"
                    >Snapshots</span
                  >
                  <div class="grid grid-cols-[1fr_auto] gap-2">
                    <input
                      class="rounded-lg border border-ink-100 bg-white px-3 py-2 text-sm"
                      bind:value={snapshotName}
                      aria-label="Nombre del snapshot"
                    />
                    <button
                      type="button"
                      class="rounded-lg border border-ink-100 px-3 py-2 text-sm font-semibold hover:bg-ink-50"
                      onclick={createManualSnapshot}
                    >
                      Crear
                    </button>
                  </div>
                  {#if snapshots.length > 0}
                    <div class="grid gap-2">
                      {#each snapshots as snapshot (snapshot.id)}
                        <button
                          type="button"
                          class="rounded-lg border border-ink-100 px-3 py-2 text-left text-sm hover:bg-ink-50"
                          onclick={() => restoreSnapshotById(snapshot.id)}
                        >
                          <span class="font-medium">{snapshot.name}</span>
                          <span class="mt-1 block text-xs text-ink-500">
                            {new Date(snapshot.createdAt).toLocaleString()}
                          </span>
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>
            </details>
          {:else}
            <p class="text-sm leading-6 text-ink-500">
              Selecciona un documento para ver sus detalles.
            </p>
          {/if}
        </div>
      </aside>
    {/if}
  </div>

  <footer
    class="flex items-center justify-between border-t border-ink-100 bg-white px-10 text-base text-ink-500"
  >
    <p>{bottomLeftStatus}</p>
    <p>{bottomRightStatus}</p>
  </footer>
</main>
