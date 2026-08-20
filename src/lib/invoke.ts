import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { logger } from './logger';

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    logger.ipc('frontend', `→ ${cmd}`, args);
    try {
        const result = await tauriInvoke<T>(cmd, args);
        logger.ipc('frontend', `← ${cmd} OK`, result);
        return result;
    } catch (err) {
        logger.error('frontend', `← ${cmd} FAIL`, err);
        throw err;
    }
}
