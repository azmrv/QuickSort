type LogLevel = 'INFO' | 'WARN' | 'ERROR' | 'ACTION' | 'IPC';

interface LogEntry {
    timestamp: string;
    level: LogLevel;
    source: string;
    message: string;
    data?: unknown;
}

const logs: LogEntry[] = [];
const MAX_LOGS = 500;

function formatTime(): string {
    return new Date().toLocaleTimeString('ru-RU', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        fractionalSecondDigits: 3,
    } as Intl.DateTimeFormatOptions);
}

function addEntry(level: LogLevel, source: string, message: string, data?: unknown) {
    const entry: LogEntry = { timestamp: formatTime(), level, source, message, data };
    logs.push(entry);
    if (logs.length > MAX_LOGS) logs.shift();

    const prefix = `[${entry.timestamp}] [${level}] [${source}]`;
    const line = data ? `${prefix} ${message}` : `${prefix} ${message}`;
    switch (level) {
        case 'ERROR': console.error(line, data ?? ''); break;
        case 'WARN':  console.warn(line, data ?? '');  break;
        default:      console.log(line, data ?? '');
    }
}

export const logger = {
    info:    (source: string, msg: string, data?: unknown) => addEntry('INFO', source, msg, data),
    warn:    (source: string, msg: string, data?: unknown) => addEntry('WARN', source, msg, data),
    error:   (source: string, msg: string, data?: unknown) => addEntry('ERROR', source, msg, data),
    action:  (source: string, msg: string, data?: unknown) => addEntry('ACTION', source, msg, data),
    ipc:     (source: string, msg: string, data?: unknown) => addEntry('IPC', source, msg, data),

    getLogs: (): LogEntry[] => [...logs],
    clear:   () => { logs.length = 0; },
};
