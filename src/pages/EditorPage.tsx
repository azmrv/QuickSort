import { useState, useEffect } from 'react';
import { App } from 'antd';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';
import { useTranslation } from '../i18n/useTranslation';
import FolderList from '../components/FolderList';
import AddFolderButton from '../components/AddFolderButton';
import StatusIndicator from '../components/StatusIndicator';
import { Folder } from '../types';

const AUTO_COLORS = [
    '#ef4444', '#f97316', '#eab308', '#22c55e', '#06b6d4',
    '#3b82f6', '#8b5cf6', '#ec4899', '#f43f5e', '#14b8a6',
    '#a855f7', '#6366f1', '#0ea5e9', '#10b981', '#84cc16',
];

function assignAutoColors(folders: Folder[]): Folder[] {
    let colorIdx = 0;
    return folders.map(f => {
        if (!f.color) {
            const color = AUTO_COLORS[colorIdx % AUTO_COLORS.length];
            colorIdx++;
            return { ...f, color };
        }
        return f;
    });
}

const EditorPage: React.FC = () => {
    const { t } = useTranslation();
    const [folders, setFolders] = useState<Folder[]>([]);
    const { message } = App.useApp();

    useEffect(() => {
        logger.action('EditorPage', 'mount — loading folders');
        invoke<Folder[]>('get_folders_v2')
            .then(rawFolders => {
                // Auto-assign colors to folders that don't have one
                const folders = assignAutoColors(rawFolders);
                setFolders(folders);
                logger.info('EditorPage', `loaded ${folders.length} folders`);

                // Save auto-assigned colors back to backend
                const needsColor = rawFolders.filter(f => !f.color);
                if (needsColor.length > 0) {
                    Promise.all(
                        needsColor.map(f => {
                            const updated = folders.find(uf => uf.id === f.id);
                            return updated?.color
                                ? invoke('set_folder_color_v2', { id: f.id, color: updated.color })
                                : Promise.resolve();
                        })
                    ).then(() => {
                        logger.info('EditorPage', `auto-assigned colors to ${needsColor.length} folders`);
                    }).catch(err => {
                        logger.error('EditorPage', 'failed to save auto-colors', err);
                    });
                }
            })
            .catch(err => {
                logger.error('EditorPage', 'failed to load folders', err);
                message.error(`${t('editor.load_error')} ${err}`);
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
                message.error(`${t('editor.add_error')} ${err}`);
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
            message.error(t('editor.favorite_error'));
        }
    };

    const handleSetColor = async (id: string, color: string | null) => {
        logger.action('EditorPage', `set color: ${id} → ${color ?? 'none'}`);
        setFolders(folders.map((f) => (f.id === id ? { ...f, color } : f)));

        try {
            await invoke('set_folder_color_v2', { id, color });
            logger.info('EditorPage', `color saved: ${id}`);
        } catch (err) {
            logger.error('EditorPage', 'set color failed', err);
            setFolders(folders);
            message.error(t('editor.color_error'));
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
                onSetColor={handleSetColor}
                onApply={handleApply}
            />
        </div>
    );
};

export default EditorPage;
