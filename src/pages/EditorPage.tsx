import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Typography, Switch, Space, Button, Popconfirm, message } from 'antd';
import { BulbOutlined, DeleteOutlined, CheckCircleOutlined, CloseCircleOutlined } from '@ant-design/icons';
import FolderList from '../components/FolderList';
import AddFolderButton from '../components/AddFolderButton';
import ApplyButton from '../components/ApplyButton';
import StatusIndicator from '../components/StatusIndicator';
import { Folder } from '../types';

const { Title } = Typography;

interface EditorPageProps {
    isDark: boolean;
    onToggleTheme: (checked: boolean) => void;
}

const EditorPage: React.FC<EditorPageProps> = ({ isDark, onToggleTheme }) => {
    const [folders, setFolders] = useState<Folder[]>([]);

    useEffect(() => {
        invoke<Folder[]>('get_folders').then(setFolders).catch(console.error);
    }, []);

    const handleAddFolder = (name: string, path: string) => {
        const newFolder: Folder = {
            id: crypto.randomUUID(),
            name,
            path,
            favorite: false,
            order: folders.length + 1,
            stats: { use_count: 0, last_used: null },
        };
        setFolders([...folders, newFolder]);
    };

    const handleRename = (id: string, newName: string) => {
        setFolders(folders.map((f) => (f.id === id ? { ...f, name: newName } : f)));
    };

    const handleToggleFavorite = async (id: string) => {
        setFolders(folders.map((f) => (f.id === id ? { ...f, favorite: !f.favorite } : f)));
        try {
            await invoke('toggle_favorite', { id });
        } catch (err) {
            console.error(err);
        }
    };

    const handleApply = async (newFolders: Folder[]) => {
        setFolders(newFolders);
    };

    const handleDeleteMenu = async () => {
        await invoke('update_folders', { folders: [] });
        setFolders([]);
    };

    const handleRegisterComServer = async () => {
        try {
            const msg = await invoke<string>('register_com_server');
            message.success(msg);
        } catch (err) {
            message.error(`Ошибка регистрации: ${err}`);
        }
    };

    const handleUnregisterComServer = async () => {
        try {
            const msg = await invoke<string>('unregister_com_server');
            message.success(msg);
        } catch (err) {
            message.error(`Ошибка удаления: ${err}`);
        }
    };

    return (
        <div style={{ maxWidth: 640, margin: '0 auto', padding: 24, minHeight: '100vh' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 24 }}>
                <Title level={3} style={{ margin: 0 }}>QuickSort</Title>
                <Space>
                    <BulbOutlined />
                    <Switch checked={isDark} onChange={onToggleTheme} checkedChildren="🌙" unCheckedChildren="☀️" />
                </Space>
            </div>
            <StatusIndicator />
            <AddFolderButton onFolderAdded={handleAddFolder} />
            <FolderList
                folders={folders}
                onRename={handleRename}
                onToggleFavorite={handleToggleFavorite}
                onApply={handleApply}
            />
            <div style={{ marginTop: 16 }}>
                <ApplyButton folders={folders} onSuccess={() => {}} />
            </div>
            <div style={{ marginTop: 8, display: 'flex', gap: 8 }}>
                <Popconfirm title="Удалить все пункты меню?" onConfirm={handleDeleteMenu} okText="Да" cancelText="Нет">
                    <Button danger icon={<DeleteOutlined />} block disabled={folders.length === 0}>
                        Удалить меню
                    </Button>
                </Popconfirm>
            </div>
            <div style={{ marginTop: 8, display: 'flex', gap: 8 }}>
                <Button icon={<CheckCircleOutlined />} onClick={handleRegisterComServer} block>
                    Зарегистрировать COM-сервер
                </Button>
                <Button icon={<CloseCircleOutlined />} onClick={handleUnregisterComServer} block>
                    Удалить COM-сервер
                </Button>
            </div>
        </div>
    );
};

export default EditorPage;