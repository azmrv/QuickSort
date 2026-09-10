// Shared label helpers for the unified Operations table and the details
// popup. Free of React state; only needs the i18n `t` function.

import type { OperationRow } from './types';

export type Translate = (key: string, params?: Record<string, string | number>) => string;

export const STATUS_COLORS: Record<string, string> = {
    queued: '#6b7280',
    pending: '#6b7280',
    running: '#3b82f6',
    executing: '#3b82f6',
    completed: '#22c55e',
    failed: '#ef4444',
    canceled: '#9ca3af',
    undone: '#f59e0b',
    unknown: '#6b7280',
};

export const SOURCE_COLORS: Record<string, string> = {
    Search: '#3b82f6',
    Selector: '#8b5cf6',
    ContextMenu: '#22c55e',
    Api: '#6b7280',
};

/** Status text for a unified row (job namespace vs history namespace). */
export const getStatusLabel = (row: OperationRow, t: Translate): string => {
    if (row.kind === 'job') {
        return t(`queue.status.${row.statusKey}`);
    }
    if (row.statusKey === 'completed') {
        return t('history.state.completed', { count: row.statusCount ?? 0 });
    }
    if (row.statusKey === 'failed' && row.error) {
        return `${t('history.state.failed')} ${row.error}`;
    }
    return t(`history.state.${row.statusKey}`);
};

/** Localized operation-type label (history.operation.*). */
export const getTypeLabel = (row: OperationRow, t: Translate): string =>
    t(`history.operation.${row.operationType.toLowerCase()}`);

/** Localized source label; jobs (no source) show an em dash. */
export const getSourceLabel = (row: OperationRow, t: Translate): string => {
    if (!row.source) return '\u2014';
    const key = row.source === 'ContextMenu' ? 'context_menu' : row.source.toLowerCase();
    return t(`operations.source.${key}`);
};