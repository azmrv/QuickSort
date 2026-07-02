import { useState, useEffect } from 'react';
import { ConfigProvider, theme, App as AntApp, Button } from 'antd';
import EditorPage from './pages/EditorPage';
import SelectorPage from './pages/SelectorPage';

function App() {
    const [mode, setMode] = useState<'editor' | 'selector'>('editor');
    const [isDark, setIsDark] = useState(true);

    useEffect(() => {
        document.body.style.backgroundColor = isDark ? '#141414' : '#ffffff';
        document.body.style.color = isDark ? 'rgba(255,255,255,0.85)' : 'rgba(0,0,0,0.88)';
    }, [isDark]);

    return (
        <ConfigProvider theme={{
            algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
            token: { colorPrimary: '#1677ff' }
        }}>
            <AntApp>
                <div style={{ position: 'fixed', top: 10, right: 10, zIndex: 1000 }}>
                    <Button onClick={() => setMode(m => m === 'editor' ? 'selector' : 'editor')}>
                        {mode === 'editor' ? 'Показать Selector' : 'Показать Editor'}
                    </Button>
                </div>
                {mode === 'editor' ? (
                    <EditorPage isDark={isDark} onToggleTheme={setIsDark} />
                ) : (
                    <SelectorPage file={null} />
                )}
            </AntApp>
        </ConfigProvider>
    );
}

export default App;