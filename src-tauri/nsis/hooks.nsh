; QuickSort NSIS installer hooks
; Included by tauri's custom installer template via installerHooks option.
;
; POSTINSTALL  - register the COM handler right after install so the context
;                menu works without requiring the user to launch the app first.
; PREUNINSTALL - remove COM registry keys and unload the shell extension DLL
;                before the uninstaller deletes files.
; POSTUNINSTALL - delete per-user application settings when the checkbox is ticked.

; Runs after files are copied, registry keys written, and shortcuts created.
; ExecWait blocks until the register call completes (explorer restart is inside).
!macro NSIS_HOOK_POSTINSTALL
  IfFileExists "$INSTDIR\${MAINBINARYNAME}.exe" 0 +2
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --register'
!macroend

; Runs at the very start of uninstall, before any files are removed.
; Remove COM registry keys directly from NSIS (no elevation issues), then
; restart Explorer so the shell extension DLL is unloaded from memory and
; subsequent file deletion succeeds.
!macro NSIS_HOOK_PREUNINSTALL
  ; Delete CLSID and handler keys so Explorer will not show the menu anymore.
  DeleteRegKey HKCU "Software\Classes\CLSID\{12345678-1234-1234-1234-1234567890AB}"
  DeleteRegKey HKCU "Software\Classes\AllFilesystemObjects\shellex\ContextMenuHandlers\QuickSort"
  ; Also clean stale handler keys from older versions that no longer ship.
  DeleteRegKey HKCU "Software\Classes\*\shellex\ContextMenuHandlers\QuickSort"
  DeleteRegKey HKCU "Software\Classes\Directory\Background\shellex\ContextMenuHandlers\QuickSort"
  DeleteRegKey HKCU "Software\Classes\Directory\shellex\ContextMenuHandlers\QuickSort"
  DeleteRegKey HKCU "Software\Classes\Drive\shellex\ContextMenuHandlers\QuickSort"
  ; Kill Explorer so the DLL mapping is released before the uninstaller
  ; tries to delete the DLL file.
  nsExec::ExecToStack 'taskkill /f /im explorer.exe'
  Pop $0
  Sleep 500
  nsExec::ExecToStack 'start explorer.exe'
  Pop $0
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
