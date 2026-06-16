import { z } from 'zod';

const tiptapJsonSchema: z.ZodType<unknown> = z.unknown();

export const documentRecordSchema = z.object({
  id: z.string().uuid(),
  binderItemId: z.string().uuid(),
  contentJson: tiptapJsonSchema,
  contentPlainText: z.string(),
  wordCount: z.number().int().nonnegative(),
  characterCount: z.number().int().nonnegative(),
  revision: z.number().int().nonnegative(),
  createdAt: z.string().datetime(),
  updatedAt: z.string().datetime(),
});

export const getDocumentRequestSchema = z.object({
  sessionId: z.string().uuid(),
  binderItemId: z.string().uuid(),
});

export const saveDocumentRequestSchema = z.object({
  sessionId: z.string().uuid(),
  binderItemId: z.string().uuid(),
  contentJson: tiptapJsonSchema,
  contentPlainText: z.string(),
  expectedRevision: z.number().int().nonnegative().nullable(),
});

export const documentRecoveryStateSchema = z.object({
  binderItemId: z.string().uuid(),
  contentJson: tiptapJsonSchema,
  contentPlainText: z.string(),
  revision: z.number().int().nonnegative(),
  updatedAt: z.string().datetime(),
});

export const recordDocumentRecoveryRequestSchema = z.object({
  sessionId: z.string().uuid(),
  binderItemId: z.string().uuid(),
  contentJson: tiptapJsonSchema,
  contentPlainText: z.string(),
  revision: z.number().int().nonnegative(),
});

export const getDocumentRecoveryRequestSchema = z.object({
  sessionId: z.string().uuid(),
  binderItemId: z.string().uuid(),
});

export const clearDocumentRecoveryRequestSchema =
  getDocumentRecoveryRequestSchema;

export type DocumentRecord = z.infer<typeof documentRecordSchema>;
export type GetDocumentRequest = z.infer<typeof getDocumentRequestSchema>;
export type SaveDocumentRequest = z.infer<typeof saveDocumentRequestSchema>;
export type DocumentRecoveryState = z.infer<typeof documentRecoveryStateSchema>;
export type RecordDocumentRecoveryRequest = z.infer<
  typeof recordDocumentRecoveryRequestSchema
>;
export type GetDocumentRecoveryRequest = z.infer<
  typeof getDocumentRecoveryRequestSchema
>;
export type ClearDocumentRecoveryRequest = z.infer<
  typeof clearDocumentRecoveryRequestSchema
>;
