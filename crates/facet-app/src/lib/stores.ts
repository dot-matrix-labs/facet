import { writable } from 'svelte/store';
import type { DebugLogEntry, DebugEventType } from './types';

// Browser state
export const browserLaunched = writable(false);
export const isNavigating = writable(false);
export const currentUrl = writable('');
export const currentTitle = writable('');

// Debug logs
export const debugLogs = writable<DebugLogEntry[]>([]);

// Add a debug log entry
export function addDebugLog(event: DebugEventType) {
  const entry: DebugLogEntry = {
    id: crypto.randomUUID(),
    timestamp: new Date(),
    event,
    level: getLogLevel(event),
  };

  debugLogs.update((logs) => [...logs, entry]);
}

// Clear debug logs
export function clearDebugLogs() {
  debugLogs.set([]);
}

// Determine log level based on event type
function getLogLevel(event: DebugEventType): 'info' | 'success' | 'error' | 'warning' {
  switch (event.type) {
    case 'Error':
      return 'error';
    case 'Success':
    case 'ChromeLaunched':
    case 'PageLoaded':
      return 'success';
    case 'ChromeDownloading':
    case 'ChromeLaunching':
    case 'PageNavigating':
    case 'PageLoading':
    case 'PageParsing':
    case 'Info':
      return 'info';
    default:
      return 'info';
  }
}
