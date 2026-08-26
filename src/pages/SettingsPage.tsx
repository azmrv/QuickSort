import { useState, useEffect } from 'react';
import { App } from 'antd';
import { invoke } from '../lib/invoke';
import { emit } from '@tauri-apps/api/event';
import { logger } from '../lib/logger';
import { useTranslation } from '../i18n/useTranslation';
import { LOCALE_LABELS, type Locale } from '../i18n/translations';

const LOG_LEVELS = ['info', 'debug', 'warn', 'error'];

interface Settings {
    default_operation: 'Move' | 'Copy';
    default_overwrite_policy: 'Skip' | 'Overwrite' | 'AutoRename';
    duplicate_check: {
        enabled: boolean;
        mode: 'name' | 'size' | 'content';
    };
    theme_mode: 'System' | 'Light' | 'Dark';
    locale: Locale;
}

const SettingsPage: React.FC = () => {
    const { t } = useTranslation();
    const { message } = App.useApp();
    const [logLevel, setLogLevel] = useState('info');
    const [settings, setSettings] = useState<Settings>({
        default_operation: 'Move',
        default_overwrite_policy: 'Skip',
        duplicate_check: {
            enabled: true,
            mode: 'name',
        },
        theme_mode: 'System',
        locale: 'en',
    });
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        invoke<Settings>('get_settings')
            .then(setSettings)
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
                                    theme_mode: e.target.value as 'System' | 'Light' | 'Dark',
                                };
                                saveSettings(newSettings);
                            }}
                            style={selectStyle}
                        >
                            <option value="System">{t('settings.appearance.theme.system')}</option>
                            <option value="Light">{t('settings.appearance.theme.light')}</option>
                            <option value="Dark">{t('settings.appearance.theme.dark')}</option>
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
                <select
                    value={logLevel}
                    onChange={(e) => {
                        const level = e.target.value;
                        setLogLevel(level);
                        logger.action('SettingsPage', `log level changed → ${level}`);
                    }}
                    style={selectStyle}
                >
                    {LOG_LEVELS.map(l => (
                        <option key={l} value={l}>{l.toUpperCase()}</option>
                    ))}
                </select>
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
