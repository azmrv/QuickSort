import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { List, Typography } from 'antd';

interface LogEntry {
    timestamp: string;
    event: string;
    status: string;
}

const LogPage = () => {
    const [logs, setLogs] = useState<LogEntry[]>([]);
    const loadLogs = () => {
        invoke<LogEntry[]>('get_logs').then(setLogs).catch(console.error);
    };
    useEffect(() => {
        loadLogs();
    }, []);

    return (
        <div>
            <List
                dataSource={logs}
                renderItem={(item) => (
                    <List.Item>
                        <Typography.Text code>{item.timestamp}</Typography.Text> — {item.event} ({item.status})
                    </List.Item>
                )}
            />
        </div>
    );
};
export default LogPage;