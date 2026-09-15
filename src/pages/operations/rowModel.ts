// Pure mapping from wire DTOs (get_jobs / get_operations) to the unified
// `OperationRow` used by the single Operations table, plus format/sort helpers.
// Kept free of React/i18n so it is unit-testable.

import type {
    JobDto,
    JobStatusKey,
    JobStatusName,
    OperationDto,
    OperationRow,
    OperationState,
    OperationStatusKey,
} from './types';

// Explorer-style byte formatting, e.g. "1.2 MB".
export const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    const units = ['KB', 'MB', 'GB', 'TB'];
    let value = bytes;
    let unitIndex = -1;
    do {
        value /= 1024;
        unitIndex += 1;
    } while (value >= 1024 && unitIndex < units.length - 1);
    return `${value.toFixed(1)} ${units[unitIndex]}`;
};

const JOB_STATUS_KEYS: Record<JobStatusName, JobStatusKey> = {
    Queued: 'queued',
    Running: 'running',
    Completed: 'completed',
    Failed: 'failed',
    Canceled: 'canceled',
};

const JOB_STATUS_RANKS: Record<JobStatusName, number> = {
    Queued: 0,
    Running: 1,
    Completed: 2,
    Failed: 3,
    Canceled: 4,
};

/** Normalizes a serde OperationState into { key, rank, count?, error?, size?. }. */
const parseOperationState = (state: OperationState): {
    key: OperationStatusKey;
    rank: number;
    count: number | undefined;
    error: string | null;
    size: number | undefined;
} => {
    if (typeof state === 'string') {
        switch (state) {
            case 'Pending':
                return { key: 'pending', rank: 0, count: undefined, error: null, size: undefined };
            case 'Executing':
                return { key: 'executing', rank: 1, count: undefined, error: null, size: undefined };
            case 'Undone':
                return { key: 'undone', rank: 4, count: undefined, error: null, size: undefined };
            default:
                return { key: 'unknown', rank: 5, count: undefined, error: null, size: undefined };
        }
    }
    if (typeof state === 'object' && state !== null) {
        if ('Completed' in state) {
            return {
                key: 'completed',
                rank: 2,
                count: state.Completed.processed_files,
                error: null,
                size: state.Completed.bytes_processed,
            };
        }
        if ('Failed' in state) {
            return {
                key: 'failed',
                rank: 3,
                count: undefined,
                error: state.Failed.reason,
                size: undefined,
            };
        }
    }
    return { key: 'unknown', rank: 5, count: undefined, error: null, size: undefined };
};

export const jobToRow = (job: JobDto): OperationRow => {
    return {
        key: `job.${job.id}`,
        kind: 'job',
        jobId: job.id,
        operationId: job.operation_id,
        operationType: job.operation_type,
        statusKey: JOB_STATUS_KEYS[job.status],
        stateRank: JOB_STATUS_RANKS[job.status],
        source: null,
        files: job.source_paths,
        filesCount: job.source_paths.length,
        target: null,
        sizeBytes: 0,
        createdAtMs: job.created_at * 1000,
        progress: job.progress,
        error: job.error,
        undoable: false,
        repeatable: false,
        cancellable: job.status === 'Queued' || job.status === 'Running',
    };
};

export const operationToRow = (op: OperationDto): OperationRow => {
    const parsed = parseOperationState(op.state);
    const complete = parsed.key === 'completed';
    return {
        key: `op.${op.id}`,
        kind: 'operation',
        jobId: null,
        operationId: op.id,
        operationType: op.operation_type,
        statusKey: parsed.key,
        stateRank: parsed.rank,
        statusCount: parsed.count,
        source: op.source ?? null,
        files: op.source_paths,
        filesCount: op.source_paths.length,
        target: op.target_folder_path,
        sizeBytes: parsed.size ?? op.bytes_processed ?? 0,
        createdAtMs: Date.parse(op.created_at),
        progress: null,
        error: parsed.error,
        undoable: complete && op.operation_type !== 'Delete',
        repeatable: complete || parsed.key === 'undone',
        cancellable: false,
    };
};

/** Ascending comparator by createdAtMs. */
export const compareRowsByCreatedAtAsc = (a: OperationRow, b: OperationRow): number =>
    a.createdAtMs - b.createdAtMs;

/** Ascending comparator by state rank; ties broken by newest-first, then key. */
export const compareRowsByStateAsc = (a: OperationRow, b: OperationRow): number => {
    if (a.stateRank !== b.stateRank) return a.stateRank - b.stateRank;
    if (a.createdAtMs !== b.createdAtMs) return b.createdAtMs - a.createdAtMs;
    return a.key < b.key ? -1 : a.key > b.key ? 1 : 0;
};