/**
 * Pure tree-building helpers for the folder hierarchy (0.2.6 feature 4f).
 *
 * All functions are deterministic and side-effect free — safe for unit tests.
 */

import { Folder } from '../types';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** A folder enriched with its resolved children for tree rendering. */
export interface FolderTreeNode extends Folder {
    children: FolderTreeNode[];
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** Maximum nesting depth allowed by the domain model and DLL constraints. */
export const MAX_FOLDER_DEPTH = 10;

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/**
 * Determine whether attaching `candidateChildId` as a child of
 * `candidateParentId` would form a cycle.
 *
 * Walks the parent chain from `candidateParentId` upward; if the walk
 * encounters `candidateChildId`, the attachment is cyclic.
 */
function wouldCreateCycle(
    nodesById: Map<string, Folder>,
    candidateChildId: string,
    candidateParentId: string,
): boolean {
    let current: Folder | undefined = nodesById.get(candidateParentId);
    let hops = 0;
    while (current && hops < MAX_FOLDER_DEPTH) {
        if (current.id === candidateChildId) return true;
        const nextParent = current.parent_id;
        current = nextParent ? nodesById.get(nextParent) : undefined;
        hops++;
    }
    // Exhausted the hop budget — treat as unsafe (cycle or depth > MAX above).
    return current !== undefined;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Build a hierarchical tree from a flat list of folders.
 *
 * - Folders whose `parent_id` is absent, points to a non-existent id, or
 *   would create a cycle are placed at the root level.
 * - Children are **not** sorted — the caller should order the list before
 *   passing it here or sort the result separately.
 *
 * @param folders - Flat array returned by `get_folders_v2`.
 * @returns Array of root-level `FolderTreeNode`s with nested `children`.
 */
export function buildFolderTree(folders: Folder[]): FolderTreeNode[] {
    const byId = new Map<string, Folder>();
    for (const f of folders) {
        byId.set(f.id, f);
    }

    const nodeMap = new Map<string, FolderTreeNode>();
    for (const f of folders) {
        nodeMap.set(f.id, { ...f, children: [] });
    }

    const roots: FolderTreeNode[] = [];

    for (const f of folders) {
        const node = nodeMap.get(f.id)!;
        const pid = f.parent_id;

        if (
            pid !== undefined &&
            pid !== null &&
            byId.has(pid) &&
            !wouldCreateCycle(byId, f.id, pid)
        ) {
            nodeMap.get(pid)!.children.push(node);
        } else {
            roots.push(node);
        }
    }

    return roots;
}

/**
 * Compute the nesting depth of `folderId` (root folders have depth 1).
 *
 * Walks up the parent chain; guards against cycles with a hop limit equal to
 * {@link MAX_FOLDER_DEPTH}.
 *
 * @param folders  - Flat folder list.
 * @param folderId - Id of the folder to measure.
 * @returns Depth value in the range [1, MAX_FOLDER_DEPTH].
 */
export function getFolderDepth(folders: Folder[], folderId: string): number {
    const byId = new Map(folders.map(f => [f.id, f]));
    let depth = 1;
    let current = byId.get(folderId);
    const seen = new Set<string>();

    while (current?.parent_id) {
        if (seen.has(current.id)) return MAX_FOLDER_DEPTH; // cycle guard
        seen.add(current.id);
        const parent = byId.get(current.parent_id);
        if (!parent) break; // dangling parent link — treat the folder as a root
        current = parent;
        depth++;
        if (depth > MAX_FOLDER_DEPTH) return MAX_FOLDER_DEPTH;
    }

    return Math.min(depth, MAX_FOLDER_DEPTH);
}

/**
 * Return the subset of folders that may still accept a child — i.e. those
 * whose depth is strictly less than {@link MAX_FOLDER_DEPTH}.
 *
 * Used to populate the parent-selector dropdown at folder creation time.
 *
 * @param folders - Flat folder list.
 * @returns Eligible parent folders.
 */
export function availableParents(folders: Folder[]): Folder[] {
    return folders.filter(f => getFolderDepth(folders, f.id) < MAX_FOLDER_DEPTH);
}
