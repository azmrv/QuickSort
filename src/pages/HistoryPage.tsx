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

type SortMode = 'date' | 'status';
type SortDir = 1 | -1;

const CLEAR_COUNTDOWN = 5;

const HistoryPage = () => {
    const { t } = useTranslation();
    const [operations, setOperations] = useState<Operation[]>([]);
    const [loading, setLoading] = useState(true);
    // Tracks operations whose Undo/Repeat cannot be performed anymore (e.g. the
    // target/source files no longer exist). Once the backend rejects the action
    // with "Operation not undoable", the id is added here so the buttons stay
    // disabled instead of re-raising the error on every click.
    const [unavailable, setUnavailable] = useState<Set<string>>(new Set());
    const [sortBy, setSortBy] = useState<SortMode>('date');
    const [sortDir, setSortDir] = useState<SortDir>(-1);
    // Countdown state for the "clear history" confirmation. While > 0 the action
    // is still pending and can be cancelled; when it reaches 0 the clear runs.
    const [clearSeconds, setClearSeconds] = useState<number | null>(null);
    const { message, modal } = App.useApp();

    // The backend reports an impossible action as `UndoNotPossible`, whose
    // Display message starts with this prefix. Match on it to treat the failure
    // as "action not available" rather than a transient error.
    const isUnavailableError = (err: unknown): boolean =>
        String(err).includes('Operation not undoable');

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

    // Drives the countdown dialog: each tick decrements the counter, and when
    // it hits zero the history is actually cleared and the dialog closes.
    useEffect(() => {
        if (clearSeconds === null || clearSeconds <= 0) return;
        const timer = setTimeout(() => {
            setClearSeconds(prev => {
                if (prev === null) return null;
                if (prev <= 1) {
                    performClear();
                    return null;
                }
                return prev - 1;
            });
        }, 1000);
        return () => clearTimeout(timer);
    }, [clearSeconds]);

    const handleUndo = async (operationId: string) => {
        try {
            await invoke('undo_operation_v2', { operationId });
            message.success(t('history.undo_success'));
            loadOperations();
        } catch (err) {
            if (isUnavailableError(err)) {
                setUnavailable(prev => new Set(prev).add(operationId));
                message.info(t('history.action_unavailable'));
            } else {
                message.error(`${t('history.undo_error')} ${err}`);
            }
        }
    };

    const handleRepeat = async (operationId: string) => {
        try {
            await invoke('repeat_operation_v2', { operationId });
            message.success(t('history.repeat_success'));
            loadOperations();
        } catch (err) {
            if (isUnavailableError(err)) {
                setUnavailable(prev => new Set(prev).add(operationId));
                message.info(t('history.action_unavailable'));
            } else {
                message.error(`${t('history.repeat_error')} ${err}`);
            }
        }
    };

    const handleDelete = async (operationId: string) => {
        modal.confirm({
            title: t('history.delete_confirm'),
            okText: t('history.delete'),
            cancelText: t('history.clear_cancel'),
            onOk: async () => {
                try {
                    await invoke('delete_operation', { operationId });
                    message.success(t('history.delete_success'));
                    loadOperations();
                } catch (err) {
                    logger.error('HistoryPage', 'failed to delete operation', err);
                    message.error(`${t('history.delete_error')} ${err}`);
                }
            },
        });
    };

    const startClear = () => {
        setClearSeconds(CLEAR_COUNTDOWN);
    };

    const performClear = async () => {
        try {
            await invoke('clear_history');
            message.success(t('history.clear_success'));
            loadOperations();
        } catch (err) {
            logger.error('HistoryPage', 'failed to clear history', err);
            message.error(`${t('history.clear_error')} ${err}`);
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

    // Numeric rank per state used for sorting by status. Keeping ranks stable
    // across locales lets "Status" sorting behave the same in every language.
    const getStateRank = (state: unknown): number => {
        if (typeof state === 'string') {
            if (state === 'Pending') return 0;
            if (state === 'Executing') return 1;
            if (state === 'Undone') return 4;
            return 5;
        }
        if (typeof state !== 'object' || state === null) return 5;
        const s = state as Record<string, unknown>;
        if ('Pending' in s) return 0;
        if ('Executing' in s) return 1;
        if ('Completed' in s) return 2;
        if ('Failed' in s) return 3;
        if ('Undone' in s) return 4;
        return 5;
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
        if (unavailable.has(op.id)) return false;
        if (typeof op.state === 'string') return false;
        if (typeof op.state !== 'object' || op.state === null) return false;
        return 'Completed' in op.state && op.operation_type !== 'Delete';
    };

    // Repeat is offered for operations that finished (Completed) and for
    // undone ones (redo). Unit variants arrive as plain strings via serde.
    const canRepeat = (op: Operation): boolean => {
        if (unavailable.has(op.id)) return false;
        if (typeof op.state === 'string') return op.state === 'Undone';
        if (typeof op.state !== 'object' || op.state === null) return false;
        return 'Completed' in op.state || 'Undone' in op.state;
    };

    const sortedOperations = [...operations].sort((a, b) => {
        if (sortBy === 'status') {
            const diff = getStateRank(a.state) - getStateRank(b.state);
            if (diff !== 0) return diff * sortDir;
        }
        const timeA = new Date(a.created_at).getTime();
        const timeB = new Date(b.created_at).getTime();
        const timeDiff = timeA - timeB;
        if (timeDiff !== 0) return timeDiff * sortDir;
        return 0;
    });

    const toggleSort = (mode: SortMode) => {
        if (sortBy === mode) {
            setSortDir(prev => (prev === 1 ? -1 : 1));
        } else {
            setSortBy(mode);
            setSortDir(-1);
        }
    };

    const sortControlStyle: React.CSSProperties = {
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

    const buttonBaseStyle: React.CSSProperties = {
        padding: '4px 8px',
        background: 'transparent',
        border: '1px solid var(--qs-border)',
        borderRadius: 'var(--qs-radius-sm)',
        color: 'var(--qs-accent)',
        fontFamily: 'var(--qs-font-mono)',
        fontSize: '11px',
        cursor: 'pointer',
        flexShrink: 0,
    };

    return (
        <div style={{ padding: 'var(--qs-space-lg)' }}>
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
                    {t('history.title')}
                </h3>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexWrap: 'wrap' }}>
                    <span style={{
                        fontFamily: 'var(--qs-font-mono)',
                        fontSize: '11px',
                        color: 'var(--qs-text-muted)',
                    }}>
                        {t('history.sort_by')}
                    </span>
                    <button
                        onClick={() => toggleSort('date')}
                        style={{
                            ...sortControlStyle,
                            color: sortBy === 'date' ? 'var(--qs-accent)' : 'var(--qs-text-secondary)',
                        }}
                    >
                        {t('history.sort_date')} {sortBy === 'date' ? (sortDir === -1 ? '\u2193' : '\u2191') : ''}
                    </button>
                    <button
                        onClick={() => toggleSort('status')}
                        style={{
                            ...sortControlStyle,
                            color: sortBy === 'status' ? 'var(--qs-accent)' : 'var(--qs-text-secondary)',
                        }}
                    >
                        {t('history.sort_status')} {sortBy === 'status' ? (sortDir === -1 ? '\u2193' : '\u2191') : ''}
                    </button>
                    <button
                        onClick={loadOperations}
                        disabled={loading}
                        style={{
                            ...sortControlStyle,
                            cursor: loading ? 'not-allowed' : 'pointer',
                            opacity: loading ? 0.7 : 1,
                        }}
                    >
                        {loading ? t('history.loading') : t('history.refresh')}
                    </button>
                    {operations.length > 0 && (
                        <button
                            onClick={startClear}
                            style={{
                                ...sortControlStyle,
                                color: 'var(--qs-danger, #ef4444)',
                                borderColor: 'var(--qs-border)',
                            }}
                        >
                            {t('history.clear')}
                        </button>
                    )}
                </div>
            </div>

            {clearSeconds !== null && (
                <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '12px',
                    padding: '12px 16px',
                    background: 'var(--qs-bg-tertiary)',
                    border: '1px solid var(--qs-danger, #ef4444)',
                    borderRadius: 'var(--qs-radius-md)',
                    marginBottom: 'var(--qs-space-lg)',
                }}>
                    <span style={{
                        fontFamily: 'var(--qs-font-body)',
                        fontSize: '13px',
                        color: 'var(--qs-text-primary)',
                        flex: 1,
                    }}>
                        {t('history.clear_confirm', { n: clearSeconds })}
                    </span>
                    <button
                        onClick={() => setClearSeconds(null)}
                        style={{
                            ...buttonBaseStyle,
                            color: 'var(--qs-accent)',
                        }}
                    >
                        {t('history.clear_cancel')}
                    </button>
                </div>
            )}

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
                    {sortedOperations.map((op) => {
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
                                        ...buttonBaseStyle,
                                        color: canUndo(op) ? 'var(--qs-accent)' : 'var(--qs-text-muted)',
                                        cursor: canUndo(op) ? 'pointer' : 'not-allowed',
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
                                        ...buttonBaseStyle,
                                        color: canRepeat(op) ? 'var(--qs-accent)' : 'var(--qs-text-muted)',
                                        cursor: canRepeat(op) ? 'pointer' : 'not-allowed',
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
                                <button
                                    onClick={() => handleDelete(op.id)}
                                    style={{
                                        ...buttonBaseStyle,
                                        color: 'var(--qs-text-muted)',
                                    }}
                                    onMouseEnter={(e) => {
                                        e.currentTarget.style.background = 'var(--qs-accent-muted)';
                                        e.currentTarget.style.color = 'var(--qs-danger, #ef4444)';
                                    }}
                                    onMouseLeave={(e) => {
                                        e.currentTarget.style.background = 'transparent';
                                        e.currentTarget.style.color = 'var(--qs-text-muted)';
                                    }}
                                >
                                    {t('history.delete')}
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
