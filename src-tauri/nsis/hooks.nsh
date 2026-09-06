; QuickSort NSIS installer hooks
; Included by tauri's custom installer template via installerHooks option.
;
; POSTINSTALL  - intentionally empty: the installer must not register anything.
;                The COM handler is registered by the application itself on
;                first launch (see src-tauri main.rs), never by the installer.
; PREUNINSTALL - remove COM registry keys before the uninstaller deletes files,
;                then restart Explorer so the shell-extension DLL is unloaded
;                from memory and can be deleted. The relaunch waits until the
;                old shell has fully exited (polling, not a blind sleep).
; POSTUNINSTALL - delete per-user application settings when the checkbox is
;                ticked, plus the publisher-named parent folder and legacy
;                per-user install leftovers.

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
  ; 1) Stop the app FIRST. A running QuickSort holds locks on its own exe and on
  ; the shell-extension DLL next to it (plus it rewrites the owner PID/registry
  ; on exit), so the uninstaller must kill it before touching files, otherwise
  ; the whole Program Files folder survives deletion and leaves traces behind.
  ; Kill both possible binary names (productName casing differs across builds).
  nsExec::Exec 'taskkill /f /im Quicksort.exe'
  nsExec::Exec 'taskkill /f /im quicksort.exe'

  ; 2) Remove COM keys so Explorer will not show the menu anymore.
  DeleteRegKey HKCU "Software\Classes\CLSID\{12345678-1234-1234-1234-1234567890AB}"
  DeleteRegKey HKCU "Software\Classes\AllFilesystemObjects\shellex\ContextMenuHandlers\QuickSort"
  ; Also clean stale handler keys from older versions that no longer ship.
  DeleteRegKey HKCU "Software\Classes\*\shellex\ContextMenuHandlers\QuickSort"
  DeleteRegKey HKCU "Software\Classes\Directory\Background\shellex\ContextMenuHandlers\QuickSort"
  DeleteRegKey HKCU "Software\Classes\Directory\shellex\ContextMenuHandlers\QuickSort"
  DeleteRegKey HKCU "Software\Classes\Drive\shellex\ContextMenuHandlers\QuickSort"

  ; 3) Restart Explorer so it unloads the mapped shell extension DLL before the
  ; uninstaller deletes it. The uninstaller is a 32-bit NSIS process, so the new
  ; shell must be launched through the Sysnative alias: a bare `explorer.exe`
  ; resolves to the WOW64 stub, which opens a folder window and never restores
  ; the shell (Start button and taskbar stay missing after uninstall).
  ;
  ; The relaunch must wait until the old shell has fully exited: spawning the
  ; new explorer.exe while the old one is still shutting down is what leaves the
  ; taskbar/Start button missing after uninstall (QA report 06.09.2026, line 84).
  ; Poll for the taskbar window (Shell_TrayWnd) to disappear for up to 5 s in
  ; 250 ms steps, mirroring restart_explorer() in src-tauri src/com.rs.
  nsExec::Exec 'taskkill /f /im explorer.exe'

  ; Wait for the old shell to die (locale-independent: check for the taskbar
  ; window, not a localized "no tasks running" tasklist message). FindWindow
  ; takes (class name, window name); the taskbar window class is Shell_TrayWnd.
  StrCpy $0 0
qs_explorer_poll:
  IntOp $0 $0 + 1
  ${If} $0 > 20
    Goto qs_explorer_relaunch
  ${EndIf}
  Sleep 250
  System::Call 'user32::FindWindow(t"Shell_TrayWnd",p0)p.r1'
  ${If} $1 != 0
    Goto qs_explorer_poll
  ${EndIf}
qs_explorer_relaunch:
  nsExec::Exec '"$WINDIR\Sysnative\explorer.exe"'
!macroend

; Runs after files, registry keys, and shortcuts have been removed.
; Always clean the install directory if it is now empty (leaving it behind is
; reported as a post-uninstall trace), then remove the per-user application
; data when the "delete settings" checkbox was ticked.
!macro NSIS_HOOK_POSTUNINSTALL
  ; Force-remove the install directory and every leftover in it. A bare RMDir
  ; only removes an empty folder, so a single surviving file (e.g. a shell DLL
  ; mapped by Explorer) keeps the whole directory on disk as an uninstall trace.
  RMDir /r "$INSTDIR"

  ; Per-user installs from 0.2.6 land in a publisher-named parent
  ; ("$LOCALAPPDATA\pr0math3us\Quicksort", see RestorePreviousInstallLocation).
  ; Remove the now-empty parent and the legacy per-user location
  ; ("$LOCALAPPDATA\Programs\Quicksort") used by builds before 0.2.6
  ; (QA report 06.09.2026, p. 90-91). RMDir without /r removes only empty
  ; folders, so active leftover installs are never deleted. Per-machine
  ; installs never use these per-user paths, so skip them.
  ${If} $MultiUser.InstallMode == "CurrentUser"
    RMDir "$LOCALAPPDATA\pr0math3us"
    RMDir "$LOCALAPPDATA\Programs\${PRODUCTNAME}"
  ${EndIf}

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
