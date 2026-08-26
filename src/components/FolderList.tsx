import { useState } from 'react';
import { Folder } from '../types';
import { useTranslation } from '../i18n/useTranslation';

interface FolderListProps {
    folders: Folder[];
    onRename: (id: string, newName: string) => void;
    onToggleFavorite: (id: string) => void;
    onSetColor: (id: string, color: string | null) => void;
    onApply: (folders: Folder[]) => void;
}

const DEFAULT_COLOR = '#4a9eff';

const FolderList: React.FC<FolderListProps> = ({ folders, onRename, onToggleFavorite, onSetColor, onApply }) => {
    const { t } = useTranslation();
    const [editingId, setEditingId] = useState<string | null>(null);
    const [editValue, setEditValue] = useState('');

    const startEdit = (id: string, currentName: string) => {
        setEditingId(id);
        setEditValue(currentName);
    };

    const confirmEdit = () => {
        if (editingId && editValue.trim()) {
            onRename(editingId, editValue.trim());
        }
        setEditingId(null);
    };

    const handleRemove = (id: string) => {
        const updated = folders.filter(f => f.id !== id);
        onApply(updated);
    };

    if (folders.length === 0) {
        return (
            <div className="empty-state">
                <div className="empty-state-icon">📁</div>
                <div className="empty-state-title">{t('folder_list.empty_title')}</div>
                <div className="empty-state-description">
                    {t('folder_list.empty_description')}
                </div>
            </div>
        );
    }

    return (
        <div className="folder-list">
            {folders.map((folder, index) => (
                <div
                    key={folder.id}
                    className={`folder-card ${folder.favorite ? 'favorite' : ''}`}
                    style={{
                        animationDelay: `${index * 50}ms`,
                        ...(folder.color ? {
                            borderLeft: `3px solid ${folder.color}`,
                            background: `color-mix(in srgb, ${folder.color} 8%, transparent)`,
                        } : {}),
                    }}
                >
                    <div className="folder-icon">📁</div>
                    <div className="folder-info">
                        {editingId === folder.id ? (
                            <input
                                type="text"
                                value={editValue}
                                onChange={e => setEditValue(e.target.value)}
                                onKeyDown={e => e.key === 'Enter' && confirmEdit()}
                                onBlur={confirmEdit}
                                autoFocus
                                style={{
                                    background: 'var(--qs-bg-tertiary)',
                                    border: '1px solid var(--qs-accent)',
                                    borderRadius: 'var(--qs-radius-sm)',
                                    padding: '4px 8px',
                                    color: 'var(--qs-text-primary)',
                                    fontFamily: 'var(--qs-font-body)',
                                    fontSize: '14px',
                                    fontWeight: 500,
                                    width: '100%',
                                    outline: 'none',
                                }}
                            />
                        ) : (
                            <div className="folder-name">{folder.name}</div>
                        )}
                        <div className="folder-path">{folder.path}</div>
                    </div>
                    <div className="folder-actions">
                        <button
                            className={`folder-action-btn star ${folder.favorite ? 'active' : ''}`}
                            onClick={() => onToggleFavorite(folder.id)}
                            title={folder.favorite ? t('folder_list.remove_favorite') : t('folder_list.add_favorite')}
                        >
                            {folder.favorite ? '★' : '☆'}
                        </button>
                        <label
                            className={`folder-action-btn color ${folder.color ? '' : 'empty'}`}
                            title={folder.color ? t('folder_list.color', { color: folder.color }) : t('folder_list.set_color')}
                            onContextMenu={(e) => {
                                e.preventDefault();
                                if (folder.color) {
                                    onSetColor(folder.id, null);
                                }
                            }}
                        >
                            <span
                                className="color-dot"
                                style={{ background: folder.color ?? 'transparent' }}
                            />
                            <input
                                type="color"
                                value={folder.color ?? DEFAULT_COLOR}
                                onChange={(e) => onSetColor(folder.id, e.target.value.toUpperCase())}
                            />
                        </label>
                        {editingId === folder.id ? (
                            <button
                                className="folder-action-btn"
                                onClick={confirmEdit}
                                title={t('folder_list.save')}
                            >
                                ✓
                            </button>
                        ) : (
                            <button
                                className="folder-action-btn"
                                onClick={() => startEdit(folder.id, folder.name)}
                                title={t('folder_list.rename')}
                            >
                                ✎
                            </button>
                        )}
                        <button
                            className="folder-action-btn danger"
                            onClick={() => handleRemove(folder.id)}
                            title={t('folder_list.delete')}
                        >
                            ×
                        </button>
                    </div>
                </div>
            ))}
        </div>
    );
};

export default FolderList;
