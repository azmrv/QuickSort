/**
 * Tree view of the folder hierarchy (0.2.6 feature 4f, view #3).
 *
 * Renders folders as a collapsible antd Table grouped by `parent_id`.
 * Column set per spec: name / path / last_used / order / color.
 */

import { Table } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { Folder } from '../types';
import { buildFolderTree, FolderTreeNode } from '../lib/folderTree';
import { useTranslation } from '../i18n/useTranslation';

interface FolderTreeProps {
    folders: Folder[];
}

/** Format a `last_used` timestamp for display; fall back to a label when absent. */
function formatLastUsed(value: string | null, neverLabel: string): string {
    if (!value) return neverLabel;
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;
    return date.toLocaleString();
}

const FolderTree: React.FC<FolderTreeProps> = ({ folders }) => {
    const { t } = useTranslation();
    const treeData = buildFolderTree(folders);

    const columns: ColumnsType<FolderTreeNode> = [
        {
            title: t('folder_tree.col.name'),
            dataIndex: 'name',
            key: 'name',
            render: (name: string, record) => (
                <span style={{ display: 'inline-flex', alignItems: 'center', gap: 8 }}>
                    {record.color && (
                        <span
                            aria-hidden
                            style={{
                                width: 10,
                                height: 10,
                                borderRadius: '50%',
                                background: record.color,
                                display: 'inline-block',
                            }}
                        />
                    )}
                    {name}
                </span>
            ),
        },
        {
            title: t('folder_tree.col.path'),
            dataIndex: 'path',
            key: 'path',
        },
        {
            title: t('folder_tree.col.last_used'),
            key: 'last_used',
            width: 180,
            render: (_, record) => formatLastUsed(record.stats.last_used, t('folder_tree.last_used_never')),
        },
        {
            title: t('folder_tree.col.order'),
            dataIndex: 'order',
            key: 'order',
            width: 80,
        },
        {
            title: t('folder_tree.col.color'),
            key: 'color',
            width: 80,
            render: (_, record) =>
                record.color ? (
                    <span
                        aria-hidden
                        style={{
                            width: 14,
                            height: 14,
                            borderRadius: '50%',
                            background: record.color,
                            display: 'inline-block',
                            border: '1px solid rgba(128,128,128,0.35)',
                        }}
                    />
                ) : null,
        },
    ];

    if (folders.length === 0) {
        return (
            <div className="empty-state">
                <div className="empty-state-icon">📁</div>
                <div className="empty-state-title">{t('folder_list.empty_title')}</div>
                <div className="empty-state-description">
                    {t('folder_list.empty_description')}
                </div>
            </div>
        );
    }

    return (
        <Table<FolderTreeNode>
            rowKey="id"
            columns={columns}
            dataSource={treeData}
            pagination={false}
            size="middle"
            expandable={{ defaultExpandAllRows: true }}
        />
    );
};

export default FolderTree;