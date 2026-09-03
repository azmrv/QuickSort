import { useState, useEffect, useRef } from 'react';
import { logger } from '../lib/logger';
import { App } from 'antd';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from '../i18n/useTranslation';

interface BackendLog {
    timestamp: string;
    level: string;
    target: string;
    message: string;
}

type LogLevel = 'ALL' | 'DEBUG' | 'INFO' | 'WARN' | 'ERROR';
type LogSortKey = 'time' | 'level' | 'source' | 'target' | 'message';
type LogSortDir = 1 | -1;

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
    const [sortKey, setSortKey] = useState<LogSortKey>('time');
    const [sortDir, setSortDir] = useState<LogSortDir>(-1);
    const { message } = App.useApp();
    const endRef = useRef<HTMLDivElement>(null);
    const listRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        logger.action('LogPage', 'mount');
        const unlisten = listen<BackendLog>('backend-log', (event) => {
            setBackendLogs(prev => [...prev.slice(-500), event.payload]);
        });
        return () => { unlisten.then(fn => fn()); };
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

    const sortedLogs = [...allLogs].sort((a, b) => {
        const va = getSortValue(a, sortKey);
        const vb = getSortValue(b, sortKey);
        if (va < vb) return -1 * sortDir;
        if (va > vb) return 1 * sortDir;
        return 0;
    });

    const handleSort = (key: LogSortKey) => {
        if (sortKey === key) {
            setSortDir(prev => (prev === 1 ? -1 : 1));
        } else {
            setSortKey(key);
            setSortDir(-1);
        }
    };

    const handleCopyAll = async () => {
        const text = sortedLogs.map(l =>
            `[${l.timestamp}] [${l.level}] [${l.source}]${l.target ? ` [${l.target}]` : ''} ${l.message}`
        ).join('\n');
        await navigator.clipboard.writeText(text);
        message.success(t('log.copy_success', { count: sortedLogs.length }));
    };

    const columns: { key: LogSortKey; label: string }[] = [
        { key: 'time', label: t('log.col.time') },
        { key: 'level', label: t('log.col.level') },
        { key: 'source', label: t('log.col.source') },
        { key: 'target', label: t('log.col.target') },
        { key: 'message', label: t('log.col.message') },
    ];

    const headerStyle: React.CSSProperties = {
        padding: '8px 10px',
        background: 'var(--qs-bg-tertiary)',
        color: 'var(--qs-text-secondary)',
        fontFamily: 'var(--qs-font-mono)',
        fontSize: '11px',
        fontWeight: 600,
        textAlign: 'left',
        borderBottom: '1px solid var(--qs-border)',
        whiteSpace: 'nowrap',
        cursor: 'pointer',
        userSelect: 'none',
    };

    const cellStyle: React.CSSProperties = {
        padding: '6px 10px',
        borderBottom: '1px solid var(--qs-border)',
        fontFamily: 'var(--qs-font-mono)',
        fontSize: '11px',
        color: 'var(--qs-text-primary)',
        verticalAlign: 'top',
        whiteSpace: 'nowrap',
    };

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
                    <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                        <thead>
                            <tr>
                                {columns.map((col) => (
                                    <th
                                        key={col.key}
                                        onClick={() => handleSort(col.key)}
                                        style={{
                                            ...headerStyle,
                                            color: sortKey === col.key ? 'var(--qs-accent)' : 'var(--qs-text-secondary)',
                                        }}
                                    >
                                        {col.label}
                                        {sortKey === col.key ? ` ${sortDir === -1 ? '\u2193' : '\u2191'}` : ''}
                                    </th>
                                ))}
                            </tr>
                        </thead>
                        <tbody>
                            {sortedLogs.map((log, i) => (
                                <tr key={i} style={{ background: 'var(--qs-bg-secondary)' }}>
                                    <td style={{ ...cellStyle, color: 'var(--qs-text-muted)' }}>{log.timestamp.slice(11, 23)}</td>
                                    <td style={{ ...cellStyle, color: LEVEL_COLORS[log.level] ?? 'var(--qs-text-secondary)' }}>{log.level}</td>
                                    <td style={{ ...cellStyle, color: 'var(--qs-text-muted)' }}>[{log.source}]</td>
                                    <td style={{ ...cellStyle, color: '#8b5cf6', maxWidth: '160px', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                                        {log.target || '\u2014'}
                                    </td>
                                    <td style={{ ...cellStyle, whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>{log.message}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                )}
                <div ref={endRef} />
            </div>
        </div>
    );
};

export default LogPage;
