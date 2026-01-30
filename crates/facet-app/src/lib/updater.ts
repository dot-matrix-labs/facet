import { check, type DownloadEvent } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export interface UpdateCheckResult {
  available: boolean;
  currentVersion: string;
  version?: string;
  date?: string;
  body?: string; // Release notes
}

export interface UpdateProgress {
  downloaded: number;
  total: number;
  percentage: number;
}

export type UpdaterCallback = (_progress: UpdateProgress) => void;

/**
 * Check for available updates silently (no user prompts)
 * @returns Update information if available, null otherwise
 */
export async function checkForUpdates(): Promise<UpdateCheckResult | null> {
  try {
    console.log('[Updater] Checking for updates...');

    const update = await check();

    if (!update) {
      // Just log to console, don't show error to user
      console.warn('[Updater] Failed to check for updates - update service returned null');
      console.warn('[Updater] This may be due to network issues or missing endpoint');
      return null;
    }

    if (!update.available) {
      console.log('[Updater] No update available, running latest version');
      console.log(`[Updater] Current version: ${update.currentVersion}`);
      return {
        available: false,
        currentVersion: update.currentVersion,
      };
    }

    console.log(`[Updater] Update available: ${update.version}`);
    console.log(`[Updater] Current version: ${update.currentVersion}`);
    console.log(`[Updater] Release date: ${update.date}`);

    return {
      available: true,
      currentVersion: update.currentVersion,
      version: update.version,
      date: update.date,
      body: update.body,
    };
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    // Just log to console, don't show error to user
    console.warn('[Updater] Error checking for updates:', errorMsg);
    console.warn('[Updater] Full error:', error);
    console.warn('[Updater] This may be due to network issues or missing endpoint');
    return null;
  }
}

/**
 * Download and install an available update
 * @param onProgress Optional callback for download progress
 * @returns True if update was successful, false otherwise
 */
export async function downloadAndInstallUpdate(onProgress?: UpdaterCallback): Promise<boolean> {
  try {
    console.log('[Updater] Starting update download and installation...');

    const update = await check();

    if (!update || !update.available) {
      const errorMsg = '[Updater] No update available to install';
      console.error(errorMsg);
      throw new Error(errorMsg);
    }

    console.log(`[Updater] Downloading version ${update.version}...`);

    let totalBytes = 0;
    let downloadedBytes = 0;

    await update.downloadAndInstall((event: DownloadEvent) => {
      switch (event.event) {
        case 'Started':
          totalBytes = event.data.contentLength || 0;
          console.log(`[Updater] Download started: ${totalBytes} bytes`);
          if (onProgress) {
            onProgress({
              downloaded: 0,
              total: totalBytes,
              percentage: 0,
            });
          }
          break;

        case 'Progress': {
          downloadedBytes += event.data.chunkLength;
          const percentage = totalBytes > 0 ? (downloadedBytes / totalBytes) * 100 : 0;
          console.log(
            `[Updater] Progress: ${downloadedBytes}/${totalBytes} bytes (${percentage.toFixed(1)}%)`
          );
          if (onProgress) {
            onProgress({
              downloaded: downloadedBytes,
              total: totalBytes,
              percentage,
            });
          }
          break;
        }

        case 'Finished':
          console.log('[Updater] Download complete, installing...');
          if (onProgress) {
            onProgress({
              downloaded: totalBytes,
              total: totalBytes,
              percentage: 100,
            });
          }
          break;
      }
    });

    console.log('[Updater] Update installed successfully, relaunching app...');

    // Small delay to ensure progress UI updates
    await new Promise((resolve) => setTimeout(resolve, 500));

    // Relaunch the application
    await relaunch();

    return true;
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    console.error('[Updater] Error during update:', errorMsg);
    console.error('[Updater] Full error:', error);
    throw new Error(`Update failed: ${errorMsg}`);
  }
}

/**
 * Format bytes to human-readable string
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';

  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}
