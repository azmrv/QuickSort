import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';
import { SearchResult, FileSearchResult } from '../types';
import { Modal } from 'antd';
import '../styles/CommandPalette.css';

interface Props {
    open: boolean;
    onClose: () => void;
}

export default function CommandPalette({ open, onClose }: Props) {
    const [query, setQuery] = useState('');
    const [results, setResults] = useState<SearchResult | null>(null);
    const [loading, setLoading] = useState(false);
    const [selectedIdx, setSelectedIdx] = useState(0);
    const inputRef = useRef<HTMLInputElement>(null);
    const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    // Focus input on open
    useEffect(() => {
        if (open) {
            setQuery('');
            setResults(null);
            setSelectedIdx(0);
            setTimeout(() => inputRef.current?.focus(), 50);
        }
    }, [open]);

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
            setSelectedIdx(0);
        } catch (e) {
            logger.error('CommandPalette', `search failed: ${e}`);
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
    const maxVisible = 12;
    const visibleItems = allItems.slice(0, maxVisible);

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'ArrowDown') {
            e.preventDefault();
            setSelectedIdx((i) => Math.min(i + 1, visibleItems.length - 1));
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            setSelectedIdx((i) => Math.max(i - 1, 0));
        } else if (e.key === 'Enter' && visibleItems[selectedIdx]) {
            e.preventDefault();
            handleSelect(visibleItems[selectedIdx]);
        } else if (e.key === 'Escape') {
            onClose();
        }
    };

    const handleSelect = (item: FileSearchResult) => {
        logger.action('CommandPalette', `selected: ${item.path}`);
        // TODO: open file in explorer or execute action
        onClose();
    };

    const formatSize = (bytes: number): string => {
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
        return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
    };

    return (
        <Modal
            open={open}
            footer={null}
            closable={false}
            onCancel={onClose}
            width={640}
            styles={{ body: { padding: 0 } }}
            className="command-palette-modal"
        >
            <div className="command-palette">
                <div className="command-palette-input-row">
                    <span className="command-palette-icon">{'>'}_</span>
                    <input
                        ref={inputRef}
                        className="command-palette-input"
                        placeholder="Search files... (ext:pdf, size:>10mb, folders:)"
                        value={query}
                        onChange={(e) => handleInputChange(e.target.value)}
                        onKeyDown={handleKeyDown}
                        spellCheck={false}
                        autoComplete="off"
                    />
                    {loading && <span className="command-palette-spinner" />}
                </div>

                {visibleItems.length > 0 && (
                    <div className="command-palette-results">
                        {visibleItems.map((item, idx) => (
                            <div
                                key={item.path}
                                className={`command-palette-item ${idx === selectedIdx ? 'selected' : ''}`}
                                onClick={() => handleSelect(item)}
                                onMouseEnter={() => setSelectedIdx(idx)}
                            >
                                <span className="command-palette-item-icon">
                                    {item.is_directory ? '📁' : '📄'}
                                </span>
                                <div className="command-palette-item-info">
                                    <span className="command-palette-item-name">{item.name}</span>
                                    <span className="command-palette-item-path">{item.path}</span>
                                </div>
                                <span className="command-palette-item-size">
                                    {item.is_directory ? '—' : formatSize(item.size)}
                                </span>
                            </div>
                        ))}
                        {allItems.length > maxVisible && (
                            <div className="command-palette-overflow">
                                +{allItems.length - maxVisible} more results
                            </div>
                        )}
                    </div>
                )}

                {query && results && !loading && visibleItems.length === 0 && (
                    <div className="command-palette-empty">No results for "{query}"</div>
                )}

                {results && (
                    <div className="command-palette-footer">
                        <span>{results.total_count} results in {results.search_time_ms}ms</span>
                        {results.truncated && <span className="command-palette-truncated">Truncated</span>}
                    </div>
                )}
            </div>
        </Modal>
    );
}
