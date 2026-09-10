import { describe, expect, it } from 'vitest';
import { computeToolbarState } from './toolbarState';
import type { OperationRow } from './types';

const row = (
    kind: OperationRow['kind'],
    overrides: Partial<OperationRow> = {},
): OperationRow => {
    const base: OperationRow = {
        key: `${kind}.${overrides.operationId ?? overrides.jobId ?? 'x'}`,
        kind,
        jobId: kind === 'job' ? 'job-x' : null,
        operationId: kind === 'operation' ? 'op-x' : null,
        operationType: 'Move',
        statusKey: 'completed',
        stateRank: 2,
        source: null,
        files: [],
        filesCount: 0,
        target: null,
        sizeBytes: 0,
        createdAtMs: 0,
        progress: null,
        error: null,
        undoable: false,
        repeatable: false,
        cancellable: false,
        ...overrides,
    };
    return base;
};

const op = (overrides: Partial<OperationRow> = {}): OperationRow =>
    row('operation', { operationId: 'op-1', ...overrides });

const job = (overrides: Partial<OperationRow> = {}): OperationRow =>
    row('job', { jobId: 'job-1', ...overrides });

const keys = (...rows: OperationRow[]): Set<string> => new Set(rows.map((r) => r.key));

describe('computeToolbarState', () => {
    it('disables every action for an empty selection', () => {
        const state = computeToolbarState(new Set(), [op(), job()]);
        expect(state).toMatchObject({
            selectionCount: 0,
            canUndo: false,
            canRepeat: false,
            canDelete: false,
            canCancel: false,
        });
    });

    it('ignores stale keys that do not resolve to a row', () => {
        const state = computeToolbarState(new Set(['job.dropped', 'op.ghost']), [op()]);
        expect(state.selectionCount).toBe(0);
        expect(state.canDelete).toBe(false);
    });

    it('enables undo/repeat/delete on an undoable operation', () => {
        const target = op({ undoable: true, repeatable: true });
        const state = computeToolbarState(keys(target), [target]);
        expect(state).toMatchObject({
            selectionCount: 1,
            undoCount: 1,
            repeatCount: 1,
            deleteCount: 1,
            canUndo: true,
            canRepeat: true,
            canDelete: true,
            canCancel: false,
        });
    });

    it('a completed Delete operation is repeatable but not undoable', () => {
        const target = op({ undoable: false, repeatable: true });
        const state = computeToolbarState(keys(target), [target]);
        expect(state.canUndo).toBe(false);
        expect(state.canRepeat).toBe(true);
        expect(state.canDelete).toBe(true);
    });

    it('selected jobs enable only Cancel for queued/running ones', () => {
        const queued = job({ statusKey: 'queued', cancellable: true });
        const done = job({ statusKey: 'completed', cancellable: false });
        const state = computeToolbarState(keys(queued, done), [queued, done]);
        expect(state.canCancel).toBe(true);
        expect(state.cancelCount).toBe(1);
        expect(state.canUndo).toBe(false);
        expect(state.canDelete).toBe(false);
    });

    it('a mixed selection reports per-group counts', () => {
        const undoableOp = op({ operationId: 'op-u', undoable: true, repeatable: false });
        const failedOp = op({ operationId: 'op-f', statusKey: 'failed', stateRank: 3 });
        const runningJob = job({ jobId: 'job-r', statusKey: 'running', cancellable: true });
        const rows = [undoableOp, failedOp, runningJob];
        const state = computeToolbarState(keys(...rows), rows);
        expect(state).toMatchObject({
            selectionCount: 3,
            undoCount: 1,
            repeatCount: 0,
            deleteCount: 2,
            cancelCount: 1,
            canUndo: true,
            canRepeat: false,
            canDelete: true,
            canCancel: true,
        });
    });

    it('partial validity still enables an action (at least one applicable row)', () => {
        const valid = op({ operationId: 'op-ok', undoable: true });
        const invalid = op({ operationId: 'op-rm', operationType: 'Delete', undoable: false });
        const rows = [valid, invalid];
        const state = computeToolbarState(keys(...rows), rows);
        expect(state.selectionCount).toBe(2);
        expect(state.canUndo).toBe(true);
        expect(state.undoCount).toBe(1);
        expect(state.canDelete).toBe(true);
    });
});