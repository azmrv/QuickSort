#!/usr/bin/env python3
"""Shared IPC client for QuickSort Unix Domain Socket communication.

This module provides a thin client that talks to the QuickSort Tauri app
over a Unix Domain Socket using the same framing protocol as the Windows
Named Pipe client: `[u32 LE length][JSON payload]`.
"""

import os
import json
import socket
import struct
from typing import Optional, Dict, Any

# Unix socket path (matches Tauri app's UnixSocketTransport)
SOCKET_PATH = f"/run/user/{os.getuid()}/quicksort.sock"


class QuickSortIpcClient:
    """IPC client for communicating with QuickSort via Unix Domain Socket."""

    def __init__(self, socket_path: str = SOCKET_PATH):
        """Initialize IPC client.

        Args:
            socket_path: Path to the Unix Domain Socket.
        """
        self.socket_path = socket_path

    def send_command(self, command: str, data: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Send a command to QuickSort via the Unix socket.

        Uses the same framing protocol as the Windows client:
        `[u32 LE length][JSON payload]`.

        Args:
            command: Command name (e.g., "select_folder", "ping").
            data: Additional data to send with the command.

        Returns:
            Response dictionary from QuickSort. Returns an error dict when
            QuickSort is not running or the socket is unreachable.
        """
        try:
            sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            sock.connect(self.socket_path)

            # Frame: [u32 LE length][JSON payload]
            payload_data = {"command": command}
            if data:
                payload_data.update(data)
            payload = json.dumps(payload_data).encode()
            sock.send(struct.pack("<I", len(payload)) + payload)

            # Read the 4-byte length prefix, then the payload.
            length_data = self._recv_exact(sock, 4)
            if not length_data:
                sock.close()
                return {"status": "error", "message": "No response from QuickSort"}
            length = struct.unpack("<I", length_data)[0]
            response_data = self._recv_exact(sock, length)
            sock.close()

            return json.loads(response_data)
        except (ConnectionRefusedError, FileNotFoundError, OSError) as e:
            return {"status": "error", "message": "QuickSort not running: %s" % e}

    @staticmethod
    def _recv_exact(sock: socket.socket, n: int) -> Optional[bytes]:
        """Receive exactly n bytes from the socket.

        Args:
            sock: The connected socket to read from.
            n: Number of bytes to receive.

        Returns:
            The received bytes, or None if the connection was closed early.
        """
        data = b""
        while len(data) < n:
            chunk = sock.recv(n - len(data))
            if not chunk:
                return None
            data += chunk
        return data

    def is_running(self) -> bool:
        """Return True if the QuickSort Tauri app is reachable.

        Returns:
            True when a ping command returns an Ok status.
        """
        result = self.send_command("ping")
        return result.get("status") == "ok"

    def select_folder(self, files: list, operation: str = "move") -> Dict[str, Any]:
        """Send a select_folder command to QuickSort.

        Args:
            files: List of absolute file paths to operate on.
            operation: Operation type ("move" or "copy").

        Returns:
            Response dictionary from QuickSort.
        """
        return self.send_command("select_folder", {
            "files": files,
            "operation": operation,
        })
