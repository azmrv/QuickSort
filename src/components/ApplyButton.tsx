import { Button, message } from 'antd';
import { CheckOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { Folder } from '../types';

interface ApplyButtonProps {
    folders: Folder[];
    onSuccess?: () => void;
}

const ApplyButton: React.FC<ApplyButtonProps> = ({ folders, onSuccess }) => {
    const handleApply = async () => {
        try {
            await invoke('update_folders', { folders });
            message.success('Контекстное меню обновлено!');
            onSuccess?.();
        } catch (err) {
            message.error(`Ошибка: ${err}`);
        }
    };

    return (
        <Button type="primary" icon={<CheckOutlined />} onClick={handleApply} block disabled={folders.length === 0}>
            Применить
        </Button>
    );
};

export default ApplyButton;