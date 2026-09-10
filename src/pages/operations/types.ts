// Unified Operations table row types (0.2.6 feature 4c).
//
// Live queue jobs (Queued/Running) and finished operations (history) are
// normalized into a single `OperationRow` so the grid renders one table.

export type OperationSourceLabel = 'Search' | 'Selector' | 'ContextMenu' | 'Api';

export type RowKind = 'job' | 'operation';

/** Progress counters for an active queue job. */
export interface RowProgress {
    current: number;
    total: number;
}

/** Unified row of the single Operations table. */
export interface OperationRow {
    /** Stable unique key: `job.<id>` | `op.<id>` (used by rowKeyGetter). */
    key: string;
    kind: RowKind;

    /** Back-reference to the source entity (exactly one is non-null). */
    jobId: string | null;
    operationId: string | null;

    /** Move/Copy/Delete/Rename (wire form: PascalCase). */
    operationType: string;

    /** Status label key root: `queue.status.*` for jobs, `history.state.*` for operations. */
    statusKey: string;
    /** Stable sort rank for the status column (locale independent). */
    stateRank: number;

    /** Operation origin (null for legacy records without `source`). */
    source: OperationSourceLabel | null;

    /** Top-level objects from `source_paths` (single level, not recursive). */
    files: string[];
    /** Total object count for the "Objects" column. */
    filesCount: number;

    /** Target folder for Move/Copy. */
    target: string | null;

    /** Aggregated size in bytes for the operation. */
    sizeBytes: number;

    /** Unix milliseconds for sorting and display. */
    createdAtMs: number;

    /** Progress for active jobs only. */
    progress: RowProgress | null;

    /** Error text (job.error or Failed operation reason). */
    error: string | null;

    /** Action availability flags, computed by the row mapping (S2). */
    undoable: boolean;
    repeatable: boolean;
    cancellable: boolean;
}