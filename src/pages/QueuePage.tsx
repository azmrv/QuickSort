import { useCallback, useEffect, useState } from 'react';
import { DataGrid, type Column } from 'react-data-grid';
import 'react-data-grid/lib/styles.css';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';
import { useTranslation } from '../i18n/useTranslation';

type JobStatus = 'Queued' | 'Running' | 'Completed' | 'Failed' | 'Canceled';

interface Job {
    id: string;
    operation_type: 'Move' | 'Copy' | 'Delete' | 'Rename';
    source_paths: string[];
    status: JobStatus;
    progress: { current: number; total: number };
    operation_id: string | null;
    error: string | null;
    created_at: number;
    updated_at: number;
}

const STATUS_COLORS: Record<JobStatus, string> = {
    Queued: '#6b7280',
    Running: '#3b82f6',
    Completed: '#22c55e',
    Failed: '#ef4444',
    Canceled: '#9ca3af',
};

const QueuePage = () => {
    const { t } = useTranslation();
    const [jobs, setJobs] = useState<Job[]>([]);
    const [loading, setLoading] = useState(true);
    const [columns, setColumns] = useState<Column<Job>[]>([]);

    const getStatusLabel = (status: JobStatus): string => {
        switch (status) {
            case 'Queued': return t('queue.status.queued');
            case 'Running': return t('queue.status.running');
            case 'Completed': return t('queue.status.completed');
            case 'Failed': return t('queue.status.failed');
            case 'Canceled': return t('queue.status.canceled');
        }
    };

    const getOperationLabel = (type: Job['operation_type']): string => {
        switch (type) {
            case 'Move': return t('queue.operation.move');
            case 'Copy': return t('queue.operation.copy');
            case 'Delete': return t('queue.operation.delete');
            case 'Rename': return t('queue.operation.rename');
        }
    };

    const loadJobs = useCallback(() => {
        invoke<Job[]>('get_jobs')
            .then(setJobs)
            .catch((err) => logger.error('QueuePage', 'failed to load jobs', err))
            .finally(() => setLoading(false));
    }, []);

    useEffect(() => {
        logger.action('QueuePage', 'mount');
        loadJobs();

        const unlisten = listen<{ job: Job }>('job-status', (event) => {
            const updated = event.payload.job;
            setJobs((prev) => {
                const idx = prev.findIndex((j) => j.id === updated.id);
                if (idx === -1) return [updated, ...prev];
                const next = [...prev];
                next[idx] = updated;
                return next;
            });
        });
        return () => { unlisten.then((fn) => fn()); };
    }, [loadJobs]);

    useEffect(() => {
        setColumns([
            {
                key: 'type',
                name: t('queue.col.type'),
                width: 110,
                resizable: true,
                draggable: true,
                renderCell: ({ row }) => <span>{getOperationLabel(row.operation_type)}</span>,
            },
            {
                key: 'path',
                name: t('queue.col.path'),
                resizable: true,
                draggable: true,
                renderCell: ({ row }) => (
                    <span title={row.source_paths.join('\n')}>
                        {row.source_paths[0]}
                        {row.source_paths.length > 1 ? ` +${row.source_paths.length - 1}` : ''}
                    </span>
                ),
            },
            {
                key: 'status',
                name: t('queue.col.status'),
                width: 120,
                resizable: true,
                draggable: true,
                renderCell: ({ row }) => (
                    <span style={{ color: STATUS_COLORS[row.status] }}>
                        {getStatusLabel(row.status)}
                    </span>
                ),
            },
            {
                key: 'progress',
                name: t('queue.col.progress'),
                width: 180,
                resizable: true,
                draggable: true,
                renderCell: ({ row }) => <ProgressBar current={row.progress.current} total={row.progress.total} />,
            },
            {
                key: 'created_at',
                name: t('queue.col.date'),
                width: 160,
                resizable: true,
                draggable: true,
                renderCell: ({ row }) => (
                    <span>{new Date(row.created_at * 1000).toLocaleString()}</span>
                ),
            },
            {
                key: 'actions',
                name: t('queue.col.actions'),
                width: 100,
                resizable: false,
                draggable: false,
                frozen: 'end',
                renderCell: ({ row }) =>
                    row.status === 'Queued' ? (
                        <button
                            onClick={() => handleCancel(row.id)}
                            style={actionButtonStyle}
                        >
                            {t('queue.cancel')}
                        </button>
                    ) : null,
            },
        ]);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [t]);

    const handleCancel = async (id: string) => {
        try {
            await invoke('cancel_job', { jobId: id });
            loadJobs();
        } catch (err) {
            logger.error('QueuePage', 'failed to cancel job', err);
        }
    };

    const handleColumnsReorder = useCallback((sourceKey: string, targetKey: string) => {
        setColumns((prev) => {
            const sourceIdx = prev.findIndex((c) => c.key === sourceKey);
            const targetIdx = prev.findIndex((c) => c.key === targetKey);
            if (sourceIdx === -1 || targetIdx === -1) return prev;
            const next = [...prev];
            const [moved] = next.splice(sourceIdx, 1);
            next.splice(targetIdx, 0, moved);
            return next;
        });
    }, []);

    return (
        <div style={{ padding: 'var(--qs-space-lg)' }}>
            <div style={toolbarRowStyle}>
                <h3 style={titleStyle}>{t('queue.title')}</h3>
                <button
                    onClick={loadJobs}
                    disabled={loading}
                    style={toolbarButtonStyle}
                >
                    {t('queue.refresh')}
                </button>
            </div>

            {jobs.length === 0 ? (
                <div style={emptyStyle}>
                    <div style={{ fontSize: '32px', marginBottom: 'var(--qs-space-md)' }}>{'\uD83D\uDDC4\uFE0F'}</div>
                    <div>{t('queue.empty')}</div>
                </div>
            ) : (
                <div style={gridContainerStyle}>
                    <DataGrid<Job>
                        columns={columns}
                        rows={jobs}
                        rowKeyGetter={(row) => row.id}
                        defaultColumnOptions={{ resizable: true, draggable: true }}
                        onColumnsReorder={handleColumnsReorder}
                        onRowsChange={() => {}}
                        direction="ltr"
                    />
                </div>
            )}
        </div>
    );
};

interface ProgressBarProps {
    current: number;
    total: number;
}

const ProgressBar = ({ current, total }: ProgressBarProps) => {
    const pct = total > 0 ? Math.min(100, Math.round((current / total) * 100)) : 0;
    return (
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', width: '100%' }}>
            <div style={{ flex: 1, height: '8px', background: 'var(--qs-bg-tertiary)', borderRadius: '4px', overflow: 'hidden' }}>
                <div style={{ width: `${pct}%`, height: '100%', background: 'var(--qs-accent)', transition: 'width .2s' }} />
            </div>
            <span style={{ fontFamily: 'var(--qs-font-mono)', fontSize: '11px', color: 'var(--qs-text-secondary)', whiteSpace: 'nowrap' }}>
                {current}/{total}
            </span>
        </div>
    );
};

const toolbarRowStyle: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 'var(--qs-space-lg)',
    gap: '8px',
};

const titleStyle: React.CSSProperties = {
    fontFamily: 'var(--qs-font-display)',
    fontSize: '16px',
    fontWeight: 600,
    color: 'var(--qs-text-primary)',
    margin: 0,
};

const toolbarButtonStyle: React.CSSProperties = {
    padding: '6px 12px',
    background: 'var(--qs-bg-tertiary)',
    border: '1px solid var(--qs-border)',
    borderRadius: 'var(--qs-radius-sm)',
    color: 'var(--qs-text-secondary)',
    fontFamily: 'var(--qs-font-mono)',
    fontSize: '12px',
    cursor: 'pointer',
};

const actionButtonStyle: React.CSSProperties = {
    padding: '2px 8px',
    background: 'transparent',
    border: '1px solid var(--qs-border)',
    borderRadius: 'var(--qs-radius-sm)',
    color: 'var(--qs-accent)',
    fontFamily: 'var(--qs-font-mono)',
    fontSize: '11px',
    cursor: 'pointer',
};

const emptyStyle: React.CSSProperties = {
    textAlign: 'center',
    padding: 'var(--qs-space-2xl)',
    color: 'var(--qs-text-muted)',
};

const gridContainerStyle: React.CSSProperties = {
    height: '60vh',
    border: '1px solid var(--qs-border)',
    borderRadius: 'var(--qs-radius-md)',
    overflow: 'hidden',
};

export default QueuePage;
