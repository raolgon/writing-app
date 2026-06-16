# Architecture

## Current repository state

The repository is effectively empty. It contains no application source files, package configuration, Tauri setup, tests, or existing Git history visible from this workspace. The only directories present before Phase 1 were environment-managed `.agents`, `.codex`, and `.git` placeholders.

This means the application can be scaffolded cleanly in Phase 2 without needing to preserve existing runtime code or UI conventions.

## Product constraints

The application is a local-first desktop writing tool. The architecture must preserve these constraints:

- No remote backend.
- No accounts, subscriptions, telemetry, sync, or cloud services.
- All project data is stored in a user-selected folder.
- Projects remain portable and understandable outside the application.
- Core content is stored as structured TipTap JSON plus derived plain text.
- Native file, lock, backup, database, and export operations are handled through Tauri/Rust.
- UI state is kept separate from persisted project state.

## Initial technology decisions

| Area            | Decision                                                     | Reason                                                                                                                                                                                                                   |
| --------------- | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Desktop shell   | Tauri 2                                                      | Required stack; smaller local desktop container than Electron.                                                                                                                                                           |
| Frontend        | SvelteKit with strict TypeScript                             | Required stack; supports component-level UI and typed services.                                                                                                                                                          |
| Native layer    | Rust Tauri commands                                          | Required for local filesystem, SQLite, locks, backups, export, and future encryption boundary.                                                                                                                           |
| Database        | SQLite in each project folder                                | Portable, durable, local, supports transactions and FTS5.                                                                                                                                                                |
| ORM             | Start with a typed Rust repository layer rather than Drizzle | The canonical database access should live in Rust because Tauri commands own filesystem and project locks. Drizzle can be reconsidered only if direct frontend-side SQLite access becomes necessary and secure in Tauri. |
| Validation      | Zod in frontend, Rust validation at command boundaries       | Zod protects UI inputs and command payloads; Rust validates trusted persistence operations.                                                                                                                              |
| Editor          | TipTap                                                       | Required; stores document JSON as primary rich text format.                                                                                                                                                              |
| Styling         | Tailwind CSS with accessible custom components               | Required; avoids premature dependency on a complex component system. Melt UI can be added where it reduces accessibility work.                                                                                           |
| Tests           | Vitest, Playwright, Rust tests                               | Unit tests for frontend/domain utilities, E2E for flows, Rust tests for persistence and filesystem safety.                                                                                                               |
| Package manager | pnpm                                                         | Required.                                                                                                                                                                                                                |

## High-level runtime model

The application has one active writable project per window for the MVP.

1. The user creates or opens a project folder.
2. Rust validates `project.json`, acquires a local write lock, opens `project.db`, and runs migrations.
3. Rust returns a typed project session summary to the SvelteKit app.
4. The UI loads only the binder tree, selection, visible document, metadata, notes, and view state needed for the current screen.
5. Document edits are debounced in the frontend and saved through explicit Tauri commands.
6. Rust persists changes inside SQLite transactions, updates search indexes, updates session recovery state, and writes `project.json` atomically when required.
7. Backups and exports are written inside the project folder.

## Source layout

Planned structure:

```text
src/
  lib/
    components/
      app-shell/
      binder/
      corkboard/
      editor/
      inspector/
      outline/
      shared/
    i18n/
    schemas/
    services/
      commands/
      export/
      search/
    stores/
      editor/
      project/
      saving/
      selection/
      session/
      ui/
    types/
    utils/
  routes/
src-tauri/
  src/
    backup/
    commands/
    database/
    domain/
    errors/
    export/
    filesystem/
    locks/
    migrations/
    search/
    session/
```

### Frontend responsibilities

- Render the desktop UI.
- Manage temporary UI state, current selection, editor state, and save indicators.
- Validate command inputs with Zod before invoking Tauri.
- Keep stores small and purpose-specific.
- Avoid keeping all document content in memory.
- Convert TipTap editor state into JSON payloads for save commands.
- Provide accessible keyboard interactions and visible focus states.

### Rust responsibilities

- Create and open projects.
- Validate project folders and format versions.
- Acquire and release project locks.
- Own SQLite connections and migrations.
- Execute project mutations in transactions.
- Maintain FTS5 search data.
- Write `project.json` atomically.
- Manage backups, recovery records, and exports.
- Return typed errors suitable for user-facing messages.

## Domain model

The MVP uses these aggregate boundaries:

- `Project`: format metadata, settings, custom metadata schema, and version.
- `BinderItem`: hierarchical project structure and non-destructive deletion state.
- `Document`: rich content, plain text, counts, and revision.
- `DocumentMetadata`: export flags, labels, status, targets, keywords, and custom fields.
- `ProjectNote`: project-level or binder-item-specific notes.
- `Snapshot`: manual historical document versions.
- `ProjectSession`: active lock, last opened document, recovery markers, and dirty state metadata.
- `BackupManifest`: backup records and retention policy.

## Persistence design

Each project folder contains:

```text
project-name/
  project.json
  project.db
  assets/
  backups/
  exports/
```

SQLite is the source of truth for structured content. `project.json` is the source of truth for human-readable project metadata and format compatibility checks.

Persistence rules:

- Use SQLite transactions for all multi-table mutations.
- Enable foreign keys on every connection.
- Use WAL mode unless it conflicts with portability or backup needs.
- Use FTS5 for global search.
- Store TipTap JSON as text containing canonical JSON.
- Store derived plain text for search, export, and recovery.
- Never hard-delete user content during normal delete operations; move items to an internal trash state.
- Atomic writes for `project.json`: write temp file, flush, then rename.

## Project locks

The MVP will use a local lock file in the project folder, for example `.write-lock`.

The lock should contain:

- Application name and version.
- Hostname where available.
- Process id.
- Created timestamp.
- Last heartbeat timestamp.

Opening a locked project for writing must fail with a clear message unless the lock is stale. Stale lock recovery must be explicit and conservative.

## Recovery model

Recovery has three layers:

1. Debounced autosave for normal editing.
2. Immediate save when switching documents or closing the window.
3. Session recovery records for dirty editor state and last active document.

The MVP should write recovery records locally and reconcile them when the project is reopened. The user should see a clear recovery prompt if unsaved content is detected.

## Backup model

Backups are local only and stored in `backups/`.

Initial backup strategy:

- Create automatic backups on project open or close once the project has changed.
- Include `project.json`, `project.db`, and optionally assets.
- Store a manifest with timestamp, application version, format version, and size.
- Apply configurable retention by count.
- Use a temporary backup name and rename after successful completion.

## Export model

Exports are implemented through a Rust-side `Exporter` interface:

```text
Exporter
  id
  display_name
  extension
  export_document(input, options)
  export_collection(input, options)
  export_project(input, options)
```

Initial exporters:

- Plain text `.txt`
- Markdown `.md`
- HTML
- Full project JSON

Future exporters should be able to add DOCX, PDF, EPUB, Fountain, and Final Draft FDX without changing the core persistence model.

## Search model

SQLite FTS5 will index:

- Binder item titles.
- Binder item synopses.
- Document plain text.
- Project note content.
- Keywords from document metadata.

Search results return:

- Matched binder item or note id.
- Display title.
- Project path.
- Snippet.
- Match count where practical.

The frontend opens the selected document and applies temporary highlight state. The highlight is UI-only and is not persisted.

## UI architecture

The main UI has four zones:

- Left binder tree.
- Central work area with editor, board, or outline view.
- Right inspector.
- Top application bar.

Panel sizes, focus mode, theme, editor width, font size, line height, and active view are UI preferences. Project-specific display preferences may be persisted in `project.settings`; machine-specific preferences may live in the system config directory.

The initial visual direction is restrained and writing-focused:

- Semantic layout.
- Resizable panels.
- High contrast in light and dark themes.
- No decorative gradients or cloud-connected affordances.
- Clear save and lock states.

## State management

Stores should remain small:

- `projectSessionStore`: open project, paths, lock state, format version.
- `binderStore`: tree summary and expanded state.
- `selectionStore`: selected binder item and active folder context.
- `editorStore`: current document content, revision, dirty state.
- `savingStore`: queued, saving, saved, failed, recovery-needed state.
- `searchStore`: query, results, active result.
- `uiStore`: panel sizes, active view, theme, focus mode.

The application must not load every document body into a global store.

## Error handling

Rust commands return typed application errors with stable codes:

- `ProjectNotFound`
- `UnsupportedProjectVersion`
- `ProjectLocked`
- `ProjectCorrupt`
- `MigrationFailed`
- `DatabaseError`
- `FilesystemError`
- `ValidationError`
- `BackupFailed`
- `ExportFailed`

The frontend maps these codes to clear Spanish UI messages.

## Accessibility baseline

The MVP must support:

- Keyboard access to primary commands.
- Visible focus for all interactive controls.
- Semantic buttons, tree controls, text fields, and tables.
- ARIA only where native semantics are insufficient.
- Non-color status labels alongside colors.
- Reduced-motion support.
- Resizable text and panels without content overlap.

## Important risks

- Tauri 2 filesystem and dialog permissions must be configured carefully so project folders are user-selected but not hidden.
- SQLite FTS5 availability should be verified in the bundled SQLite library used by Rust.
- Autosave and TipTap updates can create race conditions if document switching is not handled with revisions.
- Lock files can become stale after crashes; stale lock recovery must avoid data loss.
- Exporting rich TipTap content to Markdown and HTML requires a deterministic serializer.
- Drag-and-drop tree reordering needs strong transaction boundaries to avoid corrupt positions.
- Playwright coverage for Tauri desktop can be more complex than browser-only SvelteKit testing; the first E2E may target the web shell plus mocked Tauri commands, with later Tauri integration tests added once the app runs.

## MVP scope

The MVP includes:

- Project creation and opening.
- Portable project folder format.
- SQLite migrations and repositories.
- Binder CRUD, nesting, reordering, and trash.
- Rich text editor with TipTap JSON persistence and plain-text derivation.
- Autosave, immediate save on switch, word and character counts.
- Inspector metadata, notes, and manual snapshots.
- Board and outline views for folder children.
- Global local search using FTS5.
- Export to text, Markdown, HTML, and full project JSON.
- Local backups with retention.
- Recovery after unexpected closure.
- Unit and E2E tests for core flows.

The MVP does not include:

- Cloud sync.
- Accounts.
- Collaboration.
- Telemetry.
- Encryption.
- AI features.
- Specialized screenplay editing.
- DOCX, PDF, EPUB, Fountain, or FDX export.

## Phase 1 outcome

Phase 1 establishes the architecture and project format documentation only. No runtime code, UI components, dependencies, or Tauri setup are implemented in this phase.
