import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from './lib/invoke';
import { logger } from './lib/logger';
import { ConfigProvider, theme, App as AntApp } from 'antd';
import { LanguageProvider, useTranslation } from './i18n/LanguageContext';
import type { Locale } from './i18n/translations';
import EditorPage from './pages/EditorPage';
import SelectorPage from './pages/SelectorPage';
import LogPage from './pages/LogPage';
import HistoryPage from './pages/HistoryPage';
import SettingsPage from './pages/SettingsPage';
import AboutPage from './pages/AboutPage';
import PluginsPage from './pages/PluginsPage';
import CommandPalette from './components/CommandPalette';
import './styles/App.css';

interface Settings {
    theme_mode: 'System' | 'Light' | 'Dark';
    locale: Locale;
    [key: string]: unknown;
}

function deriveIsDark(themeMode: string, systemDark: boolean): boolean {
    switch (themeMode) {
        case 'Light': return false;
        case 'Dark': return true;
        default: return systemDark;
    }
}

function AppContent() {
    const { t } = useTranslation();
    const [mode, setMode] = useState<'editor' | 'selector'>('editor');
    const [selectFiles, setSelectFiles] = useState<string[]>([]);
    const [themeMode, setThemeMode] = useState<string>('System');
    const [isDark, setIsDark] = useState(() => {
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches) {
            return false;
        }
        return true;
    });
    const [activeTab, setActiveTab] = useState('folders');
    const [version, setVersion] = useState('0.0.0');
    const [paletteOpen, setPaletteOpen] = useState(false);

    // Load settings and apply theme on startup
    useEffect(() => {
        logger.info('App', 'startup');
        invoke<string>('get_app_version').then(setVersion);
        invoke<Settings>('get_settings').then((settings) => {
            setThemeMode(settings.theme_mode || 'System');
            const systemDark = window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? true;
            setIsDark(deriveIsDark(settings.theme_mode || 'System', systemDark));
            logger.info('App', `loaded settings: theme=${settings.theme_mode}, locale=${settings.locale}`);
        }).catch((err) => {
            logger.error('App', 'Failed to load settings', err);
        });
        invoke<string[]>('get_pending_files').then((files) => {
            if (files && files.length > 0) {
                logger.info('App', `pending files: ${files.length}`);
                setSelectFiles(files);
                setMode('selector');
            }
        });
    }, []);

    // Listen for single-instance forwarded files (second launch while already running)
    useEffect(() => {
        const unlisten = listen<{ files: string[] }>('pending-file', (event) => {
            const files = event.payload.files;
            logger.info('App', `single-instance pending files: ${files.length}`);
            setSelectFiles(files);
            setMode('selector');
        });
        return () => { unlisten.then((fn) => fn()); };
    }, []);

    // Listen for Windows system theme changes — only apply when theme_mode === 'system'
    useEffect(() => {
        const mq = window.matchMedia('(prefers-color-scheme: dark)');
        const handler = (e: MediaQueryListEvent) => {
            if (themeMode === 'System') {
                setIsDark(e.matches);
                logger.info('App', `system theme changed → ${e.matches ? 'dark' : 'light'}`);
            }
        };
        mq.addEventListener('change', handler);
        return () => mq.removeEventListener('change', handler);
    }, [themeMode]);

    // Listen for settings-changed events from SettingsPage
    useEffect(() => {
        const unlisten = listen<Settings>('settings-changed', (event) => {
            const newSettings = event.payload;
            setThemeMode(newSettings.theme_mode);
            const systemDark = window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? true;
            setIsDark(deriveIsDark(newSettings.theme_mode, systemDark));
            logger.info('App', `settings-changed: theme=${newSettings.theme_mode}`);
        });
        return () => { unlisten.then((fn) => fn()); };
    }, []);

    // Global keyboard shortcut: Ctrl+Shift+Space opens Command Palette
    useEffect(() => {
        const handler = (e: KeyboardEvent) => {
            if (e.ctrlKey && e.shiftKey && e.code === 'Space') {
                e.preventDefault();
                setPaletteOpen((prev) => !prev);
            }
        };
        window.addEventListener('keydown', handler);
        return () => window.removeEventListener('keydown', handler);
    }, []);

    useEffect(() => {
        document.body.style.backgroundColor = isDark ? '#0a0a0b' : '#f8f9fa';
        document.body.style.color = isDark ? '#e8e8ec' : '#1a1a1d';
    }, [isDark]);

    const TABS = [
        { key: 'folders', label: t('tab.folders'), content: <EditorPage /> },
        { key: 'history', label: t('tab.history'), content: <HistoryPage /> },
        { key: 'plugins', label: t('tab.plugins'), content: <PluginsPage /> },
        { key: 'log', label: t('tab.log'), content: <LogPage /> },
        { key: 'settings', label: t('tab.settings'), content: <SettingsPage /> },
        { key: 'about', label: t('tab.about'), content: <AboutPage /> },
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
                    <SelectorPage files={selectFiles} onClose={() => setMode('editor')} />
                )}
                <CommandPalette open={paletteOpen} onClose={() => setPaletteOpen(false)} />
            </AntApp>
        </ConfigProvider>
    );
}

function App() {
    const [locale, setLocale] = useState<Locale>('en');
    const [localeLoaded, setLocaleLoaded] = useState(false);

    useEffect(() => {
        invoke<Settings>('get_settings').then((settings) => {
            setLocale(settings.locale || 'en');
            setLocaleLoaded(true);
        }).catch(() => {
            setLocaleLoaded(true);
        });
    }, []);

    useEffect(() => {
        const unlisten = listen<Settings>('settings-changed', (event) => {
            if (event.payload.locale) {
                setLocale(event.payload.locale);
            }
        });
        return () => { unlisten.then((fn) => fn()); };
    }, []);

    if (!localeLoaded) {
        return null;
    }

    return (
        <LanguageProvider initialLocale={locale}>
            <AppContent />
        </LanguageProvider>
    );
}

export default App;
