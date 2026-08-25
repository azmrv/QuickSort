import { useState, useEffect } from 'react';
import { App } from 'antd';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';
import { Folder, OperationCommand } from '../types';

interface SelectorPageProps {
    file: string | null;
    onClose: () => void;
}

const SelectorPage: React.FC<SelectorPageProps> = ({ file, onClose }) => {
    const [folders, setFolders] = useState<Folder[]>([]);
    const [search, setSearch] = useState('');
    const [showAddFolder, setShowAddFolder] = useState(false);
    const [newFolderName, setNewFolderName] = useState('');
    const [newFolderPath, setNewFolderPath] = useState('');
    const { message } = App.useApp();

    const loadFolders = () => {
        invoke<Folder[]>('get_folders_v2')
            .then(folders => {
                setFolders(folders);
                logger.info('SelectorPage', `loaded ${folders.length} folders`);
            })
            .catch(err => {
                logger.error('SelectorPage', 'failed to load folders', err);
                message.error(`Ошибка загрузки: ${err}`);
            });
    };

    useEffect(() => {
        logger.action('SelectorPage', `mount — loading folders for file: ${file}`);
        loadFolders();
    }, []);

    const filtered = folders.filter(
        (f) =>
            f.name.toLowerCase().includes(search.toLowerCase()) ||
            f.path.toLowerCase().includes(search.toLowerCase())
    );

    const favoriteFolders = filtered.filter(f => f.favorite);
    const otherFolders = filtered.filter(f => !f.favorite);

    const handleSelect = async (folder: Folder) => {
        if (!file) {
            message.error('Нет файла для перемещения');
            return;
        }
        logger.action('SelectorPage', `move file "${file}" -> "${folder.name}" (${folder.path})`);
        try {
            const command: OperationCommand = {
                operation_type: 'Move',
                source_paths: [file],
                target_folder_id: folder.id,
                target_paths: null,
                overwrite_policy: 'Skip',
            };
            const result = await invoke('execute_operation_v2', { command });
            logger.info('SelectorPage', 'file moved successfully', result);
            message.success(`Файл перемещён в ${folder.name}`);
            onClose();
        } catch (err) {
            logger.error('SelectorPage', 'move file failed', err);
            message.error(`Ошибка: ${err}`);
        }
    };

    const handleAddFolder = async () => {
        if (!newFolderName.trim() || !newFolderPath.trim()) {
            message.error('Заполните имя и путь');
            return;
        }
        try {
            await invoke('add_folder_v2', { name: newFolderName.trim(), path: newFolderPath.trim() });
            message.success(`Папка "${newFolderName}" добавлена`);
            setNewFolderName('');
            setNewFolderPath('');
            setShowAddFolder(false);
            loadFolders();
        } catch (err) {
            message.error(`Ошибка: ${err}`);
        }
    };

    const inputStyle = {
        width: '100%',
        background: 'var(--qs-bg-secondary)',
        border: '1px solid var(--qs-border)',
        borderRadius: 'var(--qs-radius-sm)',
        padding: '8px 12px',
        color: 'var(--qs-text-primary)',
        fontFamily: 'var(--qs-font-body)',
        fontSize: '13px',
        outline: 'none',
    };

    const renderFolderItem = (folder: Folder, index: number) => (
        <div
            key={folder.id}
            className="selector-item"
            onClick={() => handleSelect(folder)}
            style={{ animationDelay: `${index * 30}ms` }}
        >
            <div className="selector-item-icon">
                {folder.favorite ? '\u2605' : '\uD83D\uDCC1'}
            </div>
            <div className="selector-item-info">
                <div className="selector-item-name">{folder.name}</div>
                <div className="selector-item-path">{folder.path}</div>
            </div>
            <div className="selector-item-shortcut">Enter</div>
        </div>
    );

    return (
        <div className="selector-container">
            <header className="selector-header">
                <div className="selector-title">Переместить файл в:</div>
                <div className="selector-file">{file || 'Файл не выбран'}</div>
            </header>

            <div className="selector-search">
                <input
                    type="text"
                    placeholder="Поиск папки..."
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                    style={{
                        width: '100%',
                        background: 'var(--qs-bg-tertiary)',
                        border: '1px solid var(--qs-border)',
                        borderRadius: 'var(--qs-radius-md)',
                        padding: '12px 16px',
                        color: 'var(--qs-text-primary)',
                        fontFamily: 'var(--qs-font-body)',
                        fontSize: '14px',
                        outline: 'none',
                    }}
                    onFocus={(e) => {
                        e.target.style.borderColor = 'var(--qs-accent)';
                        e.target.style.boxShadow = '0 0 0 2px var(--qs-accent-muted)';
                    }}
                    onBlur={(e) => {
                        e.target.style.borderColor = 'var(--qs-border)';
                        e.target.style.boxShadow = 'none';
                    }}
                />
            </div>

            <div className="selector-list">
                {filtered.length === 0 ? (
                    <div className="empty-state">
                        <div className="empty-state-icon">{'\uD83D\uDD0D'}</div>
                        <div className="empty-state-title">Папки не найдены</div>
                        <div className="empty-state-description">
                            {search ? 'Попробуйте другой запрос' : 'Добавьте папки в настройках'}
                        </div>
                    </div>
                ) : (
                    <>
                        {favoriteFolders.length > 0 && (
                            <div className="selector-section">
                                <div className="selector-section-label">Избранные</div>
                                {favoriteFolders.map((f, i) => renderFolderItem(f, i))}
                            </div>
                        )}
                        {otherFolders.length > 0 && (
                            <div className="selector-section">
                                {favoriteFolders.length > 0 ? (
                                    <div className="selector-section-label">Остальные</div>
                                ) : (
                                    <div className="selector-section-label">Все папки</div>
                                )}
                                {otherFolders.map((f, i) => renderFolderItem(f, i + favoriteFolders.length))}
                            </div>
                        )}
                    </>
                )}
            </div>

            {showAddFolder && (
                <div style={{
                    padding: '12px 16px',
                    borderTop: '1px solid var(--qs-border)',
                    background: 'var(--qs-bg-tertiary)',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '8px',
                }}>
                    <input
                        type="text"
                        placeholder="Имя папки"
                        value={newFolderName}
                        onChange={(e) => setNewFolderName(e.target.value)}
                        style={inputStyle}
                    />
                    <input
                        type="text"
                        placeholder="Путь (C:\Users\...)"
                        value={newFolderPath}
                        onChange={(e) => setNewFolderPath(e.target.value)}
                        style={inputStyle}
                    />
                    <button
                        onClick={handleAddFolder}
                        style={{
                            padding: '8px',
                            background: 'var(--qs-accent)',
                            border: 'none',
                            borderRadius: 'var(--qs-radius-sm)',
                            color: 'var(--qs-bg-primary)',
                            fontFamily: 'var(--qs-font-body)',
                            fontSize: '13px',
                            fontWeight: 600,
                            cursor: 'pointer',
                        }}
                    >
                        Добавить
                    </button>
                </div>
            )}

            <div style={{
                padding: 'var(--qs-space-md) var(--qs-space-xl)',
                borderTop: '1px solid var(--qs-border)',
                background: 'var(--qs-bg-secondary)',
                display: 'flex',
                gap: '8px',
            }}>
                <button
                    onClick={() => setShowAddFolder(!showAddFolder)}
                    style={{
                        flex: 1,
                        padding: '12px',
                        background: showAddFolder ? 'var(--qs-accent-muted)' : 'var(--qs-bg-tertiary)',
                        border: '1px solid var(--qs-border)',
                        borderRadius: 'var(--qs-radius-md)',
                        color: showAddFolder ? 'var(--qs-accent)' : 'var(--qs-text-secondary)',
                        fontFamily: 'var(--qs-font-body)',
                        fontSize: '14px',
                        fontWeight: 500,
                        cursor: 'pointer',
                        transition: 'all var(--qs-transition-fast)',
                    }}
                >
                    + Добавить папку
                </button>
                <button
                    onClick={onClose}
                    style={{
                        flex: 1,
                        padding: '12px',
                        background: 'var(--qs-bg-tertiary)',
                        border: '1px solid var(--qs-border)',
                        borderRadius: 'var(--qs-radius-md)',
                        color: 'var(--qs-text-secondary)',
                        fontFamily: 'var(--qs-font-body)',
                        fontSize: '14px',
                        fontWeight: 500,
                        cursor: 'pointer',
                        transition: 'all var(--qs-transition-fast)',
                    }}
                >
                    Отмена (Esc)
                </button>
            </div>
        </div>
    );
};

export default SelectorPage;
