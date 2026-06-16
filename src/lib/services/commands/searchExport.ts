import { z } from 'zod';

import { commandErrorSchema } from '$lib/schemas/project';
import {
  exportedFileSchema,
  exportProjectRequestSchema,
  searchProjectRequestSchema,
  searchResultsSchema,
  type ExportProjectRequest,
  type ExportedFile,
  type SearchProjectRequest,
  type SearchResult,
} from '$lib/schemas/searchExport';
import { TauriCommandError } from './project';
import { invokeCommand } from './runtime';

export async function searchProject(
  request: SearchProjectRequest,
): Promise<SearchResult[]> {
  return invokeSearchExportCommand(
    'search_project',
    { request: searchProjectRequestSchema.parse(request) },
    searchResultsSchema,
  );
}

export async function exportProject(
  request: ExportProjectRequest,
): Promise<ExportedFile> {
  return invokeSearchExportCommand(
    'export_project',
    { request: exportProjectRequestSchema.parse(request) },
    exportedFileSchema,
  );
}

async function invokeSearchExportCommand<T>(
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
