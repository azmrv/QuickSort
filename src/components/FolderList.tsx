import { List, Button, Typography, Popconfirm, Input, Switch } from 'antd';
import { FolderOpenOutlined, DeleteOutlined, EditOutlined, StarOutlined } from '@ant-design/icons';
import { useState } from 'react';
import { Folder } from '../types';

const { Text } = Typography;

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
        return <div style={{ textAlign: 'center', padding: '24px', color: '#888' }}>Нет добавленных папок.</div>;
    }

    return (
        <List
            dataSource={folders}
            renderItem={(folder) => (
                <List.Item
                    actions={[
                        <Switch
                            checked={folder.is_favorite}
                            onChange={() => onToggleFavorite(folder.id)}
                            checkedChildren={<StarOutlined />}
                            unCheckedChildren={<StarOutlined />}
                        />,
                        editingId === folder.id ? (
                            <Button type="link" onClick={confirmEdit} icon={<EditOutlined />}>Сохранить</Button>
                        ) : (
                            <Button type="text" icon={<EditOutlined />} onClick={() => startEdit(folder.id, folder.name)} />
                        ),
                        <Popconfirm title="Удалить папку?" onConfirm={() => handleRemove(folder.id)} okText="Да" cancelText="Нет">
                            <Button type="text" danger icon={<DeleteOutlined />} />
                        </Popconfirm>,
                    ]}
                >
                    <List.Item.Meta
                        avatar={<FolderOpenOutlined style={{ fontSize: '24px', color: '#faad14' }} />}
                        title={
                            editingId === folder.id ? (
                                <Input value={editValue} onChange={e => setEditValue(e.target.value)} onPressEnter={confirmEdit} onBlur={confirmEdit} autoFocus />
                            ) : (
                                <Text strong>{folder.name}</Text>
                            )
                        }
                        description={<Text type="secondary">{folder.path}</Text>}
                    />
                </List.Item>
            )}
        />
    );
};

export default FolderList;