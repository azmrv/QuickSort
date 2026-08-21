import { useState, useEffect } from 'react';
import { App } from 'antd';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';
import FolderList from '../components/FolderList';
import AddFolderButton from '../components/AddFolderButton';
import StatusIndicator from '../components/StatusIndicator';
import { Folder } from '../types';

const EditorPage: React.FC = () => {
    const [folders, setFolders] = useState<Folder[]>([]);
    const { message } = App.useApp();

    useEffect(() => {
        logger.action('EditorPage', 'mount — loading folders');
        invoke<Folder[]>('get_folders_v2')
            .then(folders => {
                setFolders(folders);
                logger.info('EditorPage', `loaded ${folders.length} folders`);
            })
            .catch(err => {
                logger.error('EditorPage', 'failed to load folders', err);
                message.error(`Ошибка загрузки: ${err}`);
            });
    }, []);

    const handleAddFolder = (name: string, path: string) => {
        logger.action('EditorPage', `add folder: ${name} → ${path}`);
        invoke('add_folder_v2', { name, path })
            .then(() => {
                logger.info('EditorPage', 'folder added, reloading list');
                return invoke<Folder[]>('get_folders_v2');
            })
            .then(folders => {
                setFolders(folders);
                logger.info('EditorPage', `reloaded ${folders.length} folders`);
            })
            .catch(err => {
                logger.error('EditorPage', 'add folder failed', err);
                message.error(`Ошибка добавления: ${err}`);
            });
    };

    const handleRename = (id: string, newName: string) => {
        logger.action('EditorPage', `rename folder ${id} → "${newName}"`);
        setFolders(folders.map((f) => (f.id === id ? { ...f, name: newName } : f)));
    };

    const handleToggleFavorite = async (id: string) => {
        const folder = folders.find(f => f.id === id);
        if (!folder) return;
        const newOrder = folder.favorite ? 0 : folders.filter(f => f.favorite).length + 1;
        logger.action('EditorPage', `toggle favorite: ${id} (favorite=${!folder.favorite}, order=${newOrder})`);

        setFolders(folders.map((f) =>
            f.id === id ? { ...f, favorite: !f.favorite, order: newOrder } : f
        ));

        try {
            await invoke('toggle_favorite_v2', { id, order: newOrder });
            logger.info('EditorPage', `favorite toggled: ${id}`);
        } catch (err) {
            logger.error('EditorPage', 'toggle favorite failed', err);
            setFolders(folders);
            message.error('Ошибка обновления избранного');
        }
    };

    const handleApply = async (newFolders: Folder[]) => {
        setFolders(newFolders);
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
        </div>
    );
};

export default EditorPage;
