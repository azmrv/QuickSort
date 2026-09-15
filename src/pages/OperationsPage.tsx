import OperationsTable from './operations/OperationsTable';

/**
 * Operations tab (0.2.6 feature 4c): a single unified Operations table that
 * merges the live job queue and the operation history (see OperationsTable).
 */
const OperationsPage = () => {
    return <OperationsTable />;
};

export default OperationsPage;