import { describe, expect, it } from 'vitest';

import {
  documentMetadataSchema,
  inspectorDataSchema,
  saveDocumentMetadataRequestSchema,
} from './inspector';

describe('inspector schemas', () => {
  it('validates document metadata', () => {
    const result = documentMetadataSchema.safeParse({
      documentId: '018fe3c4-5678-7abc-9def-0123456789ab',
      label: 'A',
      status: 'Borrador',
      targetWordCount: 1200,
      keywords: ['escena', 'inicio'],
      customFields: {},
      includeInExport: true,
    });

    expect(result.success).toBe(true);
  });

  it('validates metadata save requests', () => {
    const result = saveDocumentMetadataRequestSchema.safeParse({
      sessionId: '018fe3c4-5678-7abc-9def-0123456789ab',
      binderItemId: '018fe3c4-5678-7abc-9def-0123456789ac',
      label: null,
      status: 'Revisar',
      targetWordCount: null,
      keywords: [],
      customFields: {},
      includeInExport: false,
    });

    expect(result.success).toBe(true);
  });

  it('validates aggregate inspector data', () => {
    const result = inspectorDataSchema.safeParse({
      metadata: null,
      notes: [],
      snapshots: [],
    });

    expect(result.success).toBe(true);
  });
});
