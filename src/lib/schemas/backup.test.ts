import { describe, expect, it } from 'vitest';

import { backupRecordSchema, createBackupRequestSchema } from './backup';

describe('backup schemas', () => {
  it('validates manual backup requests', () => {
    const result = createBackupRequestSchema.safeParse({
      sessionId: '018fe3c4-5678-7abc-9def-0123456789ab',
    });

    expect(result.success).toBe(true);
  });

  it('validates backup records', () => {
    const result = backupRecordSchema.safeParse({
      id: '018fe3c4-5678-7abc-9def-0123456789ac',
      createdAt: '2026-06-15T00:00:00.000Z',
      path: 'backups/20260615T000000Z',
      kind: 'manual',
      formatVersion: 1,
      sizeBytes: 2048,
      status: 'complete',
    });

    expect(result.success).toBe(true);
  });
});
