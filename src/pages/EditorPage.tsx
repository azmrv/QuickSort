import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { message } from 'antd';
import FolderList from '../components/FolderList';
import AddFolderButton from '../components/AddFolderButton';
import StatusIndicator from '../components/StatusIndicator';
import { Folder } from '../types';

interface EditorPageProps {
    isDark: boolean;
    onToggleTheme: (checked: boolean) => void;
}

const EditorPage: React.FC<EditorPageProps> = () => {
    const [folders, setFolders] = useState<Folder[]>([]);

    // Load folders via the new command
    useEffect(() => {
        invoke<Folder[]>('get_folders_v2')
            .then(setFolders)
            .catch(console.error);
    }, []);

    const handleAddFolder = (name: string, path: string) => {
        // Call the new add_folder_v2 command
        invoke('add_folder_v2', { name, path })
            .then(() => {
                // Reload the list after successful addition
                return invoke<Folder[]>('get_folders_v2');
            })
            .then(setFolders)
            .catch(err => message.error(`Ошибка добавления: ${err}`));
    };

    const handleRename = (id: string, newName: string) => {
        // Update locally for now
        setFolders(folders.map((f) => (f.id === id ? { ...f, name: newName } : f)));
        // TODO: Call rename_folder_v2 when implemented
    };

    const handleToggleFavorite = async (id: string) => {
        // Find the folder and its current order
        const folder = folders.find(f => f.id === id);
        if (!folder) return;
        const newOrder = folder.favorite ? 0 : folders.filter(f => f.favorite).length + 1;

        // Optimistic UI update
        setFolders(folders.map((f) =>
            f.id === id ? { ...f, favorite: !f.favorite, order: newOrder } : f
        ));

        try {
            await invoke('toggle_favorite_v2', { id, order: newOrder });
        } catch (err) {
            console.error(err);
            // Rollback on error
            setFolders(folders);
            message.error('Ошибка обновления избранного');
        }
    };

    const handleApply = async (newFolders: Folder[]) => {
        // Apply changes (reordering) — update local list for now
        setFolders(newFolders);
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
        <div>
            <StatusIndicator />
            <AddFolderButton onFolderAdded={handleAddFolder} />
            <FolderList
                folders={folders}
                onRename={handleRename}
                onToggleFavorite={handleToggleFavorite}
                onApply={handleApply}
            />
            <div className="action-bar">
                <button 
                    className="add-folder-btn"
                    onClick={handleRegisterComServer}
                >
                    Зарегистрировать COM
                </button>
                <button 
                    className="add-folder-btn"
                    onClick={handleUnregisterComServer}
                >
                    Удалить COM
                </button>
            </div>
        </div>
    );
};

export default EditorPage;
