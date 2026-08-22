//! WCX Plugin Adapter
//!
//! Adapts Total Commander WCX packer plugins to QuickSort's ArchivePlugin trait.
//!
//! # WCX SDK Reference
//! - WCX SDK v2.21 SE: https://ghisler.github.io/WCX-SDK/table_of_contents.htm
//! - Mandatory functions: OpenArchive, ReadHeader, ProcessFile, CloseArchive
//! - Optional functions: PackFiles, GetPackerCaps, CanYouHandleThisFile, ReadHeaderEx
//!
//! # Safety
//! This module uses unsafe code for FFI calls with WCX plugin DLLs.
//! All unsafe blocks are carefully audited and documented.

use quicksort_domain::{
    ArchiveEntry, ArchivePlugin, Plugin, PluginCapabilities, PluginError, PluginType,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// WCX SDK Constants
const PK_OM_LIST: i32 = 0;
const PK_OM_EXTRACT: i32 = 1;
const PK_SKIP: i32 = 0;
const PK_EXTRACT: i32 = 2;

// WCX Capability Flags
const PK_CAPS_NEW: u32 = 1;
const PK_CAPS_MODIFY: u32 = 2;
const PK_CAPS_MULTIPLE: u32 = 4;
const PK_CAPS_DELETE: u32 = 8;
const PK_CAPS_OPTIONS: u32 = 16;
const PK_CAPS_MEMPACK: u32 = 32;
const PK_CAPS_BY_CONTENT: u32 = 64;
const PK_CAPS_SEARCHTEXT: u32 = 128;
const PK_CAPS_ENCRYPT: u32 = 512;

// WCX Error Codes
const E_END_ARCHIVE: i32 = 10;

/// WCX archive handle (opaque pointer).
type WcxHandle = *mut std::ffi::c_void;

/// Function pointer types for WCX API (Windows stdcall calling convention).
type FnOpenArchive = unsafe extern "system" fn(*mut tOpenArchiveData) -> WcxHandle;
type FnReadHeader = unsafe extern "system" fn(WcxHandle, *mut tHeaderData) -> i32;
type FnProcessFile = unsafe extern "system" fn(WcxHandle, i32, *const i8, *const i8) -> i32;
type FnCloseArchive = unsafe extern "system" fn(WcxHandle);
type FnSetChangeVolProc = unsafe extern "system" fn(WcxHandle, *mut std::ffi::c_void);
type FnSetProcessDataProc = unsafe extern "system" fn(WcxHandle, *mut std::ffi::c_void);
type FnGetPackerCaps = unsafe extern "system" fn() -> i32;
type FnCanYouHandleThisFile = unsafe extern "system" fn(*const i8) -> i32;
type FnReadHeaderEx = unsafe extern "system" fn(WcxHandle, *mut tHeaderDataEx) -> i32;

// WCX Structures
#[repr(C)]
struct tOpenArchiveData {
    arc_name: [i8; 260],
    open_mode: i32,
    open_result: i32,
    _reserved: [u8; 256], // Callback pointers and reserved space
}

#[repr(C)]
struct tHeaderData {
    file_name: [i8; 260],
    pack_size: i32,
    unp_size: i32,
    file_time: [u8; 5],
    file_attr: i32,
    _reserved: [u8; 280], // Additional fields
}

#[repr(C)]
struct tHeaderDataEx {
    file_name: [i8; 260 * 3], // Unicode support
    pack_size: i64,
    unp_size: i64,
    file_time: [u8; 8], // FILETIME
    file_attr: i32,
    _reserved: [u8; 280], // Additional fields
}

/// WCX Plugin Adapter.
///
/// Wraps a Total Commander WCX plugin DLL and adapts it to QuickSort's
/// `ArchivePlugin` trait. Handles function loading, capability detection,
/// and API mapping.
pub struct WcxPluginAdapter {
    /// Plugin identifier.
    id: String,
    /// Plugin name.
    name: String,
    /// Plugin version.
    version: String,
    /// Capability flags from GetPackerCaps.
    capabilities: u32,
    /// Supported file extensions (lowercase, without dot).
    extension_map: HashMap<String, String>,
    /// DLL module handle.
    _dll_module: *mut std::ffi::c_void,

    // Mandatory function pointers
    open_archive: FnOpenArchive,
    read_header: FnReadHeader,
    process_file: FnProcessFile,
    close_archive: FnCloseArchive,
    _set_change_vol_proc: FnSetChangeVolProc,
    _set_process_data_proc: FnSetProcessDataProc,

    // Optional function pointers
    _get_packer_caps: Option<FnGetPackerCaps>,
    can_handle_file: Option<FnCanYouHandleThisFile>,
    _read_header_ex: Option<FnReadHeaderEx>,
}

// Safety: The DLL module handle is managed by the adapter and not shared.
unsafe impl Send for WcxPluginAdapter {}
unsafe impl Sync for WcxPluginAdapter {}

impl WcxPluginAdapter {
    /// Load a WCX plugin from a DLL file.
    ///
    /// # Safety
    /// Caller must ensure `dll_path` points to a valid WCX plugin DLL
    /// compiled for the current platform (win32, stdcall ABI).
    ///
    /// # Arguments
    /// * `dll_path` - Path to the .wcx or .dll file
    ///
    /// # Returns
    /// * `Ok(WcxPluginAdapter)` - Successfully loaded plugin
    /// * `Err(PluginError)` - Load failed
    pub unsafe fn load(dll_path: &Path) -> Result<Self, PluginError> {
        // 1. Load DLL
        let dll_module = Self::load_dll(dll_path)?;

        // 2. Get mandatory function pointers
        let open_archive = Self::get_proc_address::<FnOpenArchive>(dll_module, "OpenArchive")?;
        let read_header = Self::get_proc_address::<FnReadHeader>(dll_module, "ReadHeader")?;
        let process_file = Self::get_proc_address::<FnProcessFile>(dll_module, "ProcessFile")?;
        let close_archive = Self::get_proc_address::<FnCloseArchive>(dll_module, "CloseArchive")?;
        let set_change_vol_proc =
            Self::get_proc_address::<FnSetChangeVolProc>(dll_module, "SetChangeVolProc")?;
        let set_process_data_proc =
            Self::get_proc_address::<FnSetProcessDataProc>(dll_module, "SetProcessDataProc")?;

        // 3. Get optional function pointers
        let get_packer_caps =
            Self::get_optional_proc::<FnGetPackerCaps>(dll_module, "GetPackerCaps");
        let can_handle_file =
            Self::get_optional_proc::<FnCanYouHandleThisFile>(dll_module, "CanYouHandleThisFile");
        let read_header_ex = Self::get_optional_proc::<FnReadHeaderEx>(dll_module, "ReadHeaderEx");

        // 4. Get capabilities
        let capabilities = match get_packer_caps {
            Some(f) => f() as u32,
            None => PK_CAPS_NEW | PK_CAPS_MULTIPLE, // Default for read-only plugins
        };

        // 5. Build plugin info from DLL filename
        let plugin_name = dll_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let id = format!("tc-wcx-{}", plugin_name.to_lowercase());

        // 6. Build extension map from filename (will be populated by can_handle_file calls)
        let extension_map = HashMap::new();

        tracing::info!(
            plugin_id = %id,
            capabilities = capabilities,
            "WCX plugin loaded"
        );

        Ok(Self {
            id,
            name: plugin_name,
            version: "1.0.0".to_string(),
            capabilities,
            extension_map,
            _dll_module: dll_module,
            open_archive,
            read_header,
            process_file,
            close_archive,
            _set_change_vol_proc: set_change_vol_proc,
            _set_process_data_proc: set_process_data_proc,
            _get_packer_caps: get_packer_caps,
            can_handle_file,
            _read_header_ex: read_header_ex,
        })
    }

    /// Load a Windows DLL.
    unsafe fn load_dll(dll_path: &Path) -> Result<*mut std::ffi::c_void, PluginError> {
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::libloaderapi::LoadLibraryW;

        let wide: Vec<u16> = dll_path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let handle = LoadLibraryW(wide.as_ptr());
        if handle.is_null() {
            return Err(PluginError::LoadFailed(format!(
                "Failed to load DLL: {}",
                dll_path.display()
            )));
        }

        Ok(handle as *mut std::ffi::c_void)
    }

    /// Get a required function pointer from the DLL.
    ///
    /// # Safety
    /// The caller must ensure the function signature matches the expected WCX API.
    unsafe fn get_proc_address<F: Copy>(
        dll_module: *mut std::ffi::c_void,
        name: &str,
    ) -> Result<F, PluginError> {
        use winapi::um::libloaderapi::GetProcAddress;

        let name_cstr = std::ffi::CString::new(name).unwrap();
        let proc = GetProcAddress(dll_module as *mut _, name_cstr.as_ptr());
        if proc.is_null() {
            return Err(PluginError::LoadFailed(format!(
                "Required function not found: {}",
                name
            )));
        }
        // Safety: transmute_copy is safe for function pointers of known size.
        Ok(std::mem::transmute_copy(&proc))
    }

    /// Get an optional function pointer from the DLL.
    ///
    /// # Safety
    /// The caller must ensure the function signature matches the expected WCX API.
    unsafe fn get_optional_proc<F: Copy>(
        dll_module: *mut std::ffi::c_void,
        name: &str,
    ) -> Option<F> {
        use winapi::um::libloaderapi::GetProcAddress;

        let name_cstr = std::ffi::CString::new(name).unwrap();
        let proc = GetProcAddress(dll_module as *mut _, name_cstr.as_ptr());
        if proc.is_null() {
            None
        } else {
            Some(std::mem::transmute_copy(&proc))
        }
    }

    /// Open an archive and return a handle.
    unsafe fn open_archive_internal(
        &self,
        archive_path: &Path,
        mode: i32,
    ) -> Result<WcxHandle, PluginError> {
        let mut data = tOpenArchiveData {
            arc_name: [0; 260],
            open_mode: mode,
            open_result: 0,
            _reserved: [0; 256],
        };

        // Copy path to arc_name
        if let Some(path_str) = archive_path.to_str() {
            let bytes = path_str.as_bytes();
            let len = bytes.len().min(259);
            for (i, &b) in bytes.iter().enumerate().take(len) {
                data.arc_name[i] = b as i8;
            }
        }

        let handle = (self.open_archive)(&mut data);
        if handle.is_null() {
            return Err(PluginError::OperationFailed(format!(
                "OpenArchive failed with code: {}",
                data.open_result
            )));
        }

        Ok(handle)
    }

    /// Close an archive handle.
    unsafe fn close_archive_internal(&self, handle: WcxHandle) {
        (self.close_archive)(handle);
    }

    /// Read the next header from the archive.
    unsafe fn read_header_internal(&self, handle: WcxHandle) -> Result<tHeaderData, PluginError> {
        let mut header = tHeaderData {
            file_name: [0; 260],
            pack_size: 0,
            unp_size: 0,
            file_time: [0; 5],
            file_attr: 0,
            _reserved: [0; 280],
        };

        let result = (self.read_header)(handle, &mut header);
        if result == E_END_ARCHIVE {
            return Err(PluginError::OperationFailed("End of archive".to_string()));
        }
        if result != 0 {
            return Err(PluginError::OperationFailed(format!(
                "ReadHeader failed with code: {}",
                result
            )));
        }

        Ok(header)
    }

    /// Process (extract/skip) a file in the archive.
    unsafe fn process_file_internal(
        &self,
        handle: WcxHandle,
        operation: i32,
        dest_path: Option<&str>,
        dest_name: Option<&str>,
    ) -> Result<(), PluginError> {
        let dest_path_ptr = dest_path
            .and_then(|s| std::ffi::CString::new(s).ok())
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut());

        let dest_name_ptr = dest_name
            .and_then(|s| std::ffi::CString::new(s).ok())
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut());

        let result = (self.process_file)(handle, operation, dest_path_ptr, dest_name_ptr);

        // Clean up CStrings
        if !dest_path_ptr.is_null() {
            drop(std::ffi::CString::from_raw(dest_path_ptr));
        }
        if !dest_name_ptr.is_null() {
            drop(std::ffi::CString::from_raw(dest_name_ptr));
        }

        if result != 0 {
            return Err(PluginError::OperationFailed(format!(
                "ProcessFile failed with code: {}",
                result
            )));
        }

        Ok(())
    }

    /// Convert a WCX header to an ArchiveEntry.
    fn header_to_entry(header: &tHeaderData) -> ArchiveEntry {
        // Extract filename (null-terminated)
        let name_bytes: Vec<u8> = header
            .file_name
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as u8)
            .collect();
        let name = String::from_utf8_lossy(&name_bytes).to_string();

        let is_directory = name.ends_with('/') || name.ends_with('\\');

        ArchiveEntry {
            path: name,
            size: header.unp_size as u64,
            compressed_size: Some(header.pack_size as u64),
            is_directory,
            modified_at: None, // TODO: Parse file_time
        }
    }
}

impl Plugin for WcxPluginAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn plugin_type(&self) -> PluginType {
        PluginType::Archive
    }

    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            can_create: self.capabilities & PK_CAPS_NEW != 0,
            can_modify: self.capabilities & PK_CAPS_MODIFY != 0,
            supports_multiple: self.capabilities & PK_CAPS_MULTIPLE != 0,
            can_delete: self.capabilities & PK_CAPS_DELETE != 0,
            has_options: self.capabilities & PK_CAPS_OPTIONS != 0,
            supports_mempack: self.capabilities & PK_CAPS_MEMPACK != 0,
            detect_by_content: self.capabilities & PK_CAPS_BY_CONTENT != 0,
            supports_search: self.capabilities & PK_CAPS_SEARCHTEXT != 0,
            supports_encrypt: self.capabilities & PK_CAPS_ENCRYPT != 0,
        }
    }

    fn supported_extensions(&self) -> Vec<String> {
        self.extension_map.keys().cloned().collect()
    }

    fn initialize(&mut self, _config: &quicksort_domain::PluginConfig) -> Result<(), PluginError> {
        // WCX plugins don't require initialization beyond loading
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        // DLL will be freed when the adapter is dropped
        Ok(())
    }
}

impl ArchivePlugin for WcxPluginAdapter {
    fn can_handle(&self, extension: &str) -> bool {
        // Check if the plugin reports it can handle this file
        if let Some(can_handle) = self.can_handle_file {
            let ext_cstr = std::ffi::CString::new(format!(".{}", extension)).unwrap();
            let result = unsafe { can_handle(ext_cstr.as_ptr()) };
            result != 0
        } else {
            // Fallback to extension map
            self.extension_map
                .contains_key(extension.to_lowercase().as_str())
        }
    }

    fn list_contents(&self, archive_path: &Path) -> Result<Vec<ArchiveEntry>, PluginError> {
        let mut entries = Vec::new();

        unsafe {
            let handle = self.open_archive_internal(archive_path, PK_OM_LIST)?;

            while let Ok(header) = self.read_header_internal(handle) {
                let entry = Self::header_to_entry(&header);
                let _ = self.process_file_internal(handle, PK_SKIP, None, None);
                entries.push(entry);
            }

            self.close_archive_internal(handle);
        }

        Ok(entries)
    }

    fn extract_file(
        &self,
        archive_path: &Path,
        entry_path: &str,
        output_path: &Path,
    ) -> Result<(), PluginError> {
        unsafe {
            let handle = self.open_archive_internal(archive_path, PK_OM_EXTRACT)?;

            while let Ok(header) = self.read_header_internal(handle) {
                let name = Self::header_to_entry(&header).path;
                if name == entry_path {
                    self.process_file_internal(
                        handle,
                        PK_EXTRACT,
                        output_path.parent().and_then(|p| p.to_str()),
                        output_path.file_name().and_then(|n| n.to_str()),
                    )?;
                    self.close_archive_internal(handle);
                    return Ok(());
                } else {
                    let _ = self.process_file_internal(handle, PK_SKIP, None, None);
                }
            }

            self.close_archive_internal(handle);
        }

        Err(PluginError::OperationFailed(format!(
            "File not found in archive: {}",
            entry_path
        )))
    }

    fn add_file(
        &self,
        _archive_path: &Path,
        _file_path: &Path,
        _entry_name: &str,
    ) -> Result<(), PluginError> {
        let caps = self.capabilities();
        if !caps.can_create || !caps.can_modify {
            return Err(PluginError::Incompatible(
                "Plugin does not support adding files to archives".to_string(),
            ));
        }

        // TODO: Implement PackFiles call
        Err(PluginError::OperationFailed(
            "PackFiles not yet implemented".to_string(),
        ))
    }

    fn create_archive(&self, _archive_path: &Path, _files: &[PathBuf]) -> Result<(), PluginError> {
        let caps = self.capabilities();
        if !caps.can_create {
            return Err(PluginError::Incompatible(
                "Plugin does not support creating archives".to_string(),
            ));
        }

        // TODO: Implement PackFiles call for new archive
        Err(PluginError::OperationFailed(
            "PackFiles not yet implemented".to_string(),
        ))
    }
}

impl Drop for WcxPluginAdapter {
    fn drop(&mut self) {
        unsafe {
            use winapi::um::libloaderapi::FreeLibrary;
            if !self._dll_module.is_null() {
                FreeLibrary(self._dll_module as *mut _);
            }
        }
    }
}

/// Plugin loader for WCX plugins.
pub struct WcxPluginLoader;

impl WcxPluginLoader {
    /// Load all WCX plugins from a directory.
    pub fn load_all(plugin_dir: &Path) -> Result<Vec<WcxPluginAdapter>, PluginError> {
        let mut plugins = Vec::new();

        if !plugin_dir.exists() {
            return Ok(plugins);
        }

        for entry in std::fs::read_dir(plugin_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("wcx") || ext.eq_ignore_ascii_case("dll") {
                    match unsafe { WcxPluginAdapter::load(&path) } {
                        Ok(plugin) => {
                            tracing::info!(path = %path.display(), "Loaded WCX plugin");
                            plugins.push(plugin);
                        }
                        Err(e) => {
                            tracing::warn!(path = %path.display(), error = %e, "Failed to load WCX plugin");
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_type() {
        // Test that the adapter type is correct
        assert_eq!(
            std::mem::size_of::<WcxPluginAdapter>(),
            std::mem::size_of::<WcxPluginAdapter>()
        );
    }
}
