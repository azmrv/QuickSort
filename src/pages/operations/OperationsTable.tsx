import { useCallback, useEffect, useMemo, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import {
    DataGrid,
    SelectColumn,
    SELECT_COLUMN_KEY,
    type Column,
    type SortColumn,
} from 'react-data-grid';
import 'react-data-grid/lib/styles.css';
import { invoke } from '../../lib/invoke';
import { logger } from '../../lib/logger';
import { useTranslation } from '../../i18n/useTranslation';
import DetailsPopup from './DetailsPopup';
import OperationsToolbar from './OperationsToolbar';
import { getSourceLabel, getStatusLabel, getTypeLabel, SOURCE_COLORS, STATUS_COLORS } from './labels';
import {
    compareRowsByCreatedAtAsc,
    compareRowsByStateAsc,
    formatBytes,
    jobToRow,
    operationToRow,
} from './rowModel';
import {
    compareFilesFirst,
    compareSourceRank,
    compareTypeRank,
    filterJobRows,
    orderRows,
} from './rowOrder';
import type { JobDto, OperationDto, OperationRow } from './types';

// Payload of the `operation-progress` Tauri event (backend emitter, src-tauri/src/progress.rs).
interface ProgressPayload {
    current: number;
    total: number;
    phase: string;
    detail?: string | null;
}

const DEFAULT_COLUMN_ORDER = [SELECT_COLUMN_KEY, 'status', 'type', 'source', 'objects', 'files', 'size', 'created_at'];

// react-data-grid v7 has no per-column sortComparator (unlike v6, where it
// lived on Column). Sorting is consumer-driven: the grid only reports
// sortColumns, so we sort the rows ourselves with these comparators.
const SORT_COMPARATORS: Record<string, (a: OperationRow, b: OperationRow) => number> = {
    status: compareRowsByStateAsc,
    type: compareTypeRank,
    source: compareSourceRank,
    objects: (a, b) => a.filesCount - b.filesCount,
    files: compareFilesFirst,
    size: (a, b) => a.sizeBytes - b.sizeBytes,
    created_at: compareRowsByCreatedAtAsc,
};

const OperationsTable = () => {
    const { t } = useTranslation();
    const [jobs, setJobs] = useState<JobDto[]>([]);
    const [operations, setOperations] = useState<OperationDto[]>([]);
    const [loading, setLoading] = useState(true);
    const [selectedKeys, setSelectedKeys] = useState<Set<string>>(new Set());
    const [sortColumns, setSortColumns] = useState<SortColumn[]>([]);
    const [columnOrder, setColumnOrder] = useState<string[]>(DEFAULT_COLUMN_ORDER);
    const [detailsRow, setDetailsRow] = useState<OperationRow | null>(null);

    const loadOperations = useCallback(() => {
        invoke<OperationDto[]>('get_operations')
            .then(setOperations)
            .catch((err) => logger.error('OperationsTable', 'failed to load operations', err));
    }, []);

    const loadJobs = useCallback(() => {
        invoke<JobDto[]>('get_jobs')
            .then(setJobs)
            .catch((err) => logger.error('OperationsTable', 'failed to load jobs', err));
    }, []);

    const reload = useCallback(() => {
        setLoading(true);
        const finish = () => setLoading(false);
        Promise.all([loadJobs(), loadOperations()]).then(finish, finish);
    }, [loadJobs, loadOperations]);

    useEffect(() => {
        logger.action('OperationsTable', 'mount');
        reload();

        const refreshIfVisible = () => {
            if (document.visibilityState === 'visible') {
                reload();
            }
        };
        document.addEventListener('visibilitychange', refreshIfVisible);
        window.addEventListener('focus', refreshIfVisible);
        const interval = setInterval(() => {
            if (document.visibilityState === 'visible') {
                reload();
            }
        }, 5000);

        return () => {
            document.removeEventListener('visibilitychange', refreshIfVisible);
            window.removeEventListener('focus', refreshIfVisible);
            clearInterval(interval);
        };
    }, [reload]);

    // Live queue updates. A job reaching a terminal status just created an
    // operation — refetch history right away instead of waiting for the poll.
    useEffect(() => {
        const unlisten = listen<{ job: JobDto }>('job-status', (event) => {
            const updated = event.payload.job;
            setJobs((prev) => {
                const idx = prev.findIndex((j) => j.id === updated.id);
                if (idx === -1) return [updated, ...prev];
                const next = [...prev];
                next[idx] = updated;
                return next;
            });
            if (
                updated.status === 'Completed' ||
                updated.status === 'Failed' ||
                updated.status === 'Canceled'
            ) {
                loadOperations();
            }
        });
        // The queue serializes execution, so at most one job is Running at a
        // time; an `operation-progress` event therefore belongs to the running
        // job (the payload carries no id). Update its progress in place.
        const unlistenProgress = listen<ProgressPayload>('operation-progress', (event) => {
            const { current, total } = event.payload;
            setJobs((prev) => {
                const idx = prev.findIndex((j) => j.status === 'Running');
                if (idx === -1) return prev;
                const next = [...prev];
                next[idx] = { ...next[idx], progress: { current, total } };
                return next;
            });
        });
        return () => {
            unlisten.then((fn) => fn());
            unlistenProgress.then((fn) => fn());
        };
    }, [loadOperations]);

    // Display order lives in rowOrder.orderRows (unit-tested); an active sort
    // column re-sorts those rows with the matching comparator.
    const rows = useMemo(() => {
        const ordered = orderRows(filterJobRows(jobs).map(jobToRow), operations.map(operationToRow));
        if (sortColumns.length === 0) return ordered;
        // Single-column sort only; multi-sort (v7 multiSort prop) is out of 0.2.6 scope.
        const { columnKey, direction } = sortColumns[0];
        const comparator = SORT_COMPARATORS[columnKey];
        if (!comparator) return ordered;
        const sorted = ordered.slice().sort(comparator);
        return direction === 'DESC' ? sorted.reverse() : sorted;
    }, [jobs, operations, sortColumns]);

    const columns: Column<OperationRow>[] = useMemo(() => {
        return [
            SelectColumn,
            {
                key: 'status',
                name: t('operations.col.status'),
                width: 170,
                sortable: true,
                resizable: true,
                draggable: true,
                renderCell: ({ row }) => {
                    const color = STATUS_COLORS[row.statusKey] ?? '#6b7280';
                    const progress = row.progress;
                    const showProgress =
                        row.kind === 'job' &&
                        row.statusKey === 'running' &&
                        progress !== null &&
                        progress.total > 0;
                    return (
                        <span style={{ display: 'inline-flex', alignItems: 'center', gap: '6px' }}>
                            <span style={{
                                display: 'inline-block',
                                width: '8px',
                                height: '8px',
                                borderRadius: '50%',
                                background: color,
                                flexShrink: 0,
                            }} />
                            <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                                {getStatusLabel(row, t)}
                            </span>
                            {showProgress && progress !== null && (
                                <span style={{
                                    fontFamily: 'var(--qs-font-mono)',
                                    fontSize: '10px',
                                    color: 'var(--qs-text-muted)',
                                    whiteSpace: 'nowrap',
                                }}>
                                    {progress.current}/{progress.total}
                                </span>
                            )}
                        </span>
                    );
                },
            },
            {
                key: 'type',
                name: t('operations.col.type'),
                width: 100,
                sortable: true,
                resizable: true,
                draggable: true,
                renderCell: ({ row }) => (
                    <span>{getTypeLabel(row, t)}</span>
                ),
            },
            {
                key: 'source',
                name: t('operations.col.source'),
                width: 140,
                sortable: true,
                resizable: true,
                draggable: true,
                renderCell: ({ row }) => {
                    const color = row.source ? (SOURCE_COLORS[row.source] ?? '#6b7280') : '#6b7280';
                    return (
                        <span style={{ display: 'inline-flex', alignItems: 'center', gap: '6px' }}>
                            <span style={{
                                display: 'inline-block',
                                width: '8px',
                                height: '8px',
                                borderRadius: '50%',
                                background: color,
                                flexShrink: 0,
                            }} />
                            <span style={{ color: 'var(--qs-text-secondary)' }}>{getSourceLabel(row, t)}</span>
                        </span>
                    );
                },
            },
            {
                key: 'objects',
                name: t('operations.col.objects'),
                width: 90,
                sortable: true,
                resizable: true,
                draggable: true,
                renderCell: ({ row }) => <span>{row.filesCount}</span>,
            },
            {
                key: 'files',
                name: t('operations.col.files'),
                sortable: true,
                resizable: true,
                draggable: true,
                renderCell: ({ row }) => (
                    <span title={row.files.join('\n')} style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                        {row.files[0]}
                        {row.filesCount > 1 ? ` +${row.filesCount - 1}` : ''}
                    </span>
                ),
            },
            {
                key: 'size',
                name: t('operations.col.size'),
                width: 110,
                sortable: true,
                resizable: true,
                draggable: true,
                renderCell: ({ row }) => (
                    <span style={{ color: 'var(--qs-text-muted)' }}>
                        {row.sizeBytes > 0 ? formatBytes(row.sizeBytes) : '\u2014'}
                    </span>
                ),
            },
            {
                key: 'created_at',
                name: t('operations.col.date'),
                width: 160,
                sortable: true,
                resizable: true,
                draggable: true,
                renderCell: ({ row }) => (
                    <span style={{ color: 'var(--qs-text-muted)' }}>
                        {new Date(row.createdAtMs).toLocaleString()}
                    </span>
                ),
            },
        ];
    }, [t]);

    const handleColumnsReorder = useCallback((sourceKey: string, targetKey: string) => {
        setColumnOrder((prev) => {
            const srcIdx = prev.findIndex((k) => k === sourceKey);
            const tgtIdx = prev.findIndex((k) => k === targetKey);
            if (srcIdx === -1 || tgtIdx === -1) return prev;
            const next = [...prev];
            const [moved] = next.splice(srcIdx, 1);
            next.splice(tgtIdx, 0, moved);
            return next;
        });
    }, []);

    const orderedColumns = useMemo(() => {
        const byKey = new Map(columns.map((c) => [c.key, c]));
        return columnOrder
            .map((key) => byKey.get(key))
            .filter((c): c is Column<OperationRow> => Boolean(c));
    }, [columns, columnOrder]);

    return (
        <div style={{ padding: 'var(--qs-space-lg)' }}>
            <OperationsToolbar
                selectedKeys={selectedKeys}
                rows={rows}
                loading={loading}
                hasOperations={operations.length > 0}
                onRefresh={reload}
                onChanged={reload}
            />

            {loading && rows.length === 0 ? (
                <div style={{
                    textAlign: 'center',
                    padding: 'var(--qs-space-2xl)',
                    color: 'var(--qs-text-muted)',
                }}>
                    <div style={{ fontSize: '32px', marginBottom: 'var(--qs-space-md)' }}>{'\u231B'}</div>
                    <div>{t('operations.loading')}</div>
                </div>
            ) : rows.length === 0 ? (
                <div style={{
                    textAlign: 'center',
                    padding: 'var(--qs-space-2xl)',
                    color: 'var(--qs-text-muted)',
                }}>
                    <div style={{ fontSize: '32px', marginBottom: 'var(--qs-space-md)' }}>{'\uD83D\uDCCB'}</div>
                    <div>{t('operations.empty')}</div>
                </div>
            ) : (
                <div className="operations-grid-wrapper">
                    <DataGrid<OperationRow>
                        columns={orderedColumns}
                        rows={rows}
                        rowKeyGetter={(row) => row.key}
                        rowHeight={28}
                        headerRowHeight={32}
                        selectedRows={selectedKeys}
                        onSelectedRowsChange={(keys) => {
                            const strings: string[] = [];
                            keys.forEach((k) => {
                                if (typeof k === 'string') strings.push(k);
                            });
                            setSelectedKeys(new Set(strings));
                        }}
                        sortColumns={sortColumns}
                        onSortColumnsChange={setSortColumns}
                        onColumnsReorder={handleColumnsReorder}
                        onCellClick={(args) => {
                            if (args.column.key !== SELECT_COLUMN_KEY) {
                                setDetailsRow(args.row);
                            }
                        }}
                        onRowsChange={() => {}}
                        className="rdg operations-grid"
                        direction="ltr"
                    />
                </div>
            )}

            <DetailsPopup row={detailsRow} onClose={() => setDetailsRow(null)} />
        </div>
    );
};

export default OperationsTable;