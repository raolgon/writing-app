import { z } from 'zod';

import { PROJECT_FORMAT, PROJECT_FORMAT_VERSION } from '$lib/config/app';

export const projectTypeSchema = z.enum(['blank', 'novel', 'screenplay']);

export const createProjectRequestSchema = z.object({
  folderPath: z.string().min(1),
  title: z.string().min(1),
  description: z.string(),
  projectType: projectTypeSchema,
});

export const createDefaultProjectRequestSchema = z.object({
  title: z.string().min(1),
  description: z.string(),
  projectType: projectTypeSchema,
});

export const openProjectRequestSchema = z.object({
  folderPath: z.string().min(1),
});

export const closeProjectRequestSchema = z.object({
  sessionId: z.string().uuid(),
});

export const projectSummarySchema = z.object({
  id: z.string().uuid(),
  title: z.string().min(1),
  description: z.string(),
  projectType: projectTypeSchema,
  path: z.string().min(1),
  formatVersion: z.literal(PROJECT_FORMAT_VERSION),
  databaseSchemaVersion: z.number().int().positive(),
  lastOpenedAt: z.string().datetime().nullable(),
});

export const projectSessionSchema = z.object({
  sessionId: z.string().uuid(),
  project: projectSummarySchema,
});

export const closeProjectResponseSchema = z.object({
  closed: z.boolean(),
});

export const commandErrorSchema = z.object({
  code: z.string().min(1),
  message: z.string().min(1),
});

export const projectJsonSchema = z.object({
  format: z.literal(PROJECT_FORMAT),
  formatVersion: z.literal(PROJECT_FORMAT_VERSION),
  appVersion: z.string().min(1),
  id: z.string().uuid(),
  title: z.string().min(1),
  description: z.string(),
  projectType: projectTypeSchema,
  createdAt: z.string().datetime(),
  updatedAt: z.string().datetime(),
  lastOpenedAt: z.string().datetime().nullable(),
  settings: z.record(z.unknown()),
  customMetadataSchema: z.array(z.record(z.unknown())),
  database: z.object({
    file: z.literal('project.db'),
    schemaVersion: z.number().int().positive(),
  }),
});

export type ProjectJson = z.infer<typeof projectJsonSchema>;
export type ProjectType = z.infer<typeof projectTypeSchema>;
export type CreateProjectRequest = z.infer<typeof createProjectRequestSchema>;
export type CreateDefaultProjectRequest = z.infer<
  typeof createDefaultProjectRequestSchema
>;
export type OpenProjectRequest = z.infer<typeof openProjectRequestSchema>;
export type CloseProjectRequest = z.infer<typeof closeProjectRequestSchema>;
export type ProjectSummary = z.infer<typeof projectSummarySchema>;
export type ProjectSession = z.infer<typeof projectSessionSchema>;
export type CloseProjectResponse = z.infer<typeof closeProjectResponseSchema>;
export type CommandError = z.infer<typeof commandErrorSchema>;
