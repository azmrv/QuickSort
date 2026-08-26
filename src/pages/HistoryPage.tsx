import { useState, useEffect } from 'react';
import { App } from 'antd';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';
import { useTranslation } from '../i18n/useTranslation';

interface Operation {
    id: string;
    operation_type: string;
    state: unknown;
    source_paths: string[];
    target_folder_path: string | null;
    created_at: string;
    updated_at: string;
}

const HistoryPage = () => {
    const { t } = useTranslation();
    const [operations, setOperations] = useState<Operation[]>([]);
    const [loading, setLoading] = useState(true);
    const { message } = App.useApp();

    const loadOperations = () => {
        setLoading(true);
        invoke<Operation[]>('get_operations')
            .then(ops => {
                setOperations(ops);
                logger.info('HistoryPage', `loaded ${ops.length} operations`);
            })
            .catch(err => {
                logger.error('HistoryPage', 'failed to load operations', err);
                message.error(`${t('editor.load_error')} ${err}`);
            })
            .finally(() => setLoading(false));
    };

    useEffect(() => {
        logger.action('HistoryPage', 'mount');
        loadOperations();

        const refreshIfVisible = () => {
            if (document.visibilityState === 'visible') {
                loadOperations();
            }
        };
        document.addEventListener('visibilitychange', refreshIfVisible);
        window.addEventListener('focus', refreshIfVisible);

        // Periodic polling every 5 seconds while visible — covers Tauri
        // webview quirks where focus/visibility events don't fire reliably.
        const interval = setInterval(() => {
            if (document.visibilityState === 'visible') {
                loadOperations();
            }
        }, 5000);

        return () => {
            document.removeEventListener('visibilitychange', refreshIfVisible);
            window.removeEventListener('focus', refreshIfVisible);
            clearInterval(interval);
        };
    }, []);

    const handleUndo = async (operationId: string) => {
        try {
            await invoke('undo_operation_v2', { operationId });
            message.success(t('history.undo_success'));
            loadOperations();
        } catch (err) {
            message.error(`${t('history.undo_error')} ${err}`);
        }
    };

    const handleRepeat = async (operationId: string) => {
        try {
            await invoke('repeat_operation_v2', { operationId });
            message.success(t('history.repeat_success'));
            loadOperations();
        } catch (err) {
            message.error(`${t('history.repeat_error')} ${err}`);
        }
    };

    const getStateLabel = (state: unknown): { text: string; color: string } => {
        // Serde serializes unit variants as strings ("Undone"), not objects.
        if (typeof state === 'string') {
            if (state === 'Undone') return { text: t('history.state.undone'), color: '#f59e0b' };
            if (state === 'Pending') return { text: t('history.state.pending'), color: '#6b7280' };
            if (state === 'Executing') return { text: t('history.state.executing'), color: '#3b82f6' };
            return { text: state, color: '#6b7280' };
        }
        if (typeof state !== 'object' || state === null) {
            return { text: t('history.state.unknown'), color: '#6b7280' };
        }
        const s = state as Record<string, unknown>;
        if ('Completed' in s) {
            const completed = s.Completed as { processed_files: number; bytes_processed: number };
            return { text: t('history.state.completed', { count: completed.processed_files }), color: '#22c55e' };
        }
        if ('Failed' in s) {
            const failed = s.Failed as { reason: string };
            return { text: `${t('history.state.failed')} ${failed.reason}`, color: '#ef4444' };
        }
        if ('Undone' in s) {
            return { text: t('history.state.undone'), color: '#f59e0b' };
        }
        if ('Executing' in s) {
            return { text: t('history.state.executing'), color: '#3b82f6' };
        }
        if ('Pending' in s) {
            return { text: t('history.state.pending'), color: '#6b7280' };
        }
        return { text: t('history.state.unknown'), color: '#6b7280' };
    };

    const getOperationLabel = (type: string): string => {
        switch (type) {
            case 'Move': return t('history.operation.move');
            case 'Copy': return t('history.operation.copy');
            case 'Delete': return t('history.operation.delete');
            case 'Rename': return t('history.operation.rename');
            default: return type;
        }
    };

    const canUndo = (op: Operation): boolean => {
        if (typeof op.state === 'string') return false;
        if (typeof op.state !== 'object' || op.state === null) return false;
        return 'Completed' in op.state && op.operation_type !== 'Delete';
    };

    // Repeat is offered for operations that finished (Completed) and for
    // undone ones (redo). Unit variants arrive as plain strings via serde.
    const canRepeat = (op: Operation): boolean => {
        if (typeof op.state === 'string') return op.state === 'Undone';
        if (typeof op.state !== 'object' || op.state === null) return false;
        return 'Completed' in op.state || 'Undone' in op.state;
    };

    return (
        <div style={{ padding: 'var(--qs-space-lg)' }}>
            <div style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                marginBottom: 'var(--qs-space-lg)',
            }}>
                <h3 style={{
                    fontFamily: 'var(--qs-font-display)',
                    fontSize: '16px',
                    fontWeight: 600,
                    color: 'var(--qs-text-primary)',
                    margin: 0,
                }}>
                    {t('history.title')}
                </h3>
                <button
                    onClick={loadOperations}
                    disabled={loading}
                    style={{
                        padding: '6px 12px',
                        background: 'var(--qs-bg-tertiary)',
                        border: '1px solid var(--qs-border)',
                        borderRadius: 'var(--qs-radius-sm)',
                        color: 'var(--qs-text-secondary)',
                        fontFamily: 'var(--qs-font-mono)',
                        fontSize: '12px',
                        cursor: loading ? 'not-allowed' : 'pointer',
                    }}
                >
                    {loading ? t('history.loading') : t('history.refresh')}
                </button>
            </div>

            {operations.length === 0 ? (
                <div style={{
                    textAlign: 'center',
                    padding: 'var(--qs-space-2xl)',
                    color: 'var(--qs-text-muted)',
                }}>
                    <div style={{ fontSize: '32px', marginBottom: 'var(--qs-space-md)' }}>{'\uD83D\uDCCB'}</div>
                    <div>{t('history.empty')}</div>
                </div>
            ) : (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                    {operations.map((op) => {
                        const state = getStateLabel(op.state);
                        return (
                            <div
                                key={op.id}
                                style={{
                                    padding: '12px 16px',
                                    background: 'var(--qs-bg-secondary)',
                                    border: '1px solid var(--qs-border)',
                                    borderRadius: 'var(--qs-radius-md)',
                                    display: 'flex',
                                    alignItems: 'center',
                                    gap: '12px',
                                }}
                            >
                                <div style={{
                                    width: '8px',
                                    height: '8px',
                                    borderRadius: '50%',
                                    background: state.color,
                                    flexShrink: 0,
                                }} />
                                <div style={{ flex: 1, minWidth: 0 }}>
                                    <div style={{
                                        fontFamily: 'var(--qs-font-body)',
                                        fontSize: '13px',
                                        fontWeight: 500,
                                        color: 'var(--qs-text-primary)',
                                        marginBottom: '2px',
                                    }}>
                                        {getOperationLabel(op.operation_type)}
                                    </div>
                                    <div style={{
                                        fontFamily: 'var(--qs-font-mono)',
                                        fontSize: '11px',
                                        color: 'var(--qs-text-muted)',
                                        overflow: 'hidden',
                                        textOverflow: 'ellipsis',
                                        whiteSpace: 'nowrap',
                                    }}>
                                        {op.source_paths[0]}
                                        {op.source_paths.length > 1 && ` +${op.source_paths.length - 1}`}
                                    </div>
                                </div>
                                <div style={{
                                    fontFamily: 'var(--qs-font-mono)',
                                    fontSize: '11px',
                                    color: state.color,
                                    flexShrink: 0,
                                }}>
                                    {state.text}
                                </div>
                                <div style={{
                                    fontFamily: 'var(--qs-font-mono)',
                                    fontSize: '11px',
                                    color: 'var(--qs-text-muted)',
                                    flexShrink: 0,
                                }}>
                                    {new Date(op.created_at).toLocaleTimeString('ru-RU')}
                                </div>
                                <button
                                    onClick={() => handleUndo(op.id)}
                                    disabled={!canUndo(op)}
                                    style={{
                                        padding: '4px 8px',
                                        background: 'transparent',
                                        border: '1px solid var(--qs-border)',
                                        borderRadius: 'var(--qs-radius-sm)',
                                        color: canUndo(op) ? 'var(--qs-accent)' : 'var(--qs-text-muted)',
                                        fontFamily: 'var(--qs-font-mono)',
                                        fontSize: '11px',
                                        cursor: canUndo(op) ? 'pointer' : 'not-allowed',
                                        flexShrink: 0,
                                        opacity: canUndo(op) ? 1 : 0.7,
                                    }}
                                    onMouseEnter={(e) => {
                                        if (canUndo(op)) e.currentTarget.style.background = 'var(--qs-accent-muted)';
                                    }}
                                    onMouseLeave={(e) => {
                                        e.currentTarget.style.background = 'transparent';
                                    }}
                                >
                                    {t('history.undo')}
                                </button>
                                <button
                                    onClick={() => handleRepeat(op.id)}
                                    disabled={!canRepeat(op)}
                                    style={{
                                        padding: '4px 8px',
                                        background: 'transparent',
                                        border: '1px solid var(--qs-border)',
                                        borderRadius: 'var(--qs-radius-sm)',
                                        color: canRepeat(op) ? 'var(--qs-accent)' : 'var(--qs-text-muted)',
                                        fontFamily: 'var(--qs-font-mono)',
                                        fontSize: '11px',
                                        cursor: canRepeat(op) ? 'pointer' : 'not-allowed',
                                        flexShrink: 0,
                                        opacity: canRepeat(op) ? 1 : 0.7,
                                    }}
                                    onMouseEnter={(e) => {
                                        if (canRepeat(op)) e.currentTarget.style.background = 'var(--qs-accent-muted)';
                                    }}
                                    onMouseLeave={(e) => {
                                        e.currentTarget.style.background = 'transparent';
                                    }}
                                >
                                    {t('history.repeat')}
                                </button>
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
};

export default HistoryPage;
