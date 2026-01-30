import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { DebugEventType } from './types';
import { addDebugLog } from './stores';

let unlistenFn: UnlistenFn | null = null;

export async function setupEventListeners(): Promise<void> {
  // Listen to debug events from backend
  unlistenFn = await listen<DebugEventType>('debug-event', (event) => {
    addDebugLog(event.payload);
  });
}

export function cleanupEventListeners(): void {
  if (unlistenFn) {
    unlistenFn();
    unlistenFn = null;
  }
}
