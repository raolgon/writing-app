import { writable } from 'svelte/store';

export type SaveStatus = 'idle' | 'dirty' | 'saving' | 'saved' | 'error';

export const saveStatusStore = writable<SaveStatus>('idle');
export const saveErrorStore = writable<string | null>(null);
