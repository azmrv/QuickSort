//! Unix Domain Socket transport for Linux/macOS.
//!
//! Uses a Unix Domain Socket at `$XDG_RUNTIME_DIR/quicksort-{uid}.sock`
//! (or `/tmp/quicksort-{uid}.sock` as fallback) for IPC.

use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::Mutex;

use super::transport::{IpcStream, IpcTransport};

// ---------------------------------------------------------------------------
// Unix Socket Stream
// ---------------------------------------------------------------------------

/// A connected Unix Domain Socket client stream.
pub struct UnixSocketStream {
    stream: UnixStream,
}

impl IpcStream for UnixSocketStream {
    fn read_frame(&mut self) -> io::Result<Vec<u8>> {
        use std::io::Read;

        // Read the 4-byte length prefix (little-endian u32)
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf)?;
        let payload_len = u32::from_le_bytes(len_buf) as usize;

        // Read the payload
        let mut payload = vec![0u8; payload_len];
        self.stream.read_exact(&mut payload)?;
        Ok(payload)
    }

    fn write_frame(&mut self, data: &[u8]) -> io::Result<()> {
        use std::io::Write;

        // Write the 4-byte length prefix (little-endian u32)
        let len_bytes = (data.len() as u32).to_le_bytes();
        self.stream.write_all(&len_bytes)?;

        // Write the payload
        self.stream.write_all(data)?;
        self.stream.flush()?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Unix Socket Transport
// ---------------------------------------------------------------------------

/// Unix Domain Socket IPC transport for Linux/macOS.
pub struct UnixSocketTransport {
    path: PathBuf,
    listener: Mutex<Option<UnixListener>>,
}

impl UnixSocketTransport {
    pub fn new() -> Self {
        let uid = unsafe { libc::getuid() };
        let base = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
        let path = PathBuf::from(format!("{}/quicksort-{}.sock", base, uid));
        Self {
            path,
            listener: Mutex::new(None),
        }
    }
}

impl IpcTransport for UnixSocketTransport {
    type Stream = UnixSocketStream;

    fn start(&self) -> io::Result<()> {
        // Remove stale socket if exists
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }

        let listener = UnixListener::bind(&self.path)?;

        // Set permissions to owner-only (0600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.path, perms)?;
        }

        *self.listener.lock().unwrap() = Some(listener);
        tracing::info!("Unix socket listening on {:?}", self.path);
        Ok(())
    }

    fn accept(&self) -> io::Result<UnixSocketStream> {
        let guard = self.listener.lock().unwrap();
        let listener = guard
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "transport not started"))?;

        let (stream, _addr) = listener.accept()?;
        tracing::info!("Client connected to Unix socket");
        Ok(UnixSocketStream { stream })
    }

    fn cleanup(&self) {
        let _ = std::fs::remove_file(&self.path);
    }

    fn name(&self) -> &str {
        "UnixSocket"
    }
}
