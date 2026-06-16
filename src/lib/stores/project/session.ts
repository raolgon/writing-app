import { writable } from 'svelte/store';

import type { ProjectSession } from '$lib/schemas/project';

export const projectSessionStore = writable<ProjectSession | null>(null);
