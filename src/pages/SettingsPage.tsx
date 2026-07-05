import { Button, message, Space } from 'antd';
import { invoke } from '@tauri-apps/api/core';

const SettingsPage: React.FC = () => {
    const handleRegister = async () => {
        try {
            const msg = await invoke<string>('register_com_server');
            message.success(msg);
        } catch (err) {
            message.error(`Ошибка: ${err}`);
        }
    };
    const handleUnregister = async () => {
        try {
            const msg = await invoke<string>('unregister_com_server');
            message.success(msg);
        } catch (err) {
            message.error(`Ошибка: ${err}`);
        }
    };
    return (
        <Space direction="vertical" style={{ width: '100%' }}>
            <Button type="primary" onClick={handleRegister}>Зарегистрировать COM-сервер</Button>
            <Button danger onClick={handleUnregister}>Удалить COM-сервер</Button>
        </Space>
    );
};
export default SettingsPage;