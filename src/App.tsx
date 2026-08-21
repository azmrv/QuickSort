import { useState, useEffect } from 'react';
import { invoke } from './lib/invoke';
import { logger } from './lib/logger';
import { ConfigProvider, theme, App as AntApp } from 'antd';
import EditorPage from './pages/EditorPage';
import SelectorPage from './pages/SelectorPage';
import LogPage from './pages/LogPage';
import HistoryPage from './pages/HistoryPage';
import SettingsPage from './pages/SettingsPage';
import AboutPage from './pages/AboutPage';
import './styles/App.css';

function App() {
    const [mode, setMode] = useState<'editor' | 'selector'>('editor');
    const [selectFile, setSelectFile] = useState<string | null>(null);
    const [isDark, setIsDark] = useState(true);
    const [activeTab, setActiveTab] = useState('folders');
    const [version, setVersion] = useState('0.0.0');

    useEffect(() => {
        logger.info('App', 'startup');
        invoke<string>('get_app_version').then(setVersion);
        invoke<string | null>('get_pending_file').then((file) => {
            if (file) {
                logger.info('App', `pending file: ${file}`);
                setSelectFile(file);
                setMode('selector');
            }
        });
    }, []);

    useEffect(() => {
        document.body.style.backgroundColor = isDark ? '#0a0a0b' : '#f8f9fa';
        document.body.style.color = isDark ? '#e8e8ec' : '#1a1a1d';
    }, [isDark]);

    const TABS = [
        { key: 'folders', label: 'Папки', content: <EditorPage /> },
        { key: 'history', label: 'История', content: <HistoryPage /> },
        { key: 'log', label: 'Лог', content: <LogPage /> },
        { key: 'settings', label: 'Настройки', content: <SettingsPage /> },
        { key: 'about', label: 'О программе', content: <AboutPage /> },
    ];

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
                {mode === 'editor' ? (
                    <div className="app-layout">
                        <header className="app-header">
                            <div className="app-logo">
                                <div className="app-logo-icon">Q</div>
                                <span className="app-logo-text">QuickSort</span>
                                <span className="app-logo-version">v{version}</span>
                            </div>
                            <div className="theme-toggle" onClick={() => setIsDark(!isDark)}>
                                <span className="theme-toggle-icon">{isDark ? '🌙' : '☀️'}</span>
                            </div>
                        </header>
                        <main className="app-main">
                            <div className="tab-nav">
                                {TABS.map(t => (
                                    <button
                                        key={t.key}
                                        className={`tab-nav-btn ${activeTab === t.key ? 'active' : ''}`}
                                        onClick={() => {
                                            setActiveTab(t.key);
                                            logger.action('App', `tab switch → ${t.key}`);
                                        }}
                                    >
                                        {t.label}
                                    </button>
                                ))}
                            </div>
                            <div className="tab-content">
                                {TABS.filter(t => t.key === activeTab).map(t => (
                                    <div key={t.key} className="tab-panel">
                                        {t.content}
                                    </div>
                                ))}
                            </div>
                        </main>
                    </div>
                ) : (
                    <SelectorPage file={selectFile} onClose={() => setMode('editor')} />
                )}
            </AntApp>
        </ConfigProvider>
    );
}

export default App;
