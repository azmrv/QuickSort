/**
 * Unit tests for the pure folder-tree helpers (0.2.6 feature 4f).
 */

import { describe, it, expect } from 'vitest';
import {
    buildFolderTree,
    availableParents,
    getFolderDepth,
    MAX_FOLDER_DEPTH,
} from './folderTree';
import { Folder } from '../types';

function makeFolder(
    id: string,
    parentId?: string | null,
    overwrite: Partial<Folder> = {},
): Folder {
    return {
        id,
        name: `Folder ${id}`,
        path: `C:\\${id}`,
        favorite: false,
        order: 0,
        stats: { use_count: 0, last_used: null },
        parent_id: parentId,
        ...overwrite,
    };
}

describe('buildFolderTree', () => {
    it('moves orphan folders with a missing parent to the roots', () => {
        const folders = [
            makeFolder('a', 'missing-parent'),
            makeFolder('b'),
        ];
        const tree = buildFolderTree(folders);
        const ids = tree.map(n => n.id).sort();
        expect(ids).toEqual(['a', 'b']);
        // The orphan must not be nested anywhere.
        expect(tree.every(n => n.children.length === 0)).toBe(true);
    });

    it('nests children under their parent', () => {
        const folders = [
            makeFolder('root'),
            makeFolder('child', 'root'),
            makeFolder('grandchild', 'child'),
        ];
        const tree = buildFolderTree(folders);
        expect(tree).toHaveLength(1);
        expect(tree[0].id).toBe('root');
        expect(tree[0].children.map(c => c.id)).toEqual(['child']);
        expect(tree[0].children[0].children.map(c => c.id)).toEqual(['grandchild']);
    });

    it('promotes folders in a self-cycle to roots', () => {
        const folders = [makeFolder('a', 'a'), makeFolder('b', 'c'), makeFolder('c', 'b')];
        const tree = buildFolderTree(folders);
        expect(tree).toHaveLength(3);
        expect(tree.every(n => n.children.length === 0)).toBe(true);
    });

    it('treats null parent_id as root', () => {
        const folders = [makeFolder('root', null)];
        const tree = buildFolderTree(folders);
        expect(tree).toHaveLength(1);
        expect(tree[0].id).toBe('root');
    });

    it('handles a long but valid chain deep within the depth limit', () => {
        const folders: Folder[] = [];
        let prev: string | null = null;
        for (let i = 0; i < MAX_FOLDER_DEPTH; i++) {
            const id = `d${i}`;
            folders.push(makeFolder(id, prev));
            prev = id;
        }
        const tree = buildFolderTree(folders);
        expect(tree).toHaveLength(1);
        // Walk the chain and confirm every level nests exactly one child.
        let node = tree[0];
        let depth = 1;
        while (node.children.length > 0) {
            expect(node.children).toHaveLength(1);
            node = node.children[0];
            depth++;
        }
        expect(depth).toBe(MAX_FOLDER_DEPTH);
    });

    it('keeps folder fields intact on tree nodes', () => {
        const folders = [
            makeFolder('root', null, {
                name: 'My Root',
                path: 'D:\\Data',
                favorite: true,
                order: 3,
                color: '#ff0000',
            }),
            makeFolder('child', 'root'),
        ];
        const tree = buildFolderTree(folders);
        expect(tree[0].name).toBe('My Root');
        expect(tree[0].path).toBe('D:\\Data');
        expect(tree[0].favorite).toBe(true);
        expect(tree[0].order).toBe(3);
        expect(tree[0].color).toBe('#ff0000');
        expect(tree[0].stats).toEqual({ use_count: 0, last_used: null });
    });
});

describe('getFolderDepth', () => {
    it('returns 1 for a root folder', () => {
        const folders = [makeFolder('root')];
        expect(getFolderDepth(folders, 'root')).toBe(1);
    });

    it('returns the nesting depth for a nested folder', () => {
        const folders = [
            makeFolder('root'),
            makeFolder('child', 'root'),
            makeFolder('grandchild', 'child'),
        ];
        expect(getFolderDepth(folders, 'root')).toBe(1);
        expect(getFolderDepth(folders, 'child')).toBe(2);
        expect(getFolderDepth(folders, 'grandchild')).toBe(3);
    });

    it('returns 1 for a folder whose parent is missing', () => {
        const folders = [makeFolder('orphan', 'missing')];
        expect(getFolderDepth(folders, 'orphan')).toBe(1);
    });

    it('returns 1 for an unknown folder id', () => {
        expect(getFolderDepth([], 'nope')).toBe(1);
    });

    it('returns MAX_FOLDER_DEPTH for a cycle (hop limit, no infinite loop)', () => {
        const folders = [makeFolder('a', 'b'), makeFolder('b', 'a')];
        expect(getFolderDepth(folders, 'a')).toBe(MAX_FOLDER_DEPTH);
        expect(getFolderDepth(folders, 'b')).toBe(MAX_FOLDER_DEPTH);
    });
});

describe('availableParents', () => {
    it('excludes folders whose depth already reached the max', () => {
        // Build a chain of MAX depth: d0 is the root, d{MAX-1} is the deepest.
        const folders: Folder[] = [];
        let prev: string | null = null;
        for (let i = 0; i < MAX_FOLDER_DEPTH; i++) {
            const id = `d${i}`;
            folders.push(makeFolder(id, prev));
            prev = id;
        }

        const parents = availableParents(folders);
        expect(parents).toHaveLength(MAX_FOLDER_DEPTH - 1);
        expect(parents[0].id).toBe('d0');
        // The deepest node cannot accept a child (depth == MAX).
        expect(parents.some(p => p.id === `d${MAX_FOLDER_DEPTH - 1}`)).toBe(false);
    });

    it('returns all folders when none reach the depth limit', () => {
        const folders = [
            makeFolder('a'),
            makeFolder('b', 'a'),
        ];
        const parents = availableParents(folders);
        expect(parents).toHaveLength(2);
    });

    it('excludes folders involved in cycles from being parents', () => {
        const folders = [makeFolder('a', 'b'), makeFolder('b', 'a')];
        const parents = availableParents(folders);
        // Both nodes sit in a cycle -> depth == MAX -> no eligible parents.
        expect(parents).toHaveLength(0);
    });
});