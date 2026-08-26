import { open } from '@tauri-apps/plugin-dialog';
import { useTranslation } from '../i18n/useTranslation';

interface AddFolderButtonProps {
    onFolderAdded: (name: string, path: string) => void;
}

const AddFolderButton: React.FC<AddFolderButtonProps> = ({ onFolderAdded }) => {
    const { t } = useTranslation();

    const handleClick = async () => {
        const selected = await open({ directory: true });
        if (selected && typeof selected === 'string') {
            const name = selected.split('\\').pop() || selected;
            onFolderAdded(name, selected);
        }
    };

    return (
        <button className="add-folder-btn" onClick={handleClick}>
            <span className="add-folder-btn-icon">+</span>
            {t('add_folder_button')}
        </button>
    );
};

export default AddFolderButton;
