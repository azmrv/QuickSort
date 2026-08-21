export interface Folder {
    id: string;
    name: string;
    path: string;
    favorite: boolean;
    order: number;
    stats: {
        use_count: number;
        last_used: string | null;
    };
}

export interface OperationCommand {
    operation_type: 'Move' | 'Copy' | 'Delete' | 'Rename';
    source_paths: string[];
    target_folder_id: string | null;
    target_paths: string[] | null;
    overwrite_policy: 'Skip' | 'Overwrite' | 'AutoRename' | 'Ask';
}

export interface OperationResult {
    operation_id: string;
    state: 'Pending' | 'Executing' | 'Completed' | 'Failed' | 'Undone';
    processed_files: number;
    bytes_moved: number;
}

export type ConflictResolution = 'Skip' | 'AddWithTimestamp' | 'Replace' | 'Rename' | 'Cancel' | 'Ask';

export interface ConflictContext {
    remembered: ConflictResolution | null;
    is_chosen: boolean;
    files_processed: number;
    files_skipped: number;
    files_renamed: number;
    files_overwritten: number;
}

// ---------------------------------------------------------------------------
// Plugin types
// ---------------------------------------------------------------------------

export type PluginType = 'Archive' | 'Content' | 'FileSystem' | 'Lister';

export interface PluginInfoDto {
    id: string;
    name: string;
    version: string;
    plugin_type: PluginType;
    enabled: boolean;
    path: string;
}

export interface PluginConfig {
    enabled: boolean;
    priority: number;
    custom_settings: Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// Search types
// ---------------------------------------------------------------------------

export interface FileSearchResult {
    path: string;
    name: string;
    size: number;
    is_directory: boolean;
    modified_at: number | null;
}

export interface SearchResult {
    files: FileSearchResult[];
    total_count: number;
    search_time_ms: number;
    truncated: boolean;
}
