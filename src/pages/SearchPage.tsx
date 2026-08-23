import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';
import { SearchResult, FileSearchResult } from '../types';

export default function SearchPage() {
    const [query, setQuery] = useState('');
    const [results, setResults] = useState<SearchResult | null>(null);
    const [loading, setLoading] = useState(false);
    const inputRef = useRef<HTMLInputElement>(null);
    const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    useEffect(() => {
        inputRef.current?.focus();
    }, []);

    const doSearch = useCallback(async (text: string) => {
        if (!text.trim()) {
            setResults(null);
            return;
        }
        setLoading(true);
        try {
            const dirs: string[] = await invoke('get_folders_v2').then((folders: any) =>
                folders.map((f: any) => f.path)
            );
            const res = await invoke<SearchResult>('search_files', {
                query: text,
                directories: dirs,
            });
            setResults(res);
        } catch (e) {
            logger.error('SearchPage', `search failed: ${e}`);
        } finally {
            setLoading(false);
        }
    }, []);

    const handleInputChange = (value: string) => {
        setQuery(value);
        if (debounceRef.current) clearTimeout(debounceRef.current);
        debounceRef.current = setTimeout(() => doSearch(value), 300);
    };

    const allItems: FileSearchResult[] = results?.files ?? [];

    const formatSize = (bytes: number): string => {
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
        return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
    };

    const formatTimestamp = (ts: number | null): string => {
        if (!ts) return '';
        return new Date(ts * 1000).toLocaleDateString('ru-RU', {
            day: '2-digit',
            month: '2-digit',
            year: '2-digit',
        });
    };

    const handleSelect = (item: FileSearchResult) => {
        logger.action('SearchPage', `selected: ${item.path}`);
        // TODO: open file / reveal in explorer
    };

    return (
        <div className="page-container" style={{ padding: 'var(--qs-space-lg)' }}>
            <div style={{ marginBottom: 'var(--qs-space-lg)' }}>
                <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 'var(--qs-space-sm)',
                    background: 'var(--qs-bg-secondary)',
                    border: '1px solid var(--qs-border)',
                    borderRadius: 'var(--qs-radius-md)',
                    padding: 'var(--qs-space-sm) var(--qs-space-md)',
                }}>
                    <span style={{ color: 'var(--qs-text-muted)', fontFamily: 'var(--qs-font-mono)' }}>
                        {'>'}_
                    </span>
                    <input
                        ref={inputRef}
                        style={{
                            flex: 1,
                            background: 'none',
                            border: 'none',
                            outline: 'none',
                            color: 'var(--qs-text-primary)',
                            fontFamily: 'var(--qs-font-mono)',
                            fontSize: '14px',
                        }}
                        placeholder="Поиск файлов... (ext:pdf, size:>10mb, folders:)"
                        value={query}
                        onChange={(e) => handleInputChange(e.target.value)}
                        spellCheck={false}
                        autoComplete="off"
                    />
                    {loading && (
                        <span style={{
                            width: '16px',
                            height: '16px',
                            border: '2px solid var(--qs-border)',
                            borderTopColor: 'var(--qs-accent)',
                            borderRadius: '50%',
                            animation: 'spin 0.6s linear infinite',
                        }} />
                    )}
                </div>
                {query && results && (
                    <div style={{
                        marginTop: 'var(--qs-space-xs)',
                        fontSize: '12px',
                        color: 'var(--qs-text-muted)',
                        fontFamily: 'var(--qs-font-mono)',
                    }}>
                        {results.total_count} результатов за {results.search_time_ms}мс
                        {results.truncated && ' (обрезано)'}
                    </div>
                )}
            </div>

            {allItems.length > 0 && (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
                    {allItems.map((item) => (
                        <div
                            key={item.path}
                            onClick={() => handleSelect(item)}
                            style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: 'var(--qs-space-sm)',
                                padding: 'var(--qs-space-xs) var(--qs-space-sm)',
                                borderRadius: 'var(--qs-radius-sm)',
                                cursor: 'pointer',
                                transition: 'background var(--qs-transition-fast)',
                            }}
                            onMouseEnter={(e) => {
                                e.currentTarget.style.background = 'var(--qs-bg-hover)';
                            }}
                            onMouseLeave={(e) => {
                                e.currentTarget.style.background = 'none';
                            }}
                        >
                            <span style={{ fontSize: '16px', width: '24px', textAlign: 'center' }}>
                                {item.is_directory ? '📁' : '📄'}
                            </span>
                            <div style={{ flex: 1, minWidth: 0 }}>
                                <div style={{
                                    fontSize: '13px',
                                    color: 'var(--qs-text-primary)',
                                    overflow: 'hidden',
                                    textOverflow: 'ellipsis',
                                    whiteSpace: 'nowrap',
                                }}>
                                    {item.name}
                                </div>
                                <div style={{
                                    fontSize: '11px',
                                    color: 'var(--qs-text-muted)',
                                    fontFamily: 'var(--qs-font-mono)',
                                    overflow: 'hidden',
                                    textOverflow: 'ellipsis',
                                    whiteSpace: 'nowrap',
                                }}>
                                    {item.path}
                                </div>
                            </div>
                            {item.modified_at && (
                                <span style={{
                                    fontSize: '11px',
                                    color: 'var(--qs-text-muted)',
                                    fontFamily: 'var(--qs-font-mono)',
                                    whiteSpace: 'nowrap',
                                }}>
                                    {formatTimestamp(item.modified_at)}
                                </span>
                            )}
                            <span style={{
                                fontSize: '12px',
                                color: 'var(--qs-text-muted)',
                                fontFamily: 'var(--qs-font-mono)',
                                whiteSpace: 'nowrap',
                            }}>
                                {item.is_directory ? '—' : formatSize(item.size)}
                            </span>
                        </div>
                    ))}
                </div>
            )}

            {query && !loading && results && allItems.length === 0 && (
                <div style={{
                    textAlign: 'center',
                    padding: 'var(--qs-space-2xl)',
                    color: 'var(--qs-text-muted)',
                    fontFamily: 'var(--qs-font-mono)',
                    fontSize: '13px',
                }}>
                    Нет результатов для «{query}»
                </div>
            )}

            {!query && (
                <div style={{
                    textAlign: 'center',
                    padding: 'var(--qs-space-2xl)',
                    color: 'var(--qs-text-muted)',
                    fontSize: '13px',
                }}>
                    Введите запрос для поиска файлов по отслеживаемым папкам.
                    <br />
                    <span style={{ fontFamily: 'var(--qs-font-mono)', fontSize: '12px', marginTop: '8px', display: 'block' }}>
                        Примеры: <code style={{ color: 'var(--qs-accent)' }}>ext:pdf</code>{' '}
                        <code style={{ color: 'var(--qs-accent)' }}>size:&gt;10mb</code>{' '}
                        <code style={{ color: 'var(--qs-accent)' }}>folders:</code>
                    </span>
                </div>
            )}
        </div>
    );
}
