import { z } from 'zod';

const jsonValueSchema: z.ZodType<unknown> = z.unknown();

export const documentMetadataSchema = z.object({
  documentId: z.string().uuid(),
  label: z.string().nullable(),
  status: z.string().nullable(),
  targetWordCount: z.number().int().nonnegative().nullable(),
  keywords: z.array(z.string()),
  customFields: jsonValueSchema,
  includeInExport: z.boolean(),
});

export const projectNoteSchema = z.object({
  id: z.string().uuid(),
  projectId: z.string().uuid(),
  binderItemId: z.string().uuid().nullable(),
  title: z.string().min(1),
  content: z.string(),
  createdAt: z.string().datetime(),
  updatedAt: z.string().datetime(),
});

export const snapshotSchema = z.object({
  id: z.string().uuid(),
  documentId: z.string().uuid(),
  name: z.string().min(1),
  contentJson: jsonValueSchema,
  contentPlainText: z.string(),
  createdAt: z.string().datetime(),
});

export const inspectorDataSchema = z.object({
  metadata: documentMetadataSchema.nullable(),
  notes: z.array(projectNoteSchema),
  snapshots: z.array(snapshotSchema),
});

export const getInspectorDataRequestSchema = z.object({
  sessionId: z.string().uuid(),
  binderItemId: z.string().uuid(),
});

export const saveBinderSynopsisRequestSchema = z.object({
  sessionId: z.string().uuid(),
  itemId: z.string().uuid(),
  synopsis: z.string(),
});

export const saveDocumentMetadataRequestSchema = z.object({
  sessionId: z.string().uuid(),
  binderItemId: z.string().uuid(),
  label: z.string().nullable(),
  status: z.string().nullable(),
  targetWordCount: z.number().int().nonnegative().nullable(),
  keywords: z.array(z.string()),
  customFields: jsonValueSchema,
  includeInExport: z.boolean(),
});

export const saveProjectNoteRequestSchema = z.object({
  sessionId: z.string().uuid(),
  id: z.string().uuid().nullable(),
  binderItemId: z.string().uuid().nullable(),
  title: z.string().min(1),
  content: z.string(),
});

export const createSnapshotRequestSchema = z.object({
  sessionId: z.string().uuid(),
  binderItemId: z.string().uuid(),
  name: z.string().min(1),
});

export const restoreSnapshotRequestSchema = z.object({
  sessionId: z.string().uuid(),
  snapshotId: z.string().uuid(),
});

export type DocumentMetadata = z.infer<typeof documentMetadataSchema>;
export type ProjectNote = z.infer<typeof projectNoteSchema>;
export type Snapshot = z.infer<typeof snapshotSchema>;
export type InspectorData = z.infer<typeof inspectorDataSchema>;
export type GetInspectorDataRequest = z.infer<
  typeof getInspectorDataRequestSchema
>;
export type SaveBinderSynopsisRequest = z.infer<
  typeof saveBinderSynopsisRequestSchema
>;
export type SaveDocumentMetadataRequest = z.infer<
  typeof saveDocumentMetadataRequestSchema
>;
export type SaveProjectNoteRequest = z.infer<
  typeof saveProjectNoteRequestSchema
>;
export type CreateSnapshotRequest = z.infer<typeof createSnapshotRequestSchema>;
export type RestoreSnapshotRequest = z.infer<
  typeof restoreSnapshotRequestSchema
>;
