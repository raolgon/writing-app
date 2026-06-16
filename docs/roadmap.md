# Roadmap

## Phase 1: Analysis and architecture

Status: complete.

Completed:

- Inspected the repository.
- Confirmed there is no existing app source to preserve.
- Defined the initial architecture.
- Defined the portable project folder format.
- Chose a Rust-owned SQLite persistence layer for the first implementation.
- Defined MVP scope and non-goals.
- Documented risks and testing expectations.

Created:

- `docs/architecture.md`
- `docs/project-format.md`
- `docs/roadmap.md`

No runtime code was implemented in this phase.

## Phase 2: Project foundation

Status: complete.

Completed:

- Scaffolded SvelteKit.
- Added Tauri 2 project structure.
- Configured strict TypeScript.
- Added Tailwind CSS.
- Added Zod and initial project format validation schema.
- Added Vitest.
- Added Playwright.
- Configured ESLint and Prettier.
- Added pnpm lockfile and workspace settings.
- Added an initial accessible application shell only.
- Added placeholder Rust modules for commands, domain, database, migrations, and errors.
- Added a minimal Tauri command returning application metadata.
- Added a temporary local Tauri icon placeholder required by `generate_context!()`.

Verification results:

- `pnpm install`: passed.
- `pnpm format:check`: passed.
- `pnpm check`: passed with 0 errors and 0 warnings.
- `pnpm lint`: passed.
- `pnpm test:unit`: passed, 1 file and 2 tests.
- `pnpm build`: passed.
- `pnpm test:e2e`: passed, 1 Chromium smoke test.
- `cargo test`: passed, 2 Rust tests.
- `pnpm tauri build`: passed and produced `src-tauri/target/release/local-writer`.

Native dependency note:

- Native compilation originally failed because `dbus-1` was not discoverable through `pkg-config`.
- After local Tauri/Linux prerequisites were installed, `cargo test` and `pnpm tauri build` completed successfully.
- Tauri emitted a warning that the bundle identifier `dev.local-writer.app` ends with `.app`, which is not recommended for macOS because it conflicts with the app bundle extension. This should be renamed before distribution.

## Phase 3: Persistence foundation

Status: complete.

Completed:

- Implemented project creation in Rust.
- Implemented project opening and validation in Rust.
- Implemented atomic `project.json` writes.
- Implemented local `project.db` creation.
- Implemented SQLite migration runner.
- Added the initial schema for projects, binder items, documents, metadata, notes, snapshots, session state, and backup records.
- Implemented typed Rust project persistence services.
- Added Tauri commands for `create_project`, `create_default_project`, `open_project`, and `close_project`.
- Added local `.write-lock` write locks.
- Added structured command errors with stable codes.
- Added TypeScript/Zod command contracts and frontend command service wrappers.
- Updated the initial project creation UX to avoid asking for a filesystem path. New projects are created under a user-owned default directory (`Documents/Local Writer Projects` when available, otherwise `Local Writer Projects` in the user home), with stable folder-name sanitization and collision suffixes.

Verification results:

- `cargo fmt --check`: passed.
- `cargo test`: passed, 7 Rust tests.
- `pnpm format:check`: passed.
- `pnpm check`: passed with 0 errors and 0 warnings.
- `pnpm lint`: passed.
- `pnpm test:unit`: passed, 1 file and 4 tests.
- `pnpm build`: passed.
- `pnpm test:e2e`: passed, 1 Chromium smoke test.
- `pnpm tauri build`: passed and produced `src-tauri/target/release/local-writer`.

Notes:

- The Tauri bundle identifier warning from Phase 2 remains: `dev.local-writer.app` ends with `.app`, which should be changed before distribution.
- The persistence foundation is intentionally command/service level only. Binder CRUD, document editing, autosave, backups, and recovery are still scheduled for later phases.

## Phase 4: Binder

Status: complete.

Completed:

- Added Rust binder repository operations.
- Added Tauri commands for listing, creating, renaming, expanding/collapsing, duplicating, moving, reordering, trashing, and restoring binder items.
- Added transaction-backed sibling reorder and move operations.
- Added cycle prevention when moving items between parents.
- Added non-destructive trash via `trashed_at`.
- Added empty document rows and metadata for non-folder binder items so Phase 5 can attach editor content without a schema change.
- Added TypeScript/Zod schemas for binder commands and returned binder items.
- Added frontend command wrappers.
- Added small stores for project session, binder items/tree, and selection.
- Replaced the placeholder sidebar with a connected Binder UI.
- Added inline rename, contextual menu, expand/collapse, duplicate, trash, and root/child creation controls.
- Added basic drag-and-drop: dropping on a folder nests at the end; dropping on a non-folder moves before that item.

Verification results:

- `cargo test`: passed, 11 Rust tests.
- `cargo fmt --check`: passed.
- `pnpm format:check`: passed.
- `pnpm check`: passed with 0 errors and 0 warnings.
- `pnpm lint`: passed.
- `pnpm test:unit`: passed, 3 files and 8 tests.
- `pnpm build`: passed.
- `pnpm test:e2e`: passed, 1 Chromium smoke test.
- `pnpm tauri build`: passed and produced `src-tauri/target/release/local-writer`.

Notes:

- The Binder UI is intentionally functional and compact. It is not the final interaction design.
- Full keyboard reordering and polished drag affordances can be improved later, but the persistence and command layer already supports move and reorder operations.

## Phase 5: Editor

Status: complete.

Completed:

- Added Rust document repository operations.
- Added Tauri commands for `get_document`, `save_document`, `record_document_recovery`, `get_document_recovery`, and `clear_document_recovery`.
- Stored TipTap JSON as the primary document content.
- Derived and persisted plain text for search/export foundations.
- Calculated word and character counts.
- Added optimistic revision checks and `RevisionConflict` errors.
- Added initial recovery records in `session_state`.
- Added TypeScript/Zod document schemas and command wrappers.
- Added small editor and saving stores.
- Added a TipTap editor in the central pane.
- Added formatting controls for bold, italic, underline, strikethrough, heading, lists, quote, separator, link, undo, and redo.
- Added `Ctrl/Cmd+S` immediate save, `Ctrl/Cmd+K` link editing, and `F11` focus mode.
- Added debounced recovery recording and autosave.
- Added immediate save when switching selected Binder items.
- Added configurable text width, font size, and line height controls.

Verification results:

- `cargo fmt --check`: passed.
- `cargo test`: passed, 14 Rust tests.
- `pnpm format:check`: passed.
- `pnpm check`: passed with 0 errors and 0 warnings.
- `pnpm lint`: passed.
- `pnpm test:unit`: passed, 4 files and 10 tests.
- `pnpm build`: passed.
- `pnpm test:e2e`: passed, 2 Chromium tests.
- `pnpm tauri build`: passed and produced `src-tauri/target/release/local-writer`.

Notes:

- The E2E persistence flow uses a mocked Tauri IPC layer in browser Playwright. The real persistence path is covered by Rust tests and Tauri build verification.
- Editor preferences are UI-local in this phase. Persisting them into project settings can be added later.

## Phase 6: Inspector

Status: complete.

Completed:

- Added typed Rust inspector persistence services.
- Added Tauri commands for inspector aggregate loading, synopsis updates, metadata updates, note save, snapshot creation, and snapshot restore.
- Added TypeScript/Zod inspector schemas and command wrappers.
- Replaced the placeholder right inspector with editable fields for synopsis, status, label, target word count, keywords, include-in-export, document notes, and snapshots.
- Added debounced local saves for synopsis, metadata, and notes.
- Added manual snapshot creation and restore. Restoring a snapshot updates document content, counts, revision, and remounts the TipTap editor with restored JSON.
- Added Rust tests for inspector metadata/note persistence and snapshot creation/restore.
- Added Zod unit tests for inspector contracts.

Verification results:

- `cargo fmt --check`: passed.
- `cargo test`: passed, 18 Rust tests.
- `pnpm format:check`: passed.
- `pnpm check`: passed with 0 errors and 0 warnings.
- `pnpm lint`: passed.
- `pnpm test:unit`: passed, 5 files and 14 tests.
- `pnpm build`: passed.
- `pnpm test:e2e`: passed, 2 Chromium tests.

Notes:

- `customFields` are persisted through the command contract, but the MVP UI does not yet include a dynamic custom-field editor.
- The inspector currently exposes one primary note textarea for the selected document. The storage layer supports multiple project notes.

## Phase 7: Board and outline

Status: complete.

Completed:

- Board view for child documents of a folder.
- Card reordering from the board using the existing persisted binder move command.
- Quick synopsis editing on board cards using the existing inspector synopsis command.
- New scene creation from the board.
- Outline table for folder children.
- Sortable outline columns for title, binder order, status, and updated date.
- Resizable outline column controls.
- Shared folder-view helpers for direct children, board document children, and outline sorting.
- Playwright coverage for board quick edit, scene creation, card reorder, and outline sorting.

Verification results:

- `cargo fmt --check`: passed.
- `cargo test`: passed, 18 Rust tests.
- `pnpm format:check`: passed.
- `pnpm check`: passed with 0 errors and 0 warnings.
- `pnpm lint`: passed.
- `pnpm test:unit`: passed, 6 files and 17 tests.
- `pnpm build`: passed.
- `pnpm test:e2e`: passed, 4 Chromium tests.

Notes:

- This phase did not require new Rust persistence commands. It reuses the Phase 4 binder move command and the Phase 6 synopsis command.
- The board currently reorders by dropping one card before another card. More elaborate lane/status workflows can be added later.

## Phase 8: Search and export

Status: complete.

Completed:

- Add SQLite FTS5 index.
- Search titles, synopses, document text, notes, and keywords.
- Show snippets and binder paths.
- Open search results and apply temporary highlight.
- Export document, folder, and included documents.
- Implement TXT, Markdown, HTML, and full JSON exporters.
- Added Tauri commands for project search and export.
- Added TypeScript/Zod schemas and frontend command wrappers for search/export.
- Added Figma-aligned search and export views to the application shell.
- Added local browser download for exported files.

Verification results:

- `cargo fmt --check`: passed.
- `cargo test`: passed, 20 Rust tests.
- `pnpm format:check`: passed.
- `pnpm check`: passed with 0 errors and 0 warnings.
- `pnpm lint`: passed.
- `pnpm test:unit`: passed, 7 files and 20 tests.
- `pnpm build`: passed.
- `pnpm test:e2e`: passed, 6 Chromium tests.

## Phase 9: Backups and stability

Status: complete.

Completed:

- Automatic local backups.
- Manual backup command.
- Backup retention.
- Backup records.
- Recovery prompts for unsaved state.
- Crash-resilience checks.
- Polish error messages.
- Fill critical test gaps.
- Added directory-based local backups under `backups/`.
- Added `backup.json` manifests with app, project, database, and kind metadata.
- Added SQLite backup record listing.
- Added best-effort automatic backup on project close.
- Added manual backup creation from the export/stability view.
- Added browser fallback backup records for the Vite development runtime.
- Kept existing unsaved recovery prompt flow for simulated crash recovery.

Verification results:

- `cargo fmt --check`: passed.
- `cargo test`: passed, 22 Rust tests.
- `pnpm format:check`: passed.
- `pnpm check`: passed with 0 errors and 0 warnings.
- `pnpm lint`: passed.
- `pnpm test:unit`: passed, 8 files and 22 tests.
- `pnpm build`: passed.
- `pnpm test:e2e`: passed, 6 Chromium tests.

## Phase 10: MVP readiness

Status: complete.

Completed:

- Audited the MVP definition after Phase 9.
- Added a UI path to open an existing project folder from the project title control.
- Reused the existing `open_project` and `close_project` commands so reopening releases the previous project lock and creates the automatic close backup.
- Kept browser/Vite fallback behavior compatible with project opening.
- Added E2E coverage for opening an existing project and loading its binder/document content.

Verification results:

- `cargo fmt --check`: passed.
- `cargo test`: passed, 22 Rust tests.
- `pnpm format:check`: passed.
- `pnpm check`: passed with 0 errors and 0 warnings.
- `pnpm lint`: passed.
- `pnpm test:unit`: passed, 8 files and 22 tests.
- `pnpm build`: passed.
- `pnpm test:e2e`: passed, 7 Chromium tests.

## MVP definition

The MVP is complete only when a user can:

- Create a local project folder.
- Reopen it without accounts or network access.
- Organize documents hierarchically.
- Write rich text that autosaves.
- Edit metadata and notes.
- View folder children as board cards and outline rows.
- Search project text locally.
- Export to initial formats.
- Create and rotate local backups.
- Recover unsaved work after an unexpected close.

## Deferred features

Deferred until after the MVP:

- Encryption.
- Specialized screenplay editor.
- Fountain export.
- Final Draft FDX export.
- DOCX, PDF, and EPUB export.
- Plugin system.
- Cloud sync.
- Collaboration.
- Authentication.
- AI features.
- Telemetry.
