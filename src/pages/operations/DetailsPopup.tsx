import { Modal } from 'antd';
import { useTranslation } from '../../i18n/useTranslation';
import { formatBytes } from './rowModel';
import { getSourceLabel, getStatusLabel, getTypeLabel } from './labels';
import type { OperationRow } from './types';

interface DetailsPopupProps {
    row: OperationRow | null;
    onClose: () => void;
}

const fieldLabelStyle: React.CSSProperties = {
    fontFamily: 'var(--qs-font-mono)',
    fontSize: '11px',
    textTransform: 'uppercase',
    letterSpacing: '0.04em',
    color: 'var(--qs-text-muted)',
    marginBottom: '4px',
};

const fieldValueStyle: React.CSSProperties = {
    fontFamily: 'var(--qs-font-mono)',
    fontSize: '12px',
    color: 'var(--qs-text-primary)',
    wordBreak: 'break-all',
};

const DetailsPopup = ({ row, onClose }: DetailsPopupProps) => {
    const { t } = useTranslation();
    if (!row) return null;

    const progress = row.progress;
    const pct =
        progress !== null && progress.total > 0
            ? Math.min(100, Math.round((progress.current / progress.total) * 100))
            : 0;

    return (
        <Modal
            open
            title={t('operations.details.title')}
            onCancel={onClose}
            width={560}
            destroyOnHidden
            footer={[
                <button
                    key="close"
                    onClick={onClose}
                    style={{
                        padding: '6px 16px',
                        background: 'transparent',
                        border: '1px solid var(--qs-border)',
                        borderRadius: 'var(--qs-radius-sm)',
                        color: 'var(--qs-accent)',
                        fontFamily: 'var(--qs-font-mono)',
                        fontSize: '12px',
                        cursor: 'pointer',
                    }}
                >
                    {t('operations.details.close')}
                </button>,
            ]}
        >
            <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
                    <div>
                        <div style={fieldLabelStyle}>{t('operations.col.type')}</div>
                        <div style={fieldValueStyle}>{getTypeLabel(row, t)}</div>
                    </div>
                    <div>
                        <div style={fieldLabelStyle}>{t('operations.col.source')}</div>
                        <div style={fieldValueStyle}>{getSourceLabel(row, t)}</div>
                    </div>
                </div>

                <div>
                    <div style={fieldLabelStyle}>{t('operations.col.status')}</div>
                    <div style={fieldValueStyle}>{getStatusLabel(row, t)}</div>
                </div>

                <div>
                    <div style={fieldLabelStyle}>{t('operations.col.date')}</div>
                    <div style={fieldValueStyle}>{new Date(row.createdAtMs).toLocaleString()}</div>
                </div>

                {row.sizeBytes > 0 && (
                    <div>
                        <div style={fieldLabelStyle}>{t('operations.col.size')}</div>
                        <div style={fieldValueStyle}>
                            {formatBytes(row.sizeBytes)}
                            {row.filesCount > 1 ? `  (${row.filesCount})` : ''}
                        </div>
                    </div>
                )}

                {progress !== null && (
                    <div>
                        <div style={fieldLabelStyle}>{t('operations.details.progress')}</div>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                            <div style={{
                                flex: 1,
                                height: '8px',
                                background: 'var(--qs-bg-tertiary)',
                                borderRadius: '4px',
                                overflow: 'hidden',
                            }}>
                                <div style={{
                                    width: `${pct}%`,
                                    height: '100%',
                                    background: 'var(--qs-accent)',
                                    transition: 'width .2s',
                                }} />
                            </div>
                            <span style={{
                                fontFamily: 'var(--qs-font-mono)',
                                fontSize: '11px',
                                color: 'var(--qs-text-secondary)',
                                whiteSpace: 'nowrap',
                            }}>
                                {progress.current}/{progress.total}
                            </span>
                        </div>
                    </div>
                )}

                {row.target !== null && (
                    <div>
                        <div style={fieldLabelStyle}>{t('operations.details.target')}</div>
                        <div style={fieldValueStyle}>{row.target}</div>
                    </div>
                )}

                {row.error !== null && (
                    <div>
                        <div style={fieldLabelStyle}>{t('operations.details.error')}</div>
                        <div style={{ ...fieldValueStyle, color: 'var(--qs-danger, #ef4444)' }}>
                            {row.error}
                        </div>
                    </div>
                )}

                <div>
                    <div style={fieldLabelStyle}>{t('operations.details.files')}</div>
                    <div style={{
                        maxHeight: '200px',
                        overflowY: 'auto',
                        border: '1px solid var(--qs-border)',
                        borderRadius: 'var(--qs-radius-sm)',
                        padding: '8px',
                    }}>
                        {row.files.length === 0 ? (
                            <div style={{ ...fieldValueStyle, color: 'var(--qs-text-muted)' }}>
                                {'\u2014'}
                            </div>
                        ) : (
                            row.files.map((file) => (
                                <div key={file} style={{ ...fieldValueStyle, margin: '2px 0' }}>
                                    {file}
                                </div>
                            ))
                        )}
                    </div>
                </div>
            </div>
        </Modal>
    );
};

export default DetailsPopup;