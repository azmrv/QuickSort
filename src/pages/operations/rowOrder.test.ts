import { describe, expect, it } from 'vitest';
import {
    compareFilesFirst,
    compareSourceRank,
    compareTypeRank,
    filterJobRows,
    orderRows,
} from './rowOrder';
import type { JobDto, OperationRow, OperationSourceLabel } from './types';

const job = (overrides: Partial<JobDto> = {}): JobDto => ({
    id: 'job-1',
    operation_type: 'Move',
    source_paths: ['C:\\Data\\a.txt'],
    status: 'Queued',
    progress: { current: 0, total: 1 },
    operation_id: null,
    error: null,
    created_at: 1_700_000_000,
    updated_at: 1_700_000_000,
    ...overrides,
});

describe('filterJobRows', () => {
    it('keeps active jobs even when an operation id is assigned', () => {
        const jobs = filterJobRows([
            job({ id: 'a', status: 'Queued', operation_id: 'op-a' }),
            job({ id: 'b', status: 'Running', operation_id: 'op-b' }),
        ]);
        expect(jobs.map((j) => j.id)).toEqual(['a', 'b']);
    });

    it('drops completed jobs that produced an operation', () => {
        expect(
            filterJobRows([job({ id: 'a', status: 'Completed', operation_id: 'op-a' })]),
        ).toEqual([]);
    });

    it('keeps terminal jobs without an operation', () => {
        const jobs = filterJobRows([
            job({ id: 'a', status: 'Completed', operation_id: null }),
            job({ id: 'b', status: 'Failed', operation_id: null }),
            job({ id: 'c', status: 'Canceled', operation_id: null }),
        ]);
        expect(jobs.map((j) => j.id)).toEqual(['a', 'b', 'c']);
    });

    it('drops failed jobs that have an equivalent operation', () => {
        expect(
            filterJobRows([job({ id: 'a', status: 'Failed', operation_id: 'op-a' })]),
        ).toEqual([]);
    });
});

const row = (
    statusKey: string,
    createdAtMs: number,
    overrides: Partial<OperationRow> = {},
): OperationRow => ({
    key: `j.${statusKey}.${createdAtMs}`,
    kind: 'job',
    jobId: null,
    operationId: null,
    operationType: 'Move',
    statusKey: statusKey as OperationRow['statusKey'],
    stateRank: 0,
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
    ...overrides,
});

describe('orderRows', () => {
    it('orders jobs only: active newest-first, then terminal newest-first', () => {
        const jobs = [
            row('completed', 5000),
            row('running', 2000),
            row('queued', 3000),
            row('failed', 1000),
        ];
        expect(orderRows(jobs, []).map((r) => r.statusKey)).toEqual([
            'queued',
            'running',
            'completed',
            'failed',
        ]);
    });

    it('orders operations only newest-first', () => {
        const ops = [row('completed', 2000), row('failed', 4000), row('completed', 1000)];
        const ordered = orderRows([], ops);
        expect(ordered.map((r) => r.createdAtMs)).toEqual([4000, 2000, 1000]);
        expect(orderRows([], ops).length).toBe(3);
    });

    it('interleaves active jobs, terminal jobs, then history', () => {
        const jobs = [
            row('queued', 2000, { key: 'q' }),
            row('failed', 7000, { key: 'f' }),
            row('running', 3000, { key: 'r' }),
        ];
        const ops = [row('completed', 6000, { key: 'op' }), row('undone', 1500, { key: 'op2' })];
        const result = orderRows(jobs, ops);
        expect(result.map((r) => r.key)).toEqual(['r', 'q', 'f', 'op', 'op2']);
    });

    it('does not mutate its inputs', () => {
        const jobs = [row('queued', 2000), row('completed', 5000)];
        const ops = [row('completed', 3000)];
        orderRows(jobs, ops);
        expect(jobs.map((r) => r.createdAtMs)).toEqual([2000, 5000]);
        expect(ops.map((r) => r.createdAtMs)).toEqual([3000]);
    });
});

describe('sort comparators', () => {
    it('returns an empty array for no rows', () => {
        expect(orderRows([], [])).toEqual([]);
    });

    it('sorts types by a stable locale-independent rank', () => {
        const rows = [row('completed', 0, { operationType: 'Delete' }), row('completed', 0, { operationType: 'Copy' })];
        const r = [...rows].sort((a, b) => compareTypeRank(a, b));
        expect(r.map((x) => x.operationType)).toEqual(['Copy', 'Delete']);
    });

    it('sorts sources by rank with null (jobs) always last', () => {
        const rows = [
            row('completed', 0, { source: 'Api' }),
            row('completed', 0, { source: 'Search' }),
            row('completed', 0, { source: null }),
        ];
        const r = [...rows].sort((a, b) => compareSourceRank(a, b));
        expect(r.map((x) => x.source)).toEqual(['Search', 'Api', null]);
    });

    it('sorts unknown type/source ranks to the end', () => {
        const rows = [
            row('completed', 0, { operationType: 'Rename', source: 'Selector' }),
            row('completed', 0, { operationType: 'Zap' as string, source: 'Zap' as OperationSourceLabel }),
        ];
        const byType = [...rows].sort((a, b) => compareTypeRank(a, b));
        expect(byType[0].operationType).toBe('Rename');
        const bySource = [...rows].sort((a, b) => compareSourceRank(a, b));
        expect(bySource[0].source).toBe('Selector');
    });

    it('compares first file paths with empty paths last', () => {
        const a = row('completed', 0, { files: ['Z:\\x'] });
        const b = row('completed', 0, { files: ['A:\\y'] });
        const empty = row('completed', 0, { files: [] });
        expect(compareFilesFirst(a, b)).toBeGreaterThan(0);
        expect(compareFilesFirst(b, empty)).toBeLessThan(0);
        expect(compareFilesFirst(a, a)).toBe(0);
    });
});