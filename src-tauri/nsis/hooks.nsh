; QuickSort NSIS installer hooks
; Included by tauri's custom installer template via installerHooks option.
;
; POSTINSTALL  - intentionally empty: the installer must not register anything.
;                The COM handler is registered by the application itself on
;                first launch (see src-tauri main.rs), never by the installer.
; PREUNINSTALL - remove COM registry keys before the uninstaller deletes files,
;                then kill Explorer so the shell-extension DLL is unloaded from
;                memory and can be deleted. The macro waits a fixed 500 ms (no
;                poll loop: it reads as a hang - QA 07.09.2026) and then
;                relaunches explorer.exe ONCE only if the shell did NOT
;                auto-restart (auto-restart is NOT guaranteed - QA 12.09.2026;
;                see the macro body for the full QA history).
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
  DetailPrint "Closing a running QuickSort, if any..."
  StrCpy $3 0 ; attempt counter
${_uniq}_retry:
  IntOp $3 $3 + 1
  ${If} $3 > 3
    DetailPrint "QuickSort: no process confirmed stopped after 3 attempts; proceeding"
    Goto ${_uniq}_done
  ${EndIf}
  ; taskkill /im is case-insensitive (one call covers every binary-name casing)
  ; and synchronous: it returns only after the process has terminated. Its exit
  ; code is the only locale-independent "is it running" signal: 0 = a process
  ; was terminated, 128 = no process matched. Parsing the output text is
  ; fragile: taskkill prints a localized "not found" message on stderr, and
  ; tasklist's CSV output prints a localized "INFO: No tasks are running..."
  ; line to STDOUT when nothing matches — so both the empty-string check and
  ; any text match break on localized Windows.
  nsExec::ExecToStack 'taskkill /f /im "${BINARY_NAME}"'
  Pop $1 ; exit code
  Pop $2 ; console output
  ${If} $1 == 0
    ; Killed. Give the process a moment to release locks on its exe and on the
    ; shell-extension DLL next to it.
    Sleep 500
    Goto ${_uniq}_done
  ${EndIf}
  Sleep 500
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
  ; under WOW64, so Explorer must be relaunched via "$WINDIR\explorer.exe" (the
  ; real 64-bit shell) — never System32\explorer.exe or Sysnative: WOW64
  ; redirects them to a stub, and System32 contains no explorer.exe at all.
  ; Full background of the Sysnative failure: wiki error-history.md
  ; (QA 06.09.2026, line 84; 07.09.2026, lines 70-72).
  DetailPrint "Restarting Windows Explorer to unload the shell extension..."
  nsExec::Exec 'taskkill /f /im explorer.exe'

  ; Wait for the old shell to release the mapped context_menu_dll.dll, then
  ; probe whether it actually came back. Since Windows 10, killing explorer.exe
  ; via taskkill USUALLY triggers an automatic restart within ~200 ms, so a
  ; poll loop would see the process never absent and burn up to 5 s — reads as
  ; a hang inside an uninstaller (QA 07.09.2026); keep the fixed 500 ms wait
  ; from that fix. BUT the auto-restart is NOT guaranteed: QA 12.09.2026 showed
  ; the shell staying dead after uninstall (user had to start it manually), so
  ; probe the process list ONCE and relaunch the 64-bit shell only when
  ; explorer.exe is absent. Launching it while the shell is already up would
  ; pop a spurious File Explorer window on top of the restored desktop — the
  ; probe guards against that (mirrors the fixed-sleep design of
  ; restart_explorer() in src-tauri src/com.rs).
  Sleep 500
  ; Same locale pitfall as QuickSortStopRunning: tasklist prints a localized
  ; "INFO: ..." line to STDOUT when nothing matches, so detect the running
  ; shell by SEARCHING the output for the image name ("explorer.exe" appears
  ; in every process row), not by emptiness or text equality. This block runs
  ; in the uninstaller only, so the un.-variant of the StrFunc function is
  ; legal here (Call to un.StrLoc inside uninstall sections is allowed).
  ; Case-insensitive search ("<"); $3 must not clobber any live register.
  nsExec::ExecToStack 'tasklist /fi "imagename eq explorer.exe" /fo csv /nh'
  Pop $1 ; exit code
  Pop $2 ; console output
  ${UnStrLoc} $3 "$2" "explorer.exe" "<"
  ${If} $3 == ""
    ; Real 64-bit shell. nsExec::Exec does not wait for the never-exiting shell
    ; process and shows no "Running: ..." popup (unlike Exec).
    nsExec::Exec '"$WINDIR\explorer.exe"'
    DetailPrint "QuickSort uninstall: Explorer did not auto-restart, relaunched"
  ${Else}
    DetailPrint "QuickSort uninstall: Explorer alive (Windows restarted it)"
  ${EndIf}
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
