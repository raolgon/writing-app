import { z } from 'zod';

import {
  backupRecordSchema,
  backupRecordsSchema,
  createBackupRequestSchema,
  listBackupsRequestSchema,
  type BackupRecord,
  type CreateBackupRequest,
  type ListBackupsRequest,
} from '$lib/schemas/backup';
import { commandErrorSchema } from '$lib/schemas/project';
import { TauriCommandError } from './project';
import { invokeCommand } from './runtime';

export async function createBackup(
  request: CreateBackupRequest,
): Promise<BackupRecord> {
  return invokeBackupCommand(
    'create_backup',
    { request: createBackupRequestSchema.parse(request) },
    backupRecordSchema,
  );
}

export async function listBackups(
  request: ListBackupsRequest,
): Promise<BackupRecord[]> {
  return invokeBackupCommand(
    'list_backups',
    { request: listBackupsRequestSchema.parse(request) },
    backupRecordsSchema,
  );
}

async function invokeBackupCommand<T>(
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
