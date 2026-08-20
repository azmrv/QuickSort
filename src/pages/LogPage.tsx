import { useState, useEffect, useRef } from 'react';
import { logger } from '../lib/logger';
import { App } from 'antd';
import { listen } from '@tauri-apps/api/event';

interface BackendLog {
    timestamp: string;
    level: string;
    target: string;
    message: string;
}

type LogLevel = 'ALL' | 'DEBUG' | 'INFO' | 'WARN' | 'ERROR';

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

const LogPage = () => {
    const [backendLogs, setBackendLogs] = useState<BackendLog[]>([]);
    const [frontendLogs, setFrontendLogs] = useState(logger.getLogs());
    const [filter, setFilter] = useState<LogLevel>('ALL');
    const [showBackend, setShowBackend] = useState(true);
    const [showFrontend, setShowFrontend] = useState(true);
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

    useEffect(() => {
        const list = listRef.current;
        if (!list) return;
        const isNearBottom = list.scrollHeight - list.scrollTop - list.clientHeight < 80;
        if (isNearBottom) {
            endRef.current?.scrollIntoView({ behavior: 'smooth' });
        }
    }, [backendLogs, frontendLogs]);

    const filterLevel = (log: { level?: string }) => {
        if (filter === 'ALL') return true;
        const logLevel = log.level ?? 'INFO';
        return (LEVEL_ORDER[logLevel] ?? 0) >= (LEVEL_ORDER[filter] ?? 0);
    };

    const backendFiltered = showBackend ? backendLogs.filter(filterLevel) : [];
    const frontendFiltered = showFrontend ? frontendLogs.filter(filterLevel) : [];

    const allLogs = [
        ...backendFiltered.map(l => ({ ...l, source: 'backend' as const })),
        ...frontendFiltered.map(l => ({ ...l, source: 'frontend' as const })),
    ].sort((a, b) => a.timestamp.localeCompare(b.timestamp));

    const handleCopyAll = async () => {
        const text = allLogs.map(l =>
            `[${l.timestamp}] [${l.level}] [${l.source}] ${l.message}`
        ).join('\n');
        await navigator.clipboard.writeText(text);
        message.success(`Скопировано ${allLogs.length} записей`);
    };

    return (
        <div className="log-page">
            <div className="log-toolbar">
                <select value={filter} onChange={(e) => setFilter(e.target.value as LogLevel)}>
                    <option value="ALL">Все уровни</option>
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

                <span className="log-count">{allLogs.length}</span>

                <button className="log-copy-btn" onClick={handleCopyAll}>
                    Копировать всё
                </button>
            </div>

            <div className="log-list" ref={listRef}>
                {allLogs.length === 0 ? (
                    <div className="log-empty">Нет записей</div>
                ) : (
                    allLogs.map((log, i) => (
                        <div key={i} className="log-entry">
                            <span className="log-time">{log.timestamp.slice(11, 23)}</span>
                            <span className="log-level" style={{ color: LEVEL_COLORS[log.level] ?? 'var(--qs-text-secondary)' }}>
                                {log.level}
                            </span>
                            <span className="log-source">[{log.source}]</span>
                            <span className="log-message">{log.message}</span>
                        </div>
                    ))
                )}
                <div ref={endRef} />
            </div>
        </div>
    );
};

export default LogPage;
