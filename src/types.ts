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
