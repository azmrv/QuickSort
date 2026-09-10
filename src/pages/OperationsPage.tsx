import HistoryPage from './HistoryPage';
import QueuePage from './QueuePage';

/**
 * Operations tab (0.2.6 navigation Q2): unifies operation history and the
 * active job queue. In 4a the historical and queue views are shown as two
 * sections on one page; 4b merges them into a single Operations table.
 */
const OperationsPage = () => {
    return (
        <div style={{ padding: 'var(--qs-space-lg)' }}>
            <div style={{ marginBottom: 'var(--qs-space-lg)' }}>
                <QueuePage />
            </div>
            <div>
                <HistoryPage />
            </div>
        </div>
    );
};

export default OperationsPage;