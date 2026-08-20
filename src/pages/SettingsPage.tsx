import { message } from 'antd';
import { invoke } from '@tauri-apps/api/core';

const SettingsPage: React.FC = () => {
    const handleRegister = async () => {
        try {
            const msg = await invoke<string>('register_com_server');
            message.success(msg);
        } catch (err) {
            message.error(`Ошибка: ${err}`);
        }
    };

    const handleUnregister = async () => {
        try {
            const msg = await invoke<string>('unregister_com_server');
            message.success(msg);
        } catch (err) {
            message.error(`Ошибка: ${err}`);
        }
    };

    return (
        <div>
            <h3 style={{
                fontFamily: 'var(--qs-font-display)',
                fontSize: '16px',
                fontWeight: 600,
                color: 'var(--qs-text-primary)',
                marginBottom: 'var(--qs-space-lg)',
            }}>
                COM-сервер
            </h3>
            
            <p style={{
                color: 'var(--qs-text-secondary)',
                marginBottom: 'var(--qs-space-lg)',
                lineHeight: 1.6,
            }}>
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
                        transition: 'all var(--qs-transition-fast)',
                    }}
                    onMouseEnter={(e) => {
                        e.currentTarget.style.background = 'var(--qs-accent-hover)';
                        e.currentTarget.style.boxShadow = 'var(--qs-shadow-glow)';
                    }}
                    onMouseLeave={(e) => {
                        e.currentTarget.style.background = 'var(--qs-accent)';
                        e.currentTarget.style.boxShadow = 'none';
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
                        transition: 'all var(--qs-transition-fast)',
                    }}
                    onMouseEnter={(e) => {
                        e.currentTarget.style.background = 'var(--qs-danger)';
                        e.currentTarget.style.color = 'white';
                    }}
                    onMouseLeave={(e) => {
                        e.currentTarget.style.background = 'var(--qs-danger-muted)';
                        e.currentTarget.style.color = 'var(--qs-danger)';
                    }}
                >
                    Удалить
                </button>
            </div>
        </div>
    );
};

export default SettingsPage;
