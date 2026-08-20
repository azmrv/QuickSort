import { useState } from 'react';
import { Folder } from '../types';

interface FolderListProps {
    folders: Folder[];
    onRename: (id: string, newName: string) => void;
    onToggleFavorite: (id: string) => void;
    onApply: (folders: Folder[]) => void;
}

const FolderList: React.FC<FolderListProps> = ({ folders, onRename, onToggleFavorite, onApply }) => {
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
                <div className="empty-state-title">Нет добавленных папок</div>
                <div className="empty-state-description">
                    Нажмите "Добавить папку" чтобы начать
                </div>
            </div>
        );
    }

    return (
        <div className="folder-list">
            {folders.map((folder, index) => (
                <div 
                    key={folder.id} 
                    className={`folder-card ${folder.is_favorite ? 'favorite' : ''}`}
                    style={{ animationDelay: `${index * 50}ms` }}
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
                            className={`folder-action-btn star ${folder.is_favorite ? 'active' : ''}`}
                            onClick={() => onToggleFavorite(folder.id)}
                            title={folder.is_favorite ? 'Убрать из избранного' : 'Добавить в избранное'}
                        >
                            {folder.is_favorite ? '★' : '☆'}
                        </button>
                        {editingId === folder.id ? (
                            <button
                                className="folder-action-btn"
                                onClick={confirmEdit}
                                title="Сохранить"
                            >
                                ✓
                            </button>
                        ) : (
                            <button
                                className="folder-action-btn"
                                onClick={() => startEdit(folder.id, folder.name)}
                                title="Переименовать"
                            >
                                ✎
                            </button>
                        )}
                        <button
                            className="folder-action-btn danger"
                            onClick={() => handleRemove(folder.id)}
                            title="Удалить"
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
