import { describe, expect, it } from 'vitest';
import { classifyUndoError, getOperationErrorMessage } from './operationErrors';

describe('classifyUndoError', () => {
    it('passes through transient kind', () => {
        expect(classifyUndoError({ kind: 'transient', message: 'file busy' })).toBe('transient');
    });

    it('passes through permanent kind', () => {
        expect(classifyUndoError({ kind: 'permanent', message: 'file gone' })).toBe('permanent');
    });

    it('passes through unknown kind', () => {
        expect(classifyUndoError({ kind: 'unknown', message: 'boom' })).toBe('unknown');
    });

    it('classifies a plain string error as unknown', () => {
        expect(classifyUndoError('Operation not undoable')).toBe('unknown');
    });

    it('classifies null/undefined as unknown', () => {
        expect(classifyUndoError(null)).toBe('unknown');
        expect(classifyUndoError(undefined)).toBe('unknown');
    });

    it('classifies non-object errors as unknown', () => {
        expect(classifyUndoError(42)).toBe('unknown');
        expect(classifyUndoError([{ kind: 'permanent' }])).toBe('unknown');
    });

    it('classifies an object without a known kind as unknown', () => {
        expect(classifyUndoError({ message: 'boom' })).toBe('unknown');
        expect(classifyUndoError({ kind: 'retryable' })).toBe('unknown');
    });
});

describe('getOperationErrorMessage', () => {
    it('returns the backend message for an OperationErrorDto', () => {
        expect(getOperationErrorMessage({ kind: 'permanent', message: 'file gone' })).toBe('file gone');
    });

    it('returns the raw string for plain string errors', () => {
        expect(getOperationErrorMessage('Operation not undoable')).toBe('Operation not undoable');
    });

    it('falls back to String() for anything else', () => {
        expect(getOperationErrorMessage(undefined)).toBe('undefined');
        expect(getOperationErrorMessage(null)).toBe('null');
        expect(getOperationErrorMessage({ kind: 'transient' })).toBe('[object Object]');
    });
});