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