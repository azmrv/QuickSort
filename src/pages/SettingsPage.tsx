import { useState } from 'react';
import { App } from 'antd';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';

const LOG_LEVELS = ['info', 'debug', 'warn', 'error'];

const SettingsPage: React.FC = () => {
    const { message } = App.useApp();
    const [logLevel, setLogLevel] = useState('info');

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
                        // TODO: send to backend when runtime filter is implemented
                    }}
                    style={selectStyle}
                >
                    {LOG_LEVELS.map(l => (
                        <option key={l} value={l}>{l.toUpperCase()}</option>
                    ))}
                </select>
            </div>
        </div>
    );
};

export default SettingsPage;
