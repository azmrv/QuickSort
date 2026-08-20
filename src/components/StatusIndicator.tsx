import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

const StatusIndicator: React.FC = () => {
    const [active, setActive] = useState(false);

    useEffect(() => {
        invoke<boolean>('check_menu_status').then(setActive).catch(console.error);
    }, []);

    return (
        <div className="status-bar">
            <div className={`status-dot ${active ? '' : 'error'}`}></div>
            <span>Контекстное меню:</span>
            <span style={{ color: active ? 'var(--qs-success)' : 'var(--qs-danger)' }}>
                {active ? 'Активно' : 'Неактивно'}
            </span>
        </div>
    );
};

export default StatusIndicator;
