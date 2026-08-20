import { useEffect, useState } from 'react';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';

const StatusIndicator: React.FC = () => {
    const [active, setActive] = useState(false);

    useEffect(() => {
        invoke<boolean>('check_menu_status')
            .then(active => {
                setActive(active);
                logger.info('StatusIndicator', `menu: ${active ? 'active' : 'inactive'}`);
            })
            .catch(err => logger.error('StatusIndicator', 'check_menu_status failed', err));
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
