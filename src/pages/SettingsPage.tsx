import { useState, useEffect } from 'react';
import { App } from 'antd';
import { invoke } from '../lib/invoke';
import { emit } from '@tauri-apps/api/event';
import { save, open } from '@tauri-apps/plugin-dialog';
import { isEnabled, enable, disable } from '@tauri-apps/plugin-autostart';
import { logger } from '../lib/logger';
import { useTranslation } from '../i18n/useTranslation';
import { LOCALE_LABELS, type Locale } from '../i18n/translations';
import LogPage from './LogPage';
import PluginsPage from './PluginsPage';

// Log levels understood by the backend tracing subscriber (src-tauri/src/logging.rs).
const LOG_LEVELS = ['trace', 'debug', 'info', 'warn', 'error'] as const;
type LogLevel = (typeof LOG_LEVELS)[number];

interface LoggingConfig {
    level: LogLevel;
    format: 'text' | 'json';
}

interface Settings {
    default_operation: 'Move' | 'Copy';
    default_overwrite_policy: 'Skip' | 'Overwrite' | 'AutoRename';
    duplicate_check: {
        enabled: boolean;
        mode: 'name' | 'size' | 'content';
    };
    theme_mode: 'system' | 'light' | 'dark';
    locale: Locale;
    logging: LoggingConfig;
}

// Defaults for the logging config so the page works even when the field
// is absent from an older settings.json on disk.
const DEFAULT_LOGGING: LoggingConfig = { level: 'info', format: 'text' };

const DEFAULT_SETTINGS: Settings = {
    default_operation: 'Move',
    default_overwrite_policy: 'Skip',
    duplicate_check: {
        enabled: true,
        mode: 'name',
    },
    theme_mode: 'system',
    locale: 'en',
    logging: DEFAULT_LOGGING,
};

const SettingsPage: React.FC = () => {
    const { t } = useTranslation();
    const { message } = App.useApp();
const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
const [loading, setLoading] = useState(true);
const [autostartEnabled, setAutostartEnabled] = useState(false);

useEffect(() => {
    // Current OS autostart state (Windows registry) is independent of settings.json,
    // so it is loaded separately from the plugin.
    isEnabled().then(setAutostartEnabled).catch((err) => {
        logger.error('SettingsPage', 'Failed to read autostart state', err);
    });
}, []);

const toggleAutostart = async () => {
    const next = !autostartEnabled;
    logger.action('SettingsPage', next ? 'enable autostart' : 'disable autostart');
    try {
        if (next) {
            await enable();
        } else {
            await disable();
        }
        setAutostartEnabled(next);
    } catch (err) {
        logger.error('SettingsPage', 'Failed to change autostart state', err);
        message.error(`Error: ${err}`);
    }
};

    useEffect(() => {
        invoke<Partial<Settings>>('get_settings')
            .then((loaded) => {
                const next: Settings = {
                    ...DEFAULT_SETTINGS,
                    ...loaded,
                    logging: {
                        ...DEFAULT_LOGGING,
                        ...(loaded.logging ?? {}),
                    },
                };
                setSettings(next);
            })
            .catch((err) => {
                logger.error('SettingsPage', 'Failed to load settings', err);
            })
            .finally(() => setLoading(false));
    }, []);

    const saveSettings = async (newSettings: Settings) => {
        setSettings(newSettings);
        try {
            await invoke('save_settings', { settings: newSettings });
            logger.info('SettingsPage', 'Settings saved');
            // Emit event so App.tsx can update theme/locale
            await emit('settings-changed', newSettings);
        } catch (err) {
            logger.error('SettingsPage', 'Failed to save settings', err);
            message.error(t('settings.save_error'));
        }
    };

    const handleRegister = async () => {
        logger.action('SettingsPage', 'register COM server');
        try {
            const msg = await invoke<string>('register_com_server');
            logger.info('SettingsPage', `COM registered: ${msg}`);
            message.success(msg);
        } catch (err) {
            logger.error('SettingsPage', 'COM register failed', err);
            message.error(`Error: ${err}`);
        }
    };

    const handleUnregister = async () => {
        logger.action('SettingsPage', 'unregister COM server');
        try {
            const msg = await invoke<string>('unregister_com_server');
            logger.info('SettingsPage', `COM unregistered: ${msg}`);
            message.success(msg);
        } catch (err) {
            logger.error('SettingsPage', 'COM unregister failed', err);
            message.error(`Error: ${err}`);
        }
    };

    const sectionStyle = {
        fontFamily: 'var(--qs-font-display)',
        fontSize: '16px',
        fontWeight: 600 as const,
        color: 'var(--qs-text-primary)',
        marginBottom: '12px',
    };

    const labelStyle = {
        color: 'var(--qs-text-secondary)',
        marginBottom: '8px',
        lineHeight: 1.6,
        fontSize: '14px',
    };

    const selectStyle = {
        width: '100%',
        padding: '10px 12px',
        background: 'var(--qs-bg-tertiary)',
        border: '1px solid var(--qs-border)',
        borderRadius: 'var(--qs-radius-md)',
        color: 'var(--qs-text-primary)',
        fontFamily: 'var(--qs-font-body)',
        fontSize: '14px',
        cursor: 'pointer' as const,
    };

    const toggleContainerStyle = {
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '12px 16px',
        background: 'var(--qs-bg-tertiary)',
        border: '1px solid var(--qs-border)',
        borderRadius: 'var(--qs-radius-md)',
    };

    const toggleStyle = (enabled: boolean) => ({
        width: '48px',
        height: '24px',
        borderRadius: '12px',
        background: enabled ? 'var(--qs-accent)' : 'var(--qs-bg-secondary)',
        border: 'none',
        cursor: 'pointer' as const,
        position: 'relative' as const,
        transition: 'background 0.2s',
    });

    const toggleDotStyle = (enabled: boolean) => ({
        width: '20px',
        height: '20px',
        borderRadius: '50%',
        background: 'white',
        position: 'absolute' as const,
        top: '2px',
        left: enabled ? '26px' : '2px',
        transition: 'left 0.2s',
    });

    if (loading) {
        return <div style={{ color: 'var(--qs-text-muted)' }}>{t('settings.loading')}</div>;
    }

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '32px' }}>
            {/* Appearance */}
            <div>
                <h3 style={sectionStyle}>{t('settings.appearance.title')}</h3>
                <p style={labelStyle}>
                    {t('settings.appearance.description')}
                </p>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    <div>
                        <p style={{ ...labelStyle, marginBottom: '4px', fontSize: '13px' }}>
                            {t('settings.appearance.theme')}
                        </p>
                        <select
                            value={settings.theme_mode}
                            onChange={(e) => {
                                const newSettings = {
                                    ...settings,
                                    theme_mode: e.target.value as 'system' | 'light' | 'dark',
                                };
                                saveSettings(newSettings);
                            }}
                            style={selectStyle}
                        >
                            <option value="system">{t('settings.appearance.theme.system')}</option>
                            <option value="light">{t('settings.appearance.theme.light')}</option>
                            <option value="dark">{t('settings.appearance.theme.dark')}</option>
                        </select>
                    </div>
                    <div>
                        <p style={{ ...labelStyle, marginBottom: '4px', fontSize: '13px' }}>
                            {t('settings.appearance.language')}
                        </p>
                        <select
                            value={settings.locale}
                            onChange={(e) => {
                                const newSettings = {
                                    ...settings,
                                    locale: e.target.value as Locale,
                                };
                                saveSettings(newSettings);
                            }}
                            style={selectStyle}
                        >
                            {Object.entries(LOCALE_LABELS).map(([code, label]) => (
                                <option key={code} value={code}>{label}</option>
                            ))}
                        </select>
                    </div>
                </div>
            </div>

            {/* COM Server */}
            <div>
                <h3 style={sectionStyle}>{t('settings.com_server.title')}</h3>
                <p style={labelStyle}>
                    {t('settings.com_server.description')}
                </p>
                <div style={{ display: 'flex', gap: 'var(--qs-space-sm)' }}>
                    <button
                        onClick={handleRegister}
                        style={{
                            flex: 1,
                            padding: 'var(--qs-space-md)',
                            background: 'var(--qs-accent)',
                            border: 'none',
                            borderRadius: 'var(--qs-radius-md)',
                            color: 'var(--qs-bg-primary)',
                            fontFamily: 'var(--qs-font-body)',
                            fontSize: '14px',
                            fontWeight: 600,
                            cursor: 'pointer',
                        }}
                    >
                        {t('settings.com_server.register')}
                    </button>
                    <button
                        onClick={handleUnregister}
                        style={{
                            flex: 1,
                            padding: 'var(--qs-space-md)',
                            background: 'var(--qs-danger-muted)',
                            border: '1px solid transparent',
                            borderRadius: 'var(--qs-radius-md)',
                            color: 'var(--qs-danger)',
                            fontFamily: 'var(--qs-font-body)',
                            fontSize: '14px',
                            fontWeight: 600,
                            cursor: 'pointer',
                        }}
                    >
                        {t('settings.com_server.unregister')}
                    </button>
                </div>
            </div>

            {/* Autostart (0.2.6 feature #23 / plan Q6) */}
            <div>
                <h3 style={sectionStyle}>{t('settings.autostart.title')}</h3>
                <p style={labelStyle}>
                    {t('settings.autostart.description')}
                </p>
                <div style={toggleContainerStyle}>
                    <span style={{ color: 'var(--qs-text-secondary)', fontSize: '14px' }}>
                        {t('settings.autostart.toggle')}
                    </span>
                    <button
                        style={toggleStyle(autostartEnabled)}
                        onClick={toggleAutostart}
                    >
                        <div style={toggleDotStyle(autostartEnabled)} />
                    </button>
                </div>
            </div>

            {/* Default Actions */}
            <div>
                <h3 style={sectionStyle}>{t('settings.default_actions.title')}</h3>
                <p style={labelStyle}>
                    {t('settings.default_actions.description')}
                </p>
                <select
                    value={settings.default_operation}
                    onChange={(e) => {
                        const newSettings = {
                            ...settings,
                            default_operation: e.target.value as 'Move' | 'Copy',
                        };
                        saveSettings(newSettings);
                    }}
                    style={selectStyle}
                >
                    <option value="Move">{t('settings.default_actions.move')}</option>
                    <option value="Copy">{t('settings.default_actions.copy')}</option>
                </select>
            </div>

            {/* Duplicate Handling */}
            <div>
                <h3 style={sectionStyle}>{t('settings.duplicates.title')}</h3>
                <p style={labelStyle}>
                    {t('settings.duplicates.description')}
                </p>
                <select
                    value={settings.default_overwrite_policy}
                    onChange={(e) => {
                        const newSettings = {
                            ...settings,
                            default_overwrite_policy: e.target.value as 'Skip' | 'Overwrite' | 'AutoRename',
                        };
                        saveSettings(newSettings);
                    }}
                    style={selectStyle}
                >
                    <option value="Skip">{t('settings.duplicates.skip')}</option>
                    <option value="Overwrite">{t('settings.duplicates.overwrite')}</option>
                    <option value="AutoRename">{t('settings.duplicates.auto_rename')}</option>
                </select>
            </div>

            {/* Duplicate Check */}
            <div>
                <h3 style={sectionStyle}>{t('settings.duplicate_check.title')}</h3>
                <p style={labelStyle}>
                    {t('settings.duplicate_check.description')}
                </p>
                <div style={toggleContainerStyle}>
                    <span style={{ color: 'var(--qs-text-secondary)', fontSize: '14px' }}>
                        {t('settings.duplicate_check.toggle')}
                    </span>
                    <button
                        style={toggleStyle(settings.duplicate_check.enabled)}
                        onClick={() => {
                            const newSettings = {
                                ...settings,
                                duplicate_check: {
                                    ...settings.duplicate_check,
                                    enabled: !settings.duplicate_check.enabled,
                                },
                            };
                            saveSettings(newSettings);
                        }}
                    >
                        <div style={toggleDotStyle(settings.duplicate_check.enabled)} />
                    </button>
                </div>

                {settings.duplicate_check.enabled && (
                    <div style={{ marginTop: '12px' }}>
                        <p style={labelStyle}>
                            {t('settings.duplicate_check.mode')}
                        </p>
                        <select
                            value={settings.duplicate_check.mode}
                            onChange={(e) => {
                                const newSettings = {
                                    ...settings,
                                    duplicate_check: {
                                        ...settings.duplicate_check,
                                        mode: e.target.value as 'name' | 'size' | 'content',
                                    },
                                };
                                saveSettings(newSettings);
                            }}
                            style={selectStyle}
                        >
                            <option value="name">{t('settings.duplicate_check.quick')}</option>
                            <option value="size">{t('settings.duplicate_check.medium')}</option>
                            <option value="content">{t('settings.duplicate_check.deep')}</option>
                        </select>
                    </div>
                )}
            </div>

            {/* Logging */}
            <div>
                <h3 style={sectionStyle}>{t('settings.logging.title')}</h3>
                <p style={labelStyle}>
                    {t('settings.logging.description')}
                </p>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    <div>
                        <p style={{ ...labelStyle, marginBottom: '4px', fontSize: '13px' }}>
                            {t('settings.logging.level')}
                        </p>
                        <select
                            value={settings.logging.level}
                            onChange={(e) => {
                                const level = e.target.value as LogLevel;
                                const newSettings = {
                                    ...settings,
                                    logging: {
                                        ...settings.logging,
                                        level,
                                    },
                                };
                                saveSettings(newSettings);
                                // Apply immediately so the current session's
                                // backend logs follow the new level without
                                // an app restart.
                                invoke('set_log_level', { level }).catch((err) => {
                                    logger.error('SettingsPage', 'Failed to apply log level', err);
                                    message.error(t('settings.logging.level_error'));
                                });
                            }}
                            style={selectStyle}
                        >
                            {LOG_LEVELS.map((l) => (
                                <option key={l} value={l}>
                                    {t(`settings.logging.level.${l}`)}
                                </option>
                            ))}
                        </select>
                    </div>
                    <div>
                        <p style={{ ...labelStyle, marginBottom: '4px', fontSize: '13px' }}>
                            {t('settings.logging.format')}
                        </p>
                        <select
                            value={settings.logging.format}
                            onChange={(e) => {
                                const newSettings = {
                                    ...settings,
                                    logging: {
                                        ...settings.logging,
                                        format: e.target.value as 'text' | 'json',
                                    },
                                };
                                saveSettings(newSettings);
                            }}
                            style={selectStyle}
                        >
                            <option value="text">{t('settings.logging.format.text')}</option>
                            <option value="json">{t('settings.logging.format.json')}</option>
                        </select>
                    </div>
                </div>
            </div>

            {/* Live log viewer — moved from the Log tab into Settings per release plan */}
            <div>
                <h3 style={sectionStyle}>{t('settings.logs.title')}</h3>
                <p style={labelStyle}>
                    {t('settings.logs.description')}
                </p>
                <div className="settings-logs-panel">
                    <LogPage />
                </div>
            </div>

            {/* Plugins — moved from the Plugins tab into Settings (0.2.6 feature #17) */}
            <div>
                <h3 style={sectionStyle}>{t('settings.plugins.title')}</h3>
                <p style={labelStyle}>
                    {t('settings.plugins.description')}
                </p>
                <div className="settings-plugins-panel">
                    <PluginsPage />
                </div>
            </div>

            {/* Backup (0.2.6 feature #20 / plan Q7) */}
            <div>
                <h3 style={sectionStyle}>{t('settings.backup.title')}</h3>
                <p style={labelStyle}>
                    {t('settings.backup.description')}
                </p>
                <div style={{ display: 'flex', gap: 'var(--qs-space-sm)' }}>
                    <button
                        onClick={async () => {
                            logger.action('SettingsPage', 'create backup');
                            try {
                                const stamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, '-');
                                const target = await save({
                                    title: t('settings.backup.create'),
                                    defaultPath: `quicksort-backup-${stamp}.zip`,
                                    filters: [{ name: 'ZIP', extensions: ['zip'] }],
                                });
                                if (!target) return;
                                await invoke('backup_data', { targetPath: target });
                                logger.info('SettingsPage', 'Backup created');
                                message.success(t('settings.backup.created'));
                            } catch (err) {
                                logger.error('SettingsPage', 'Backup failed', err);
                                message.error(`Error: ${err}`);
                            }
                        }}
                        style={{
                            flex: 1,
                            padding: 'var(--qs-space-md)',
                            background: 'var(--qs-accent)',
                            border: 'none',
                            borderRadius: 'var(--qs-radius-md)',
                            color: 'var(--qs-bg-primary)',
                            fontFamily: 'var(--qs-font-body)',
                            fontSize: '14px',
                            fontWeight: 600,
                            cursor: 'pointer',
                        }}
                    >
                        {t('settings.backup.create')}
                    </button>
                    <button
                        onClick={async () => {
                            logger.action('SettingsPage', 'restore backup');
                            try {
                                const selected = await open({
                                    title: t('settings.backup.restore'),
                                    filters: [{ name: 'ZIP', extensions: ['zip'] }],
                                    multiple: false,
                                });
                                if (!selected || typeof selected !== 'string') return;
                                await invoke('restore_data', { backupPath: selected });
                                logger.info('SettingsPage', 'Backup restored');
                                message.success(t('settings.backup.restored'));
                                const loaded = await invoke<Partial<Settings>>('get_settings');
                                setSettings({
                                    ...DEFAULT_SETTINGS,
                                    ...loaded,
                                    logging: { ...DEFAULT_LOGGING, ...(loaded.logging ?? {}) },
                                });
                            } catch (err) {
                                logger.error('SettingsPage', 'Restore failed', err);
                                message.error(`Error: ${err}`);
                            }
                        }}
                        style={{
                            flex: 1,
                            padding: 'var(--qs-space-md)',
                            background: 'var(--qs-danger-muted)',
                            border: '1px solid transparent',
                            borderRadius: 'var(--qs-radius-md)',
                            color: 'var(--qs-danger)',
                            fontFamily: 'var(--qs-font-body)',
                            fontSize: '14px',
                            fontWeight: 600,
                            cursor: 'pointer',
                        }}
                    >
                        {t('settings.backup.restore')}
                    </button>
                </div>
            </div>

            {/* Application */}
            <div>
                <h3 style={sectionStyle}>{t('settings.application.title')}</h3>
                <button
                    onClick={async () => {
                        logger.action('SettingsPage', 'quit app');
                        try {
                            await invoke('quit_app');
                        } catch (err) {
                            logger.error('SettingsPage', 'Failed to quit app', err);
                        }
                    }}
                    style={{
                        width: '100%',
                        padding: 'var(--qs-space-md)',
                        background: 'var(--qs-danger-muted)',
                        border: '1px solid transparent',
                        borderRadius: 'var(--qs-radius-md)',
                        color: 'var(--qs-danger)',
                        fontFamily: 'var(--qs-font-body)',
                        fontSize: '14px',
                        fontWeight: 600,
                        cursor: 'pointer',
                    }}
                >
                    {t('settings.application.quit')}
                </button>
            </div>
        </div>
    );
};

export default SettingsPage;
