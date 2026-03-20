import { writable } from 'svelte/store';

export type McpStatus = 'disconnected' | 'connecting' | 'connected';
export type WsStatus = 'disconnected' | 'connected';

export const mcpStatus = writable<McpStatus>('disconnected');
export const wsStatus = writable<WsStatus>('disconnected');
