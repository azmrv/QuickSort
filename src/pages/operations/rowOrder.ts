// Pure ordering/filtering helpers for the unified Operations table.
// Kept free of React/i18n so they are unit-testable (vitest).

import type { JobDto, OperationRow, OperationSourceLabel } from './types';
import { compareRowsByCreatedAtAsc } from './rowModel';

/**
 * Drops terminal jobs (Completed/Failed/Canceled) that already produced an
 * operation: the equivalent row lives in the history block, showing both
 * would duplicate the entry. Active jobs are always kept.
 */
export const filterJobRows = (jobs: JobDto[]): JobDto[] =>
    jobs.filter(
        (j) => j.status === 'Queued' || j.status === 'Running' || j.operation_id === null,
    );

const isActive = (row: OperationRow): boolean =>
    row.statusKey === 'queued' || row.statusKey === 'running';

const byNewest = (a: OperationRow, b: OperationRow): number =>
    compareRowsByCreatedAtAsc(b, a);

/** Default display order: active live jobs, then finished jobs, then history. */
export const orderRows = (
    jobRows: OperationRow[],
    operationRows: OperationRow[],
): OperationRow[] => {
    const active = jobRows.filter(isActive).sort(byNewest);
    const terminal = jobRows.filter((r) => !isActive(r)).sort(byNewest);
    const history = operationRows.slice().sort(byNewest);
    return [...active, ...terminal, ...history];
};

// Stable, locale-independent sort ranks (mirrors stateRank in rowModel).

const TYPE_RANK: Record<string, number> = {
    Copy: 0,
    Move: 1,
    Delete: 2,
    Rename: 3,
};

const SOURCE_RANK: Record<OperationSourceLabel, number> = {
    Search: 0,
    Selector: 1,
    ContextMenu: 2,
    Api: 3,
};

export const compareTypeRank = (a: OperationRow, b: OperationRow): number =>
    (TYPE_RANK[a.operationType] ?? 99) - (TYPE_RANK[b.operationType] ?? 99);

/** Sources without a rank (null) always sort to the end. */
export const compareSourceRank = (a: OperationRow, b: OperationRow): number => {
    const ra = a.source ? (SOURCE_RANK[a.source] ?? 99) : 99;
    const rb = b.source ? (SOURCE_RANK[b.source] ?? 99) : 99;
    return ra - rb;
};

/** Compares the first file path; rows without a path sort to the end. */
export const compareFilesFirst = (a: OperationRow, b: OperationRow): number => {
    const fa = a.files[0] ?? '\uffff';
    const fb = b.files[0] ?? '\uffff';
    return fa < fb ? -1 : fa > fb ? 1 : 0;
};