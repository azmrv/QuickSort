import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ConfigProvider, theme, App as AntApp } from 'antd';
import EditorPage from './pages/EditorPage';
import SelectorPage from './pages/SelectorPage';

function App() {
    const [mode, setMode] = useState<'editor' | 'selector'>('editor');
    const [selectFile, setSelectFile] = useState<string | null>(null);
    const [isDark, setIsDark] = useState(true);

    useEffect(() => {
        invoke<string | null>('get_pending_file')
            .then((file) => {
                if (file) {
                    setSelectFile(file);
                    setMode('selector');
                }
            })
            .catch(console.error);
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
                    <EditorPage isDark={isDark} onToggleTheme={setIsDark} />
                ) : (
                    <SelectorPage
                        file={selectFile}
                        onClose={() => {
                            setMode('editor');
                            setSelectFile(null);
                        }}
                    />
                )}
            </AntApp>
        </ConfigProvider>
    );
}

export default App;