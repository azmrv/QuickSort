// Pure enable/disable logic for the Operations toolbar multiselect actions.
// Kept free of React/tauri so it is unit-testable (vitest).

import type { OperationRow } from './types';

export interface ToolbarState {
    /** Rows that are actually selected (keys that resolve to an existing row). */
    selectionCount: number;
    /** Selected completed/redoable operations Undo can iterate. */
    undoCount: number;
    /** Selected operations Repeat can iterate. */
    repeatCount: number;
    /** Selected operations Delete can iterate (any operation row). */
    deleteCount: number;
    /** Selected queued/running jobs Cancel can iterate. */
    cancelCount: number;
    canUndo: boolean;
    canRepeat: boolean;
    canDelete: boolean;
    canCancel: boolean;
}

/**
 * Actions are always enabled when at least one applicable row is selected;
 * a mixed/partially-valid selection is not blocked, inapplicable rows are
 * skipped at execution time (files-manager behaviour).
 *
 * `unavailable` holds row keys whose Undo/Repeat the backend rejected as
 * permanent (e.g. target files no longer exist); they are treated as not
 * undoable so the button stays disabled instead of re-raising on every click.
 */
export const computeToolbarState = (
    selectedKeys: ReadonlySet<string>,
    rows: OperationRow[],
    unavailable: ReadonlySet<string> = new Set(),
): ToolbarState => {
    const selected = rows.filter((row) => selectedKeys.has(row.key));
    const undoable = selected.filter(
        (row) => row.kind === 'operation' && row.undoable && !unavailable.has(row.key),
    );
    const repeatable = selected.filter(
        (row) => row.kind === 'operation' && row.repeatable && !unavailable.has(row.key),
    );
    const deletable = selected.filter((row) => row.kind === 'operation');
    const cancellable = selected.filter((row) => row.kind === 'job' && row.cancellable);
    return {
        selectionCount: selected.length,
        undoCount: undoable.length,
        repeatCount: repeatable.length,
        deleteCount: deletable.length,
        cancelCount: cancellable.length,
        canUndo: undoable.length > 0,
        canRepeat: repeatable.length > 0,
        canDelete: deletable.length > 0,
        canCancel: cancellable.length > 0,
    };
};