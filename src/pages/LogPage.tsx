import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface LogEntry {
    timestamp: string;
    event: string;
    status: string;
}

const LogPage = () => {
    const [logs, setLogs] = useState<LogEntry[]>([]);

    useEffect(() => {
        invoke<LogEntry[]>('get_logs').then(setLogs).catch(console.error);
    }, []);

    if (logs.length === 0) {
        return (
            <div className="empty-state">
                <div className="empty-state-icon">📋</div>
                <div className="empty-state-title">Нет записей</div>
                <div className="empty-state-description">
                    Журнал операций будет отображаться здесь
                </div>
            </div>
        );
    }

    return (
        <div className="folder-list">
            {logs.map((log, index) => (
                <div 
                    key={index}
                    className="folder-card"
                    style={{ animationDelay: `${index * 30}ms` }}
                >
                    <div className="folder-icon" style={{
                        background: log.status === 'Success' 
                            ? 'var(--qs-success-muted)' 
                            : 'var(--qs-danger-muted)',
                        color: log.status === 'Success' 
                            ? 'var(--qs-success)' 
                            : 'var(--qs-danger)',
                    }}>
                        {log.status === 'Success' ? '✓' : '✗'}
                    </div>
                    <div className="folder-info">
                        <div className="folder-name">{log.event}</div>
                        <div className="folder-path">{log.timestamp}</div>
                    </div>
                </div>
            ))}
        </div>
    );
};

export default LogPage;
