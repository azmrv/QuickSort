import { useState, useEffect } from 'react';
import { App } from 'antd';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';
import { useTranslation } from '../i18n/useTranslation';
import { Folder, OperationCommand } from '../types';

interface SelectorPageProps {
    files: string[];
    onClose: () => void;
}

interface SelectorSettings {
    default_operation?: 'Move' | 'Copy';
    default_overwrite_policy?: 'Skip' | 'Overwrite' | 'AutoRename';
    duplicate_check?: { mode?: 'name' | 'size' | 'content' };
}

const SelectorPage: React.FC<SelectorPageProps> = ({ files, onClose }) => {
    const { t } = useTranslation();
    const [folders, setFolders] = useState<Folder[]>([]);
    const [search, setSearch] = useState('');
    const [showAddFolder, setShowAddFolder] = useState(false);
    const [newFolderName, setNewFolderName] = useState('');
    const [newFolderPath, setNewFolderPath] = useState('');
    const [operation, setOperation] = useState<'Move' | 'Copy'>('Move');
    const [overwritePolicy, setOverwritePolicy] = useState<'Skip' | 'Overwrite' | 'AutoRename'>('Skip');
    const [dupCheckMode, setDupCheckMode] = useState<'name' | 'size' | 'content'>('name');
    const { message } = App.useApp();

    const loadFolders = () => {
        invoke<Folder[]>('get_folders_v2')
            .then(folders => {
                setFolders(folders);
                logger.info('SelectorPage', `loaded ${folders.length} folders`);
            })
            .catch(err => {
                logger.error('SelectorPage', 'failed to load folders', err);
                message.error(`${t('selector.load_error')} ${err}`);
            });
    };

    useEffect(() => {
        logger.action('SelectorPage', `mount — loading folders for ${files.length} file(s)`);
        loadFolders();
        invoke<SelectorSettings>('get_settings')
            .then((settings) => {
                if (settings.default_operation) setOperation(settings.default_operation);
                if (settings.default_overwrite_policy) setOverwritePolicy(settings.default_overwrite_policy);
                if (settings.duplicate_check?.mode) setDupCheckMode(settings.duplicate_check.mode);
            })
            .catch((err) => logger.error('SelectorPage', 'failed to load settings', err));
    }, []);

    const filtered = folders.filter(
        (f) =>
            f.name.toLowerCase().includes(search.toLowerCase()) ||
            f.path.toLowerCase().includes(search.toLowerCase())
    );

    const favoriteFolders = filtered.filter(f => f.favorite);
    const otherFolders = filtered.filter(f => !f.favorite);

    const handleSelect = async (folder: Folder) => {
        if (files.length === 0) {
            message.error(t('selector.no_files'));
            return;
        }
        logger.action('SelectorPage', `queue ${files.length} file(s) -> "${folder.name}" (${folder.path})`);
        try {
            const command: OperationCommand = {
                operation_type: operation,
                source_paths: files,
                target_folder_id: folder.id,
                target_paths: null,
                overwrite_policy: overwritePolicy,
                duplicate_check_mode: dupCheckMode,
            };
            const jobId = await invoke<string>('enqueue_operation_v2', { command });
            logger.info('SelectorPage', 'job queued successfully', jobId);
            message.success(t('selector.queued', { count: files.length, folder: folder.name }));
            onClose();
        } catch (err) {
            logger.error('SelectorPage', 'queue files failed', err);
            message.error(`${t('selector.move_error')} ${err}`);
        }
    };

    const handleAddFolder = async () => {
        if (!newFolderName.trim() || !newFolderPath.trim()) {
            message.error(t('selector.fill_name_path'));
            return;
        }
        try {
            await invoke('add_folder_v2', { name: newFolderName.trim(), path: newFolderPath.trim() });
            message.success(t('selector.folder_added', { name: newFolderName }));
            setNewFolderName('');
            setNewFolderPath('');
            setShowAddFolder(false);
            loadFolders();
        } catch (err) {
            message.error(`${t('selector.move_error')} ${err}`);
        }
    };

    const handleBrowseFolder = async () => {
        const selected = await open({ directory: true });
        if (selected && typeof selected === 'string') {
            setNewFolderPath(selected);
            if (!newFolderName.trim()) {
                setNewFolderName(selected.split('\\').pop() || selected);
            }
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

    const optionLabelStyle: React.CSSProperties = {
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        fontFamily: 'var(--qs-font-body)',
        fontSize: '13px',
        color: 'var(--qs-text-secondary)',
    };

    const optionSelectStyle: React.CSSProperties = {
        background: 'var(--qs-bg-secondary)',
        border: '1px solid var(--qs-border)',
        borderRadius: 'var(--qs-radius-sm)',
        padding: '6px 10px',
        color: 'var(--qs-text-primary)',
        fontFamily: 'var(--qs-font-body)',
        fontSize: '13px',
        cursor: 'pointer',
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
                <div className="selector-title">
                    {files.length > 1 ? t('selector.title.move_many') : t('selector.title.move_one')}
                </div>
                <div className="selector-file">
                    {files.length === 0
                        ? t('selector.files_none')
                        : files.length === 1
                        ? files[0]
                        : t('selector.files_count', { count: files.length })}
                </div>
            </header>

            <div style={{
                display: 'flex',
                alignItems: 'center',
                gap: '16px',
                padding: '8px 16px',
                borderBottom: '1px solid var(--qs-border)',
                background: 'var(--qs-bg-tertiary)',
                flexWrap: 'wrap',
            }}>
                <label style={optionLabelStyle}>
                    <span>{t('selector.options.operation')}</span>
                    <select
                        value={operation}
                        onChange={(e) => setOperation(e.target.value as 'Move' | 'Copy')}
                        style={optionSelectStyle}
                    >
                        <option value="Move">{t('settings.default_actions.move')}</option>
                        <option value="Copy">{t('settings.default_actions.copy')}</option>
                    </select>
                </label>
                <label style={optionLabelStyle}>
                    <span>{t('selector.options.overwrite')}</span>
                    <select
                        value={overwritePolicy}
                        onChange={(e) => setOverwritePolicy(e.target.value as 'Skip' | 'Overwrite' | 'AutoRename')}
                        style={optionSelectStyle}
                    >
                        <option value="Skip">{t('settings.duplicates.skip')}</option>
                        <option value="Overwrite">{t('settings.duplicates.overwrite')}</option>
                        <option value="AutoRename">{t('settings.duplicates.auto_rename')}</option>
                    </select>
                </label>
                <label style={optionLabelStyle}>
                    <span>{t('selector.options.duplicate')}</span>
                    <select
                        value={dupCheckMode}
                        onChange={(e) => setDupCheckMode(e.target.value as 'name' | 'size' | 'content')}
                        style={optionSelectStyle}
                    >
                        <option value="name">{t('settings.duplicate_check.quick')}</option>
                        <option value="size">{t('settings.duplicate_check.medium')}</option>
                        <option value="content">{t('settings.duplicate_check.deep')}</option>
                    </select>
                </label>
            </div>

            <div className="selector-search">
                <input
                    type="text"
                    placeholder={t('selector.search_placeholder')}
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
                        <div className="empty-state-title">{t('selector.empty_title')}</div>
                        <div className="empty-state-description">
                            {search ? t('selector.empty_try') : t('selector.empty_add')}
                        </div>
                    </div>
                ) : (
                    <>
                        {favoriteFolders.length > 0 && (
                            <div className="selector-section">
                                <div className="selector-section-label">{t('selector.favorites')}</div>
                                {favoriteFolders.map((f, i) => renderFolderItem(f, i))}
                            </div>
                        )}
                        {otherFolders.length > 0 && (
                            <div className="selector-section">
                                {favoriteFolders.length > 0 ? (
                                    <div className="selector-section-label">{t('selector.other')}</div>
                                ) : (
                                    <div className="selector-section-label">{t('selector.all_folders')}</div>
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
                        placeholder={t('selector.add_folder_name')}
                        value={newFolderName}
                        onChange={(e) => setNewFolderName(e.target.value)}
                        style={inputStyle}
                    />
                    <div style={{ display: 'flex', gap: '8px' }}>
                        <input
                            type="text"
                            placeholder={t('selector.add_folder_path')}
                            value={newFolderPath}
                            onChange={(e) => setNewFolderPath(e.target.value)}
                            style={{ ...inputStyle, flex: 1 }}
                        />
                        <button
                            onClick={handleBrowseFolder}
                            style={{
                                padding: '8px 12px',
                                background: 'var(--qs-bg-secondary)',
                                border: '1px solid var(--qs-border)',
                                borderRadius: 'var(--qs-radius-sm)',
                                color: 'var(--qs-text-secondary)',
                                fontFamily: 'var(--qs-font-body)',
                                fontSize: '13px',
                                cursor: 'pointer',
                                whiteSpace: 'nowrap',
                            }}
                        >
                            {t('selector.browse')}
                        </button>
                    </div>
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
                        {t('selector.add_folder_confirm')}
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
                    {t('selector.add_folder')}
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
                    {t('selector.cancel')}
                </button>
            </div>
        </div>
    );
};

export default SelectorPage;
