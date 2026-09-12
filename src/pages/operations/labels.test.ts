import { describe, expect, it } from 'vitest';
import { getStatusLabel } from './labels';
import type { OperationRow } from './types';

const row = (overrides: Partial<OperationRow> = {}): OperationRow => ({
    key: 'r1',
    kind: 'job',
    jobId: null,
    operationId: null,
    operationType: 'Move',
    statusKey: 'queued',
    stateRank: 0,
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
});

// Keys-only translate stub: returns the key, so expected values read as the
// i18n path rather than a guessed translation.
const t = (key: string): string => key;

describe('getStatusLabel', () => {
    it('appends the error reason to a failed job status', () => {
        const r = row({ statusKey: 'failed', error: 'Not enough disk space on target' });
        expect(getStatusLabel(r, t)).toBe('queue.status.failed Not enough disk space on target');
    });

    it('keeps a failed job label plain when no error reason exists', () => {
        expect(getStatusLabel(row({ statusKey: 'failed', error: null }), t)).toBe(
            'queue.status.failed',
        );
    });

    it('maps non-failed job statuses to the queue namespace', () => {
        expect(getStatusLabel(row({ statusKey: 'running' }), t)).toBe('queue.status.running');
        expect(getStatusLabel(row({ statusKey: 'canceled' }), t)).toBe('queue.status.canceled');
    });

    it('appends the error reason to a failed history operation', () => {
        const r = row({
            kind: 'operation',
            statusKey: 'failed',
            error: 'Source file not found',
        });
        expect(getStatusLabel(r, t)).toBe('history.state.failed Source file not found');
    });

    it('maps completed history to the count-carrying history label', () => {
        const r = row({ kind: 'operation', statusKey: 'completed', statusCount: 3 });
        expect(getStatusLabel(r, t)).toBe('history.state.completed');
    });

    it('maps other history statuses to the history namespace', () => {
        expect(getStatusLabel(row({ kind: 'operation', statusKey: 'undone' }), t)).toBe(
            'history.state.undone',
        );
    });
});