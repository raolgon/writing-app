import { describe, expect, it } from 'vitest';

import {
  exportedFileSchema,
  exportProjectRequestSchema,
  searchResultSchema,
} from './searchExport';

describe('search and export schemas', () => {
  it('validates search results with binder paths and snippets', () => {
    const result = searchResultSchema.safeParse({
      binderItemId: '018fe3c4-5678-7abc-9def-0123456789ab',
      title: 'Capítulo 3',
      itemType: 'document',
      path: ['Manuscrito', 'Capítulo 3'],
      snippet: 'Entraron en la casa abandonada...',
      updatedAt: '2026-06-15T00:00:00.000Z',
    });

    expect(result.success).toBe(true);
  });

  it('validates export requests', () => {
    const result = exportProjectRequestSchema.safeParse({
      sessionId: '018fe3c4-5678-7abc-9def-0123456789ab',
      scope: 'included',
      format: 'markdown',
      binderItemId: null,
      includeTitles: true,
      separateScenes: true,
    });

    expect(result.success).toBe(true);
  });

  it('validates exported files', () => {
    const result = exportedFileSchema.safeParse({
      fileName: 'Borrador.md',
      mimeType: 'text/markdown',
      content: '# Capítulo 1',
    });

    expect(result.success).toBe(true);
  });
});
