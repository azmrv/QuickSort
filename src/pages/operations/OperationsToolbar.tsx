import { useRef, useState } from 'react';
import { App } from 'antd';
import { invoke } from '../../lib/invoke';
import { classifyUndoError, getOperationErrorMessage } from '../../lib/operationErrors';
import { logger } from '../../lib/logger';
import { useTranslation } from '../../i18n/useTranslation';
import { computeToolbarState } from './toolbarState';
import type { OperationRow } from './types';

interface OperationsToolbarProps {
    selectedKeys: ReadonlySet<string>;
    rows: OperationRow[];
    loading: boolean;
    hasOperations: boolean;
    onRefresh: () => void;
    /** Reload after any batch action completed (undo/repeat/delete/cancel/clear). */
    onChanged: () => void;
}

const toolbarButtonStyle: React.CSSProperties = {
    padding: '6px 12px',
    background: 'var(--qs-bg-tertiary)',
    border: '1px solid var(--qs-border)',
    borderRadius: 'var(--qs-radius-sm)',
    color: 'var(--qs-text-secondary)',
    fontFamily: 'var(--qs-font-mono)',
    fontSize: '12px',
    cursor: 'pointer',
    flexShrink: 0,
};

const actionButtonStyle: React.CSSProperties = {
    padding: '6px 12px',
    background: 'transparent',
    border: '1px solid var(--qs-border)',
    borderRadius: 'var(--qs-radius-sm)',
    color: 'var(--qs-accent)',
    fontFamily: 'var(--qs-font-mono)',
    fontSize: '12px',
    cursor: 'pointer',
    flexShrink: 0,
};

const disabledStyle: React.CSSProperties = {
    cursor: 'not-allowed',
    opacity: 0.45,
};

const OperationsToolbar = ({
    selectedKeys,
    rows,
    loading,
    hasOperations,
    onRefresh,
    onChanged,
}: OperationsToolbarProps) => {
    const { t } = useTranslation();
    const { message, modal } = App.useApp();

    // Row keys whose Undo/Repeat the backend rejected as permanent (files no
    // longer exist etc.). Their buttons stay disabled — retrying would only
    // reproduce the same error. Mirrors HistoryPage's `unavailable` set.
    const [unavailable, setUnavailable] = useState<ReadonlySet<string>>(() => new Set());

    const state = computeToolbarState(selectedKeys, rows, unavailable);

    // Sequential batch runner: never parallel so a flood of in-flight
    // requests cannot happen. Ends with one aggregate toast per action.
    // `handleError` returns true when the failure was already dealt with
    // (permanent/transient undo errors); such items do not count as failed.
    const busyRef = useRef(false);
    const [busy, setBusy] = useState(false);
    const runBatch = async (
        action: string,
        ids: string[],
        invokeFn: (id: string) => Promise<unknown>,
        errorKey: string,
        handleError?: (id: string, err: unknown) => boolean,
    ): Promise<void> => {
        if (busyRef.current) return;
        busyRef.current = true;
        setBusy(true);
        let success = 0;
        let failed = 0;
        let firstError: unknown = null;
        try {
            for (const id of ids) {
                try {
                    await invokeFn(id);
                    success += 1;
                } catch (err) {
                    const handled = handleError ? handleError(id, err) : false;
                    if (!handled) {
                        failed += 1;
                        if (firstError === null) firstError = err;
                    }
                    logger.error('OperationsToolbar', `${action} ${id} failed`, err);
                }
            }
            if (failed > 0 && success > 0) {
                message.warning(t('operations.batch.partial', { success, failed }));
            } else if (failed > 0) {
                message.error(`${t(errorKey)} ${getOperationErrorMessage(firstError)}`);
            } else if (success > 0) {
                message.success(t('operations.batch.success', { count: success }));
            }
        } finally {
            busyRef.current = false;
            setBusy(false);
        }
        onChanged();
    };

    const selectedOps = () => rows.filter((r) => r.kind === 'operation');

    const handleUndo = () => {
        const targets = selectedOps().filter((r) => r.undoable && !unavailable.has(r.key));
        const ids = targets.map((r) => String(r.operationId));
        // operationId → row.key so a permanent failure can disable exactly
        // the row it came from (unavailable is keyed the same as selection).
        const keyById = new Map(targets.map((r) => [String(r.operationId), r.key]));
        void runBatch(
            'undo',
            ids,
            (id) => invoke('undo_operation_v2', { operationId: id }),
            'history.undo_error',
            (id, err) => {
                const kind = classifyUndoError(err);
                if (kind === 'permanent') {
                    const key = keyById.get(id);
                    if (key) {
                        setUnavailable((prev) => new Set(prev).add(key));
                        message.info(t('history.action_unavailable'));
                    }
                    return true;
                }
                if (kind === 'transient') {
                    message.warning(t('operations.undo_retry'));
                    return true;
                }
                return false;
            },
        );
    };

    const handleRepeat = () => {
        const ids = selectedOps()
            .filter((r) => r.repeatable)
            .map((r) => String(r.operationId));
        void runBatch('repeat', ids, (id) => invoke('repeat_operation_v2', { operationId: id }), 'history.repeat_error');
    };

    const handleDelete = () => {
        const ids = selectedOps().map((r) => String(r.operationId));
        modal.confirm({
            title: t('operations.delete_confirm', { count: ids.length }),
            okText: t('history.delete'),
            cancelText: t('operations.cancel'),
            okButtonProps: { danger: true },
            onOk: () =>
                runBatch(
                    'delete',
                    ids,
                    (id) => invoke('delete_operation', { operationId: id }),
                    'history.delete_error',
                ),
        });
    };

    const handleCancel = () => {
        const ids = rows
            .filter((r) => r.kind === 'job' && r.cancellable)
            .map((r) => String(r.jobId));
        void runBatch('cancel', ids, (id) => invoke('cancel_job', { jobId: id }), 'queue.cancel_error');
    };

    const handleClear = () => {
        modal.confirm({
            title: t('operations.clear_confirm'),
            okText: t('operations.clear'),
            cancelText: t('operations.cancel'),
            okButtonProps: { danger: true },
            onOk: async () => {
                try {
                    await invoke('clear_history');
                    message.success(t('history.clear_success'));
                } catch (err) {
                    logger.error('OperationsToolbar', 'clear history failed', err);
                    message.error(`${t('history.clear_error')} ${err}`);
                }
                onChanged();
            },
        });
    };

    return (
        <div style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            marginBottom: 'var(--qs-space-lg)',
            gap: '8px',
            flexWrap: 'wrap',
        }}>
            <h3 style={{
                fontFamily: 'var(--qs-font-display)',
                fontSize: '16px',
                fontWeight: 600,
                color: 'var(--qs-text-primary)',
                margin: 0,
            }}>
                {t('operations.title')}
            </h3>

            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexWrap: 'wrap' }}>
                <button
                    onClick={onRefresh}
                    disabled={loading || busy}
                    style={{
                        ...toolbarButtonStyle,
                        cursor: loading || busy ? 'not-allowed' : 'pointer',
                        opacity: loading || busy ? 0.7 : 1,
                    }}
                >
                    {loading ? t('operations.loading') : t('operations.refresh')}
                </button>
                <button
                    onClick={handleClear}
                    disabled={!hasOperations || busy}
                    style={{
                        ...toolbarButtonStyle,
                        ...(hasOperations && !busy ? {} : disabledStyle),
                    }}
                >
                    {t('operations.clear')}
                </button>

                {state.selectionCount > 0 && (
                    <span style={{
                        fontFamily: 'var(--qs-font-mono)',
                        fontSize: '12px',
                        color: 'var(--qs-text-muted)',
                        whiteSpace: 'nowrap',
                    }}>
                        {t('operations.selected_count', { count: state.selectionCount })}
                    </span>
                )}

                <button
                    onClick={handleUndo}
                    disabled={busy || !state.canUndo}
                    style={{ ...actionButtonStyle, ...(state.canUndo && !busy ? {} : disabledStyle) }}
                >
                    {t('operations.undo')}
                </button>
                <button
                    onClick={handleRepeat}
                    disabled={busy || !state.canRepeat}
                    style={{ ...actionButtonStyle, ...(state.canRepeat && !busy ? {} : disabledStyle) }}
                >
                    {t('operations.repeat')}
                </button>
                <button
                    onClick={handleDelete}
                    disabled={busy || !state.canDelete}
                    style={{
                        ...actionButtonStyle,
                        color: 'var(--qs-danger, #ef4444)',
                        borderColor: 'var(--qs-danger, #ef4444)',
                        ...(state.canDelete && !busy ? {} : disabledStyle),
                    }}
                >
                    {t('operations.delete')}
                </button>
                <button
                    onClick={handleCancel}
                    disabled={busy || !state.canCancel}
                    style={{ ...actionButtonStyle, ...(state.canCancel && !busy ? {} : disabledStyle) }}
                >
                    {t('operations.cancel')}
                </button>
            </div>
        </div>
    );
};

export default OperationsToolbar;