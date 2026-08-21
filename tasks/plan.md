# Implementation Plan: Settings, Duplicates, Version, Bug Fixes

## Overview
Реализация системы настроек, проверки на дубликаты, единого источника версии и исправление критических багов (перенос, иероглифы в контекстном меню).

---

## Phase 0: Critical Bug Fixes (Immediate)

### Task 0.1: Fix JSON field name mismatch (DLL reads wrong fields)
**Description:** DLL читает `favorite`/`order`, а репозиторий пишет `is_favorite`/`sort_order`. Это ломает десериализацию в DLL и вызывает мусор в контекстном меню.

**Acceptance criteria:**
- [ ] DLL `shellext.rs` `FolderData` использует `#[serde(alias = "is_favorite")]` и `#[serde(alias = "sort_order")]`
- [ ] ИЛИ репозиторий пишет `favorite`/`order` (меньше изменений)
- [ ] Иероглифы в контекстном меню исчезли

**Files:**
- `context-menu-dll/src/shellext.rs` (строки 440-448)
- `crates/quicksort-infrastructure/src/repository/json_configuration_repository.rs` (строки 19-26)

**Scope:** S (1-2 файла)

---

### Task 0.2: Fix move_file between different drives
**Description:** `tokio_fs::rename` падает при перемещении между дисками. Нужна реализация copy+delete для cross-drive moves.

**Acceptance criteria:**
- [ ] `move_file` проверяет, находятся ли from/to на одном диске
- [ ] Если разные диски: copy + delete original
- [ ] Если один диск: rename (быстро)
- [ ] Все тесты проходят

**Files:**
- `crates/quicksort-infrastructure/src/filesystem/std_file_system.rs` (строки 41-51)

**Scope:** S (1 файл)

---

### Task 0.3: Unify version number
**Description:** Версия разбросана по 5+ файлам. Сделать Cargo.toml единственным источником.

**Acceptance criteria:**
- [ ] Добавить Tauri команду `get_app_version` → `env!("CARGO_PKG_VERSION")`
- [ ] `App.tsx` загружает версию через `invoke('get_app_version')`
- [ ] `AboutPage.tsx` загружает версию через `invoke('get_app_version')`
- [ ] Убрать хардкоженные версии из React-компонентов
- [ ] Синхронизировать `package.json` и `tauri.conf.json` с `Cargo.toml` (все → `0.2.0`)

**Files:**
- `src-tauri/src/commands/mod.rs`
- `src/App.tsx`
- `src/pages/AboutPage.tsx`
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

**Scope:** M (3-5 файлов)

---

## Checkpoint: Bug Fixes
- [ ] `cargo clippy --workspace -- -D warnings` — 0 errors
- [ ] `cargo test --workspace` — all pass
- [ ] Контекстное меню отображает русские имена корректно
- [ ] Перенос между дисками работает

---

## Phase 1: Settings System

### Task 1.1: Domain — Settings entity
**Description:** Создать сущность `Settings` в domain слое.

**Acceptance criteria:**
- [ ] `crates/quicksort-domain/src/entities/settings.rs` — `Settings`, `OperationType`, `OverwritePolicy`, `DuplicateCheckMode`, `DuplicateCheckConfig`
- [ ] Значения по умолчанию: Move, Skip, name-based check
- [ ] Unit tests

**Files:**
- `crates/quicksort-domain/src/entities/settings.rs` (новый)
- `crates/quicksort-domain/src/lib.rs` (ре-экспорт)

**Scope:** S

---

### Task 1.2: Application — SettingsRepository port + use cases
**Description:** Порт и use cases для загрузки/сохранения настроек.

**Acceptance criteria:**
- [ ] `crates/quicksort-application/src/ports/outbound/settings_repository.rs` — `load`, `save`
- [ ] `crates/quicksort-application/src/use_cases/settings.rs` — `LoadSettings`, `SaveSettings`
- [ ] Unit tests с моками

**Files:**
- `crates/quicksort-application/src/ports/outbound/settings_repository.rs` (новый)
- `crates/quicksort-application/src/ports/outbound/mod.rs` (ре-экспорт)
- `crates/quicksort-application/src/use_cases/settings.rs` (новый)
- `crates/quicksort-application/src/lib.rs`

**Scope:** M

---

### Task 1.3: Infrastructure — JsonSettingsRepository
**Description:** Реализация хранения настроек в JSON.

**Acceptance criteria:**
- [ ] `crates/quicksort-infrastructure/src/repository/json_settings_repository.rs`
- [ ] Файл: `%LOCALAPPDATA%/QuickSort/settings.json`
- [ ] Автосоздание с defaults если файла нет
- [ ] Тесты

**Files:**
- `crates/quicksort-infrastructure/src/repository/json_settings_repository.rs` (новый)
- `crates/quicksort-infrastructure/src/repository/mod.rs`

**Scope:** S

---

### Task 1.4: Adapter — Tauri commands
**Description:** Команды для фронтенда.

**Acceptance criteria:**
- [ ] `get_settings` → `SettingsDTO`
- [ ] `save_settings(SettingsDTO)` → `Result`
- [ ] Подключено к фасаду

**Files:**
- `src-tauri/src/commands/mod.rs`
- `crates/quicksort-application/src/ports/inbound/facade.rs`
- `crates/quicksort-application/src/ports/inbound/facade_impl.rs`

**Scope:** M

---

### Task 1.5: Frontend — Settings page
**Description:** Расширить страницу настроек.

**Acceptance criteria:**
- [ ] Секция "Действия по умолчанию": переключатель Move/Copy
- [ ] Секция "При обнаружении дубликатов": Skip/Overwrite/AutoRename
- [ ] Секция "Проверка дубликатов": enabled, mode (name/size/content)
- [ ] Настройки сохраняются при изменении
- [ ] Стилизовано под текущий дизайн (тёмная тема, amber акценты)

**Files:**
- `src/pages/SettingsPage.tsx`

**Scope:** M

---

## Checkpoint: Settings
- [ ] Настройки загружаются/сохраняются
- [ ] Переключатели работают
- [ ] `cargo clippy` — чисто

---

## Phase 2: Duplicate Detection

### Task 2.1: Domain — DuplicateChecker entity
**Description:** Сущность результата проверки на дубликаты.

**Acceptance criteria:**
- [ ] `DuplicateCheckResult`, `DuplicateCheckMode` в domain
- [ ] Сервисный объект `DuplicateChecker`

**Files:**
- `crates/quicksort-domain/src/entities/duplicate_check.rs` (новый)

**Scope:** S

---

### Task 2.2: Infrastructure — DuplicateChecker impl
**Description:** Реализация三种 режимов проверки.

**Acceptance criteria:**
- [ ] `NameChecker` — `Path::exists()`
- [ ] `SizeChecker` — metadata len comparison
- [ ] `ContentChecker` — SHA-256 hash
- [ ] Все три реализуют trait `DuplicateChecker`
- [ ] Unit tests

**Files:**
- `crates/quicksort-infrastructure/src/duplicate_checker/mod.rs` (новый)
- `crates/quicksort-infrastructure/src/duplicate_checker/name_checker.rs`
- `crates/quicksort-infrastructure/src/duplicate_checker/size_checker.rs`
- `crates/quicksort-infrastructure/src/duplicate_checker/content_checker.rs`

**Scope:** M

---

### Task 2.3: Pipeline — Add duplicate detection phase
**Description:** Встроить проверку в пайплайн операций.

**Acceptance criteria:**
- [ ] Pipeline: validate → detect_duplicates → resolve → execute → log
- [ ] Если дубликат найден и policy = Skip → пропустить файл
- [ ] Если AutoRename → уникальное имя с timestamp
- [ ] Интеграционные тесты

**Files:**
- `crates/quicksort-application/src/pipeline/mod.rs`
- `crates/quicksort-application/src/use_cases/execute_operation.rs`

**Scope:** M

---

### Task 2.4: Frontend — Duplicate dialog
**Description:** Диалог при обнаружении дубликатов (только для интерактивного режима).

**Acceptance criteria:**
- [ ] Модальное окно: "Файл X уже существует. Выбрать действие?"
- [ ] Кнопки: Skip, Overwrite, AutoRename, Skip All
- [ ] Отображается информация о файле (размер, дата)

**Files:**
- `src/components/DuplicateDialog.tsx` (новый)
- `src/pages/SelectorPage.tsx`

**Scope:** M

---

## Checkpoint: Duplicates
- [ ] Проверка на дубликаты работает во всех режимах
- [ ] Диалог отображается корректно
- [ ] All tests pass

---

## Phase 3: All Folders Page

### Task 3.1: Design "All Folders" page
**Description:** Страница для просмотра и выбора всех папок.

**Acceptance criteria:**
- [ ] Список всех папок с иконками
- [ ] Поиск/фильтрация по имени
- [ ] Индикатор избранного (★)
- [ ] Кнопка "Добавить папку"
- [ ] Клик по папке → выбор для операции

**Files:**
- `src/pages/AllFoldersPage.tsx` (новый)
- `src/App.tsx` (роутинг)

**Scope:** M

---

### Task 3.2: Wire "Все папки..." from DLL to page
**Description:** При клике "Все папки..." в контекстном меню открывать страницу выбора папки.

**Acceptance criteria:**
- [ ] DLL запускает `quicksort.exe select-folder --file "<path>"`
- [ ] Фронтенд переключается на AllFoldersPage
- [ ] Выбор папки → execute operation

**Files:**
- `src/App.tsx`
- `src/pages/AllFoldersPage.tsx`

**Scope:** S

---

## Checkpoint: Complete
- [ ] Все acceptance criteria выполнены
- [ ] `cargo clippy --workspace -- -D warnings` — 0 errors
- [ ] `cargo test --workspace` — all pass
- [ ] `npm run build` — без ошибок
- [ ] Ручное тестирование: контекстное меню, настройки, дубликаты

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| SHA-256 хеширование медленное для больших файлов | Medium | Default = name check; content check opt-in |
| DLL field mismatch вызывает краш | High | Task 0.1 — немедленное исправление |
| Cross-drive move падает | High | Task 0.2 — copy+delete fallback |
| Настройки DLL не синхронизируются с GUI | Medium | DLL читает settings.json напрямую |

## Open Questions
- [ ] Нужен ли прогресс-бар при content-based duplicate check?
- [ ] Сохранять ли историю операций (undo) для перемещений через контекстное меню?
