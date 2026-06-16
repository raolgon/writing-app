import { z } from 'zod';

export const backupKindSchema = z.enum(['automatic', 'manual']);

export const createBackupRequestSchema = z.object({
  sessionId: z.string().uuid(),
});

export const listBackupsRequestSchema = createBackupRequestSchema;

export const backupRecordSchema = z.object({
  id: z.string().uuid(),
  createdAt: z.string().datetime(),
  path: z.string().min(1),
  kind: backupKindSchema,
  formatVersion: z.number().int().positive(),
  sizeBytes: z.number().int().nonnegative().nullable(),
  status: z.string().min(1),
});

export const backupRecordsSchema = z.array(backupRecordSchema);

export type BackupKind = z.infer<typeof backupKindSchema>;
export type CreateBackupRequest = z.infer<typeof createBackupRequestSchema>;
export type ListBackupsRequest = z.infer<typeof listBackupsRequestSchema>;
export type BackupRecord = z.infer<typeof backupRecordSchema>;
