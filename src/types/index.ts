export interface Folder {
    id: string;          // UUID в виде строки
    name: string;
    path: string;
    favorite: boolean;
    order: number;
    stats: {
        use_count: number;
        last_used: string | null;  // ISO8601
    };
}