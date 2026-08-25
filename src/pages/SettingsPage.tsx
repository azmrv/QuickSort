import { useState, useEffect } from 'react';
import { App } from 'antd';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';

const LOG_LEVELS = ['info', 'debug', 'warn', 'error'];

interface Settings {
    default_operation: 'Move' | 'Copy';
    default_overwrite_policy: 'Skip' | 'Overwrite' | 'AutoRename';
    duplicate_check: {
        enabled: boolean;
        mode: 'name' | 'size' | 'content';
    };
}

const SettingsPage: React.FC = () => {
    const { message } = App.useApp();
    const [logLevel, setLogLevel] = useState('info');
    const [settings, setSettings] = useState<Settings>({
        default_operation: 'Move',
        default_overwrite_policy: 'Skip',
        duplicate_check: {
            enabled: true,
            mode: 'name',
        },
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
        } catch (err) {
            logger.error('SettingsPage', 'Failed to save settings', err);
            message.error('Failed to save settings');
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
            message.error(`Ошибка: ${err}`);
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
            message.error(`Ошибка: ${err}`);
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
        return <div style={{ color: 'var(--qs-text-muted)' }}>Loading...</div>;
    }

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '32px' }}>
            <div>
                <h3 style={sectionStyle}>COM-сервер</h3>
                <p style={labelStyle}>
                    Регистрация COM-сервера необходима для интеграции с контекстным меню Проводника.
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
                        Зарегистрировать
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
                        Удалить
                    </button>
                </div>
            </div>

            <div>
                <h3 style={sectionStyle}>Действия по умолчанию</h3>
                <p style={labelStyle}>
                    Выберите тип операции по умолчанию при перемещении файлов.
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
                    <option value="Move">Перемещение</option>
                    <option value="Copy">Копирование</option>
                </select>
            </div>

            <div>
                <h3 style={sectionStyle}>При обнаружении дубликатов</h3>
                <p style={labelStyle}>
                    Действие по умолчанию при обнаружении файла с таким же именем.
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
                    <option value="Skip">Пропустить</option>
                    <option value="Overwrite">Заменить</option>
                    <option value="AutoRename">Переименовать</option>
                </select>
            </div>

            <div>
                <h3 style={sectionStyle}>Проверка дубликатов</h3>
                <p style={labelStyle}>
                    Включите проверку на дубликаты перед перемещением файлов.
                </p>
                <div style={toggleContainerStyle}>
                    <span style={{ color: 'var(--qs-text-secondary)', fontSize: '14px' }}>
                        Проверка дубликатов
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
                            Режим проверки:
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
                            <option value="name">Быстрый (по имени)</option>
                            <option value="size">Средний (имя + размер)</option>
                            <option value="content">Глубокий (SHA-256 хеш)</option>
                        </select>
                    </div>
                )}
            </div>

            <div>
                <h3 style={sectionStyle}>Логирование</h3>
                <p style={labelStyle}>
                    Минимальный уровень логов, отправляемых из бэкенда в журнал.
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

            <div>
                <h3 style={sectionStyle}>Приложение</h3>
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
                    Выход из приложения
                </button>
            </div>
        </div>
    );
};

export default SettingsPage;
