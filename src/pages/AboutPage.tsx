import { useState, useEffect } from 'react';
import { invoke } from '../lib/invoke';

const AboutPage = () => {
    const [version, setVersion] = useState('0.0.0');

    useEffect(() => {
        invoke<string>('get_app_version').then(setVersion);
    }, []);

    return (
        <div style={{ 
            display: 'flex', 
            flexDirection: 'column', 
            alignItems: 'center', 
            justifyContent: 'center',
            padding: 'var(--qs-space-2xl)',
            textAlign: 'center',
        }}>
            <div style={{
                width: '80px',
                height: '80px',
                background: 'linear-gradient(135deg, var(--qs-accent), #f97316)',
                borderRadius: 'var(--qs-radius-xl)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontFamily: 'var(--qs-font-display)',
                fontWeight: 700,
                fontSize: '36px',
                color: 'var(--qs-bg-primary)',
                marginBottom: 'var(--qs-space-lg)',
                boxShadow: 'var(--qs-shadow-glow)',
            }}>
                Q
            </div>
            
            <h2 style={{
                fontFamily: 'var(--qs-font-display)',
                fontSize: '24px',
                fontWeight: 600,
                color: 'var(--qs-text-primary)',
                marginBottom: 'var(--qs-space-sm)',
            }}>
                QuickSort
            </h2>
            
            <div style={{
                fontFamily: 'var(--qs-font-mono)',
                fontSize: '13px',
                color: 'var(--qs-text-muted)',
                marginBottom: 'var(--qs-space-lg)',
            }}>
                v{version}
            </div>
            
            <p style={{
                color: 'var(--qs-text-secondary)',
                maxWidth: '400px',
                lineHeight: 1.6,
                marginBottom: 'var(--qs-space-xl)',
            }}>
                Менеджер файлов нового поколения для Windows.
                Мгновенное перемещение, обнаружение дубликатов и история операций.
            </p>
            
            <div style={{
                display: 'flex',
                gap: 'var(--qs-space-md)',
                flexWrap: 'wrap',
                justifyContent: 'center',
            }}>
                <a
                    href="https://github.com/azmrv/QuickSort"
                    target="_blank"
                    rel="noopener noreferrer"
                    style={{
                        padding: 'var(--qs-space-sm) var(--qs-space-md)',
                        background: 'var(--qs-bg-tertiary)',
                        border: '1px solid var(--qs-border)',
                        borderRadius: 'var(--qs-radius-md)',
                        color: 'var(--qs-text-secondary)',
                        fontFamily: 'var(--qs-font-mono)',
                        fontSize: '12px',
                        textDecoration: 'none',
                        transition: 'all var(--qs-transition-fast)',
                    }}
                    onMouseEnter={(e) => {
                        e.currentTarget.style.borderColor = 'var(--qs-accent)';
                        e.currentTarget.style.color = 'var(--qs-accent)';
                    }}
                    onMouseLeave={(e) => {
                        e.currentTarget.style.borderColor = 'var(--qs-border)';
                        e.currentTarget.style.color = 'var(--qs-text-secondary)';
                    }}
                >
                    GitHub
                </a>
                <a
                    href="https://t.me/Fib511"
                    target="_blank"
                    rel="noopener noreferrer"
                    style={{
                        padding: 'var(--qs-space-sm) var(--qs-space-md)',
                        background: 'var(--qs-bg-tertiary)',
                        border: '1px solid var(--qs-border)',
                        borderRadius: 'var(--qs-radius-md)',
                        color: 'var(--qs-text-secondary)',
                        fontFamily: 'var(--qs-font-mono)',
                        fontSize: '12px',
                        textDecoration: 'none',
                        transition: 'all var(--qs-transition-fast)',
                    }}
                    onMouseEnter={(e) => {
                        e.currentTarget.style.borderColor = 'var(--qs-accent)';
                        e.currentTarget.style.color = 'var(--qs-accent)';
                    }}
                    onMouseLeave={(e) => {
                        e.currentTarget.style.borderColor = 'var(--qs-border)';
                        e.currentTarget.style.color = 'var(--qs-text-secondary)';
                    }}
                >
                    Telegram
                </a>
            </div>
            
            <div style={{
                marginTop: 'var(--qs-space-2xl)',
                paddingTop: 'var(--qs-space-lg)',
                borderTop: '1px solid var(--qs-border)',
                width: '100%',
            }}>
                <div style={{
                    fontFamily: 'var(--qs-font-mono)',
                    fontSize: '11px',
                    color: 'var(--qs-text-muted)',
                }}>
                    Автор: azmrv (https://github.com/azmrv)
                </div>
            </div>
        </div>
    );
};

export default AboutPage;
