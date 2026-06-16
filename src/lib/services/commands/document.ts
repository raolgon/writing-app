import { z } from 'zod';

import {
  clearDocumentRecoveryRequestSchema,
  documentRecordSchema,
  documentRecoveryStateSchema,
  getDocumentRecoveryRequestSchema,
  getDocumentRequestSchema,
  recordDocumentRecoveryRequestSchema,
  saveDocumentRequestSchema,
  type ClearDocumentRecoveryRequest,
  type DocumentRecord,
  type DocumentRecoveryState,
  type GetDocumentRecoveryRequest,
  type GetDocumentRequest,
  type RecordDocumentRecoveryRequest,
  type SaveDocumentRequest,
} from '$lib/schemas/document';
import { commandErrorSchema } from '$lib/schemas/project';
import { TauriCommandError } from './project';
import { invokeCommand } from './runtime';

const optionalRecoverySchema = documentRecoveryStateSchema.nullable();
const emptyResponseSchema = z
  .union([z.null(), z.undefined()])
  .transform(() => undefined);

export async function getDocument(
  request: GetDocumentRequest,
): Promise<DocumentRecord> {
  return invokeDocumentCommand(
    'get_document',
    { request: getDocumentRequestSchema.parse(request) },
    documentRecordSchema,
  );
}

export async function saveDocument(
  request: SaveDocumentRequest,
): Promise<DocumentRecord> {
  return invokeDocumentCommand(
    'save_document',
    { request: saveDocumentRequestSchema.parse(request) },
    documentRecordSchema,
  );
}

export async function recordDocumentRecovery(
  request: RecordDocumentRecoveryRequest,
): Promise<DocumentRecoveryState> {
  return invokeDocumentCommand(
    'record_document_recovery',
    { request: recordDocumentRecoveryRequestSchema.parse(request) },
    documentRecoveryStateSchema,
  );
}

export async function getDocumentRecovery(
  request: GetDocumentRecoveryRequest,
): Promise<DocumentRecoveryState | null> {
  return invokeDocumentCommand(
    'get_document_recovery',
    { request: getDocumentRecoveryRequestSchema.parse(request) },
    optionalRecoverySchema,
  );
}

export async function clearDocumentRecovery(
  request: ClearDocumentRecoveryRequest,
): Promise<void> {
  await invokeDocumentCommand(
    'clear_document_recovery',
    { request: clearDocumentRecoveryRequestSchema.parse(request) },
    emptyResponseSchema,
  );
}

async function invokeDocumentCommand<T>(
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
