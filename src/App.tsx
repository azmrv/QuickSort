import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ConfigProvider, Typography, theme, Switch, Space, App as AntApp, Button, Popconfirm } from 'antd';
import { BulbOutlined, DeleteOutlined } from '@ant-design/icons';
import FolderList from './components/FolderList';
import AddFolderButton from './components/AddFolderButton';
import ApplyButton from './components/ApplyButton';
import StatusIndicator from './components/StatusIndicator';
import { TargetFolder } from './types';

const { Title } = Typography;

function App() {
    const [folders, setFolders] = useState<TargetFolder[]>([]);
    const [nextId, setNextId] = useState(1);
    const [isDark, setIsDark] = useState(true);

    useEffect(() => {
        invoke<TargetFolder[]>('get_folders').then(setFolders).catch(console.error);
    }, []);

    useEffect(() => {
        document.body.style.backgroundColor = isDark ? '#141414' : '#ffffff';
        document.body.style.color = isDark ? 'rgba(255,255,255,0.85)' : 'rgba(0,0,0,0.88)';
    }, [isDark]);

    const handleAddFolder = (name: string, path: string) => {
        const newFolder: TargetFolder = { id: `folder_${nextId}`, name, path };
        setFolders([...folders, newFolder]);
        setNextId(nextId + 1);
    };

    const handleRemoveFolder = (id: string) => {
        setFolders(folders.filter((f) => f.id !== id));
    };

    const handleRenameFolder = (id: string, newName: string) => {
        setFolders(folders.map((f) => (f.id === id ? { ...f, name: newName } : f)));
    };

    const handleDeleteMenu = async () => {
        try {
            await invoke('update_folders', { folders: [] });
            setFolders([]);
            // Обновляем статус меню (оно станет неактивным)
        } catch (err) {
            console.error(err);
        }
    };

    return (
        <ConfigProvider
            theme={{
                algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
                token: { colorPrimary: '#1677ff' },
            }}
        >
            <AntApp>
                <div style={{ maxWidth: 640, margin: '0 auto', padding: 24, minHeight: '100vh' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 24 }}>
                        <Title level={3} style={{ margin: 0 }}>QuickSort</Title>
                        <Space>
                            <BulbOutlined />
                            <Switch checked={isDark} onChange={setIsDark} checkedChildren="🌙" unCheckedChildren="☀️" />
                        </Space>
                    </div>
                    <StatusIndicator />
                    <AddFolderButton onFolderAdded={handleAddFolder} />
                    <FolderList folders={folders} onRemove={handleRemoveFolder} onRename={handleRenameFolder} />
                    <div style={{ marginTop: 16 }}>
                        <ApplyButton folders={folders} />
                    </div>
                    <div style={{ marginTop: 8 }}>
                        <Popconfirm
                            title="Удалить все пункты контекстного меню?"
                            onConfirm={handleDeleteMenu}
                            okText="Да"
                            cancelText="Нет"
                        >
                            <Button danger icon={<DeleteOutlined />} block disabled={folders.length === 0}>
                                Удалить меню
                            </Button>
                        </Popconfirm>
                    </div>
                </div>
            </AntApp>
        </ConfigProvider>
    );
}

export default App;