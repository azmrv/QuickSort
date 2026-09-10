import { App } from 'antd';
import { invoke } from '../../lib/invoke';
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

    const state = computeToolbarState(selectedKeys, rows);

    // Sequential batch runner: never parallel so a flood of in-flight
    // requests cannot happen. Ends with one aggregate toast per action.
    const runBatch = async (
        action: string,
        ids: string[],
        invokeFn: (id: string) => Promise<unknown>,
        errorKey: string,
    ): Promise<void> => {
        let success = 0;
        let failed = 0;
        let firstError: unknown = null;
        for (const id of ids) {
            try {
                await invokeFn(id);
                success += 1;
            } catch (err) {
                failed += 1;
                if (firstError === null) firstError = err;
                logger.error('OperationsToolbar', `${action} ${id} failed`, err);
            }
        }
        if (failed > 0 && success > 0) {
            message.warning(t('operations.batch.partial', { success, failed }));
        } else if (failed > 0) {
            message.error(`${t(errorKey)} ${firstError ?? ''}`);
        } else if (success > 0) {
            message.success(t('operations.batch.success', { count: success }));
        }
        onChanged();
    };

    const selectedOps = () => rows.filter((r) => r.kind === 'operation');

    const handleUndo = () => {
        const ids = selectedOps()
            .filter((r) => r.undoable)
            .map((r) => String(r.operationId));
        void runBatch('undo', ids, (id) => invoke('undo_operation_v2', { operationId: id }), 'history.undo_error');
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
                    disabled={loading}
                    style={{
                        ...toolbarButtonStyle,
                        cursor: loading ? 'not-allowed' : 'pointer',
                        opacity: loading ? 0.7 : 1,
                    }}
                >
                    {loading ? t('operations.loading') : t('operations.refresh')}
                </button>
                <button
                    onClick={handleClear}
                    disabled={!hasOperations}
                    style={{
                        ...toolbarButtonStyle,
                        ...(hasOperations ? {} : disabledStyle),
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
                    disabled={!state.canUndo}
                    style={{ ...actionButtonStyle, ...(state.canUndo ? {} : disabledStyle) }}
                >
                    {t('operations.undo')}
                </button>
                <button
                    onClick={handleRepeat}
                    disabled={!state.canRepeat}
                    style={{ ...actionButtonStyle, ...(state.canRepeat ? {} : disabledStyle) }}
                >
                    {t('operations.repeat')}
                </button>
                <button
                    onClick={handleDelete}
                    disabled={!state.canDelete}
                    style={{
                        ...actionButtonStyle,
                        color: 'var(--qs-danger, #ef4444)',
                        borderColor: 'var(--qs-danger, #ef4444)',
                        ...(state.canDelete ? {} : disabledStyle),
                    }}
                >
                    {t('operations.delete')}
                </button>
                <button
                    onClick={handleCancel}
                    disabled={!state.canCancel}
                    style={{ ...actionButtonStyle, ...(state.canCancel ? {} : disabledStyle) }}
                >
                    {t('operations.cancel')}
                </button>
            </div>
        </div>
    );
};

export default OperationsToolbar;