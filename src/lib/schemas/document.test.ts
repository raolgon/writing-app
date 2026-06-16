import { describe, expect, it } from 'vitest';

import { documentRecordSchema, saveDocumentRequestSchema } from './document';

describe('document schemas', () => {
  it('validates document records', () => {
    const result = documentRecordSchema.safeParse({
      id: '018fe3c4-5678-7abc-9def-0123456789ab',
      binderItemId: '018fe3c4-5678-7abc-9def-0123456789ac',
      contentJson: { type: 'doc', content: [] },
      contentPlainText: 'Hello',
      wordCount: 1,
      characterCount: 5,
      revision: 1,
      createdAt: '2026-06-15T00:00:00.000Z',
      updatedAt: '2026-06-15T00:00:00.000Z',
    });

    expect(result.success).toBe(true);
  });

  it('validates save requests', () => {
    const result = saveDocumentRequestSchema.safeParse({
      sessionId: '018fe3c4-5678-7abc-9def-0123456789ab',
      binderItemId: '018fe3c4-5678-7abc-9def-0123456789ac',
      contentJson: { type: 'doc', content: [] },
      contentPlainText: '',
      expectedRevision: 0,
    });

    expect(result.success).toBe(true);
  });
});
