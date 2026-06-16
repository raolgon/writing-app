import { z } from 'zod';

import {
  closeProjectRequestSchema,
  closeProjectResponseSchema,
  commandErrorSchema,
  createDefaultProjectRequestSchema,
  createProjectRequestSchema,
  openProjectRequestSchema,
  projectSessionSchema,
  type CloseProjectRequest,
  type CloseProjectResponse,
  type CommandError,
  type CreateDefaultProjectRequest,
  type CreateProjectRequest,
  type OpenProjectRequest,
  type ProjectSession,
} from '$lib/schemas/project';
import { invokeCommand } from './runtime';

export class TauriCommandError extends Error {
  readonly code: string;

  constructor(error: CommandError) {
    super(error.message);
    this.name = 'TauriCommandError';
    this.code = error.code;
  }
}

export async function createProject(
  request: CreateProjectRequest,
): Promise<ProjectSession> {
  return invokeProjectCommand(
    'create_project',
    { request: createProjectRequestSchema.parse(request) },
    projectSessionSchema,
  );
}

export async function createDefaultProject(
  request: CreateDefaultProjectRequest,
): Promise<ProjectSession> {
  return invokeProjectCommand(
    'create_default_project',
    { request: createDefaultProjectRequestSchema.parse(request) },
    projectSessionSchema,
  );
}

export async function openProject(
  request: OpenProjectRequest,
): Promise<ProjectSession> {
  return invokeProjectCommand(
    'open_project',
    { request: openProjectRequestSchema.parse(request) },
    projectSessionSchema,
  );
}

export async function closeProject(
  request: CloseProjectRequest,
): Promise<CloseProjectResponse> {
  return invokeProjectCommand(
    'close_project',
    { request: closeProjectRequestSchema.parse(request) },
    closeProjectResponseSchema,
  );
}

async function invokeProjectCommand<T>(
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
