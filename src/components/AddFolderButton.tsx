/**
 * "Add folder" trigger: opens a native directory picker, then shows a modal to
 * confirm the name and optionally choose a parent folder (0.2.6 feature 4f).
 */

import { useState } from 'react';
import { Input, Modal, Select } from 'antd';
import { open } from '@tauri-apps/plugin-dialog';
import { useTranslation } from '../i18n/useTranslation';
import { Folder } from '../types';
import { availableParents } from '../lib/folderTree';

interface AddFolderButtonProps {
    folders: Folder[];
    onFolderAdded: (name: string, path: string, parentId: string | null) => void;
}

const AddFolderButton: React.FC<AddFolderButtonProps> = ({ folders, onFolderAdded }) => {
    const { t } = useTranslation();
    const [modalOpen, setModalOpen] = useState(false);
    const [selectedPath, setSelectedPath] = useState('');
    const [name, setName] = useState('');
    const [parentId, setParentId] = useState<string | null>(null);

    const handleClick = async () => {
        const selected = await open({ directory: true });
        if (selected && typeof selected === 'string') {
            setSelectedPath(selected);
            setName(selected.split('\\').pop() || selected);
            setParentId(null);
            setModalOpen(true);
        }
    };

    const handleConfirm = () => {
        const trimmed = name.trim();
        if (!trimmed) return;
        onFolderAdded(trimmed, selectedPath, parentId);
        setModalOpen(false);
    };

    const parentOptions = availableParents(folders).map(f => ({
        value: f.id,
        label: f.name,
    }));

    return (
        <>
            <button className="add-folder-btn" onClick={handleClick}>
                <span className="add-folder-btn-icon">+</span>
                {t('add_folder_button')}
            </button>
            <Modal
                title={t('add_folder.title')}
                open={modalOpen}
                onOk={handleConfirm}
                onCancel={() => setModalOpen(false)}
                okText={t('add_folder.confirm')}
                cancelText={t('add_folder.cancel')}
                okButtonProps={{ disabled: !name.trim() }}
                width={480}
            >
                <div style={{ display: 'flex', flexDirection: 'column', gap: 12, marginTop: 12 }}>
                    <label style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                        <span>{t('selector.add_folder_name')}</span>
                        <Input value={name} onChange={e => setName(e.target.value)} />
                    </label>
                    <label style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                        <span>{t('selector.add_folder_path')}</span>
                        <Input value={selectedPath} readOnly />
                    </label>
                    <label style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                        <span>{t('add_folder.parent')}</span>
                        <Select
                            value={parentId ?? ''}
                            onChange={setParentId}
                            options={[
                                { value: '', label: t('add_folder.parent_none') },
                                ...parentOptions,
                            ]}
                        />
                    </label>
                </div>
            </Modal>
        </>
    );
};

export default AddFolderButton;