import { describe, expect, it } from 'vitest';

import {
  createProjectRequestSchema,
  createDefaultProjectRequestSchema,
  projectJsonSchema,
  projectSessionSchema,
} from './project';

describe('projectJsonSchema', () => {
  it('accepts the initial project format', () => {
    const result = projectJsonSchema.safeParse({
      format: 'local-writer-project',
      formatVersion: 1,
      appVersion: '0.1.0',
      id: '018fe3c4-5678-7abc-9def-0123456789ab',
      title: 'Draft',
      description: '',
      projectType: 'blank',
      createdAt: '2026-06-15T00:00:00.000Z',
      updatedAt: '2026-06-15T00:00:00.000Z',
      lastOpenedAt: null,
      settings: {},
      customMetadataSchema: [],
      database: {
        file: 'project.db',
        schemaVersion: 1,
      },
    });

    expect(result.success).toBe(true);
  });

  it('rejects unsupported project versions', () => {
    const result = projectJsonSchema.safeParse({
      format: 'local-writer-project',
      formatVersion: 99,
      appVersion: '0.1.0',
      id: '018fe3c4-5678-7abc-9def-0123456789ab',
      title: 'Draft',
      description: '',
      projectType: 'blank',
      createdAt: '2026-06-15T00:00:00.000Z',
      updatedAt: '2026-06-15T00:00:00.000Z',
      lastOpenedAt: null,
      settings: {},
      customMetadataSchema: [],
      database: {
        file: 'project.db',
        schemaVersion: 1,
      },
    });

    expect(result.success).toBe(false);
  });

  it('validates project creation requests', () => {
    const result = createProjectRequestSchema.safeParse({
      folderPath: '/tmp/draft',
      title: 'Draft',
      description: '',
      projectType: 'novel',
    });

    expect(result.success).toBe(true);
  });

  it('validates default project creation requests', () => {
    const result = createDefaultProjectRequestSchema.safeParse({
      title: 'Draft',
      description: '',
      projectType: 'novel',
    });

    expect(result.success).toBe(true);
  });

  it('validates project sessions returned by Tauri', () => {
    const result = projectSessionSchema.safeParse({
      sessionId: '018fe3c4-5678-7abc-9def-0123456789ab',
      project: {
        id: '018fe3c4-5678-7abc-9def-0123456789ac',
        title: 'Draft',
        description: '',
        projectType: 'blank',
        path: '/tmp/draft',
        formatVersion: 1,
        databaseSchemaVersion: 1,
        lastOpenedAt: '2026-06-15T00:00:00.000Z',
      },
    });

    expect(result.success).toBe(true);
  });
});
