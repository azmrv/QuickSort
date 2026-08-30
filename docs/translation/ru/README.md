<div align="center">

# QuickSort

**Файловый менеджер нового поколения для Windows.**

QuickSort сочетает скорость shell-расширения с мощью современной файловой системы. Нажмите правой кнопкой на файл в Проводнике, выберите папку — и QuickSort сделает всё остальное: с обнаружением дубликатов, историей операций и возможностью отмены.

[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-2-FFC131?style=flat&logo=tauri)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19-61DAFB?style=flat&logo=react)](https://react.dev)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

---

[English](../../README.md) | **Русский** | [中文](../cn/README.md) | [Deutsch](../de/README.md) | [Español](../es/README.md)

---

<div align="center">
 <a target="_blank" href="https://imageban.ru/show/2026/08/25/83fc5efaa96d7d4b84317e957f628e38/png"><img src="https://i8.imageban.ru/thumbs/2026.08.25/83fc5efaa96d7d4b84317e957f628e38.png" border="0" style='border: 1px solid #000000'></a> 
 <a target="_blank" href="https://imageban.ru/show/2026/08/25/b80ebefd51e54a070cc292b0f427fa49/png"><img src="https://i3.imageban.ru/thumbs/2026.08.25/b80ebefd51e54a070cc292b0f427fa49.png" border="0" style='border: 1px solid #000000'></a> 
 <a target="_blank" href="https://imageban.ru/show/2026/08/25/2c0472a156b387eda9d595667fd24381/png"><img src="https://i2.imageban.ru/thumbs/2026.08.25/2c0472a156b387eda9d595667fd24381.png" border="0" style='border: 1px solid #000000'></a> 
 <a target="_blank" href="https://imageban.ru/show/2026/08/25/f2007bd5ce1f0283353c986e2dd5d881/png"><img src="https://i7.imageban.ru/thumbs/2026.08.25/f2007bd5ce1f0283353c986e2dd5d881.png" border="0" style='border: 1px solid #000000'></a> 
 <a target="_blank" href="https://imageban.ru/show/2026/08/25/a784d47f47a701c2f6c01cb4f20544af/png"><img src="https://i7.imageban.ru/thumbs/2026.08.25/a784d47f47a701c2f6c01cb4f20544af.png" border="0" style='border: 1px solid #000000'></a>
</div>



## Возможности

### Основные

- **Каскадное контекстное меню** — избранные папки прямо в Проводнике (без UAC)
- **Мгновенное перемещение/копирование** — атомарные файловые операции с поддержкой разных дисков
- **Обнаружение дубликатов** — проверка по имени, размеру или SHA-256 хешу содержимого
- **История операций** — полный аудит всех файловых операций с возможностью отмены
- **Настраиваемые параметры** — тип операции по умолчанию, политика перезаписи, режим проверки дубликатов

### Интерфейс

- **Редактор папок** — добавление, переименование, избранное в удобном GUI
- **Выбор из всех папок** — поиск, фильтрация и выбор из всей библиотеки папок
- **Лог событий** — логирование бэкенда и фронтенда в реальном времени
- **Тёмная тема** — встроенные светлая и тёмная схемы с янтарными акцентами
- **Системный трей** — работает в фоне, не засоряя панель задач

### Умная установка

- **Нулевая установка** — один исполняемый файл, авто-регистрация COM-сервера при первом запуске
- **Портативный режим** — не требует прав администратора, всё работает в пространстве пользователя

## Видение

QuickSort развивается в полноценный файловый менеджер:

- **Командная строка** — интерактивный ввод с синтаксисом поиска в стиле Everything для расширенных файловых запросов, фильтрации и сортировки
- **Пакетные операции** — очередь обработки для масштабных перемещений с прогрессом
- **Умный анализ файлов** — обнаружение дубликатов по содержимому, распознавание типов, индексация метаданных

## Технологии

| Уровень | Технология |
|---------|-----------|
| Ядро | Rust (Clean Architecture + DDD) |
| GUI | Tauri 2 |
| Фронтенд | React 19 + TypeScript + Ant Design |
| Shell Extension | Windows COM (DLL) — независимый компонент |
| IPC | Named Pipes |
| WinAPI | windows-rs |

## Архитектура

QuickSort следует Clean Architecture с Domain-Driven Design:

```
Domain <- Application <- Infrastructure <- Adapters (src-tauri, context-menu-dll)
```

### Cargo Workspace

| Крейт | Роль | Описание |
|--------|------|----------|
| `quicksort-domain` | Domain | Сущности, value objects, события |
| `quicksort-application` | Application | Use cases, порты, DTO, фасад |
| `quicksort-infrastructure` | Infrastructure | JSON-репозитории, FileSystem, UUID, Clock |
| `quicksort-ipc-contract` | Contract | Контракты Named Pipe |
| `src-tauri` | Adapter | Tauri-приложение, CLI, IPC-сервер, регистрация COM |
| `context-menu-dll` | Adapter | COM Shell Extension (загружается Проводником) |

### DLL как независимый компонент

Контекстное меню DLL (`context-menu-dll`) — **независимый компонент**, который собирается и распространяется отдельно от основного приложения:

- **Приложение работает без DLL** — корректно деградирует (нет контекстного меню, все остальные функции работают)
- **DLL опциональна** — поместите `context_menu_dll.dll` рядом с `QuickSort.exe` для включения контекстного меню
- **Отдельная сборка** — DLL имеет свой процесс сборки, нет циклической зависимости
- **Обработка залоченных файлов** — build-скрипт обходит блокировки Windows Defender через паттерн переименования

```bash
# Сборка приложения (без зависимости от DLL)
npm run tauri build

# Сборка DLL отдельно
npm run build:dll

# Или безопасная сборка (обрабатывает залоченные файлы)
pwsh -NoProfile -File scripts/build-dll.ps1
```

## Сборка

### Предварительные требования

- Windows 10/11 (64-бит)
- [Rust](https://rustup.rs) (stable, `x86_64-pc-windows-msvc`)
- [Node.js](https://nodejs.org) (LTS)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Desktop development with C++)

### Команды

```bash
git clone https://github.com/azmrv/QuickSort.git
cd QuickSort
npm install
npm run tauri dev        # разработка
npm run tauri build      # продакшн-сборка
```

Установщик будет в `target/release/bundle`.

## Использование

1. Запустите `QuickSort.exe` — иконка появится в системном трее
2. Добавьте папки, отметьте избранные звёздочкой, нажмите «Применить»
3. Нажмите правой кнопкой на файл в Проводнике — выберите папку из меню QuickSort
4. Файлы перемещаются мгновенно. Обнаружение дубликатов работает автоматически.
5. Откройте вкладку «История» для просмотра или отмены операций.
6. Закройте окно — приложение продолжает работать в трее.

### Настройки

Настройте параметры на вкладке «Настройки»:
- **Операция по умолчанию**: Перемещение или Копирование
- **Политика перезаписи**: Пропустить, Перезапать или Авто-переименование
- **Режим проверки дубликатов**: Имя (быстро), Размер (средне) или Содержимое/SHA-256 (тщательно)

## Структура проекта

```
QuickSort/
  src-tauri/             # Tauri-адаптер, CLI, IPC-сервер
  context-menu-dll/      # COM Shell Extension (загружается Проводником)
  crates/
    quicksort-domain/          # Сущности, value objects, события
    quicksort-application/     # Use cases, порты, DTO
    quicksort-infrastructure/  # JSON-репозитории, FileSystem, UUID
    quicksort-ipc-contract/    # Контракты Named Pipe
  src/                   # React-фронтенд
  scripts/               # Build-хелперы (безопасная сборка DLL)
```

## Автор

**azmrv** - [@Fib511](https://t.me/Fib511) в Telegram

[![GitHub](https://img.shields.io/badge/GitHub-azmrv-181717?style=flat&logo=github)](https://github.com/azmrv)

## Благодарности

Проект вдохновлён работами многих талантливых разработчиков и проектов:

- [Christian Ghisler](https://www.ghisler.com) — [Total Commander](https://www.ghisler.com), эталон файловых менеджеров, вдохновивший архитектуру плагинов QuickSort (WCX/WDX/WFX/WLX)
- [PaulDance](https://gist.github.com/PaulDance) — пример Shell Extension, ставший основой нашего COM-сервера
- [ahaoboy](https://github.com/ahaoboy) — [rcm-com](https://github.com/ahaoboy/rcm-com) и [windows-contextmenu-manager](https://dev.to/ahaoboy/windows-contextmenu-manager-tauri-and-rust-3l9b)
- [ppound](https://github.com/ppound) — [xmp-reader](https://github.com/ppound/xmp-reader), ещё один пример Shell Extension
- [acdvs](https://github.com/acdvs) — библиотека [winctx-rs](https://github.com/acdvs/winctx-rs)
- [Microsoft](https://github.com/microsoft) — [windows-rs](https://github.com/microsoft/windows-rs) биндинги к WinAPI
- [voidtools](https://www.voidtools.com) — Everything, вдохновение для командной строки

## Лицензия

[MIT](LICENSE) — свободное использование, модификация и распространение.

---

<div align="center">

**QuickSort** — файловое управление нового поколения для Windows.

</div>



<div align="center">

 <a target="_blank" href="https://imageban.ru/show/2026/08/25/40f71dff67252d956c04edd889115666/png"><img src="https://i5.imageban.ru/thumbs/2026.08.25/40f71dff67252d956c04edd889115666.png" border="0" style='border: 1px solid #000000'></a> 
 <a target="_blank" href="https://imageban.ru/show/2026/08/25/66a4921c12ff6afc4f030dde050b02e2/png"><img src="https://i4.imageban.ru/thumbs/2026.08.25/66a4921c12ff6afc4f030dde050b02e2.png" border="0" style='border: 1px solid #000000'></a>  


</div>
