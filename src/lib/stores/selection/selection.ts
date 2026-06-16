import { writable } from 'svelte/store';

export const selectedBinderItemIdStore = writable<string | null>(null);
