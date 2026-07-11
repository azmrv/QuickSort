# Phase 2: Детальный аудит кода и планирование рефакторинга

**Дата начала:** 2026-07-11  
**Цель:** Выявить конкретные проблемы в исходном коде, составить приоритизированный список задач на рефакторинг.

---

## Шаг 1. Аудит Domain Layer (`quicksort-domain`)

**Файлы для проверки:**
- `crates/quicksort-domain/src/models/folder.rs`
- `crates/quicksort-domain/src/models/operation.rs`
- `crates/quicksort-domain/src/value_objects/windows_path.rs`
- `crates/quicksort-domain/src/events/mod.rs`

**Что проверять:**
- Все ли инварианты проверяются при создании объектов?
- Есть ли неиспользуемые методы или поля?
- Корректна ли логика `reverse()` для операций?
- Используются ли доменные события для важных изменений?

**Результат:** Список проблем с приоритетами (High/Medium/Low).

---

## Шаг 2. Аудит Application Layer (`quicksort-application`)

**Файлы для проверки:**
- `crates/quicksort-application/src/use_cases/execute_operation.rs`
- `crates/quicksort-application/src/use_cases/manage_folders.rs`
- `crates/quicksort-application/src/facade.rs`
- `crates/quicksort-application/src/ports/inbound.rs`
- `crates/quicksort-application/src/ports/outbound.rs`

**Что проверять:**
- Все ли Use Cases имеют таймауты и контексты?
- Корректно ли обрабатываются ошибки (не `anyhow` для бизнес-ошибок)?
- Используется ли фасад как единственная точка входа?
- Все ли порты документированы и стабильны?

**Результат:** Список проблем.

---

## Шаг 3. Аудит Infrastructure Layer (`quicksort-infrastructure`)

**Файлы для проверки:**
- `crates/quicksort-infrastructure/src/repositories/json_configuration.rs`
- `crates/quicksort-infrastructure/src/file_system/std_fs.rs`
- `crates/quicksort-infrastructure/src/conflict/default_resolver.rs`

**Что проверять:**
- Атомарны ли файловые операции?
- Есть ли retry-логика для временных ошибок?
- Кешируются ли данные конфигурации?
- Корректна ли обработка конфликтов?

**Результат:** Список проблем.

---

## Шаг 4. Аудит Adapters (Tauri + React)

**Файлы для проверки:**
- `src-tauri/src/commands_v2.rs`
- `src-tauri/src/main.rs`
- `src/components/FolderList.tsx`
- `src/pages/EditorPage.tsx`

**Что проверять:**
- Все ли Tauri-команды — тонкие обёртки над фасадом?
- Нет ли устаревшего кода из старых модулей (`folder/`, `move_engine/`)?
- Оптимизированы ли React-компоненты (мемоизация)?
- Обрабатываются ли ошибки на фронтенде?

**Результат:** Список проблем.

---

## Шаг 5. Составление итогового плана рефакторинга

Сгруппируйте все найденные проблемы в файле `docs/workspace/audit-issues.md` и назначьте им идентификаторы (TASK-001, TASK-002, ...). Отсортируйте по приоритету:

| ID | Слой | Описание | Приоритет |
|----|------|----------|-----------|
| TASK-001 | Domain | ... | High |
| TASK-002 | Application | ... | Medium |
| ... | ... | ... | ... |

Затем создайте файл `docs/workspace/refactoring-plan.md` с порядком выполнения задач:
1. Critical/High задачи (немедленно)
2. Medium задачи (в течение спринта)
3. Low задачи (при возможности)

---

## Чек-лист завершения Фазы 2

- [ ] Domain проанализирован, проблемы зафиксированы
- [ ] Application проанализирован, проблемы зафиксированы
- [ ] Infrastructure проанализирована, проблемы зафиксированы
- [ ] Adapters проанализированы, проблемы зафиксированы
- [ ] Итоговый план рефакторинга создан