import { Button } from 'antd';
import { FolderAddOutlined } from '@ant-design/icons';
import { open } from '@tauri-apps/plugin-dialog';

interface AddFolderButtonProps {
    onFolderAdded: (name: string, path: string) => void;
}

const AddFolderButton: React.FC<AddFolderButtonProps> = ({ onFolderAdded }) => {
    const handleClick = async () => {
        const selected = await open({ directory: true });
        if (selected && typeof selected === 'string') {
            const name = selected.split('\\').pop() || selected;
            onFolderAdded(name, selected);
        }
    };

    return (
        <Button type="dashed" icon={<FolderAddOutlined />} onClick={handleClick} block>
            Добавить папку
        </Button>
    );
};

export default AddFolderButton;