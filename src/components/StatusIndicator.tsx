import { useEffect, useState } from 'react';
import { Tag } from 'antd';
import { CheckCircleOutlined, ExclamationCircleOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';

const StatusIndicator: React.FC = () => {
    const [active, setActive] = useState(false);

    useEffect(() => {
        invoke<boolean>('check_menu_status').then(setActive).catch(console.error);
    }, []);

    return (
        <div style={{ marginBottom: 16 }}>
            Статус контекстного меню:{' '}
            {active ? (
                <Tag icon={<CheckCircleOutlined />} color="success">Активно</Tag>
            ) : (
                <Tag icon={<ExclamationCircleOutlined />} color="error">Неактивно</Tag>
            )}
        </div>
    );
};

export default StatusIndicator;