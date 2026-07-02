import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Typography, Switch, Space, Button, Popconfirm } from 'antd';
import { BulbOutlined, DeleteOutlined } from '@ant-design/icons';
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
            id: crypto.randomUUID ? crypto.randomUUID() : `${Date.now()}`,
            name,
            path,
            favorite: false,
            order: folders.length + 1,
            stats: { use_count: 0, last_used: null },
        };
        setFolders([...folders, newFolder]);
    };

    const handleRename = (id: string, newName: string) => {
        setFolders(folders.map(f => f.id === id ? { ...f, name: newName } : f));
    };

    const handleToggleFavorite = async (id: string) => {
        // Оптимистично переключаем в локальном состоянии
        setFolders(folders.map(f => f.id === id ? { ...f, favorite: !f.favorite } : f));
        try {
            await invoke('toggle_favorite', { id });
            // После вызова меню обновится на сервере, но локально уже ок
        } catch (err) {
            console.error(err);
            // Можно откатить состояние, но пока упростим
        }
    };

    const handleApply = async (newFolders: Folder[]) => {
        setFolders(newFolders);
    };

    const handleDeleteMenu = async () => {
        await invoke('update_folders', { folders: [] });
        setFolders([]);
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
                onRemove={(id) => {
                    const updated = folders.filter(f => f.id !== id);
                    setFolders(updated);
                }}
                onRename={handleRename}
                onToggleFavorite={handleToggleFavorite}
                onApply={handleApply}
            />
            <div style={{ marginTop: 16 }}>
                <ApplyButton folders={folders} onSuccess={() => {}} />
            </div>
            <div style={{ marginTop: 8 }}>
                <Popconfirm title="Удалить все пункты меню?" onConfirm={handleDeleteMenu} okText="Да" cancelText="Нет">
                    <Button danger icon={<DeleteOutlined />} block disabled={folders.length === 0}>
                        Удалить меню
                    </Button>
                </Popconfirm>
            </div>
        </div>
    );
};

export default EditorPage;