import { useEffect, useState } from 'react';
import { Typography, Table, Switch, Button, Space, Tag, message } from 'antd';
import { ReloadOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import type { PluginInfoDto } from '../types';

const { Title, Text } = Typography;

export default function PluginsPage() {
    const [plugins, setPlugins] = useState<PluginInfoDto[]>([]);
    const [loading, setLoading] = useState(false);

    const loadPlugins = async () => {
        setLoading(true);
        try {
            const result = await invoke<PluginInfoDto[]>('list_plugins');
            setPlugins(result);
        } catch (e) {
            console.error('Failed to load plugins:', e);
            message.error('Failed to load plugins');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        loadPlugins();
    }, []);

    const handleToggle = async (pluginId: string, enabled: boolean) => {
        try {
            await invoke('set_plugin_enabled', { pluginId, enabled });
            setPlugins(prev =>
                prev.map(p => (p.id === pluginId ? { ...p, enabled } : p))
            );
            message.success(`Plugin ${enabled ? 'enabled' : 'disabled'}`);
        } catch (e) {
            console.error('Failed to toggle plugin:', e);
            message.error('Failed to update plugin state');
        }
    };

    const handleRescan = async () => {
        setLoading(true);
        try {
            const result = await invoke<PluginInfoDto[]>('rescan_plugins');
            setPlugins(result);
            message.success(`Found ${result.length} plugins`);
        } catch (e) {
            console.error('Failed to rescan plugins:', e);
            message.error('Failed to rescan plugins');
        } finally {
            setLoading(false);
        }
    };

    const columns = [
        {
            title: 'Name',
            dataIndex: 'name',
            key: 'name',
            render: (text: string) => <Text strong>{text}</Text>,
        },
        {
            title: 'Type',
            dataIndex: 'plugin_type',
            key: 'plugin_type',
            render: (type: string) => {
                const colorMap: Record<string, string> = {
                    Archive: 'blue',
                    Content: 'green',
                    FileSystem: 'orange',
                    Lister: 'purple',
                };
                return <Tag color={colorMap[type] || 'default'}>{type}</Tag>;
            },
        },
        {
            title: 'Version',
            dataIndex: 'version',
            key: 'version',
        },
        {
            title: 'Enabled',
            dataIndex: 'enabled',
            key: 'enabled',
            render: (enabled: boolean, record: PluginInfoDto) => (
                <Switch
                    checked={enabled}
                    onChange={(checked) => handleToggle(record.id, checked)}
                />
            ),
        },
        {
            title: 'Path',
            dataIndex: 'path',
            key: 'path',
            render: (text: string) => (
                <Text type="secondary" style={{ fontSize: 12 }}>
                    {text}
                </Text>
            ),
        },
    ];

    return (
        <div style={{ padding: 24 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
                <Title level={4} style={{ margin: 0 }}>Plugins</Title>
                <Space>
                    <Button
                        icon={<ReloadOutlined />}
                        onClick={handleRescan}
                        loading={loading}
                    >
                        Rescan
                    </Button>
                </Space>
            </div>

            <Table
                dataSource={plugins}
                columns={columns}
                rowKey="id"
                loading={loading}
                pagination={false}
                locale={{ emptyText: 'No plugins found' }}
            />

            <div style={{ marginTop: 16 }}>
                <Text type="secondary">
                    Plugin directory: %APPDATA%/QuickSort/plugins/
                </Text>
            </div>
        </div>
    );
}
