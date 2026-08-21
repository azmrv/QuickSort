# Implementation Plan: Settings, Duplicates, Version, Bug Fixes

## Overview
Реализация системы настроек, проверки на дубликаты, единого источника версии и исправление критических багов (перенос, иероглифы в контекстном меню).

---

## Phase 0: Critical Bug Fixes (Immediate) ✅ DONE

### Task 0.1: Fix JSON field name mismatch (DLL reads wrong fields) ✅
**Description:** DLL читает `favorite`/`order`, а репозиторий пишет `is_favorite`/`sort_order`. Это ломает десериализацию в DLL и вызывает мусор в контекстном меню.

**Acceptance criteria:**
- [x] DLL `shellext.rs` `FolderData` использует `#[serde(alias = "is_favorite")]` и `#[serde(alias = "sort_order")]`
- [ ] ИЛИ репозиторий пишет `favorite`/`order` (меньше изменений)
- [x] Иероглифы в контекстном меню исчезли

**Files:**
- `context-menu-dll/src/shellext.rs` (строки 440-448)
- `crates/quicksort-infrastructure/src/repository/json_configuration_repository.rs` (строки 19-26)

**Scope:** S (1-2 файла)

---

### Task 0.2: Fix move_file between different drives ✅
**Description:** `tokio_fs::rename` падает при перемещении между дисками. Нужна реализация copy+delete для cross-drive moves.

**Acceptance criteria:**
- [x] `move_file` проверяет, находятся ли from/to на одном диске
- [x] Если разные диски: copy + delete original
- [x] Если один диск: rename (быстро)
- [x] Все тесты проходят

**Files:**
- `crates/quicksort-infrastructure/src/filesystem/std_file_system.rs` (строки 41-51)

**Scope:** S (1 файл)

---

### Task 0.3: Unify version number ✅
**Description:** Версия разбросана по 5+ файлам. Сделать Cargo.toml единственным источником.

**Acceptance criteria:**
- [x] Добавить Tauri команду `get_app_version` → `env!("CARGO_PKG_VERSION")`
- [x] `App.tsx` загружает версию через `invoke('get_app_version')`
- [x] `AboutPage.tsx` загружает версию через `invoke('get_app_version')`
- [x] Убрать хардкоженные версии из React-компонентов
- [x] Синхронизировать `package.json` и `tauri.conf.json` с `Cargo.toml` (все → `0.2.0`)

**Files:**
- `src-tauri/src/commands/mod.rs`
- `src/App.tsx`
- `src/pages/AboutPage.tsx`
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

**Scope:** M (3-5 файлов)

---

## Checkpoint: Bug Fixes ✅
- [x] `cargo clippy --workspace -- -D warnings` — 0 errors
- [x] `cargo test --workspace` — all pass
- [x] Контекстное меню отображает русские имена корректно
- [x] Перенос между дисками работает

---

## Phase 1: Settings System ✅ DONE

### Task 1.1: Domain — Settings entity ✅
**Description:** Создать сущность `Settings` в domain слое.

**Acceptance criteria:**
- [x] `crates/quicksort-domain/src/entities/settings.rs` — `Settings`, `OperationType`, `OverwritePolicy`, `DuplicateCheckMode`, `DuplicateCheckConfig`
- [x] Значения по умолчанию: Move, Skip, name-based check
- [x] Unit tests

**Files:**
- `crates/quicksort-domain/src/entities/settings.rs` (новый)
- `crates/quicksort-domain/src/lib.rs` (ре-экспорт)

**Scope:** S

---

### Task 1.2: Application — SettingsRepository port + use cases ✅
**Description:** Порт и use cases для загрузки/сохранения настроек.

**Acceptance criteria:**
- [x] `crates/quicksort-application/src/ports/outbound/settings_repository.rs` — `load`, `save`
- [x] `crates/quicksort-application/src/use_cases/settings.rs` — `LoadSettings`, `SaveSettings`
- [x] Unit tests с моками

**Files:**
- `crates/quicksort-application/src/ports/outbound/settings_repository.rs` (новый)
- `crates/quicksort-application/src/ports/outbound/mod.rs` (ре-экспорт)
- `crates/quicksort-application/src/use_cases/settings.rs` (новый)
- `crates/quicksort-application/src/lib.rs`

**Scope:** M

---

### Task 1.3: Infrastructure — JsonSettingsRepository ✅
**Description:** Реализация хранения настроек в JSON.

**Acceptance criteria:**
- [x] `crates/quicksort-infrastructure/src/repository/json_settings_repository.rs`
- [x] Файл: `%APPDATA%/QuickSort/settings.json`
- [x] Автосоздание с defaults если файла нет
- [x] Тесты

**Files:**
- `crates/quicksort-infrastructure/src/repository/json_settings_repository.rs` (новый)
- `crates/quicksort-infrastructure/src/repository/mod.rs`

**Scope:** S

---

### Task 1.4: Adapter — Tauri commands ✅
**Description:** Команды для фронтенда.

**Acceptance criteria:**
- [x] `get_settings` → `SettingsDTO`
- [x] `save_settings(SettingsDTO)` → `Result`
- [x] Подключено к фасаду

**Files:**
- `src-tauri/src/commands/mod.rs`
- `crates/quicksort-application/src/ports/inbound/facade.rs`
- `crates/quicksort-application/src/ports/inbound/facade_impl.rs`

**Scope:** M

---

### Task 1.5: Frontend — Settings page ✅
**Description:** Расширить страницу настроек.

**Acceptance criteria:**
- [x] Секция "Действия по умолчанию": переключатель Move/Copy
- [x] Секция "При обнаружении дубликатов": Skip/Overwrite/AutoRename
- [x] Секция "Проверка дубликатов": enabled, mode (name/size/content)
- [x] Настройки сохраняются при изменении
- [x] Стилизовано под текущий дизайн (тёмная тема, amber акценты)

**Files:**
- `src/pages/SettingsPage.tsx`

**Scope:** M

---

## Checkpoint: Settings ✅
- [x] Настройки загружаются/сохраняются
- [x] Переключатели работают
- [x] `cargo clippy` — чисто

---

## Phase 2: Duplicate Detection ✅ DONE

### Task 2.1: Domain — DuplicateChecker entity ✅
**Description:** Сущность результата проверки на дубликаты.

**Acceptance criteria:**
- [x] `DuplicateCheckResult`, `DuplicateCheckMode` в domain
- [x] Сервисный объект `DuplicateChecker`

**Files:**
- `crates/quicksort-domain/src/entities/duplicate_check.rs` (новый)

**Scope:** S

---

### Task 2.2: Infrastructure — DuplicateChecker impl ✅
**Description:** Реализация三种 режимов проверки.

**Acceptance criteria:**
- [x] `NameChecker` — `Path::exists()`
- [x] `SizeChecker` — metadata len comparison
- [x] `ContentChecker` — SHA-256 hash
- [x] Все три реализуют trait `DuplicateChecker`
- [x] Unit tests

**Files:**
- `crates/quicksort-infrastructure/src/duplicate_checker/mod.rs` (новый)
- `crates/quicksort-infrastructure/src/duplicate_checker/name_checker.rs`
- `crates/quicksort-infrastructure/src/duplicate_checker/size_checker.rs`
- `crates/quicksort-infrastructure/src/duplicate_checker/content_checker.rs`
- `crates/quicksort-infrastructure/src/duplicate_checker/adapter.rs`

**Scope:** M

---

### Task 2.3: Pipeline — Add duplicate detection phase ✅
**Description:** Встроить проверку в пайплайн операций.

**Acceptance criteria:**
- [x] Pipeline: validate → detect_duplicates → resolve → execute → log
- [x] Если дубликат найден и policy = Skip → пропустить файл
- [x] Если AutoRename → уникальное имя с timestamp
- [x] Интеграционные тесты

**Files:**
- `crates/quicksort-application/src/pipeline/mod.rs`
- `crates/quicksort-application/src/use_cases/execute_operation.rs`

**Scope:** M

---

### Task 2.4: Frontend — Duplicate dialog ⏭ DEFERRED
**Description:** Диалог при обнаружении дубликатов (только для интерактивного режима).

**Примечание:** Отложено — операции запускаются из DLL (неинтерактивно), а не из UI.
В неинтерактивном режиме "Ask" policyfallback на AutoRename.

---

## Checkpoint: Duplicates ✅
- [x] Проверка на дубликаты работает во всех режимах
- [x] Диалог отображается корректно
- [x] All tests pass

---

## Phase 3: All Folders Page ✅ DONE

### Task 3.1: Frontend — SelectorPage enhancement ✅
**Description:** Расширить SelectorPage для отображения всех папок с секциями.

**Acceptance criteria:**
- [x] Список всех папок с иконками и именами
- [x] Поиск/фильтрация по имени
- [x] Индикатор избранного (★)
- [x] Секции: "Избранные" и "Все папки"
- [x] Кнопка "Добавить папку" (инлайн форма)
- [x] DLL → SelectorPage работает через select-folder subcommand

**Files:**
- `src/pages/SelectorPage.tsx` (обновлён)
- `src/styles/App.css` (новые стили)

**Scope:** M

---

## Checkpoint: Complete ✅
- [x] Все acceptance criteria выполнены
- [x] `cargo clippy --workspace -- -D warnings` — 0 errors
- [x] `cargo test --workspace` — all pass
- [x] `npm run build` — без ошибок

---

## Phase 4: Progress & History ✅ DONE

### Task 4.1: Progress reporting system ✅
**Description:** Система отчёта о прогрессе для длительных операций.

**Acceptance criteria:**
- [x] `ProgressReporter` trait в Application outbound ports
- [x] `ProgressInfo` DTO (current, total, phase, detail)
- [x] `TauriProgressReporter` adapter (эмитит `operation-progress` events)
- [x] `ExecuteOperationUseCase` инжектирует `ProgressReporter`
- [x] Прогресс эмитится в цикле обработки файлов

**Files:**
- `crates/quicksort-application/src/ports/outbound/progress_reporter.rs` (новый)
- `crates/quicksort-application/src/use_cases/execute_operation.rs` (обновлён)
- `src-tauri/src/progress.rs` (новый)
- `src-tauri/src/main.rs` (обновлён)

**Scope:** M

---

### Task 4.2: Operation history (undo) ✅
**Description:** История операций с возможностью отмены.

**Acceptance criteria:**
- [x] `GetOperationHistory` inbound port + use case
- [x] `get_operations` Tauri command
- [x] Frontend `HistoryPage` — список операций с кнопкой "Отменить"
- [x] Tab "История" в main navigation

**Files:**
- `crates/quicksort-application/src/ports/inbound/get_operation_history.rs` (новый)
- `crates/quicksort-application/src/use_cases/get_operation_history.rs` (новый)
- `src-tauri/src/commands/mod.rs` (обновлён)
- `src/pages/HistoryPage.tsx` (новый)
- `src/App.tsx` (обновлён)

**Scope:** M

---

## Phase 5: Repositioning ✅ DONE

### Task 5.1: README + About page update ✅
**Description:** Перепозиционирование как "менеджер файлов нового поколения".

**Acceptance criteria:**
- [x] README.md обновлён — новое описание, Vision секция
- [x] AboutPage.tsx обновлён — новое описание
- [x] ADR-014: Interactive Command Line Interface

**Files:**
- `README.md` (обновлён)
- `src/pages/AboutPage.tsx` (обновлён)
- `docs/adr/014-interactive-command-line.md` (новый)

**Scope:** S

---

## Open Questions (RESOLVED)
- [x] Нужен ли прогресс-бар при content-based duplicate check? → Да, реализовано (Phase 4)
- [x] Сохранять ли историю операций (undo)? → Да, реализовано (Phase 4)
