import { z } from 'zod';

import { binderItemTypeSchema } from './binder';

export const exportScopeSchema = z.enum(['document', 'folder', 'included']);
export const exportFormatSchema = z.enum(['txt', 'markdown', 'html', 'json']);

export const searchProjectRequestSchema = z.object({
  sessionId: z.string().uuid(),
  query: z.string(),
});

export const searchResultSchema = z.object({
  binderItemId: z.string().uuid(),
  title: z.string().min(1),
  itemType: binderItemTypeSchema,
  path: z.array(z.string().min(1)),
  snippet: z.string(),
  updatedAt: z.string().datetime(),
});

export const searchResultsSchema = z.array(searchResultSchema);

export const exportProjectRequestSchema = z.object({
  sessionId: z.string().uuid(),
  scope: exportScopeSchema,
  format: exportFormatSchema,
  binderItemId: z.string().uuid().nullable(),
  includeTitles: z.boolean(),
  separateScenes: z.boolean(),
});

export const exportedFileSchema = z.object({
  fileName: z.string().min(1),
  mimeType: z.string().min(1),
  content: z.string(),
});

export type ExportScope = z.infer<typeof exportScopeSchema>;
export type ExportFormat = z.infer<typeof exportFormatSchema>;
export type SearchProjectRequest = z.infer<typeof searchProjectRequestSchema>;
export type SearchResult = z.infer<typeof searchResultSchema>;
export type ExportProjectRequest = z.infer<typeof exportProjectRequestSchema>;
export type ExportedFile = z.infer<typeof exportedFileSchema>;
