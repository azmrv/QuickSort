# Changelog

All notable changes to QuickSort are documented here. Format follows
[Conventional Commits](https://www.conventionalcommits.org/). Entries are
derived from `git log` (137 commits, 2026-07-02 → 2026-08-25) and the Wiki
`change-history.md`. Commit hashes are included for traceability.

---

## [Unreleased]

### Security
- Strict Content Security Policy for production builds (`script-src 'self'`, IPC allowlisted)
- DevTools disabled in production for `main` and `selector` windows
- `cargo-deny` policy (`deny.toml`) enforced in CI: advisories denied, license
  allowlist, duplicate-crate bans; 6 upstream "unmaintained" advisories
  (proc-macro-error, unic-*) are transitive with no upstream fix — tracked in `deny.toml`
- `Cargo.lock` tracked in git (locked application builds)

### Changed
- Development launches use a relaxed CSP overlay (`src-tauri/tauri.dev.conf.json`) via
  `npm run tauri:dev` (Vite HMR / React refresh require inline scripts in dev)

---

## [0.2.5.2] - 2026-09-07

### Fixed
- **nsis**: stop a running QuickSort silently (up to 3 retries) before install,
  uninstall and PREUNINSTALL instead of the blocking `CheckIfAppIsRunning`
  prompt (`39ef649`)
- **nsis**: guard `reinst_uninstall` against a missing/stale uninstaller
  (FileExists + registry cleanup) so upgrades no longer hang (`39ef649`)

### Dependencies
- Dependabot updates: actions/download-artifact 7→8, @vitejs/plugin-react + vite,
  antd, uuid, tauri-plugin-dialog, @ant-design/icons, tauri-plugin-opener,
  @tauri-apps/plugin-opener, tauri-plugin-single-instance, actions/upload-artifact
  (#25–#34)

---

## [0.2.2] - 2026-08-25

### Fixed
- **context-menu**: remove `Directory` and `Drive` handlers to prevent double
  entries for shortcuts (`.lnk`) (`7d4756c`)
- **context-menu**: improve handler registration and system-dialog filtering
  (skip Open/Save pickers) (`0da5663`)
- history persistence (InMemory → Json repository), selector dedup, theme sync
  with Windows, single-instance CLI forwarding, search removed from nav
  (`6cd3f43`)
- `cargo fmt` cleanup to pass CI format check (`9436b78`)

---

## [0.2.1] - 2026-08-24

### Changed
- decouple DLL from app build; DLL built separately as independent component
  (`df94da8`, `ed5d7f3`)
- add translations RU/CN/DE/ES (`df94da8`)

### Fixed
- **ci**: use shared target dir, fix portable ZIP path (`31ec26b`)
- **ci**: add `contents:write` permission for release creation (`c5e5aa4`)
- **ci**: fix security alerts + improve release pipeline (`be5d06a`)
- **deps**: update dependencies + fix DLL auto-copy (`3d7bab3`)
- **context-menu**: file context menu + remove Explorer restart from register
  (`c0a1568`)
- **context-menu**: context menu for files, re-move protection, UI button
  visibility, folder coloring (`6f08b8a`)

### Dependencies
- Dependabot updates: antd 6.5.0→6.6.1, tauri-plugin-dialog, async-trait,
  clap, serde_json, tokio, react (#7–#14)

---

## [0.2.0] - 2026-08-21

Major implementation push (single day, ~30 commits).

### Added
- Duplicate detection: `DuplicateChecker` with 3 modes (Name/Size/Content-SHA256)
  + `DuplicateDetectionPort` integrated into `ExecuteOperation` (`0016a2e`,
  `f8b7284`, `83a5afd`)
- User settings system (ADR-010) (`ba38c44`)
- Plugin system: traits (Phase 6/7), WCX adapter (Phase 8), PluginManager +
  PluginsPage (Phase 9) (`d768532`, `59fb805`, `cd7b7a4`)
- File search: `SearchQuery` (Everything-style, 27 tests), `FsFileSearch`,
  `search_files` command, CommandPalette (Ctrl+Shift+Space) (Phase 10a–e)
  (`cee07e9`, `f03ac29`, `ea4d00b`, `d1f2170`)
- Progress reporting, operation history, next-gen positioning (`ead544a`)
- Enhanced SelectorPage (favorite sections, inline add, search) (`09f3086`)
- Rust CI workflow (windows-latest) and release workflow (Windows installer)
  (`a9fe714`, `37f5afe`)
- License (MIT, commercial use allowed, reselling prohibited) (`3f4ff70`,
  `a515355`)

### Fixed
- **Phase 0**: JSON field mismatch, cross-drive move (copy+delete fallback),
  unified version (`9a2c6db`)
- **context-menu**: garbled glyphs (null terminator in `cch`), separator flag,
  favorites submenu, auto COM registration, smart portable auto-registration
  (`cb8377c`)

### Documentation
- README rewritten with clean design and author links (`74099d5`, `9703dd4`)

---

## [0.0.7] and earlier - 2026-07-02 → 2026-07-16

### Added
- Project start as "Quicksort framework" by iMininru (framework 0.0.1…0.0.7)
- ADR-001: Clean Architecture + DDD adopted (2026-07-07)

---

## Notes

- Many early commits use non-descriptive messages (`Update`, `Fix`, `Fixing`)
  without bodies, which limits automated changelog granularity. Adopting
  conventional commits consistently is recommended.
- A separate, deeper diff-level analysis of all 137 commits is tracked as a
  follow-up task (see Wiki `error-history.md` for the bug-level detail already
  captured).
- Generated 2026-08-25 from `git log` of `repo/`.
