// Error classification for undo/repeat batch actions (P1-5, feature 4c).
//
// Tauri v2 serializes the backend `OperationErrorDto` as a JSON object body
// with `Tauri-Response: error`; the frontend IPC layer parses it via
// `response.json()`, so `invoke` rejects with a plain object:
//     { kind: 'transient' | 'permanent' | 'unknown', message: string }
// Strings and any other shapes (arg validation, panics) fall back to `unknown`.
// Pure functions only — unit-testable (vitest).

export type UndoErrorKind = 'transient' | 'permanent' | 'unknown';

export interface OperationErrorDto {
    kind: UndoErrorKind;
    message: string;
}

/**
 * Map an `invoke` rejection to a UI decision:
 * - `transient` — retry the action is reasonable (e.g. locked file);
 * - `permanent` — the action can never succeed, disable it;
 * - `unknown` — show the generic error toast.
 */
export const classifyUndoError = (err: unknown): UndoErrorKind => {
    if (typeof err === 'object' && err !== null) {
        const kind = (err as { kind?: unknown }).kind;
        if (kind === 'transient' || kind === 'permanent' || kind === 'unknown') {
            return kind;
        }
    }
    return 'unknown';
};

/** Human-readable message for the toast (backend `message` or the raw error). */
export const getOperationErrorMessage = (err: unknown): string => {
    if (typeof err === 'object' && err !== null) {
        const message = (err as { message?: unknown }).message;
        if (typeof message === 'string') return message;
    }
    if (typeof err === 'string') return err;
    return String(err);
};