import { useEffect, useRef, useState } from 'react';
import { Box, RingProgress, Text, Group, Center } from '@mantine/core';
import { AreaChart } from '@mantine/charts';
import { invoke } from '../lib/invoke';
import { useTranslation } from '../i18n/useTranslation';
import type { SystemInfoDto } from '../types';

/**
 * Dashboard panel (0.2.6 feature #19 / plan Q4): live system resource
 * snapshot shown inside the Search tab while the query is empty.
 *
 * Polls `get_system_info` through IPC once per second. Widgets use Mantine
 * UI (`RingProgress`, `AreaChart`) with a neon-on-dark appearance. The
 * traffic-light indicator follows the plan thresholds: green ≤50%, yellow
 * 50–80%, red >80%, with critical forcing red when CPU >85% (or >80°C),
 * RAM >90% or disk >95%.
 */

interface NetworkSample {
    /** Sample index used as the X axis label. */
    t: string;
    /** Receive throughput in MB/s (rounded to 2 decimals). */
    rx: number;
    /** Transmit throughput in MB/s (rounded to 2 decimals). */
    tx: number;
}

const TRAFFIC_GREEN = 50;
const TRAFFIC_YELLOW = 80;
const CRITICAL_CPU = 85;
const CRITICAL_CPU_TEMP_C = 80;
const CRITICAL_RAM = 90;
const CRITICAL_DISK = 95;

const HISTORY_POINTS = 30;
const POLL_INTERVAL_MS = 1000;

function trafficColor(usagePercent: number): string {
    if (usagePercent <= TRAFFIC_GREEN) return 'var(--qs-success, #22c55e)';
    if (usagePercent <= TRAFFIC_YELLOW) return 'var(--qs-warn, #f59e0b)';
    return 'var(--qs-danger, #ef4444)';
}

function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function formatThroughput(bytesPerSec: number): string {
    const mbps = bytesPerSec / (1024 * 1024);
    if (mbps < 0.01) return '0 MB/s';
    return `${mbps.toFixed(2)} MB/s`;
}

function percent(used: number, total: number): number {
    return total > 0 ? (used / total) * 100 : 0;
}

type OverallStatus =
    | 'ok'
    | 'warn'
    | 'critical';

function computeStatus(
    info: SystemInfoDto | null,
    ramPct: number,
    diskPct: number,
): { status: OverallStatus; color: string } {
    if (!info) return { status: 'ok', color: trafficColor(0) };

    const cpuPct = info.cpu_usage;
    const cpuTempCritical = info.cpu_temperature !== null && info.cpu_temperature > CRITICAL_CPU_TEMP_C;

    // Critical limit forces red regardless of the 50/80 thresholds.
    if (cpuPct > CRITICAL_CPU || cpuTempCritical || ramPct > CRITICAL_RAM || diskPct > CRITICAL_DISK) {
        return { status: 'critical', color: trafficColor(101) };
    }

    const worst = Math.max(cpuPct, ramPct, diskPct);
    if (worst > TRAFFIC_YELLOW) return { status: 'warn', color: trafficColor(worst) };
    return { status: 'ok', color: trafficColor(worst) };
}

const ringProps = {
    size: 110,
    thickness: 10,
    roundCaps: true,
};

const cardStyle: React.CSSProperties = {
    background: 'var(--qs-bg-tertiary)',
    border: '1px solid var(--qs-border)',
    borderRadius: 'var(--qs-radius-md)',
    padding: 'var(--qs-space-md)',
    minWidth: '200px',
    flex: 1,
};

const metricLabelStyle: React.CSSProperties = {
    fontFamily: 'var(--qs-font-display)',
    fontSize: '11px',
    fontWeight: 600,
    letterSpacing: '1px',
    textTransform: 'uppercase',
    color: 'var(--qs-text-muted)',
};

const metricValueStyle: React.CSSProperties = {
    fontFamily: 'var(--qs-font-mono)',
    fontSize: '12px',
    color: 'var(--qs-text-secondary)',
};

const DashboardPanel = () => {
    const { t } = useTranslation();
    const [info, setInfo] = useState<SystemInfoDto | null>(null);
    const [networkSamples, setNetworkSamples] = useState<NetworkSample[]>([]);
    const counterRef = useRef(0);

    useEffect(() => {
        const poll = async () => {
            try {
                const snapshot = await invoke<SystemInfoDto>('get_system_info');
                setInfo(snapshot);

                counterRef.current += 1;
                const idx = counterRef.current;
                setNetworkSamples((prior) => {
                    const next = [
                        ...prior,
                        {
                            t: String(idx),
                            rx: Number((snapshot.network_rx_bytes_per_sec / (1024 * 1024)).toFixed(2)),
                            tx: Number((snapshot.network_tx_bytes_per_sec / (1024 * 1024)).toFixed(2)),
                        },
                    ];
                    return next.length > HISTORY_POINTS ? next.slice(next.length - HISTORY_POINTS) : next;
                });
            } catch (err) {
                // Keep the last good snapshot; the interval retries.
                console.error('Dashboard: get_system_info failed', err);
            }
        };

        poll();
        const id = setInterval(poll, POLL_INTERVAL_MS);
        return () => clearInterval(id);
    }, []);

    const ramPct = info ? percent(info.ram_used_bytes, info.ram_total_bytes) : 0;
    const diskPct = info ? percent(info.disk_used_bytes, info.disk_total_bytes) : 0;
    const { status, color } = computeStatus(info, ramPct, diskPct);

    const statusKey = status === 'ok'
        ? 'dashboard.status.ok'
        : status === 'warn'
            ? 'dashboard.status.warn'
            : 'dashboard.status.critical';

    const cpuSpeedText = info && info.cpu_frequency_mhz > 0 ? `${(info.cpu_frequency_mhz / 1000).toFixed(2)} GHz` : '—';
    const cpuTempText = info && info.cpu_temperature !== null ? `${info.cpu_temperature.toFixed(0)}°C` : '—';

    const MetricRow = ({ label, value }: { label: string; value: string }) => (
        <Group gap={6} justify="space-between" style={{ width: '100%' }}>
            <Text style={metricLabelStyle}>{label}</Text>
            <Text style={metricValueStyle}>{value}</Text>
        </Group>
    );

    return (
        <Box>
            <Center mb="md">
                <Group gap="sm">
                    <div
                        style={{
                            width: '14px',
                            height: '14px',
                            borderRadius: '50%',
                            background: color,
                            boxShadow: `0 0 12px ${color}`,
                        }}
                    />
                    <Text
                        style={{
                            fontFamily: 'var(--qs-font-display)',
                            fontSize: '13px',
                            fontWeight: 600,
                            letterSpacing: '1px',
                            textTransform: 'uppercase',
                            color: 'var(--qs-text-primary)',
                        }}
                    >
                        {t(statusKey)}
                    </Text>
                </Group>
            </Center>

            <div style={{ display: 'flex', gap: 'var(--qs-space-md)', flexWrap: 'wrap' }}>
                {/* CPU */}
                <div style={cardStyle}>
                    <Text style={metricLabelStyle} mb="sm">{t('dashboard.cpu')}</Text>
                    <Center>
                        <RingProgress
                            {...ringProps}
                            sections={[{ value: Math.round(info?.cpu_usage ?? 0), color }]}
                            label={
                                <Text ta="center" style={metricValueStyle}>
                                    {Math.round(info?.cpu_usage ?? 0)}%
                                </Text>
                            }
                        />
                    </Center>
                    <div style={{ marginTop: 'var(--qs-space-sm)', display: 'flex', flexDirection: 'column', gap: 4 }}>
                        <MetricRow label={t('dashboard.cores')} value={`${info?.cpu_cores ?? 0}`} />
                        <MetricRow label="⚡" value={cpuSpeedText} />
                        <MetricRow label={t('dashboard.temp')} value={cpuTempText} />
                    </div>
                </div>

                {/* RAM */}
                <div style={cardStyle}>
                    <Text style={metricLabelStyle} mb="sm">{t('dashboard.ram')}</Text>
                    <Center>
                        <RingProgress
                            {...ringProps}
                            sections={[{ value: Math.round(ramPct), color }]}
                            label={
                                <Text ta="center" style={metricValueStyle}>
                                    {Math.round(ramPct)}%
                                </Text>
                            }
                        />
                    </Center>
                    <div style={{ marginTop: 'var(--qs-space-sm)', display: 'flex', flexDirection: 'column', gap: 4 }}>
                        <MetricRow label="used" value={info ? formatBytes(info.ram_used_bytes) : '—'} />
                        <MetricRow label="total" value={info ? formatBytes(info.ram_total_bytes) : '—'} />
                    </div>
                </div>

                {/* Disk */}
                <div style={cardStyle}>
                    <Text style={metricLabelStyle} mb="sm">{t('dashboard.disk')}</Text>
                    <Center>
                        <RingProgress
                            {...ringProps}
                            sections={[{ value: Math.round(diskPct), color: trafficColor(diskPct) }]}
                            label={
                                <Text ta="center" style={metricValueStyle}>
                                    {Math.round(diskPct)}%
                                </Text>
                            }
                        />
                    </Center>
                    <div style={{ marginTop: 'var(--qs-space-sm)', display: 'flex', flexDirection: 'column', gap: 4 }}>
                        <MetricRow label="used" value={info ? formatBytes(info.disk_used_bytes) : '—'} />
                        <MetricRow label="total" value={info ? formatBytes(info.disk_total_bytes) : '—'} />
                        <MetricRow
                            label={t('dashboard.read')}
                            value={info ? formatThroughput(info.disk_read_bytes_per_sec) : '—'}
                        />
                        <MetricRow
                            label={t('dashboard.write')}
                            value={info ? formatThroughput(info.disk_write_bytes_per_sec) : '—'}
                        />
                    </div>
                </div>

                {/* Network */}
                <div style={{ ...cardStyle, minWidth: '260px' }}>
                    <Text style={metricLabelStyle} mb="sm">{t('dashboard.network')}</Text>
                    <AreaChart
                        h={110}
                        data={networkSamples.length > 0 ? networkSamples : [{ t: '0', rx: 0, tx: 0 }]}
                        dataKey="t"
                        series={[
                            { name: 'rx', color: 'var(--qs-accent, #f59e0b)' },
                            { name: 'tx', color: '#22c55e' },
                        ]}
                        withXAxis={false}
                        withYAxis={false}
                        withTooltip
                        gridAxis="none"
                        curveType="monotone"
                    />
                    <div style={{ marginTop: 'var(--qs-space-sm)', display: 'flex', flexDirection: 'column', gap: 4 }}>
                        <MetricRow
                            label={`${t('dashboard.rx')} (MB/s)`}
                            value={info ? formatThroughput(info.network_rx_bytes_per_sec) : '—'}
                        />
                        <MetricRow
                            label={`${t('dashboard.tx')} (MB/s)`}
                            value={info ? formatThroughput(info.network_tx_bytes_per_sec) : '—'}
                        />
                    </div>
                </div>
            </div>
        </Box>
    );
};

export default DashboardPanel;