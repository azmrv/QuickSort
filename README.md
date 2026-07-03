```markdown
# QuickSort


**QuickSort** – утилита для Windows 10/11, которая добавляет в контекстное меню Проводника каскадный пункт «QuickSort» со списком ваших избранных папок. Один клик – и файлы мгновенно перемещаются.  
Больше никаких перетаскиваний, лишних окон и постоянного поиска нужной директории.

Tauri + React + Typescript

---

## Оглавление
- [Возможности](#-возможности)
- [Технологии](#-технологии)
- [Зависимости](#-зависимости)
- [Установка и сборка](#-установка-и-сборка)
- [Использование](#-использование)
- [Благодарности](#-благодарности)
- [Лицензия](#-лицензия)

---

## ✨ Возможности

- **Каскадное контекстное меню** – избранные папки отображаются прямо в меню Проводника (без UAC и лишних окон).
- **Мгновенное перемещение** – при выборе папки файлы сразу переносятся (используется атомарный `rename`).
- **Окно выбора папки** – для доступа ко всем папкам из конфигурации (кнопка «📂 Другие папки...»).
- **Редактор папок** – удобный GUI на Tauri + React для управления списком: добавление, переименование, избранное.
- **Лог событий** – история всех операций (перемещения, изменения конфига) сохраняется локально.
- **Тёмная тема** – встроенная поддержка светлой и тёмной схем.
- **Системный трей** – приложение может работать в фоне, не загромождая панель задач.

---

## 🛠️ Технологии

- **Rust** – ядро приложения, COM-сервер, CLI-режим, работа с реестром.
- **Tauri 2** – легковесный GUI-фреймворк (вместо Electron).
- **React + TypeScript** – фронтенд редактора.
- **Ant Design** – UI-библиотека (компоненты, иконки, темизация).
- **Windows COM** – Shell Extension (DLL), загружаемый в Проводник.
- **win-ctx** – высокоуровневая работа с реестром для контекстного меню.
- **windows-rs** – официальные биндинги Microsoft к WinAPI.

---

## 📦 Зависимости

### Rust (core / Tauri)
```toml
tauri = "2"
tauri-plugin-dialog = "2"
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
winreg = "0.56"
clap = { version = "4", features = ["derive"] }
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
directories = "6"
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
parking_lot = "0.12"
winctx = "1.4"
```

### Frontend (React)
```json
"react": "^18",
"antd": "^5",
"@ant-design/icons": "^5",
"@tauri-apps/api": "^2",
"@tauri-apps/plugin-dialog": "^2",
"vite": "^7"
```

### COM-сервер (context-menu-dll)
```toml
windows = "0.61"
parking_lot = "0.12"
log = "0.4"
simplelog = "0.12"
```

Полный список зависимостей смотрите в `src-tauri/Cargo.toml`, `context-menu-dll/Cargo.toml` и `package.json`.

---

## 🔧 Установка и сборка

### Предварительные требования
- Windows 10/11 (64-бит)
- [Rust](https://rustup.rs) (stable, `x86_64-pc-windows-msvc`)
- [Node.js](https://nodejs.org) (LTS)
- [Tauri CLI](https://tauri.app) (`cargo install tauri-cli`)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/ru/visual-cpp-build-tools/) (компонент "Разработка классических приложений на C++" с Windows SDK)

### Сборка
```bash
git clone https://github.com/yourname/quicksort.git
cd quicksort
npm install
cargo build --all --release
```

### Регистрация COM-сервера (для динамического меню)
1. Соберите DLL: `cd context-menu-dll && cargo build --release`.
2. Запустите `register.reg` из корня проекта с правами администратора.
3. Перезапустите Проводник (`taskkill /f /im explorer.exe && start explorer.exe`).

### Запуск редактора
```bash
npm run tauri dev   # режим разработки
# или
cargo run           # только бэкенд (GUI без hot-reload)
```

Для продакшн-сборки:
```bash
npm run tauri build
```
Готовый установщик будет в `src-tauri/target/release/bundle`.

---

## 📖 Использование

1. Запустите `QuickSort.exe`. В системном трее появится иконка.
2. В окне редактора добавьте папки (кнопка «Добавить папку»), отметьте избранные звёздочкой и нажмите «Применить».
3. Теперь в Проводнике при правом клике на любом файле или папке появится меню **QuickSort** с вашими избранными папками.  
   Выберите папку – файл мгновенно переместится.
4. Для доступа ко всем папкам (не только избранным) используйте пункт «📂 Другие папки...» – откроется окно выбора.
5. Все действия записываются во вкладку «Лог». При закрытии окна приложение продолжает работать в трее.

---

## 🙏 Благодарности

Этот проект вдохновлён работами многих талантливых разработчиков Rust-сообщества:

- **[PaulDance](https://gist.github.com/PaulDance)** – за превосходный [Gist с примером Shell Extension](https://gist.github.com/PaulDance), который стал основой для нашего COM-сервера.
- **[ahaoboy](https://github.com/ahaoboy)** – за [rcm-com](https://github.com/ahaoboy/rcm-com) и [windows-contextmenu-manager](https://dev.to/ahaoboy/windows-contextmenu-manager-tauri-and-rust-3l9b), которые помогли разобраться в архитектуре.
- **[ppound](https://github.com/ppound)** – за [xmp-reader](https://github.com/ppound/xmp-reader), ещё один отличный пример Shell Extension на Rust.
- **[acdvs](https://github.com/acdvs)** – за библиотеку [winctx-rs](https://github.com/acdvs/winctx-rs), которая облегчила работу с контекстным меню.
- **[Microsoft](https://github.com/microsoft)** – за [windows-rs](https://github.com/microsoft/windows-rs), открывшую доступ к WinAPI из Rust.

Особая благодарность всем мейнтейнерам крейтов, использованных в проекте.

---

## 📄 Лицензия

Проект распространяется под лицензией **MIT**.  
Это учебный проект, но каждый может свободно использовать, модифицировать и распространять его в любых целях.

Полный текст лицензии: [LICENSE](LICENSE)

---

**QuickSort** – наводите порядок в файлах так же быстро, как называете папку!  
Если у вас есть идеи или предложения – создавайте issue или pull request. Вместе сделаем проводник удобнее.
```
