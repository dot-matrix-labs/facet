import { invoke } from '@tauri-apps/api/core';

/** Log levels matching the Rust LogLevel enum */
export type LogLevel = 'Debug' | 'Info' | 'Warn' | 'Error';

/** Log entry from backend */
export interface LogEntry {
  timestamp: string;
  level: string;
  source: string;
  message: string;
}

/** Intercept console methods and send to backend for persistent logging */
export function initializeConsoleLogger() {
  const originalConsole = {
    log: console.log,
    info: console.info,
    warn: console.warn,
    error: console.error,
    debug: console.debug,
  };

  // Helper to send log to backend
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sendToBackend = async (level: LogLevel, args: any[]) => {
    try {
      const message = args
        .map((arg) => {
          if (typeof arg === 'object') {
            try {
              return JSON.stringify(arg, null, 2);
            } catch {
              return String(arg);
            }
          }
          return String(arg);
        })
        .join(' ');

      await invoke('log_frontend_message', { level, message });
    } catch {
      // Silently fail if logging backend is not available
      // (e.g., user not logged in yet)
    }
  };

  // Override console methods
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  console.log = (...args: any[]) => {
    originalConsole.log(...args);
    sendToBackend('Info', args);
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  console.info = (...args: any[]) => {
    originalConsole.info(...args);
    sendToBackend('Info', args);
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  console.warn = (...args: any[]) => {
    originalConsole.warn(...args);
    sendToBackend('Warn', args);
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  console.error = (...args: any[]) => {
    originalConsole.error(...args);
    sendToBackend('Error', args);
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  console.debug = (...args: any[]) => {
    originalConsole.debug(...args);
    sendToBackend('Debug', args);
  };

  console.log('[Logger] Console logging initialized');
}

/** Get all logs from encrypted log file */
export async function getLogs(): Promise<LogEntry[]> {
  try {
    return await invoke<LogEntry[]>('get_logs');
  } catch (error) {
    console.error('[Logger] Failed to get logs:', error);
    return [];
  }
}

/** Clear all logs */
export async function clearLogs(): Promise<void> {
  try {
    await invoke('clear_logs');
    console.log('[Logger] Logs cleared');
  } catch (error) {
    console.error('[Logger] Failed to clear logs:', error);
    throw error;
  }
}

/** Get total log file size in bytes */
export async function getLogSize(): Promise<number> {
  try {
    return await invoke<number>('get_log_size');
  } catch (error) {
    console.error('[Logger] Failed to get log size:', error);
    return 0;
  }
}
