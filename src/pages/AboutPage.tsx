import { useState, useEffect, type CSSProperties, type ReactNode } from 'react';
import { invoke } from '../lib/invoke';
import { message } from 'antd';

interface Person {
    name: string;
    role: string;
    url: string | null;
}

interface Credit {
    name: string;
    url: string;
    reason: string;
}

interface Dependency {
    name: string;
    version: string;
    license: string;
    url: string;
    description: string;
}

interface DonationLink {
    platform: string;
    url: string;
    label: string;
}

interface ExternalResource {
    name: string;
    url: string;
    description: string;
    resource_type: string;
}

interface AppMetadata {
    name: string;
    version: string;
    description: string;
    license: string;
    repository: string;
    homepage: string;
    issues: string;
    discussions: string;
    telegram: string;
    authors: Person[];
    contributors: Person[];
    credits: Credit[];
    dependencies: Dependency[];
    donation_links: DonationLink[];
    external_resources: ExternalResource[];
}

const chipLinkStyle: CSSProperties = {
    padding: 'var(--qs-space-sm) var(--qs-space-md)',
    background: 'var(--qs-bg-tertiary)',
    border: '1px solid var(--qs-border)',
    borderRadius: 'var(--qs-radius-md)',
    color: 'var(--qs-text-secondary)',
    fontFamily: 'var(--qs-font-mono)',
    fontSize: '12px',
    textDecoration: 'none',
    transition: 'all var(--qs-transition-fast)',
};

const sectionStyle: CSSProperties = {
    width: '100%',
    maxWidth: '560px',
    marginTop: 'var(--qs-space-xl)',
    textAlign: 'left',
};

const sectionTitleStyle: CSSProperties = {
    fontFamily: 'var(--qs-font-display)',
    fontSize: '12px',
    fontWeight: 600,
    letterSpacing: '1px',
    textTransform: 'uppercase',
    color: 'var(--qs-text-muted)',
    marginBottom: 'var(--qs-space-md)',
};

const cardStyle: CSSProperties = {
    background: 'var(--qs-bg-tertiary)',
    border: '1px solid var(--qs-border)',
    borderRadius: 'var(--qs-radius-md)',
    padding: 'var(--qs-space-sm) var(--qs-space-md)',
};

const textLinkStyle: CSSProperties = {
    color: 'var(--qs-accent)',
    textDecoration: 'none',
};

const tagTextStyle: CSSProperties = {
    fontFamily: 'var(--qs-font-mono)',
    fontSize: '10px',
    letterSpacing: '0.5px',
    textTransform: 'uppercase',
    color: 'var(--qs-text-muted)',
};

const ChipLink = ({ href, children }: { href: string; children: ReactNode }) => (
    <a
        href={href}
        onClick={(e) => {
            e.preventDefault();
            navigator.clipboard.writeText(href).then(() => {
                message.success('Ссылка скопирована');
            });
        }}
        style={chipLinkStyle}
        onMouseEnter={(e) => {
            e.currentTarget.style.borderColor = 'var(--qs-accent)';
            e.currentTarget.style.color = 'var(--qs-accent)';
        }}
        onMouseLeave={(e) => {
            e.currentTarget.style.borderColor = 'var(--qs-border)';
            e.currentTarget.style.color = 'var(--qs-text-secondary)';
        }}
    >
        {children}
    </a>
);

const PersonRow = ({ person }: { person: Person }) => (
    <div
        style={{
            ...cardStyle,
            display: 'flex',
            alignItems: 'baseline',
            justifyContent: 'space-between',
            gap: 'var(--qs-space-md)',
        }}
    >
        <span style={{ color: 'var(--qs-text-primary)', fontWeight: 500 }}>
            {person.url ? (
                <a
                    href={person.url}
                    onClick={(e) => {
                        e.preventDefault();
                        navigator.clipboard.writeText(person.url!).then(() => {
                            message.success('Ссылка скопирована');
                        });
                    }}
                    style={textLinkStyle}
                >
                    {person.name}
                </a>
            ) : (
                person.name
            )}
        </span>
        <span style={{ ...tagTextStyle, fontSize: '11px' }}>{person.role}</span>
    </div>
);

const AboutPage = () => {
    const [metadata, setMetadata] = useState<AppMetadata | null>(null);
    const [showDependencies, setShowDependencies] = useState(false);

    useEffect(() => {
        invoke<AppMetadata>('get_app_metadata').then(setMetadata);
    }, []);

    if (!metadata) {
        return (
            <div style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                minHeight: '200px',
                padding: 'var(--qs-space-2xl)',
                fontFamily: 'var(--qs-font-mono)',
                fontSize: '13px',
                color: 'var(--qs-text-muted)',
            }}>
                Загрузка...
            </div>
        );
    }

    return (
        <div style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
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
                {metadata.name.charAt(0)}
            </div>

            <h2 style={{
                fontFamily: 'var(--qs-font-display)',
                fontSize: '24px',
                fontWeight: 600,
                color: 'var(--qs-text-primary)',
                marginBottom: 'var(--qs-space-sm)',
            }}>
                {metadata.name}
            </h2>

            <div style={{
                fontFamily: 'var(--qs-font-mono)',
                fontSize: '13px',
                color: 'var(--qs-text-muted)',
                marginBottom: 'var(--qs-space-lg)',
            }}>
                v{metadata.version}
            </div>

            <p style={{
                color: 'var(--qs-text-secondary)',
                maxWidth: '400px',
                lineHeight: 1.6,
                margin: 0,
            }}>
                {metadata.description}
            </p>

            {(metadata.repository || metadata.homepage) && (
                <div style={{
                    display: 'flex',
                    gap: 'var(--qs-space-md)',
                    flexWrap: 'wrap',
                    justifyContent: 'center',
                    marginTop: 'var(--qs-space-lg)',
                }}>
                    {metadata.repository && (
                        <ChipLink href={metadata.repository}>Репозиторий</ChipLink>
                    )}
                    {metadata.homepage && (
                        <ChipLink href={metadata.homepage}>Сайт</ChipLink>
                    )}
                </div>
            )}

            <section style={sectionStyle}>
                <div style={sectionTitleStyle}>Лицензия</div>
                <div style={{
                    ...cardStyle,
                    fontFamily: 'var(--qs-font-mono)',
                    fontSize: '12px',
                    color: 'var(--qs-text-secondary)',
                }}>
                    {metadata.license}
                </div>
            </section>

            {(metadata.authors.length > 0 || metadata.contributors.length > 0) && (
                <section style={sectionStyle}>
                    <div style={sectionTitleStyle}>Авторы</div>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--qs-space-sm)' }}>
                        {metadata.authors.map((person) => (
                            <PersonRow
                                key={`${person.name}-${person.role}`}
                                person={person}
                            />
                        ))}
                    </div>
                    {metadata.contributors.length > 0 && (
                        <>
                            <div style={{
                                ...sectionTitleStyle,
                                margin: 'var(--qs-space-lg) 0 var(--qs-space-sm)',
                            }}>
                                Контрибьюторы
                            </div>
                            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--qs-space-sm)' }}>
                                {metadata.contributors.map((person) => (
                                    <PersonRow
                                        key={`${person.name}-${person.role}`}
                                        person={person}
                                    />
                                ))}
                            </div>
                        </>
                    )}
                </section>
            )}

            {metadata.credits.length > 0 && (
                <section style={sectionStyle}>
                    <div style={sectionTitleStyle}>Благодарности</div>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--qs-space-sm)' }}>
                        {metadata.credits.map((credit) => (
                            <div key={credit.name} style={cardStyle}>
                                <div style={{ marginBottom: '4px' }}>
                                    <a
                                        href={credit.url}
                                        onClick={(e) => {
                                            e.preventDefault();
                                            navigator.clipboard.writeText(credit.url).then(() => {
                                                message.success('Ссылка скопирована');
                                            });
                                        }}
                                        style={{ ...textLinkStyle, fontWeight: 500 }}
                                    >
                                        {credit.name}
                                    </a>
                                </div>
                                <div style={{
                                    color: 'var(--qs-text-secondary)',
                                    fontSize: '12px',
                                    lineHeight: 1.5,
                                }}>
                                    {credit.reason}
                                </div>
                            </div>
                        ))}
                    </div>
                </section>
            )}

            {metadata.donation_links.length > 0 && (
                <section style={sectionStyle}>
                    <div style={sectionTitleStyle}>Поддержать проект</div>
                    <div style={{
                        display: 'flex',
                        gap: 'var(--qs-space-md)',
                        flexWrap: 'wrap',
                    }}>
                        {metadata.donation_links.map((link) => (
                            <ChipLink key={link.platform} href={link.url}>
                                {link.label}
                            </ChipLink>
                        ))}
                    </div>
                </section>
            )}

            <section style={sectionStyle}>
                <div style={sectionTitleStyle}>Обратная связь</div>
                <div style={{
                    display: 'flex',
                    flexDirection: 'column',
                    gap: 'var(--qs-space-sm)',
                }}>
                    {metadata.issues && (
                        <div style={cardStyle}>
                            <div style={{ marginBottom: '4px' }}>
                                <a
                                    href={metadata.issues}
                                    onClick={(e) => {
                                        e.preventDefault();
                                        navigator.clipboard.writeText(metadata.issues).then(() => {
                                            message.success('Ссылка скопирована');
                                        });
                                    }}
                                    style={{ ...textLinkStyle, fontWeight: 500 }}
                                >
                                    GitHub Issues
                                </a>
                            </div>
                            <div style={{
                                color: 'var(--qs-text-secondary)',
                                fontSize: '12px',
                                lineHeight: 1.5,
                            }}>
                                Сообщения о багах и предложения по улучшению
                            </div>
                        </div>
                    )}
                    {metadata.discussions && (
                        <div style={cardStyle}>
                            <div style={{ marginBottom: '4px' }}>
                                <a
                                    href={metadata.discussions}
                                    onClick={(e) => {
                                        e.preventDefault();
                                        navigator.clipboard.writeText(metadata.discussions).then(() => {
                                            message.success('Ссылка скопирована');
                                        });
                                    }}
                                    style={{ ...textLinkStyle, fontWeight: 500 }}
                                >
                                    GitHub Discussions
                                </a>
                            </div>
                            <div style={{
                                color: 'var(--qs-text-secondary)',
                                fontSize: '12px',
                                lineHeight: 1.5,
                            }}>
                                Обсуждение функционала и идей
                            </div>
                        </div>
                    )}
                    {metadata.telegram && (
                        <div style={cardStyle}>
                            <div style={{ marginBottom: '4px' }}>
                                <a
                                    href={metadata.telegram}
                                    onClick={(e) => {
                                        e.preventDefault();
                                        navigator.clipboard.writeText(metadata.telegram).then(() => {
                                            message.success('Ссылка скопирована');
                                        });
                                    }}
                                    style={{ ...textLinkStyle, fontWeight: 500 }}
                                >
                                    Telegram @Fib511
                                </a>
                            </div>
                            <div style={{
                                color: 'var(--qs-text-secondary)',
                                fontSize: '12px',
                                lineHeight: 1.5,
                            }}>
                                Прямая связь с автором
                            </div>
                        </div>
                    )}
                </div>
            </section>

            {metadata.external_resources.length > 0 && (
                <section style={sectionStyle}>
                    <div style={sectionTitleStyle}>Внешние ресурсы</div>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--qs-space-sm)' }}>
                        {metadata.external_resources.map((resource) => (
                            <div key={resource.name} style={cardStyle}>
                                <div style={{
                                    display: 'flex',
                                    alignItems: 'baseline',
                                    justifyContent: 'space-between',
                                    gap: 'var(--qs-space-sm)',
                                    marginBottom: '4px',
                                }}>
                                    <a
                                        href={resource.url}
                                        onClick={(e) => {
                                            e.preventDefault();
                                            navigator.clipboard.writeText(resource.url).then(() => {
                                                message.success('Ссылка скопирована');
                                            });
                                        }}
                                        style={{ ...textLinkStyle, fontWeight: 500 }}
                                    >
                                        {resource.name}
                                    </a>
                                    <span style={tagTextStyle}>{resource.resource_type}</span>
                                </div>
                                <div style={{
                                    color: 'var(--qs-text-secondary)',
                                    fontSize: '12px',
                                    lineHeight: 1.5,
                                }}>
                                    {resource.description}
                                </div>
                            </div>
                        ))}
                    </div>
                </section>
            )}

            {metadata.dependencies.length > 0 && (
                <section style={sectionStyle}>
                    <button
                        onClick={() => setShowDependencies((open) => !open)}
                        style={{
                            width: '100%',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'space-between',
                            padding: 'var(--qs-space-sm) var(--qs-space-md)',
                            background: 'var(--qs-bg-tertiary)',
                            border: '1px solid var(--qs-border)',
                            borderRadius: 'var(--qs-radius-md)',
                            color: 'var(--qs-text-secondary)',
                            fontFamily: 'var(--qs-font-mono)',
                            fontSize: '12px',
                            cursor: 'pointer',
                            transition: 'all var(--qs-transition-fast)',
                        }}
                    >
                        <span>Зависимости ({metadata.dependencies.length})</span>
                        <span>{showDependencies ? '▾' : '▸'}</span>
                    </button>
                    {showDependencies && (
                        <div style={{
                            marginTop: 'var(--qs-space-sm)',
                            display: 'flex',
                            flexDirection: 'column',
                            gap: 'var(--qs-space-sm)',
                        }}>
                            {metadata.dependencies.map((dependency) => (
                                <div
                                    key={dependency.name}
                                    title={dependency.description}
                                    style={{
                                        ...cardStyle,
                                        display: 'flex',
                                        alignItems: 'baseline',
                                        gap: 'var(--qs-space-sm)',
                                    }}
                                >
                                    <a
                                        href={dependency.url}
                                        onClick={(e) => {
                                            e.preventDefault();
                                            navigator.clipboard.writeText(dependency.url).then(() => {
                                                message.success('Ссылка скопирована');
                                            });
                                        }}
                                        style={textLinkStyle}
                                    >
                                        {dependency.name}
                                    </a>
                                    <span style={{
                                        fontFamily: 'var(--qs-font-mono)',
                                        fontSize: '11px',
                                        color: 'var(--qs-text-muted)',
                                    }}>
                                        {dependency.version}
                                    </span>
                                    <span style={{ ...tagTextStyle, marginLeft: 'auto' }}>
                                        {dependency.license}
                                    </span>
                                </div>
                            ))}
                        </div>
                    )}
                </section>
            )}
        </div>
    );
};

export default AboutPage;
