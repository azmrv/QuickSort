import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ConfigProvider, theme, App as AntApp, Tabs } from 'antd';
import EditorPage from './pages/EditorPage';
import SelectorPage from './pages/SelectorPage';
import LogPage from './pages/LogPage';
import SettingsPage from './pages/SettingsPage';
import AboutPage from './pages/AboutPage';

function App() {
    const [mode, setMode] = useState<'editor' | 'selector'>('editor');
    const [selectFile, setSelectFile] = useState<string | null>(null);
    const [isDark, setIsDark] = useState(true);
    const [activeTab, setActiveTab] = useState('folders');

    useEffect(() => {
        invoke<string | null>('get_pending_file').then((file) => {
            if (file) {
                setSelectFile(file);
                setMode('selector');
            }
        });
    }, []);

    useEffect(() => {
        document.body.style.backgroundColor = isDark ? '#141414' : '#ffffff';
        document.body.style.color = isDark ? 'rgba(255,255,255,0.85)' : 'rgba(0,0,0,0.88)';
    }, [isDark]);

    return (
        <ConfigProvider
            theme={{
                algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
                token: { colorPrimary: '#1677ff' },
            }}
        >
            <AntApp>
                {mode === 'editor' ? (
                    <Tabs
                        activeKey={activeTab}
                        onChange={setActiveTab}
                        style={{ maxWidth: 800, margin: '0 auto', padding: 24 }}
                        items={[
                            { key: 'folders', label: 'Папки', children: <EditorPage isDark={isDark} onToggleTheme={setIsDark} /> },
                            { key: 'log', label: 'Лог', children: <LogPage /> },
                            { key: 'settings', label: 'Настройки', children: <SettingsPage /> },
                            { key: 'about', label: 'О программе', children: <AboutPage /> },
                        ]}
                    />
                ) : (
                    <SelectorPage file={selectFile} onClose={() => setMode('editor')} />
                )}
            </AntApp>
        </ConfigProvider>
    );
}

export default App;