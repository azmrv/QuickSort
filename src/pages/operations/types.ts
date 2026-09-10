// Unified Operations table row types (0.2.6 feature 4c).
//
// Live queue jobs (Queued/Running) and finished operations (history) are
// normalized into a single `OperationRow` so the grid renders one table.

export type OperationSourceLabel = 'Search' | 'Selector' | 'ContextMenu' | 'Api';

export type RowKind = 'job' | 'operation';

/** Job status keys (namespace: `queue.status.*`). */
export type JobStatusKey = 'queued' | 'running' | 'completed' | 'failed' | 'canceled';

/** Operation state keys (namespace: `history.state.*`; `unknown` fallback). */
export type OperationStatusKey = 'pending' | 'executing' | 'completed' | 'failed' | 'undone' | 'unknown';

/** Progress counters for an active queue job. */
export interface RowProgress {
    current: number;
    total: number;
}

// ---------------------------------------------------------------------------
// Wire DTO types (get_operations / get_jobs)
// ---------------------------------------------------------------------------

/** Serde serializes unit variants as strings and payload variants as objects. */
export type OperationState =
    | 'Pending'
    | 'Executing'
    | 'Undone'
    | { Completed: { processed_files: number; bytes_processed: number } }
    | { Failed: { reason: string } };

export type OperationTypeName = 'Move' | 'Copy' | 'Delete' | 'Rename';

export interface OperationDto {
    id: string;
    operation_type: OperationTypeName;
    state: OperationState;
    source_paths: string[];
    target_folder_path: string | null;
    target_paths: string[] | null;
    processed_files: number;
    bytes_processed: number;
    source?: OperationSourceLabel;
    correlation_id: string;
    created_at: string;
    updated_at: string;
}

export type JobStatusName = 'Queued' | 'Running' | 'Completed' | 'Failed' | 'Canceled';

export interface JobDto {
    id: string;
    operation_type: OperationTypeName;
    source_paths: string[];
    status: JobStatusName;
    progress: RowProgress;
    operation_id: string | null;
    error: string | null;
    created_at: number;
    updated_at: number;
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

    /** Status label key (history.state.* / queue.status.* suffix). */
    statusKey: JobStatusKey | OperationStatusKey;
    /** Stable sort rank for the status column (locale independent). */
    stateRank: number;
    /** For completed operations: processed_files, passed to the i18n `count`. */
    statusCount?: number;

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