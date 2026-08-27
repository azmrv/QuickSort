#!/usr/bin/env python3
"""QuickSort Nautilus Extension — right-click context menu integration.

Provides "Move to..." and "Copy to..." context menu items in Nautilus (GNOME)
and Nemo (Cinnamon). File paths are forwarded to the QuickSort Tauri app via
the shared `QuickSortIpcClient` over a Unix Domain Socket.
"""

import os
import sys

# Allow importing the shared IPC module adjacent to this extension.
_EXT_DIR = os.path.dirname(os.path.abspath(__file__))
_SHARED_DIR = os.path.abspath(os.path.join(_EXT_DIR, ".."))
if _SHARED_DIR not in sys.path:
    sys.path.insert(0, _SHARED_DIR)

from shared.ipc_client import QuickSortIpcClient  # noqa: E402

from gi.repository import Nautilus, GObject  # noqa: E402


class QuickSortExtension(GObject.GObject, Nautilus.MenuProvider):
    """Provides QuickSort context menu items in Nautilus/Nemo."""

    def __init__(self):
        super().__init__()
        self._ipc = QuickSortIpcClient()

    def get_file_items(self, *args, **kwargs):
        """Add menu items when files are selected.

        Supports both the Nautilus 3.x and 4.x call conventions.
        """
        files = args[0] if len(args) == 1 else args[1]
        if not files:
            return []

        items = []

        move_item = Nautilus.MenuItem(
            name="QuickSort::MoveTo",
            label="QuickSort: Move to...",
            tip="Move selected files to a QuickSort folder",
        )
        move_item.connect("activate", self._on_move_to, files)
        items.append(move_item)

        copy_item = Nautilus.MenuItem(
            name="QuickSort::CopyTo",
            label="QuickSort: Copy to...",
            tip="Copy selected files to a QuickSort folder",
        )
        copy_item.connect("activate", self._on_copy_to, files)
        items.append(copy_item)

        return items

    def get_background_items(self, *args, **kwargs):
        """Add menu items when right-clicking on the background."""
        # The ignored argument is the current folder; keep it implicit.
        _ = args[0] if len(args) == 1 else args[1]

        open_item = Nautilus.MenuItem(
            name="QuickSort::Open",
            label="QuickSort: Open Editor",
            tip="Open QuickSort file manager",
        )
        open_item.connect("activate", self._on_open)
        return [open_item]

    def _on_move_to(self, menu, files):
        """Handle the 'Move to' action."""
        paths = [f.get_location().get_path() for f in files]
        self._ipc.select_folder(paths, "move")

    def _on_copy_to(self, menu, files):
        """Handle the 'Copy to' action."""
        paths = [f.get_location().get_path() for f in files]
        self._ipc.select_folder(paths, "copy")

    def _on_open(self, menu):
        """Handle the 'Open Editor' action by launching QuickSort."""
        os.system("quicksort &")
