import { useState, useEffect } from 'react';
import { App } from 'antd';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';

interface Operation {
    id: string;
    operation_type: string;
    state: Record<string, unknown>;
    source_paths: string[];
    target_folder_path: string | null;
    created_at: string;
    updated_at: string;
}

const HistoryPage = () => {
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
                message.error(`Ошибка загрузки: ${err}`);
            })
            .finally(() => setLoading(false));
    };

    useEffect(() => {
        logger.action('HistoryPage', 'mount');
        loadOperations();
    }, []);

    const handleUndo = async (operationId: string) => {
        try {
            await invoke('undo_operation_v2', { operationId });
            message.success('Операция отменена');
            loadOperations();
        } catch (err) {
            message.error(`Ошибка отмены: ${err}`);
        }
    };

    const getStateLabel = (state: Record<string, unknown>): { text: string; color: string } => {
        if ('Completed' in state) {
            const completed = state.Completed as { processed_files: number; bytes_processed: number };
            return { text: `Выполнено (${completed.processed_files} файлов)`, color: '#22c55e' };
        }
        if ('Failed' in state) {
            const failed = state.Failed as { reason: string };
            return { text: `Ошибка: ${failed.reason}`, color: '#ef4444' };
        }
        if ('Undone' in state) {
            return { text: 'Отменено', color: '#f59e0b' };
        }
        if ('Executing' in state) {
            return { text: 'Выполняется', color: '#3b82f6' };
        }
        if ('Pending' in state) {
            return { text: 'Ожидание', color: '#6b7280' };
        }
        return { text: 'Неизвестно', color: '#6b7280' };
    };

    const getOperationLabel = (type: string): string => {
        switch (type) {
            case 'Move': return 'Перемещение';
            case 'Copy': return 'Копирование';
            case 'Delete': return 'Удаление';
            case 'Rename': return 'Переименование';
            default: return type;
        }
    };

    const canUndo = (op: Operation): boolean => {
        return 'Completed' in op.state && op.operation_type !== 'Delete';
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
                    История операций
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
                    {loading ? 'Загрузка...' : 'Обновить'}
                </button>
            </div>

            {operations.length === 0 ? (
                <div style={{
                    textAlign: 'center',
                    padding: 'var(--qs-space-2xl)',
                    color: 'var(--qs-text-muted)',
                }}>
                    <div style={{ fontSize: '32px', marginBottom: 'var(--qs-space-md)' }}>{'\uD83D\uDCCB'}</div>
                    <div>Нет операций</div>
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
                                {canUndo(op) && (
                                    <button
                                        onClick={() => handleUndo(op.id)}
                                        style={{
                                            padding: '4px 8px',
                                            background: 'transparent',
                                            border: '1px solid var(--qs-border)',
                                            borderRadius: 'var(--qs-radius-sm)',
                                            color: 'var(--qs-accent)',
                                            fontFamily: 'var(--qs-font-mono)',
                                            fontSize: '11px',
                                            cursor: 'pointer',
                                            flexShrink: 0,
                                        }}
                                        onMouseEnter={(e) => {
                                            e.currentTarget.style.background = 'var(--qs-accent-muted)';
                                        }}
                                        onMouseLeave={(e) => {
                                            e.currentTarget.style.background = 'transparent';
                                        }}
                                    >
                                        Отменить
                                    </button>
                                )}
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
};

export default HistoryPage;
