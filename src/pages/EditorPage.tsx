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

    // Загрузка папок через новую команду
    useEffect(() => {
        invoke<Folder[]>('get_folders_v2')
            .then(setFolders)
            .catch(console.error);
    }, []);

    const handleAddFolder = (name: string, path: string) => {
        // Вызываем новую команду add_folder_v2
        invoke('add_folder_v2', { name, path })
            .then(() => {
                // После успешного добавления перезагружаем список
                return invoke<Folder[]>('get_folders_v2');
            })
            .then(setFolders)
            .catch(err => message.error(`Ошибка добавления: ${err}`));
    };

    const handleRename = (id: string, newName: string) => {
        // Пока нет отдельной команды rename_folder – обновляем локально
        setFolders(folders.map((f) => (f.id === id ? { ...f, name: newName } : f)));
        // TODO: вызвать rename_folder_v2, когда будет реализовано
    };

    const handleToggleFavorite = async (id: string) => {
        // Находим папку и её текущий порядок
        const folder = folders.find(f => f.id === id);
        if (!folder) return;
        const newOrder = folder.is_favorite ? 0 : folders.filter(f => f.is_favorite).length + 1;

        // Оптимистичное обновление UI
        setFolders(folders.map((f) =>
            f.id === id ? { ...f, is_favorite: !f.is_favorite, sort_order: newOrder } : f
        ));

        try {
            await invoke('toggle_favorite_v2', { id, order: newOrder });
        } catch (err) {
            console.error(err);
            // Откатываем при ошибке
            setFolders(folders);
            message.error('Ошибка обновления избранного');
        }
    };

    const handleApply = async (newFolders: Folder[]) => {
        // Применяем изменения (переупорядочивание) – можно вызвать update_folders_v2, но его пока нет.
        // Просто обновляем локальный список.
        setFolders(newFolders);
    };

    const handleDeleteMenu = async () => {
        // Удаляем все папки по одной
        try {
            for (const folder of folders) {
                await invoke('remove_folder_v2', { id: folder.id });
            }
            // Перезагружаем список
            const updated = await invoke<Folder[]>('get_folders_v2');
            setFolders(updated);
            message.success('Все папки удалены');
        } catch (err) {
            message.error(`Ошибка удаления: ${err}`);
        }
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