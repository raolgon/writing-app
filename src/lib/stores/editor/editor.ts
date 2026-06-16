import { writable } from 'svelte/store';

import type { DocumentRecord } from '$lib/schemas/document';

export const currentDocumentStore = writable<DocumentRecord | null>(null);
export const editorDirtyStore = writable(false);
