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

; Quietly stops a running QuickSort without any interactive prompt. The stock
; Tauri CheckIfAppIsRunning macro shows a modal MessageBox that can end up
; behind the installer window and look like a hang (QA 07.09.2026); this kills
; the process silently with up to 3 retries (500 ms pause each) and falls back
; to a log warning instead of blocking. Works for install and uninstall.
;
; Parameters:
;   BINARY_NAME - main executable name (e.g. "Quicksort.exe")
;   _uniq       - unique label prefix per insertion site (Tauri pattern, since
;                 NSIS macro labels must not collide when inserted twice).
!macro QuickSortStopRunning BINARY_NAME _uniq
  DetailPrint "Stopping a running QuickSort, if any..."
  StrCpy $3 0 ; attempt counter
${_uniq}_retry:
  IntOp $3 $3 + 1
  ${If} $3 > 3
    DetailPrint "Warning: QuickSort is still running after 3 attempts; continuing anyway"
    Goto ${_uniq}_done
  ${EndIf}
  ; taskkill /im is case-insensitive, one call covers every binary-name casing.
  ; Macro parameters are referenced with ${...} (NSIS), NOT $... like variables.
  nsExec::ExecToStack 'taskkill /f /im "${BINARY_NAME}"'
  Pop $1 ; exit code
  Pop $2 ; console output
  Sleep 500
  ; CSV no-header mode prints an EMPTY line when no process matches, so an
  ; empty output means the app is gone and we can proceed.
  nsExec::ExecToStack 'tasklist /fi "imagename eq ${BINARY_NAME}" /fo csv /nh'
  Pop $1 ; exit code
  Pop $2 ; console output
  ${If} $2 == ""
    Goto ${_uniq}_done
  ${EndIf}
  DetailPrint "QuickSort is still running, retrying (attempt $3/3)..."
  Goto ${_uniq}_retry
${_uniq}_done:
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
  !insertmacro QuickSortStopRunning "${MAINBINARYNAME}.exe" qs_preun

  ; 2) Remove COM keys so Explorer will not show the menu anymore.
  DetailPrint "Removing QuickSort context-menu registry entries..."
  DeleteRegKey HKCU "Software\Classes\CLSID\{12345678-1234-1234-1234-1234567890AB}"
  DeleteRegKey HKCU "Software\Classes\AllFilesystemObjects\shellex\ContextMenuHandlers\QuickSort"
  ; Also clean stale handler keys from older versions that no longer ship.
  DeleteRegKey HKCU "Software\Classes\*\shellex\ContextMenuHandlers\QuickSort"
  DeleteRegKey HKCU "Software\Classes\Directory\Background\shellex\ContextMenuHandlers\QuickSort"
  DeleteRegKey HKCU "Software\Classes\Directory\shellex\ContextMenuHandlers\QuickSort"
  DeleteRegKey HKCU "Software\Classes\Drive\shellex\ContextMenuHandlers\QuickSort"

  ; 3) Restart Explorer so it unloads the mapped shell extension DLL before the
  ; uninstaller deletes it. The uninstaller is a 32-bit NSIS process running
  ; under WOW64, so any "explorer.exe" path under System32 is redirected to
  ; SysWOW64 (the WOW64 stub, which opens a folder window and never restores the
  ; shell). The real 64-bit shell lives in the Windows root and is NOT subject
  ; to WOW64 redirection, so it must be launched via "$WINDIR\explorer.exe".
  ; The previous "$WINDIR\Sysnative\explorer.exe" silently failed on Windows
  ; 10/11: Sysnative maps to System32, and System32 contains NO explorer.exe
  ; (it only lives in the Windows root), leaving the taskbar/Start button
  ; missing after uninstall (QA reports 06.09.2026, line 84 and 07.09.2026,
  ; lines 70-72).
  ;
  ; The relaunch must wait until the old shell has fully exited: spawning the
  ; new explorer.exe while the old one is still shutting down is what leaves the
  ; taskbar/Start button missing after uninstall (QA report 06.09.2026, line 84).
  ; Poll the PROCESS list with tasklist CSV (locale-independent: CSV mode with
  ; no header prints an EMPTY line when no task matches), not the taskbar
  ; window - Shell_TrayWnd can disappear before the process has fully exited.
  ; Mirrors restart_explorer() in src-tauri src/com.rs.
  DetailPrint "Restarting Windows Explorer to unload the shell extension..."
  nsExec::Exec 'taskkill /f /im explorer.exe'

  ; Wait for the old shell process to fully exit (up to 5 s, 200 ms steps).
  DetailPrint "Waiting for Windows Explorer to fully exit..."
  StrCpy $0 0
qs_explorer_poll:
  IntOp $0 $0 + 1
  ${If} $0 > 25
    Goto qs_explorer_relaunch
  ${EndIf}
  Sleep 200
  nsExec::ExecToStack 'tasklist /fi "imagename eq explorer.exe" /fo csv /nh'
  Pop $1 ; exit code
  Pop $2 ; output
  ${If} $2 == ""
    Goto qs_explorer_relaunch
  ${EndIf}
  Goto qs_explorer_poll
qs_explorer_relaunch:
  ; Real 64-bit shell, not the WOW64 stub. "Exec" (unlike nsExec::Exec) does
  ; not wait for the launched program, so the installer is not blocked by the
  ; never-exiting shell process.
  Exec '"$WINDIR\explorer.exe"'
  DetailPrint "QuickSort uninstall: Explorer relaunched"
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
    ; WebView2 user-data folder (bundle-id named) left behind by the embedded
    ; webview (QA report 07.09.2026, EBWebView leftover trace). May still be
    ; locked by a running msedgewebview2 process - best effort.
    RMDir /r "$LOCALAPPDATA\com.azmrv.quicksort"
    ; Legacy identifiers from before the bundle-ID rename
    RMDir /r "$APPDATA\promatheus"
    RMDir /r "$LOCALAPPDATA\promatheus"
  ${EndIf}
!macroend
