import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Input, List, Typography, message } from 'antd';
import { FolderOpenOutlined } from '@ant-design/icons';
import { Folder } from '../types';

const { Text } = Typography;

interface SelectorPageProps {
    file: string | null;
}

const SelectorPage: React.FC<SelectorPageProps> = ({ file }) => {
    const [folders, setFolders] = useState<Folder[]>([]);
    const [search, setSearch] = useState('');

    useEffect(() => {
        invoke<Folder[]>('get_folders').then(setFolders).catch(console.error);
    }, []);

    const filtered = folders.filter(f =>
        f.name.toLowerCase().includes(search.toLowerCase()) ||
        f.path.toLowerCase().includes(search.toLowerCase())
    );

    const handleSelect = async (folder: Folder) => {
        if (!file) {
            message.error('Нет файла для перемещения');
            return;
        }
        try {
            await invoke('move_file', { src: file, destDir: folder.path });
            message.success(`Файл перемещён в ${folder.name}`);
            // В реальном приложении здесь нужно закрыть окно
        } catch (err) {
            message.error(`Ошибка: ${err}`);
        }
    };

    return (
        <div style={{ maxWidth: 500, margin: '20px auto', padding: 16 }}>
            <Text strong>Переместить: {file || 'файл не выбран'}</Text>
            <Input
                placeholder="Поиск папки..."
                value={search}
                onChange={e => setSearch(e.target.value)}
                style={{ margin: '16px 0' }}
            />
            <List
                dataSource={filtered}
                renderItem={folder => (
                    <List.Item onClick={() => handleSelect(folder)} style={{ cursor: 'pointer' }}>
                        <List.Item.Meta
                            avatar={<FolderOpenOutlined style={{ fontSize: '24px', color: '#faad14' }} />}
                            title={folder.name}
                            description={folder.path}
                        />
                    </List.Item>
                )}
            />
        </div>
    );
};

export default SelectorPage;