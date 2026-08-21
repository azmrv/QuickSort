## Спецификация проекта QuickSort

**Версия:** 3.0  
**Дата:** 2026-07-16  
**Статус:** Активна  
**Основание:** Анализ структуры репозитория, ADR, текущего состояния разработки и документации агентов.

---

### 1. ОБЩЕЕ ОПИСАНИЕ

#### 1.1. Назначение
QuickSort — утилита для Windows 10/11, интегрирующаяся в Проводник через контекстное меню. Предоставляет быстрый доступ к избранным папкам, позволяя перемещать/копировать файлы одним кликом. Включает графический редактор для управления папками, историю операций с возможностью отката и системный трей.

#### 1.2. Ключевые сценарии
- **Быстрое перемещение** – ПКМ на файле → «QuickSort» → выбор папки из подменю → перемещение.
- **Расширенный выбор** – ПКМ → «Другие папки…» → открывается окно выбора со списком всех папок.
- **Управление папками** – GUI-редактор (добавление, удаление, переименование, сортировка, избранное).
- **История и откат** – просмотр всех операций и отмена последней (включая восстановление удалённых).
- **Системный трей** – иконка в трее для быстрого открытия редактора и выхода.

---

### 2. АРХИТЕКТУРА

#### 2.1. Принципы (ADR)
- **Clean Architecture + DDD** – бизнес-логика изолирована от внешних систем (ADR-001, ADR-003).
- **Dependency Rule** – зависимости направлены внутрь, к Domain (ADR-002).
- **Domain Events** – все изменения состояния порождают события (ADR-005).
- **Стабильные порты** – интерфейсы между слоями определены и стабильны (ADR-006).
- **Операции как первичные сущности** – каждая операция (Move, Copy, Delete, Rename) представлена отдельным Use Case (ADR-004).

#### 2.2. Слои и модули (Cargo workspace)

```
QuickSort/
├── crates/
│   ├── quicksort-domain           # Чистая бизнес-логика (сущности, VO, события)
│   ├── quicksort-application      # Use Cases, порты (inbound/outbound), DTO, фасад
│   ├── quicksort-infrastructure   # Реализации портов (JSON, файловая система, UUID, часы)
│   ├── quicksort-ipc-contract     # Контракты для межпроцессного взаимодействия (Named Pipe)
│   ├── quicksort-gui              # (зарезервирован, НЕ в workspace) будущий GUI-адаптер
│   └── quicksort-shell-extension  # (зарезервирован, НЕ в workspace) будущая обёртка для COM-сервера
├── context-menu-dll/              # Отдельный крейт для COM-сервера (DLL)
│   └── src/pipe_client/           # Клиент для общения с Tauri через Named Pipe
├── src-tauri/                     # Основной GUI (Tauri 2) + IPC-сервер
│   ├── src/commands/              # Tauri-команды (обёртки над фасадом)
│   ├── src/ipc/                   # Реализация Named Pipe сервера
│   └── src/folder/                # (устаревший) – будет заменён на фасад
├── src/                           # React-фронтенд (Vite, TypeScript, Ant Design)
├── docs/                          # Документация
│   ├── adr/                       # Архитектурные решения (001–006)
│   ├── architecture/              # Модели, глоссарий, видение
│   ├── specs/                     # Детальные спецификации Use Case
│   ├── standards/                 # Стандарты кодирования, тестирования
│   └── translation/               # Многоязычные версии README
├── tests/                         # Тесты
│   ├── acceptance/                # Приёмочные тесты
│   ├── contract/                  # Контрактные тесты
│   ├── integration/               # Интеграционные тесты
│   └── property/                  # Property-based тесты
└── workflows/                     # GitHub Actions (CI, архитектурные проверки)
```

#### 2.3. Межпроцессное взаимодействие (IPC)
- **Named Pipe** – используется для связи между `context-menu-dll` (загруженной в explorer.exe) и основным процессом Tauri.
- **Контракт** – описан в крейте `quicksort-ipc-contract`.
- **Сервер** – реализован в `src-tauri/src/ipc/server.rs`.
- **Клиент** – реализован в `context-menu-dll/src/pipe_client/`.
- **Протокол** – простой текстовый/бинарный фрейминг.

#### 2.4. Коммуникация Tauri ↔ Фронтенд
- Используется `@tauri-apps/api/core` – вызовы команд через `invoke()`.
- Команды определены в `src-tauri/src/commands/` (thin wrappers над фасадом).

#### 2.5. Агенты и MCP (AI/LLM интеграция)

В проекте используются AI-агенты через OpenCode и Cline с подключёнными MCP-серверами:

| Инструмент | Назначение |
|------------|------------|
| **OpenCode** | Основной движок для агентной разработки (команды `/plan-change`, `/write-spec`, `/run-plan`). |
| **Cline (VS Code)** | Интерактивный агент-ассистент с поддержкой MCP. |
| **Graphify** | Построение графа кодовой базы для быстрой навигации и анализа зависимостей. |
| **sequential-thinking** | Разбивка сложных задач на шаги. |
| **memory-graph** | Сохранение контекста между сессиями. |
| **upstash-context7** | Поиск документации по Rust-крейтам. |

---

### 3. КЛЮЧЕВЫЕ МОДУЛИ И ИХ ОТВЕТСТВЕННОСТЬ

#### 3.1. Domain (`quicksort-domain`)
- **Сущности** – `Folder`, `Operation`.
- **Value Objects** – `FolderId`, `OperationId`, `WindowsPath`.
- **События** – `FolderAdded`, `FolderRemoved`, `OperationCompleted`, `OperationUndone`.
- **Доменные сервисы** – вычисление обратной операции, валидация путей.

**Статус:** **завершён**, тесты проходят.

#### 3.2. Application (`quicksort-application`)
- **Inbound порты:**
  - `ExecuteOperation` – выполнение Move/Copy/Delete/Rename.
  - `UndoOperation` – откат операции.
  - `GetFolders` – получение списка папок.
  - `ManageFolders` – CRUD для папок.
- **Outbound порты:**
  - `ConfigurationRepository`, `OperationRepository`, `FileSystem`, `IdGenerator`, `Clock`, `ConflictResolver`.
- **DTO** – `OperationCommand`, `OperationResult`.
- **Фасад** – `ApplicationFacade` — единая точка входа для Tauri.

**Статус:** **в процессе**. Исправляются ошибки компиляции после рефакторинга.

#### 3.3. Infrastructure (`quicksort-infrastructure`)
- **Реализации репозиториев:**
  - `JsonConfigurationRepository` – папки в `%APPDATA%\QuickSort\folders.json`.
  - `JsonOperationRepository` – история операций в `operations.json` (лимит 500 записей).
  - `InMemoryOperationRepository` – для тестов.
- **Файловая система** – `StdFileSystem`.
- **Генератор ID** – `UuidGenerator`.
- **Часы** – `SystemClock`.
- **Разрешитель конфликтов** – `DefaultConflictResolver`.

**Статус:** **планируется**, реализация начата.

#### 3.4. Context Menu DLL (`context-menu-dll`)
- **COM-сервер** – реализует `IShellExtInit` и `IContextMenu`.
- **Регистрация** – через `.reg` файл или инсталлятор.
- **Логирование** – в `%TEMP%\quicksort_dll.log`.
- **IPC клиент** – подключается к Named Pipe серверу Tauri.

**Статус:** **в разработке**, базовая регистрация работает.

#### 3.5. GUI (Tauri + React)
- **Бэкенд (Rust)** – `src-tauri/`: трей, CLI, IPC-сервер, команды-обёртки над фасадом.
- **Фронтенд (React)** – `src/`: страницы `EditorPage`, `LogPage`, `SelectorPage`, `SettingsPage`, `AboutPage`.
- **Сборка** – через Vite, Tauri CLI.

**Статус:** **функционал улучшен**, UI готов к интеграции с новым фасадом.

---

### 4. ФУНКЦИОНАЛЬНЫЕ ТРЕБОВАНИЯ

*Без изменений (F1–F38 остаются актуальными).*

---

### 5. НЕФУНКЦИОНАЛЬНЫЕ ТРЕБОВАНИЯ

*Без изменений (NF1–NF16 остаются актуальными).*

---

### 6. ДОМЕННАЯ МОДЕЛЬ

*Без изменений — модели `Folder`, `Operation`, Value Objects и события определены корректно.*

---

### 7. ТЕХНИЧЕСКИЙ СТЕК (подробно)

| Компонент | Технологии | Версии |
|-----------|------------|--------|
| **Backend Rust** | Rust 2024, Tokio, Serde, Serde_json, anyhow, thiserror, tracing, tracing-subscriber, uuid, chrono, camino | — |
| **GUI Framework** | Tauri 2, tauri-plugin-dialog, tauri-plugin-opener, tauri-plugin-window | 2.x |
| **Frontend** | React 19, TypeScript 5, Vite 7, Ant Design 5, @ant-design/icons | — |
| **Windows Integration** | winreg (реестр), windows-rs (COM), win-ctx (устаревает) | — |
| **IPC** | Named Pipes (tokio::net::windows::named_pipe) | — |
| **Тестирование** | cargo test, proptest, mockall | — |
| **AI/LLM интеграция** | OpenCode, Cline, MCP-серверы (Graphify, sequential-thinking, memory-graph, upstash-context7) | — |
| **CI/CD** | GitHub Actions (workflows/ci.yml, architecture-tests.yml) | — |

---

### 8. РАЗВЁРТЫВАНИЕ

*Без изменений — режимы запуска и установка остаются актуальными.*

---

### 9. ТЕСТИРОВАНИЕ

*Без изменений — структура тестов сохраняется.*

---

### 10. СТАТУС РАЗРАБОТКИ (актуально на 2026-08-20)

| Компонент | Статус | Примечания |
|-----------|--------|------------|
| Domain (Folder, Operation) | ✅ **Завершено** | Сущности, VO, события, тесты — готовы |
| Application (Use Cases) | ❌ **Не компилируется** | 13+ ошибок: неправильные имена, дублирующие методы, несуществующие импорты |
| Infrastructure (репозитории, FS) | ✅ **Завершено** | JsonConfigurationRepository, InMemoryOperationRepository, StdFileSystem — реализованы |
| Tauri-команды | ❌ **Не компиляется** | Несуществующие модули (`folder`, `move_engine`), `unimplemented!()` для Undo |
| Фронтенд (React) | ✅ **Завершено** | Все страницы работают (Editor, Selector, Log, Settings, About) |
| IPC (Named Pipe) | ⏳ **Placeholder** | Сервер запускается, но не подключён к фасаду |
| Context Menu DLL (COM) | ⏳ **В разработке** | Базовая регистрация и IPC — работают |
| UndoOperationUseCase | ⏳ **Не подключён** | Use case написан, но `unimplemented!()` в main.rs |
| Интеграция DLL с IPC | ⏳ **Частично** | Клиент подключается, требуется отладка |
| Удаление старого кода | ⏳ **Планируется** | Legacy-модули (`folder`, `move_engine`, `activity_log`) ещё активны |
| Многоязычность | ⏳ **Частично** | Есть перевод README на русский, китайский, испанский, немецкий |

---

### 11. ПЛАНЫ НА БЛИЖАЙШЕЕ БУДУЩЕЕ

1. **Исправить ошибки компиляции в `quicksort-application`** — текущий блокер.
2. **Завершить реализацию `UndoOperationUseCase`** — откат Move/Copy/Rename.
3. **Реализовать `JsonConfigurationRepository` и `JsonOperationRepository`** — инфраструктура хранения.
4. **Интегрировать фасад с Tauri-командами** — подключить Use Cases к GUI.
5. **Завершить интеграцию COM DLL** — стабильная работа через Named Pipe.
6. **Добавить поддержку Copy, Delete, Rename** — расширить `ExecuteOperation`.
7. **Миграция на SQLite** — замена JSON-репозиториев (долгосрочно).
8. **Сборка инсталлятора** — MSI или NSIS для установки.

---

### 12. РИСКИ И МИТИГАЦИИ

*Без изменений — риски остаются актуальными.*

---

### 13. РУКОВОДСТВО ДЛЯ AI-АГЕНТОВ (OpenCode / Cline)

#### 13.1. Контекст проекта
- **Корень:** `D:\ITProjects\RUST\quicksort`
- **Основная ветка:** `main`
- **Документация:** папка `docs/` — содержит ADR, архитектуру, стандарты, роли агентов.
- **Конфигурация агентов:** `opencode/` (OpenCode), `.clinerules.md` (Cline).
- **MCP-серверы:** Graphify, sequential-thinking, memory-graph, upstash-context7.

#### 13.2. Ключевые команды OpenCode
- `/init` – инициализация проекта.
- `/plan-change` – инициация новой задачи.
- `/write-spec` – создание спецификации.
- `/write-test-plan` – план тестирования.
- `/write-plan` – поэтапный план.
- `/run-plan` – выполнение плана.
- `/review` – ревью кода.
- `/commit` – коммит.
- `/pr` – создание Pull Request.

#### 13.3. Требования к изменениям
- **Dependency Rule** — не нарушать.
- **Документирование** — все публичные функции `///`.
- **Тесты** — минимум 3 сценария на Use Case.
- **Логирование** — `tracing::info!`, `tracing::error!`.
- **Обработка ошибок** — `anyhow::Result` в Application, конкретные ошибки в Domain.
- **Форматирование** — `cargo fmt`, `npx prettier`.

#### 13.4. Доступные ресурсы
- **SPECIFICATION.md** – этот документ.
- **ADR** – `docs/adr/`.
- **Стандарты** – `docs/standards/`.
- **Модели** – `docs/architecture/`.
- **Роли агентов** – `docs/opencode/agents/roles/`.

---

### 14. ИСТОРИЯ ИЗМЕНЕНИЙ

| Дата | Версия | Изменения |
|------|--------|-----------|
| 2026-07-11 | 1.0 | Первоначальная версия |
| 2026-07-11 | 2.0 | Добавлены IPC, COM-сервер, тесты, планы |
| 2026-07-16 | 3.0 | **Актуализированы статусы, добавлены агенты, MCP, обновлены планы и архитектура** |
| 2026-08-20 | 3.1 | **Добавлены ADR-007 (ApplicationFacade), ADR-008 (DTO Design), ADR-009 (Legacy Migration). Исправлена документация: домен зависит от chrono/serde/thiserror, статус Application исправлен на "не компилируется"** |

