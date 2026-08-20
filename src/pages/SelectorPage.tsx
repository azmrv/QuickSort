import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { message } from 'antd';
import { Folder, OperationCommand } from '../types';

interface SelectorPageProps {
    file: string | null;
    onClose: () => void;
}

const SelectorPage: React.FC<SelectorPageProps> = ({ file, onClose }) => {
    const [folders, setFolders] = useState<Folder[]>([]);
    const [search, setSearch] = useState('');

    useEffect(() => {
        invoke<Folder[]>('get_folders_v2')
            .then(setFolders)
            .catch(console.error);
    }, []);

    const filtered = folders.filter(
        (f) =>
            f.name.toLowerCase().includes(search.toLowerCase()) ||
            f.path.toLowerCase().includes(search.toLowerCase())
    );

    const handleSelect = async (folder: Folder) => {
        if (!file) {
            message.error('Нет файла для перемещения');
            return;
        }
        try {
            // Form the Move command
            const command: OperationCommand = {
                operation_type: 'Move',
                source_paths: [file],
                target_folder_id: folder.id,
                target_paths: null,
                overwrite_policy: 'Skip',
            };
            await invoke('execute_operation_v2', { command });
            message.success(`Файл перемещён в ${folder.name}`);
            onClose(); // Return to editor
        } catch (err) {
            message.error(`Ошибка: ${err}`);
        }
    };

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
                        <div className="empty-state-icon">🔍</div>
                        <div className="empty-state-title">Папки не найдены</div>
                        <div className="empty-state-description">
                            {search ? 'Попробуйте другой запрос' : 'Добавьте папки в настройках'}
                        </div>
                    </div>
                ) : (
                    filtered.map((folder, index) => (
                        <div
                            key={folder.id}
                            className="selector-item"
                            onClick={() => handleSelect(folder)}
                            style={{ animationDelay: `${index * 30}ms` }}
                        >
                            <div className="selector-item-icon">📁</div>
                            <div className="selector-item-info">
                                <div className="selector-item-name">{folder.name}</div>
                                <div className="selector-item-path">{folder.path}</div>
                            </div>
                            <div className="selector-item-shortcut">Enter</div>
                        </div>
                    ))
                )}
            </div>

            <div style={{ 
                padding: 'var(--qs-space-md) var(--qs-space-xl)',
                borderTop: '1px solid var(--qs-border)',
                background: 'var(--qs-bg-secondary)',
            }}>
                <button
                    onClick={onClose}
                    style={{
                        width: '100%',
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
                    onMouseEnter={(e) => {
                        e.currentTarget.style.background = 'var(--qs-bg-hover)';
                        e.currentTarget.style.borderColor = 'var(--qs-border-hover)';
                        e.currentTarget.style.color = 'var(--qs-text-primary)';
                    }}
                    onMouseLeave={(e) => {
                        e.currentTarget.style.background = 'var(--qs-bg-tertiary)';
                        e.currentTarget.style.borderColor = 'var(--qs-border)';
                        e.currentTarget.style.color = 'var(--qs-text-secondary)';
                    }}
                >
                    Отмена (Esc)
                </button>
            </div>
        </div>
    );
};

export default SelectorPage;
