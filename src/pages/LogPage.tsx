import { useState, useEffect, useRef, useMemo } from 'react';
import { DataGrid, type Column, type SortColumn } from 'react-data-grid';
import 'react-data-grid/lib/styles.css';
import { logger } from '../lib/logger';
import { invoke } from '../lib/invoke';
import { App } from 'antd';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from '../i18n/useTranslation';

interface BackendLog {
    timestamp: string;
    level: string;
    target: string;
    message: string;
}

// Payload of the `operation-progress` Tauri event (backend emitter, src-tauri/src/progress.rs).
interface ProgressPayload {
    current: number;
    total: number;
    phase: string;
    detail?: string | null;
}

// Job DTO from the backend operation queue (src-tauri/src/queue).
interface JobDto {
    id: string;
    operation_type: string;
    source_paths: string[];
    status: string;
    progress: { current: number; total: number };
    operation_id?: string | null;
    error?: string | null;
    created_at: number;
    updated_at: number;
}

interface JobStatusPayload {
    job: JobDto;
}

type LogLevel = 'ALL' | 'DEBUG' | 'INFO' | 'WARN' | 'ERROR';
type LogSortKey = 'time' | 'level' | 'source' | 'target' | 'message';

const LEVEL_ORDER: Record<string, number> = { TRACE: 0, DEBUG: 1, INFO: 2, WARN: 3, ERROR: 4 };

const LEVEL_COLORS: Record<string, string> = {
    ERROR: '#ef4444',
    WARN: '#f59e0b',
    INFO: '#3b82f6',
    DEBUG: '#6b7280',
    TRACE: '#9ca3af',
    ACTION: '#8b5cf6',
    IPC: '#06b6d4',
};

interface MergedLog {
    timestamp: string;
    level: string;
    target?: string;
    message: string;
    source: 'backend' | 'frontend';
}

const LogPage = () => {
    const { t } = useTranslation();
    const [backendLogs, setBackendLogs] = useState<BackendLog[]>([]);
    const [frontendLogs, setFrontendLogs] = useState(logger.getLogs());
    const [filter, setFilter] = useState<LogLevel>('ALL');
    const [showBackend, setShowBackend] = useState(true);
    const [showFrontend, setShowFrontend] = useState(true);
    const [sortColumns, setSortColumns] = useState<SortColumn[]>([{ columnKey: 'time', direction: 'DESC' }]);
    const [columnOrder, setColumnOrder] = useState<string[]>(['time', 'level', 'source', 'target', 'message']);
    const { message } = App.useApp();
    const endRef = useRef<HTMLDivElement>(null);
    const listRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        logger.action('LogPage', 'mount');
        invoke<BackendLog[]>('get_logs')
            .then((history) => {
                setBackendLogs(history.slice(-500));
            })
            .catch((err) => logger.error('LogPage', 'get_logs failed', err));
        const unlistenLog = listen<BackendLog>('backend-log', (event) => {
            setBackendLogs(prev => [...prev.slice(-500), event.payload]);
        });
        const unlistenProgress = listen<ProgressPayload>('operation-progress', (event) => {
            const p = event.payload;
            const entry: BackendLog = {
                timestamp: new Date().toISOString(),
                level: 'INFO',
                target: 'operation',
                message: `${p.phase} ${p.current}/${p.total}${p.detail ? ` \u2014 ${p.detail}` : ''}`,
            };
            setBackendLogs(prev => [...prev.slice(-500), entry]);
        });
        const unlistenJob = listen<JobStatusPayload>('job-status', (event) => {
            const job = event.payload.job;
            const level = job.error ? 'ERROR' : 'INFO';
            const src = job.source_paths[0] ?? '';
            const ext = job.source_paths.length > 1 ? ` +${job.source_paths.length - 1}` : '';
            const detail = job.error ?? `(${job.progress.current}/${job.progress.total})`;
            const entry: BackendLog = {
                timestamp: new Date().toISOString(),
                level,
                target: 'queue',
                message: `job ${job.operation_type} ${job.status} ${src}${ext} ${detail}`,
            };
            setBackendLogs(prev => [...prev.slice(-500), entry]);
        });
        return () => {
            unlistenLog.then(fn => fn());
            unlistenProgress.then(fn => fn());
            unlistenJob.then(fn => fn());
        };
    }, []);

    useEffect(() => {
        const interval = setInterval(() => setFrontendLogs(logger.getLogs()), 500);
        return () => clearInterval(interval);
    }, []);

    const filterLevel = (log: { level?: string }) => {
        if (filter === 'ALL') return true;
        const logLevel = log.level ?? 'INFO';
        return (LEVEL_ORDER[logLevel] ?? 0) >= (LEVEL_ORDER[filter] ?? 0);
    };

    const backendFiltered = showBackend ? backendLogs.filter(filterLevel) : [];
    const frontendFiltered = showFrontend ? frontendLogs.filter(filterLevel) : [];

    const allLogs: MergedLog[] = [
        ...backendFiltered.map(l => ({ ...l, source: 'backend' as const })),
        ...frontendFiltered.map(l => ({ ...l, source: 'frontend' as const })),
    ];

    const getSortValue = (log: MergedLog, key: LogSortKey): string | number => {
        switch (key) {
            case 'time': return log.timestamp;
            case 'level': return LEVEL_ORDER[log.level] ?? 5;
            case 'source': return log.source;
            case 'target': return log.target ?? '';
            case 'message': return log.message;
        }
    };

    const sortedLogs = useMemo(() => {
        const combined = [...allLogs];
        if (sortColumns.length === 0) return combined;
        return combined.sort((a, b) => {
            for (const { columnKey, direction } of sortColumns) {
                const va = getSortValue(a, columnKey as LogSortKey);
                const vb = getSortValue(b, columnKey as LogSortKey);
                if (va < vb) return direction === 'ASC' ? -1 : 1;
                if (va > vb) return direction === 'ASC' ? 1 : -1;
            }
            return 0;
        });
    }, [allLogs, sortColumns]);

    const handleCopyAll = async () => {
        const text = sortedLogs.map(l =>
            `[${l.timestamp}] [${l.level}] [${l.source}]${l.target ? ` [${l.target}]` : ''} ${l.message}`
        ).join('\n');
        await navigator.clipboard.writeText(text);
        message.success(t('log.copy_success', { count: sortedLogs.length }));
    };

    const allColumns: Column<MergedLog>[] = [
        {
            key: 'time',
            name: t('log.col.time'),
            width: 110,
            sortable: true,
            resizable: true,
            draggable: true,
            renderCell: ({ row }) => (
                <span style={{ color: 'var(--qs-text-muted)' }}>{row.timestamp.slice(11, 23)}</span>
            ),
        },
        {
            key: 'level',
            name: t('log.col.level'),
            width: 90,
            sortable: true,
            resizable: true,
            draggable: true,
            renderCell: ({ row }) => (
                <span style={{ color: LEVEL_COLORS[row.level] ?? 'var(--qs-text-secondary)' }}>{row.level}</span>
            ),
        },
        {
            key: 'source',
            name: t('log.col.source'),
            width: 90,
            sortable: true,
            resizable: true,
            draggable: true,
            renderCell: ({ row }) => (
                <span style={{ color: 'var(--qs-text-muted)' }}>[{row.source}]</span>
            ),
        },
        {
            key: 'target',
            name: t('log.col.target'),
            width: 160,
            sortable: true,
            resizable: true,
            draggable: true,
            renderCell: ({ row }) => (
                <span style={{ color: '#8b5cf6' }}>{row.target || '\u2014'}</span>
            ),
        },
        {
            key: 'message',
            name: t('log.col.message'),
            sortable: true,
            resizable: true,
            draggable: true,
            renderCell: ({ row }) => (
                <span style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>{row.message}</span>
            ),
        },
    ];

    const handleColumnsReorder = (sourceKey: string, targetKey: string) => {
        setColumnOrder((prev) => {
            const srcIdx = prev.findIndex((k) => k === sourceKey);
            const tgtIdx = prev.findIndex((k) => k === targetKey);
            if (srcIdx === -1 || tgtIdx === -1) return prev;
            const next = [...prev];
            const [moved] = next.splice(srcIdx, 1);
            next.splice(tgtIdx, 0, moved);
            return next;
        });
    };

    const orderedColumns = useMemo(
        () => {
            const byKey = new Map(allColumns.map((c) => [c.key, c]));
            return columnOrder
                .map((key) => byKey.get(key))
                .filter((c): c is Column<MergedLog> => Boolean(c));
        },
        // eslint-disable-next-line react-hooks/exhaustive-deps
        [columnOrder, allColumns]
    );

    return (
        <div className="log-page">
            <div className="log-toolbar">
                <select value={filter} onChange={(e) => setFilter(e.target.value as LogLevel)}>
                    <option value="ALL">{t('log.all_levels')}</option>
                    <option value="ERROR">ERROR+</option>
                    <option value="WARN">WARN+</option>
                    <option value="INFO">INFO+</option>
                    <option value="DEBUG">DEBUG+</option>
                </select>

                <label>
                    <input type="checkbox" checked={showBackend} onChange={(e) => setShowBackend(e.target.checked)} />
                    Backend
                </label>
                <label>
                    <input type="checkbox" checked={showFrontend} onChange={(e) => setShowFrontend(e.target.checked)} />
                    Frontend
                </label>

                <span className="log-count">{sortedLogs.length}</span>

                <button className="log-copy-btn" onClick={handleCopyAll}>
                    {t('log.copy_all')}
                </button>
            </div>

            <div className="log-list" ref={listRef}>
                {sortedLogs.length === 0 ? (
                    <div className="log-empty">{t('log.empty')}</div>
                ) : (
                    <DataGrid<MergedLog>
                        columns={orderedColumns}
                        rows={sortedLogs}
                        rowKeyGetter={(row) => `${row.timestamp}-${row.level}-${row.source}-${row.message}`}
                        sortColumns={sortColumns}
                        onSortColumnsChange={setSortColumns}
                        onColumnsReorder={handleColumnsReorder}
                        onRowsChange={() => {}}
                        direction="ltr"
                    />
                )}
                <div ref={endRef} />
            </div>
        </div>
    );
};

export default LogPage;
