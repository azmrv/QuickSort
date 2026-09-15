import { describe, expect, it } from 'vitest';
import {
    compareRowsByCreatedAtAsc,
    compareRowsByStateAsc,
    formatBytes,
    jobToRow,
    operationToRow,
} from './rowModel';
import type { JobDto, OperationDto, OperationRow, OperationState, OperationTypeName } from './types';

const job = (overrides: Partial<JobDto> = {}): JobDto => ({
    id: 'job-1',
    operation_type: 'Move',
    source_paths: ['C:\\Data\\a.txt', 'C:\\Data\\b.txt'],
    status: 'Queued',
    progress: { current: 0, total: 2 },
    operation_id: null,
    error: null,
    created_at: 1_700_000_000,
    updated_at: 1_700_000_000,
    ...overrides,
});

const operation = (
    state: OperationState,
    overrides: Partial<OperationDto> = {},
): OperationDto => ({
    id: 'op-1',
    operation_type: 'Move',
    state,
    source_paths: ['C:\\Data\\folder_a'],
    target_folder_path: 'C:\\Data\\dest',
    target_paths: null,
    processed_files: 10,
    bytes_processed: 2048,
    source: 'ContextMenu',
    correlation_id: '00000000-0000-0000-0000-000000000000',
    created_at: '2026-09-10T12:00:00Z',
    updated_at: '2026-09-10T12:05:00Z',
    ...overrides,
});

describe('jobToRow', () => {
    it('maps every scalar field', () => {
        const row = jobToRow(job());
        expect(row).toMatchObject({
            key: 'job.job-1',
            kind: 'job',
            jobId: 'job-1',
            operationId: null,
            operationType: 'Move',
            statusKey: 'queued',
            stateRank: 0,
            source: null,
            files: ['C:\\Data\\a.txt', 'C:\\Data\\b.txt'],
            filesCount: 2,
            target: null,
            sizeBytes: 0,
            createdAtMs: 1_700_000_000_000,
            progress: { current: 0, total: 2 },
            error: null,
            undoable: false,
            repeatable: false,
        });
        expect(row.cancellable).toBe(true);
    });

    it('maps status to the queue.status.* key and stable rank', () => {
        const cases: Array<[JobDto['status'], string, number]> = [
            ['Queued', 'queued', 0],
            ['Running', 'running', 1],
            ['Completed', 'completed', 2],
            ['Failed', 'failed', 3],
            ['Canceled', 'canceled', 4],
        ];
        for (const [status, key, rank] of cases) {
            const row = jobToRow(job({ status }));
            expect(row.statusKey).toBe(key);
            expect(row.stateRank).toBe(rank);
        }
    });

    it('only queued/running jobs are cancellable', () => {
        expect(jobToRow(job({ status: 'Queued' })).cancellable).toBe(true);
        expect(jobToRow(job({ status: 'Running' })).cancellable).toBe(true);
        expect(jobToRow(job({ status: 'Completed' })).cancellable).toBe(false);
        expect(jobToRow(job({ status: 'Failed' })).cancellable).toBe(false);
        expect(jobToRow(job({ status: 'Canceled' })).cancellable).toBe(false);
    });

    it('propagates the error, progress and operation id', () => {
        const row = jobToRow(
            job({ status: 'Failed', error: 'disk full', progress: { current: 1, total: 2 }, operation_id: 'op-x' }),
        );
        expect(row.error).toBe('disk full');
        expect(row.progress).toEqual({ current: 1, total: 2 });
        expect(row.operationId).toBe('op-x');
    });
});

describe('operationToRow', () => {
    it('maps Pending', () => {
        const row = operationToRow(operation('Pending'));
        expect(row).toMatchObject({
            key: 'op.op-1',
            kind: 'operation',
            operationId: 'op-1',
            jobId: null,
            statusKey: 'pending',
            stateRank: 0,
            files: ['C:\\Data\\folder_a'],
            filesCount: 1,
            target: 'C:\\Data\\dest',
            sizeBytes: 2048,
            error: null,
            undoable: false,
            repeatable: false,
            cancellable: false,
        });
    });

    it('maps Executing', () => {
        const row = operationToRow(operation('Executing'));
        expect(row.statusKey).toBe('executing');
        expect(row.stateRank).toBe(1);
    });

    it('maps Completed with count and size from the payload', () => {
        const row = operationToRow(
            operation({
                Completed: { processed_files: 7, bytes_processed: 4096 },
            }),
        );
        expect(row.statusKey).toBe('completed');
        expect(row.stateRank).toBe(2);
        expect(row.statusCount).toBe(7);
        expect(row.sizeBytes).toBe(4096);
        expect(row.undoable).toBe(true);
        expect(row.repeatable).toBe(true);
    });

    it('completed Delete operations are not undoable', () => {
        const row = operationToRow(
            operation(
                { Completed: { processed_files: 1, bytes_processed: 0 } },
                { operation_type: 'Delete' as OperationTypeName },
            ),
        );
        expect(row.undoable).toBe(false);
        expect(row.repeatable).toBe(true);
    });

    it('maps Failed, extracting the reason and keeping top-level size', () => {
        const row = operationToRow(operation({ Failed: { reason: 'access denied' } }));
        expect(row.statusKey).toBe('failed');
        expect(row.stateRank).toBe(3);
        expect(row.error).toBe('access denied');
        expect(row.sizeBytes).toBe(2048);
        expect(row.repeatable).toBe(false);
        expect(row.undoable).toBe(false);
    });

    it('maps Undone as repeatable only', () => {
        const row = operationToRow(operation('Undone'));
        expect(row.statusKey).toBe('undone');
        expect(row.stateRank).toBe(4);
        expect(row.repeatable).toBe(true);
        expect(row.undoable).toBe(false);
    });

    it('falls back to unknown for unrecognized states', () => {
        const row = operationToRow(operation('Weird' as OperationState));
        expect(row.statusKey).toBe('unknown');
        expect(row.stateRank).toBe(5);
    });

    it('propagates the source instead of nulling it', () => {
        expect(operationToRow(operation('Completed' as OperationState)).source).toBe('ContextMenu');
    });
});

describe('formatBytes', () => {
    it('handles zero and small values', () => {
        expect(formatBytes(0)).toBe('0 B');
        expect(formatBytes(512)).toBe('512 B');
    });

    it('formats KB, MB and GB with one fractional digit', () => {
        expect(formatBytes(1024)).toBe('1.0 KB');
        expect(formatBytes(1024 * 1024)).toBe('1.0 MB');
        expect(formatBytes(5.5 * 1024 * 1024)).toBe('5.5 MB');
        expect(formatBytes(1024 ** 3)).toBe('1.0 GB');
    });

    it('keeps the largest unit for large values', () => {
        expect(formatBytes(9 * 1024 ** 4)).toBe('9.0 TB');
    });
});

describe('sort comparators', () => {
    const makeRow = (stateRank: number, createdAtMs: number, key: string): OperationRow => ({
        key,
        kind: 'operation' as const,
        jobId: null,
        operationId: key,
        operationType: 'Move',
        statusKey: 'completed',
        stateRank,
        source: null,
        files: [],
        filesCount: 0,
        target: null,
        sizeBytes: 0,
        createdAtMs,
        progress: null,
        error: null,
        undoable: false,
        repeatable: false,
        cancellable: false,
    });

    it('sorts by creation time ascending', () => {
        const early = makeRow(2, 1000, 'a');
        const late = makeRow(2, 2000, 'b');
        expect(compareRowsByCreatedAtAsc(early, late)).toBeLessThan(0);
        expect(compareRowsByCreatedAtAsc(late, early)).toBeGreaterThan(0);
    });

    it('sorts by state rank with created-at tie-break', () => {
        const rankLow = makeRow(1, 5000, 'c');
        const rankHigh = makeRow(3, 0, 'd');
        expect(compareRowsByStateAsc(rankLow, rankHigh)).toBeLessThan(0);
        expect(compareRowsByStateAsc(rankHigh, rankLow)).toBeGreaterThan(0);
        expect(compareRowsByStateAsc(rankLow, makeRow(1, 7000, 'e'))).toBeGreaterThan(0);
        expect(compareRowsByStateAsc(rankLow, makeRow(1, 1000, 'f'))).toBeLessThan(0);
    });
});