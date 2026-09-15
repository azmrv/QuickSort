export interface Folder {
    id: string;
    name: string;
    path: string;
    favorite: boolean;
    order: number;
    color?: string | null;
    /** Parent folder id for tree view (0.2.6 feature 4f). Null/absent = root folder. */
    parent_id?: string | null;
    stats: {
        use_count: number;
        last_used: string | null;
    };
}

export interface AddFoldersFromPathsResult {
    /** Number of folders successfully added. */
    added: number;
    /** Human-readable reasons for paths that were not added (per path). */
    skipped: string[];
}

export type OperationSource = 'Search' | 'Selector' | 'ContextMenu' | 'Api';

export interface OperationCommand {
    operation_type: 'Move' | 'Copy' | 'Delete' | 'Rename';
    source_paths: string[];
    target_folder_id: string | null;
    target_paths: string[] | null;
    overwrite_policy: 'Skip' | 'Overwrite' | 'AutoRename' | 'Ask';
    duplicate_check_mode: 'name' | 'size' | 'content';
    source?: OperationSource;
    correlation_id?: string | null;
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

// ---------------------------------------------------------------------------
// Dashboard types (0.2.6 feature #19 / plan Q4)
// ---------------------------------------------------------------------------

export interface SystemInfoDto {
    /** Average CPU utilization across all cores (0–100). */
    cpu_usage: number;
    /** Number of logical CPU cores. */
    cpu_cores: number;
    /** Current CPU frequency in MHz (0 if unavailable). */
    cpu_frequency_mhz: number;
    /** CPU die temperature in °C, if available. */
    cpu_temperature: number | null;
    /** Used RAM in bytes. */
    ram_used_bytes: number;
    /** Total physical RAM in bytes. */
    ram_total_bytes: number;
    /** Used disk space across all fixed drives in bytes. */
    disk_used_bytes: number;
    /** Total disk space across all fixed drives in bytes. */
    disk_total_bytes: number;
    /** Disk read throughput in bytes/sec. */
    disk_read_bytes_per_sec: number;
    /** Disk write throughput in bytes/sec. */
    disk_write_bytes_per_sec: number;
    /** Network receive throughput in bytes/sec. */
    network_rx_bytes_per_sec: number;
    /** Network transmit throughput in bytes/sec. */
    network_tx_bytes_per_sec: number;
}
