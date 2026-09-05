import { useEffect, useState } from 'react';
import { invoke } from '../lib/invoke';
import { logger } from '../lib/logger';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from '../i18n/useTranslation';

type QueueState = 'free' | 'working' | 'waiting' | 'done';

interface QueueJob {
    status: string;
}

const HeaderStatus: React.FC = () => {
    const { t } = useTranslation();
    const [comActive, setComActive] = useState(false);
    const [queueState, setQueueState] = useState<QueueState>('free');

    useEffect(() => {
        invoke<boolean>('check_menu_status')
            .then((active) => {
                setComActive(active);
                logger.info('HeaderStatus', `menu: ${active ? 'active' : 'inactive'}`);
            })
            .catch((err) => logger.error('HeaderStatus', 'check_menu_status failed', err));

        const unlistenCom = listen<string>('com-status', (event) => {
            const status = event.payload;
            logger.info('HeaderStatus', `COM status event: ${status}`);
            setComActive(status === 'active');
        });

        const refreshQueue = () => {
            invoke<QueueJob[]>('get_jobs')
                .then((jobs) => {
                    let next: QueueState = 'free';
                    if (jobs.length === 0) {
                        next = 'free';
                    } else if (jobs.some((j) => j.status === 'Running')) {
                        next = 'working';
                    } else if (jobs.some((j) => j.status === 'Queued')) {
                        next = 'waiting';
                    } else {
                        next = 'done';
                    }
                    setQueueState(next);
                    logger.info('HeaderStatus', `queue state: ${next} (${jobs.length} jobs)`);
                })
                .catch((err) => logger.error('HeaderStatus', 'get_jobs failed', err));
        };
        refreshQueue();
        const unlistenJob = listen('job-status', refreshQueue);

        return () => {
            unlistenCom.then((fn) => fn());
            unlistenJob.then((fn) => fn());
        };
    }, []);

    const dotClassName = (variant: 'ok' | 'warn' | 'err') =>
        variant === 'warn' ? 'status-dot warning' : variant === 'err' ? 'status-dot error' : 'status-dot';

    const queueVariant: 'ok' | 'warn' | 'err' =
        queueState === 'waiting' ? 'warn' : queueState === 'working' || queueState === 'done' || queueState === 'free' ? 'ok' : 'err';

    const queueColor = queueState === 'working' ? 'var(--qs-success)' : queueState === 'done' ? 'var(--qs-text-muted)' : 'var(--qs-accent)';

    const itemStyle: React.CSSProperties = {
        display: 'flex',
        alignItems: 'center',
        gap: '6px',
        fontFamily: 'var(--qs-font-mono)',
        fontSize: '12px',
        color: 'var(--qs-text-secondary)',
        whiteSpace: 'nowrap',
    };

    return (
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--qs-space-md)', flexShrink: 0 }}>
            <span style={itemStyle}>
                <span className={dotClassName(comActive ? 'ok' : 'err')} />
                <span>{t('status.context_menu')}</span>
                <span style={{ color: comActive ? 'var(--qs-success)' : 'var(--qs-danger)', fontWeight: 600 }}>
                    {comActive ? t('status.active') : t('status.inactive')}
                </span>
            </span>
            <span style={itemStyle}>
                <span className={dotClassName(queueVariant)} />
                <span>{t('status.queue')}</span>
                <span style={{ color: queueColor, fontWeight: 600, textTransform: 'capitalize' }}>
                    {t(`status.queue_state.${queueState}`)}
                </span>
            </span>
        </div>
    );
};

export default HeaderStatus;