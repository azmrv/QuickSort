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

const LogPage = () => {
    const [backendLogs, setBackendLogs] = useState<BackendLog[]>([]);
    const [frontendLogs, setFrontendLogs] = useState(logger.getLogs());
    const [filter, setFilter] = useState<LogLevel>('ALL');
    const [showBackend, setShowBackend] = useState(true);
    const [showFrontend, setShowFrontend] = useState(true);
    const { message } = App.useApp();
    const endRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        logger.action('LogPage', 'mount — listening for backend logs');
        const unlisten = listen<BackendLog>('backend-log', (event) => {
            setBackendLogs(prev => [...prev.slice(-500), event.payload]);
        });
        return () => { unlisten.then(fn => fn()); };
    }, []);

    useEffect(() => {
        const interval = setInterval(() => {
            setFrontendLogs(logger.getLogs());
        }, 500);
        return () => clearInterval(interval);
    }, []);

    useEffect(() => {
        endRef.current?.scrollIntoView({ behavior: 'smooth' });
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

    const levelColors: Record<string, string> = {
        ERROR: '#ef4444',
        WARN: '#f59e0b',
        INFO: '#3b82f6',
        DEBUG: '#6b7280',
        TRACE: '#9ca3af',
        ACTION: '#8b5cf6',
        IPC: '#06b6d4',
    };

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            <div style={{
                display: 'flex',
                gap: '8px',
                marginBottom: '12px',
                flexWrap: 'wrap',
                alignItems: 'center',
            }}>
                <select
                    value={filter}
                    onChange={(e) => setFilter(e.target.value as LogLevel)}
                    style={{
                        padding: '6px 12px',
                        background: 'var(--qs-bg-tertiary)',
                        border: '1px solid var(--qs-border)',
                        borderRadius: 'var(--qs-radius-sm)',
                        color: 'var(--qs-text-primary)',
                        fontFamily: 'var(--qs-font-body)',
                        fontSize: '13px',
                        cursor: 'pointer',
                    }}
                >
                    <option value="ALL">Все уровни</option>
                    <option value="ERROR">ERROR+</option>
                    <option value="WARN">WARN+</option>
                    <option value="INFO">INFO+</option>
                    <option value="DEBUG">DEBUG+</option>
                </select>

                <label style={{ display: 'flex', alignItems: 'center', gap: '4px', color: 'var(--qs-text-secondary)', fontSize: '13px', cursor: 'pointer' }}>
                    <input type="checkbox" checked={showBackend} onChange={(e) => setShowBackend(e.target.checked)} />
                    Backend
                </label>
                <label style={{ display: 'flex', alignItems: 'center', gap: '4px', color: 'var(--qs-text-secondary)', fontSize: '13px', cursor: 'pointer' }}>
                    <input type="checkbox" checked={showFrontend} onChange={(e) => setShowFrontend(e.target.checked)} />
                    Frontend
                </label>

                <div style={{ flex: 1 }} />

                <button
                    onClick={handleCopyAll}
                    style={{
                        padding: '6px 16px',
                        background: 'var(--qs-accent)',
                        border: 'none',
                        borderRadius: 'var(--qs-radius-sm)',
                        color: 'var(--qs-bg-primary)',
                        fontFamily: 'var(--qs-font-body)',
                        fontSize: '13px',
                        fontWeight: 600,
                        cursor: 'pointer',
                    }}
                >
                    Копировать всё ({allLogs.length})
                </button>
            </div>

            <div style={{
                flex: 1,
                overflow: 'auto',
                background: 'var(--qs-bg-primary)',
                border: '1px solid var(--qs-border)',
                borderRadius: 'var(--qs-radius-md)',
                padding: '8px',
                fontFamily: 'var(--qs-font-mono)',
                fontSize: '12px',
                lineHeight: 1.6,
            }}>
                {allLogs.length === 0 ? (
                    <div style={{ textAlign: 'center', color: 'var(--qs-text-secondary)', padding: '40px' }}>
                        Нет записей
                    </div>
                ) : (
                    allLogs.map((log, i) => (
                        <div key={i} style={{ display: 'flex', gap: '8px', whiteSpace: 'nowrap' }}>
                            <span style={{ color: 'var(--qs-text-secondary)', flexShrink: 0 }}>
                                {log.timestamp.slice(11, 23)}
                            </span>
                            <span style={{
                                color: levelColors[log.level] ?? 'var(--qs-text-secondary)',
                                fontWeight: 600,
                                flexShrink: 0,
                                width: '44px',
                            }}>
                                {log.level}
                            </span>
                            <span style={{
                                color: 'var(--qs-text-secondary)',
                                flexShrink: 0,
                                opacity: 0.6,
                            }}>
                                [{log.source}]
                            </span>
                            <span style={{ color: 'var(--qs-text-primary)' }}>
                                {log.message}
                            </span>
                        </div>
                    ))
                )}
                <div ref={endRef} />
            </div>
        </div>
    );
};

export default LogPage;
