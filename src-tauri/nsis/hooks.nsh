; QuickSort NSIS installer hooks
; Included by tauri's custom installer template via installerHooks option.
;
; POSTINSTALL  - intentionally empty: the installer must not register anything.
;                The COM handler is registered by the application itself on
;                first launch (see src-tauri main.rs), never by the installer.
; PREUNINSTALL - remove COM registry keys before the uninstaller deletes files.
;                Explorer is intentionally NOT restarted (product requirement:
;                the installer/uninstaller never interferes with Explorer).
; POSTUNINSTALL - delete per-user application settings when the checkbox is ticked.

; Runs after files are copied, registry keys written, and shortcuts created.
; Nothing to do here: registration belongs to the application (first launch).
!macro NSIS_HOOK_POSTINSTALL
!macroend

; Runs at the very start of uninstall, before any files are removed.
; Remove COM registry keys directly from NSIS (no elevation issues), then
; restart Explorer so the shell extension DLL is unloaded from memory and the
; subsequent file deletion of context_menu_dll.dll succeeds (otherwise the DLL
; stays mapped by Explorer and is left behind as a trace).
!macro NSIS_HOOK_PREUNINSTALL
  ; Delete CLSID and handler keys so Explorer will not show the menu anymore.
  DeleteRegKey HKCU "Software\Classes\CLSID\{12345678-1234-1234-1234-1234567890AB}"
  DeleteRegKey HKCU "Software\Classes\AllFilesystemObjects\shellex\ContextMenuHandlers\QuickSort"
  ; Also clean stale handler keys from older versions that no longer ship.
  DeleteRegKey HKCU "Software\Classes\*\shellex\ContextMenuHandlers\QuickSort"
  DeleteRegKey HKCU "Software\Classes\Directory\Background\shellex\ContextMenuHandlers\QuickSort"
  DeleteRegKey HKCU "Software\Classes\Directory\shellex\ContextMenuHandlers\QuickSort"
  DeleteRegKey HKCU "Software\Classes\Drive\shellex\ContextMenuHandlers\QuickSort"

  ; Restart Explorer so it unloads the mapped shell extension DLL before the
  ; uninstaller deletes it. nsExec::Exec waits for taskkill to finish (Explorer
  ; is gone when it returns); we then relaunch the shell.
  nsExec::Exec 'taskkill /f /im explorer.exe'
  Sleep 500
  Exec 'explorer.exe'
!macroend

; Runs after files, registry keys, and shortcuts have been removed.
; When the "delete settings" checkbox was ticked, remove the per-user
; application data (settings, operation history, folders config).
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    SetShellVarContext current
    ; Per-user application data (current bundle ID)
    RMDir /r "$APPDATA\QuickSort"
    RMDir /r "$LOCALAPPDATA\QuickSort"
    ; Legacy identifiers from before the bundle-ID rename
    RMDir /r "$APPDATA\promatheus"
    RMDir /r "$LOCALAPPDATA\promatheus"
  ${EndIf}
!macroend
