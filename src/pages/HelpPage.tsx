import type { CSSProperties, ReactNode } from 'react';
import { useTranslation } from '../i18n/useTranslation';

/**
 * Help tab (0.2.6 feature #1 / plan question Q3): a standalone reference page
 * with short usage guidance for each area of the app. Interactive hints are
 * rendered as plain <code> chips to keep the page dependency-free.
 */
const HelpPage = () => {
    const { t } = useTranslation();

    const sectionStyle: CSSProperties = {
        background: 'var(--qs-bg-tertiary)',
        border: '1px solid var(--qs-border)',
        borderRadius: 'var(--qs-radius-md)',
        padding: 'var(--qs-space-md)',
    };

    const sectionTitleStyle: CSSProperties = {
        fontFamily: 'var(--qs-font-display)',
        fontSize: '12px',
        fontWeight: 600,
        letterSpacing: '1px',
        textTransform: 'uppercase',
        color: 'var(--qs-accent)',
        marginBottom: 'var(--qs-space-sm)',
    };

    const bodyStyle: CSSProperties = {
        color: 'var(--qs-text-secondary)',
        fontSize: '13px',
        lineHeight: 1.7,
        margin: 0,
    };

    const hintChipStyle: CSSProperties = {
        display: 'inline-block',
        padding: '2px 8px',
        margin: '2px 0',
        background: 'var(--qs-bg-elevated)',
        border: '1px solid var(--qs-border)',
        borderRadius: 'var(--qs-radius-sm)',
        color: 'var(--qs-accent)',
        fontFamily: 'var(--qs-font-mono)',
        fontSize: '12px',
    };

    const Section = ({ title, children }: { title: string; children: ReactNode }) => (
        <section style={sectionStyle}>
            <div style={sectionTitleStyle}>{title}</div>
            <p style={bodyStyle}>{children}</p>
        </section>
    );

    const Hotkey = ({ keys }: { keys: string }) => (
        <div style={hintChipStyle}>{keys}</div>
    );

    return (
        <div style={{ padding: 'var(--qs-space-lg)', maxWidth: '640px' }}>
            <h3 style={{
                fontFamily: 'var(--qs-font-display)',
                fontSize: '16px',
                fontWeight: 600,
                color: 'var(--qs-text-primary)',
                margin: 0,
                marginBottom: 'var(--qs-space-md)',
            }}>
                {t('help.title')}
            </h3>
            <p style={bodyStyle}>{t('help.intro')}</p>

            <div style={{
                display: 'flex',
                flexDirection: 'column',
                gap: 'var(--qs-space-md)',
                marginTop: 'var(--qs-space-lg)',
            }}>
                <Section title={t('help.section.overview')}>
                    {t('help.overview.body')}
                </Section>

                <Section title={t('help.section.search')}>
                    {t('help.search.body')}
                    <div style={{ marginTop: 'var(--qs-space-sm)' }}>
                        <Hotkey keys={t('help.hotkey.palette')} />
                    </div>
                </Section>

                <Section title={t('help.section.folders')}>
                    {t('help.folders.body')}
                </Section>

                <Section title={t('help.section.operations')}>
                    {t('help.operations.body')}
                </Section>

                <Section title={t('help.section.settings')}>
                    {t('help.settings.body')}
                </Section>

                <Section title={t('help.section.context_menu')}>
                    {t('help.context_menu.body')}
                </Section>

                <Section title={t('help.section.hotkeys')}>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--qs-space-sm)' }}>
                        <Hotkey keys={t('help.hotkey.palette')} />
                        <Hotkey keys={t('help.hotkey.selector')} />
                    </div>
                </Section>
            </div>
        </div>
    );
};

export default HelpPage;