import { useState, useEffect } from 'react';
import { invoke } from './lib/invoke';
import { logger } from './lib/logger';
import { ConfigProvider, theme, App as AntApp, Tabs } from 'antd';
import EditorPage from './pages/EditorPage';
import SelectorPage from './pages/SelectorPage';
import LogPage from './pages/LogPage';
import SettingsPage from './pages/SettingsPage';
import AboutPage from './pages/AboutPage';
import './styles/App.css';

function App() {
    const [mode, setMode] = useState<'editor' | 'selector'>('editor');
    const [selectFile, setSelectFile] = useState<string | null>(null);
    const [isDark, setIsDark] = useState(true);
    const [activeTab, setActiveTab] = useState('folders');

    useEffect(() => {
        logger.info('App', 'startup — checking pending file');
        invoke<string | null>('get_pending_file').then((file) => {
            if (file) {
                logger.info('App', `pending file received: ${file} — switching to selector`);
                setSelectFile(file);
                setMode('selector');
            }
        });
    }, []);

    useEffect(() => {
        document.body.style.backgroundColor = isDark ? '#0a0a0b' : '#f8f9fa';
        document.body.style.color = isDark ? '#e8e8ec' : '#1a1a1d';
    }, [isDark]);

    return (
        <ConfigProvider
            theme={{
                algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
                token: {
                    colorPrimary: '#f59e0b',
                    colorBgContainer: isDark ? '#111113' : '#ffffff',
                    colorBgElevated: isDark ? '#1a1a1d' : '#f5f5f5',
                    colorBorder: isDark ? '#2a2a2e' : '#e5e5e5',
                    colorText: isDark ? '#e8e8ec' : '#1a1a1d',
                    colorTextSecondary: isDark ? '#8b8b94' : '#6b7280',
                    borderRadius: 8,
                    fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, sans-serif",
                },
            }}
        >
            <AntApp>
                <div className="app-container">
                    {mode === 'editor' ? (
                        <>
                            <header className="app-header">
                                <div className="app-logo">
                                    <div className="app-logo-icon">Q</div>
                                    <span className="app-logo-text">QuickSort</span>
                                    <span className="app-logo-version">v0.2.0</span>
                                </div>
                                <div 
                                    className="theme-toggle"
                                    onClick={() => setIsDark(!isDark)}
                                >
                                    <span className="theme-toggle-icon">
                                        {isDark ? '🌙' : '☀️'}
                                    </span>
                                </div>
                            </header>
                            <main className="app-content page-enter">
                                <Tabs
                                    activeKey={activeTab}
                                    onChange={(key) => {
                                        setActiveTab(key);
                                        logger.action('App', `tab switch → ${key}`);
                                    }}
                                    items={[
                                        { 
                                            key: 'folders', 
                                            label: 'Папки', 
                                            children: <EditorPage /> 
                                        },
                                        { 
                                            key: 'log', 
                                            label: 'Лог', 
                                            children: <LogPage /> 
                                        },
                                        { 
                                            key: 'settings', 
                                            label: 'Настройки', 
                                            children: <SettingsPage /> 
                                        },
                                        { 
                                            key: 'about', 
                                            label: 'О программе', 
                                            children: <AboutPage /> 
                                        },
                                    ]}
                                />
                            </main>
                        </>
                    ) : (
                        <SelectorPage file={selectFile} onClose={() => setMode('editor')} />
                    )}
                </div>
            </AntApp>
        </ConfigProvider>
    );
}

export default App;
