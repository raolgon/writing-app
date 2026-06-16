import { z } from 'zod';

import { binderItemSchema, type BinderItem } from '$lib/schemas/binder';
import {
  documentRecordSchema,
  type DocumentRecord,
} from '$lib/schemas/document';
import {
  createSnapshotRequestSchema,
  documentMetadataSchema,
  getInspectorDataRequestSchema,
  inspectorDataSchema,
  projectNoteSchema,
  restoreSnapshotRequestSchema,
  saveBinderSynopsisRequestSchema,
  saveDocumentMetadataRequestSchema,
  saveProjectNoteRequestSchema,
  snapshotSchema,
  type CreateSnapshotRequest,
  type DocumentMetadata,
  type GetInspectorDataRequest,
  type InspectorData,
  type ProjectNote,
  type RestoreSnapshotRequest,
  type SaveBinderSynopsisRequest,
  type SaveDocumentMetadataRequest,
  type SaveProjectNoteRequest,
  type Snapshot,
} from '$lib/schemas/inspector';
import { commandErrorSchema } from '$lib/schemas/project';
import { TauriCommandError } from './project';
import { invokeCommand } from './runtime';

export async function getInspectorData(
  request: GetInspectorDataRequest,
): Promise<InspectorData> {
  return invokeInspectorCommand(
    'get_inspector_data',
    { request: getInspectorDataRequestSchema.parse(request) },
    inspectorDataSchema,
  );
}

export async function saveBinderSynopsis(
  request: SaveBinderSynopsisRequest,
): Promise<BinderItem> {
  return invokeInspectorCommand(
    'save_binder_synopsis',
    { request: saveBinderSynopsisRequestSchema.parse(request) },
    binderItemSchema,
  );
}

export async function saveDocumentMetadata(
  request: SaveDocumentMetadataRequest,
): Promise<DocumentMetadata> {
  return invokeInspectorCommand(
    'save_document_metadata',
    { request: saveDocumentMetadataRequestSchema.parse(request) },
    documentMetadataSchema,
  );
}

export async function saveProjectNote(
  request: SaveProjectNoteRequest,
): Promise<ProjectNote> {
  return invokeInspectorCommand(
    'save_project_note',
    { request: saveProjectNoteRequestSchema.parse(request) },
    projectNoteSchema,
  );
}

export async function createSnapshot(
  request: CreateSnapshotRequest,
): Promise<Snapshot> {
  return invokeInspectorCommand(
    'create_snapshot',
    { request: createSnapshotRequestSchema.parse(request) },
    snapshotSchema,
  );
}

export async function restoreSnapshot(
  request: RestoreSnapshotRequest,
): Promise<DocumentRecord> {
  return invokeInspectorCommand(
    'restore_snapshot',
    { request: restoreSnapshotRequestSchema.parse(request) },
    documentRecordSchema,
  );
}

async function invokeInspectorCommand<T>(
  command: string,
  args: Record<string, unknown>,
  schema: z.ZodType<T>,
): Promise<T> {
  try {
    const result = await invokeCommand<unknown>(command, args);
    return schema.parse(result);
  } catch (error) {
    const parsed = commandErrorSchema.safeParse(error);
    if (parsed.success) {
      throw new TauriCommandError(parsed.data);
    }

    throw error;
  }
}
