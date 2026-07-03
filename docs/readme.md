# QuickSort – README (Многоязычный)

- [English](#english)
- [中文 (Chinese)](#中文)
- [Español (Spanish)](#español)
- [Deutsch (German)](#deutsch)

---

## English

# QuickSort

**QuickSort** – a Windows 10/11 utility that adds a cascading "QuickSort" item to the Explorer context menu with your favorite folders. One click – and files are moved instantly.  
No more dragging, extra windows, or hunting for the right directory.

Tauri + React + TypeScript

---

## Table of Contents
- [Features](#-features)
- [Technologies](#-technologies)
- [Dependencies](#-dependencies)
- [Installation and Build](#-installation-and-build)
- [Usage](#-usage)
- [Acknowledgments](#-acknowledgments)
- [License](#-license)

---

## ✨ Features

- **Cascading context menu** – favorite folders appear directly in the Explorer menu (no UAC or extra windows).
- **Instant moving** – selecting a folder immediately moves the files (using atomic `rename`).
- **Folder picker** – access all folders from the configuration (via the “📂 Other folders...” button).
- **Folder editor** – a convenient Tauri + React GUI to manage the list: add, rename, mark as favorite.
- **Event log** – a local history of all operations (moves, config changes).
- **Dark theme** – built‑in support for light and dark color schemes.
- **System tray** – the app can run in the background, keeping the taskbar clean.

---

## 🛠️ Technologies

- **Rust** – core of the application, COM server, CLI mode, registry handling.
- **Tauri 2** – lightweight GUI framework (instead of Electron).
- **React + TypeScript** – editor frontend.
- **Ant Design** – UI library (components, icons, theming).
- **Windows COM** – Shell Extension (DLL) loaded by Explorer.
- **win-ctx** – high‑level registry wrapper for context menus.
- **windows-rs** – official Microsoft WinAPI bindings for Rust.

---

## 📖 Usage

1. Launch `QuickSort.exe`. An icon appears in the system tray.
2. In the editor window, add folders (click “Add Folder”), mark favorites with a star, and click “Apply”.
3. Now, right‑click any file or folder in Explorer and you’ll see the **QuickSort** menu with your favorite folders.  
   Choose a folder – the file moves instantly.
4. To access all folders (not just favorites), use “📂 Other folders...” – a picker window opens.
5. Every action is recorded in the “Log” tab. When you close the window, the app keeps running in the tray.

---

## 🙏 Acknowledgments

This project is inspired by the work of many talented Rust developers:

- **[PaulDance](https://gist.github.com/PaulDance)** – for the excellent [Gist with a Shell Extension example](https://gist.github.com/PaulDance), which became the foundation of our COM server.
- **[ahaoboy](https://github.com/ahaoboy)** – for [rcm-com](https://github.com/ahaoboy/rcm-com) and [windows-contextmenu-manager](https://dev.to/ahaoboy/windows-contextmenu-manager-tauri-and-rust-3l9b), which helped understand the architecture.
- **[ppound](https://github.com/ppound)** – for [xmp-reader](https://github.com/ppound/xmp-reader), another excellent Shell Extension example in Rust.
- **[acdvs](https://github.com/acdvs)** – for the [winctx-rs](https://github.com/acdvs/winctx-rs) library, which simplified context menu handling.
- **[Microsoft](https://github.com/microsoft)** – for [windows-rs](https://github.com/microsoft/windows-rs), opening WinAPI access from Rust.

Special thanks to all maintainers of the crates used in this project.

---

## 📄 License

This project is distributed under the **MIT** license.  
It is an educational project, but everyone is free to use, modify, and distribute it for any purpose.

Full license text: [LICENSE](LICENSE)

---

**QuickSort** – tidy up your files as fast as you name a folder!  
If you have ideas or suggestions, open an issue or pull request. Let’s make Explorer more convenient together.

## 📦 Dependencies

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

### COM server (context-menu-dll)
```toml
windows = "0.61"
parking_lot = "0.12"
log = "0.4"
simplelog = "0.12"
```

See the full list of dependencies in `src-tauri/Cargo.toml`, `context-menu-dll/Cargo.toml`, and `package.json`.

---

## 🔧 Installation and Build

### Prerequisites
- Windows 10/11 (64‑bit)
- [Rust](https://rustup.rs) (stable, `x86_64-pc-windows-msvc`)
- [Node.js](https://nodejs.org) (LTS)
- [Tauri CLI](https://tauri.app) (`cargo install tauri-cli`)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (component “Desktop development with C++” with Windows SDK)

### Build
```bash
git clone https://github.com/yourname/quicksort.git
cd quicksort
npm install
cargo build --all --release
```

### COM server registration (for dynamic menu)
1. Build the DLL: `cd context-menu-dll && cargo build --release`.
2. Run `register.reg` from the project root with administrator privileges.
3. Restart Explorer (`taskkill /f /im explorer.exe && start explorer.exe`).

### Launch the editor
```bash
npm run tauri dev   # development mode
# or
cargo run           # backend only (GUI without hot‑reload)
```

For production build:
```bash
npm run tauri build
```

The installer will be in `src-tauri/target/release/bundle`.

---

## 中文

# QuickSort

**QuickSort** – 一款适用于 Windows 10/11 的工具，可在资源管理器右键菜单中添加级联的“QuickSort”菜单，列出你收藏的文件夹。只需单击一下，文件就会立即移动。  
告别拖拽、多余的窗口和不断查找目标目录的烦恼。

Tauri + React + TypeScript

---

## 目录
- [功能](#-功能-1)
- [技术栈](#-技术栈-1)
- [安装与构建](#-安装与构建-1)
- [使用说明](#-使用说明-1)
- [致谢](#-致谢-1)
- [许可证](#-许可证-1)

---

## ✨ 功能

- **级联右键菜单** – 收藏的文件夹直接显示在资源管理器菜单中（无需 UAC 或额外窗口）。
- **即时移动** – 选择文件夹后文件立即转移（使用原子 `rename`）。
- **文件夹选择器** – 通过“📂 其他文件夹...”按钮可访问配置中的所有文件夹。
- **文件夹编辑器** – 通过 Tauri + React 的图形界面方便地管理列表：添加、重命名、设为收藏。
- **事件日志** – 所有操作（移动、配置更改）的历史记录保存在本地。
- **深色主题** – 内置支持浅色和深色配色方案。
- **系统托盘** – 应用程序可在后台运行，不占用任务栏。

---

## 🛠️ 技术栈

- **Rust** – 应用核心、COM 服务器、CLI 模式、注册表操作。
- **Tauri 2** – 轻量级 GUI 框架（替代 Electron）。
- **React + TypeScript** – 编辑器前端。
- **Ant Design** – UI 库（组件、图标、主题）。
- **Windows COM** – 由资源管理器加载的 Shell Extension（DLL）。
- **win-ctx** – 上下文菜单的高层注册表封装。
- **windows-rs** – 微软官方 WinAPI 的 Rust 绑定。

---

## 📖 使用说明

1. 启动 `QuickSort.exe`，系统托盘会出现图标。
2. 在编辑器窗口中添加文件夹（点击“添加文件夹”），用星号标记收藏，然后点击“应用”。
3. 现在，在资源管理器中右键单击任意文件或文件夹，即可看到 **QuickSort** 菜单，其中包含你的收藏文件夹。  
   选择文件夹 – 文件立即移动。
4. 若要访问所有文件夹（不仅是收藏），请使用“📂 其他文件夹...” – 会打开选择窗口。
5. 所有操作都会记录在“日志”选项卡中。关闭窗口后，应用程序继续在托盘中运行。

---

## 🙏 致谢

本项目受到许多优秀 Rust 开发者工作的启发：

- **[PaulDance](https://gist.github.com/PaulDance)** – 感谢出色的 [Shell Extension 示例 Gist](https://gist.github.com/PaulDance)，它成为我们 COM 服务器的基础。
- **[ahaoboy](https://github.com/ahaoboy)** – 感谢 [rcm-com](https://github.com/ahaoboy/rcm-com) 和 [windows-contextmenu-manager](https://dev.to/ahaoboy/windows-contextmenu-manager-tauri-and-rust-3l9b)，帮助理解了架构。
- **[ppound](https://github.com/ppound)** – 感谢 [xmp-reader](https://github.com/ppound/xmp-reader)，另一个优秀的 Rust Shell Extension 示例。
- **[acdvs](https://github.com/acdvs)** – 感谢 [winctx-rs](https://github.com/acdvs/winctx-rs) 库，简化了上下文菜单的处理。
- **[Microsoft](https://github.com/microsoft)** – 感谢 [windows-rs](https://github.com/microsoft/windows-rs)，为 Rust 打开了 WinAPI 的大门。

特别感谢本项目所用 crates 的所有维护者。

---

## 📄 许可证

本项目基于 **MIT** 许可证分发。  
这是一个学习项目，但任何人都可以自由地将其用于任何目的，包括使用、修改和分发。

完整许可证文本：[LICENSE](LICENSE)

---

**QuickSort** – 像给文件夹命名一样快速地整理文件！  
如果你有想法或建议，欢迎提交 issue 或 pull request。让我们一起让资源管理器更加便捷。

## 📦 依赖

完整依赖列表请参阅 `src-tauri/Cargo.toml`、`context-menu-dll/Cargo.toml` 和 `package.json`。

---

## 🔧 安装与构建

安装包位于 `src-tauri/target/release/bundle`。

---

## Español

# QuickSort

**QuickSort** – una utilidad para Windows 10/11 que añade un elemento en cascada "QuickSort" al menú contextual del Explorador con tus carpetas favoritas. Un solo clic – y los archivos se mueven al instante.  
Se acabó arrastrar, abrir ventanas extra o buscar constantemente el directorio deseado.

Tauri + React + TypeScript

---

## Tabla de contenidos
- [Características](#-características)
- [Tecnologías](#-tecnologías)
- [Uso](#-uso)
- [Agradecimientos](#-agradecimientos)
- [Licencia](#-licencia)

---

## ✨ Características

- **Menú contextual en cascada** – las carpetas favoritas aparecen directamente en el menú del Explorador (sin UAC ni ventanas adicionales).
- **Movimiento instantáneo** – al seleccionar una carpeta, los archivos se transfieren de inmediato (usando `rename` atómico).
- **Selector de carpetas** – acceso a todas las carpetas de la configuración mediante el botón “📂 Otras carpetas...”.
- **Editor de carpetas** – una cómoda interfaz gráfica con Tauri + React para gestionar la lista: añadir, renombrar, marcar como favorita.
- **Registro de eventos** – historial local de todas las operaciones (movimientos, cambios de configuración).
- **Tema oscuro** – soporte integrado para esquemas claro y oscuro.
- **Bandeja del sistema** – la aplicación puede ejecutarse en segundo plano, manteniendo limpia la barra de tareas.

---

## 🛠️ Tecnologías

- **Rust** – núcleo de la aplicación, servidor COM, modo CLI, manejo del registro.
- **Tauri 2** – framework GUI ligero (en lugar de Electron).
- **React + TypeScript** – frontend del editor.
- **Ant Design** – biblioteca de UI (componentes, iconos, temas).
- **Windows COM** – Shell Extension (DLL) cargada por el Explorador.
- **win-ctx** – envoltorio de alto nivel sobre el registro para menús contextuales.
- **windows-rs** – bindings oficiales de Microsoft para WinAPI en Rust.

---

## 📖 Uso

1. Ejecuta `QuickSort.exe`. Aparecerá un icono en la bandeja del sistema.
2. En la ventana del editor, añade carpetas (clic en “Agregar carpeta”), marca las favoritas con una estrella y pulsa “Aplicar”.
3. Ahora, al hacer clic derecho en cualquier archivo o carpeta del Explorador, verás el menú **QuickSort** con tus carpetas favoritas.  
   Selecciona una carpeta – el archivo se mueve instantáneamente.
4. Para acceder a todas las carpetas (no solo las favoritas), usa “📂 Otras carpetas...” – se abrirá una ventana de selección.
5. Todas las acciones se registran en la pestaña “Registro”. Al cerrar la ventana, la aplicación sigue funcionando en la bandeja.

---

## 🙏 Agradecimientos

Este proyecto está inspirado en el trabajo de muchos talentosos desarrolladores de Rust:

- **[PaulDance](https://gist.github.com/PaulDance)** – por el excelente [Gist con ejemplo de Shell Extension](https://gist.github.com/PaulDance), que sirvió de base para nuestro servidor COM.
- **[ahaoboy](https://github.com/ahaoboy)** – por [rcm-com](https://github.com/ahaoboy/rcm-com) y [windows-contextmenu-manager](https://dev.to/ahaoboy/windows-contextmenu-manager-tauri-and-rust-3l9b), que ayudaron a comprender la arquitectura.
- **[ppound](https://github.com/ppound)** – por [xmp-reader](https://github.com/ppound/xmp-reader), otro magnífico ejemplo de Shell Extension en Rust.
- **[acdvs](https://github.com/acdvs)** – por la librería [winctx-rs](https://github.com/acdvs/winctx-rs), que facilitó el trabajo con los menús contextuales.
- **[Microsoft](https://github.com/microsoft)** – por [windows-rs](https://github.com/microsoft/windows-rs), que abrió el acceso a WinAPI desde Rust.

Un agradecimiento especial a todos los mantenedores de los crates utilizados en el proyecto.

---

## 📄 Licencia

El proyecto se distribuye bajo la licencia **MIT**.  
Es un proyecto educativo, pero cualquiera puede usarlo, modificarlo y distribuirlo libremente para cualquier propósito.

Texto completo de la licencia: [LICENSE](LICENSE)

---

**QuickSort** – ¡ordena tus archivos tan rápido como nombras una carpeta!  
Si tienes ideas o sugerencias, abre un issue o un pull request. Hagamos juntos que el Explorador sea más cómodo.


El instalador estará en `src-tauri/target/release/bundle`.

---

## Deutsch

# QuickSort

**QuickSort** – ein Windows-10/11-Dienstprogramm, das einen kaskadierenden Eintrag „QuickSort“ mit Ihren Lieblingsordnern zum Explorer-Kontextmenü hinzufügt. Ein Klick – und Dateien werden sofort verschoben.  
Schluss mit Ziehen, zusätzlichen Fenstern und ständigem Suchen nach dem richtigen Verzeichnis.

Tauri + React + TypeScript

---

## Inhaltsverzeichnis
- [Funktionen](#-funktionen)
- [Technologien](#-technologien)
- [Verwendung](#-verwendung)
- [Danksagungen](#-danksagungen)
- [Lizenz](#-lizenz)

---

## ✨ Funktionen

- **Kaskadierendes Kontextmenü** – Lieblingsordner erscheinen direkt im Explorer-Menü (ohne UAC oder zusätzliche Fenster).
- **Sofortiges Verschieben** – nach Auswahl eines Ordners werden die Dateien umgehend verschoben (mittels atomarem `rename`).
- **Ordnerauswahl** – über die Schaltfläche „📂 Weitere Ordner...“ Zugriff auf alle konfigurierten Ordner.
- **Ordner-Editor** – eine praktische GUI mit Tauri + React zum Verwalten der Liste: Hinzufügen, Umbenennen, als Favorit markieren.
- **Ereignisprotokoll** – lokaler Verlauf aller Vorgänge (Verschiebungen, Konfigurationsänderungen).
- **Dunkles Design** – integrierte Unterstützung für helle und dunkle Farbschemas.
- **System-Tray** – die Anwendung kann im Hintergrund laufen, ohne die Taskleiste zu überladen.

---

## 🛠️ Technologien

- **Rust** – Kern der Anwendung, COM-Server, CLI-Modus, Registry-Handling.
- **Tauri 2** – schlankes GUI-Framework (statt Electron).
- **React + TypeScript** – Editor-Frontend.
- **Ant Design** – UI-Bibliothek (Komponenten, Icons, Theming).
- **Windows COM** – Shell Extension (DLL), die vom Explorer geladen wird.
- **win-ctx** – High-Level-Registry-Wrapper für Kontextmenüs.
- **windows-rs** – offizielle Microsoft-WinAPI-Bindings für Rust.

---

## 📖 Verwendung

1. Starten Sie `QuickSort.exe`. Ein Symbol erscheint im System-Tray.
2. Fügen Sie im Editor-Fenster Ordner hinzu („Ordner hinzufügen“), markieren Sie Favoriten mit einem Stern und klicken Sie auf „Übernehmen“.
3. Wenn Sie nun im Explorer mit der rechten Maustaste auf eine Datei oder einen Ordner klicken, erscheint das Menü **QuickSort** mit Ihren Lieblingsordnern.  
   Wählen Sie einen Ordner – die Datei wird sofort verschoben.
4. Für den Zugriff auf alle Ordner (nicht nur Favoriten) verwenden Sie „📂 Weitere Ordner...“ – ein Auswahlfenster öffnet sich.
5. Alle Aktionen werden im Reiter „Protokoll“ aufgezeichnet. Beim Schließen des Fensters läuft die App weiter im Tray.

---

## 🙏 Danksagungen

Dieses Projekt ist von der Arbeit vieler talentierter Rust-Entwickler inspiriert:

- **[PaulDance](https://gist.github.com/PaulDance)** – für das hervorragende [Gist mit Shell-Extension-Beispiel](https://gist.github.com/PaulDance), das die Grundlage für unseren COM-Server bildete.
- **[ahaoboy](https://github.com/ahaoboy)** – für [rcm-com](https://github.com/ahaoboy/rcm-com) und [windows-contextmenu-manager](https://dev.to/ahaoboy/windows-contextmenu-manager-tauri-and-rust-3l9b), die halfen, die Architektur zu verstehen.
- **[ppound](https://github.com/ppound)** – für [xmp-reader](https://github.com/ppound/xmp-reader), ein weiteres ausgezeichnetes Shell-Extension-Beispiel in Rust.
- **[acdvs](https://github.com/acdvs)** – für die Bibliothek [winctx-rs](https://github.com/acdvs/winctx-rs), die die Arbeit mit Kontextmenüs erleichterte.
- **[Microsoft](https://github.com/microsoft)** – für [windows-rs](https://github.com/microsoft/windows-rs), das den Zugriff auf die WinAPI aus Rust ermöglichte.

Besonderer Dank gilt allen Maintainern der im Projekt verwendeten Crates.

---

## 📄 Lizenz

Das Projekt wird unter der **MIT**-Lizenz verbreitet.  
Es handelt sich um ein Lernprojekt, aber jeder darf es für beliebige Zwecke frei verwenden, modifizieren und weitergeben.

Vollständiger Lizenztext: [LICENSE](LICENSE)

---

**QuickSort** – räumen Sie Ihre Dateien so schnell auf, wie Sie einen Ordner benennen!  
Wenn Sie Ideen oder Vorschläge haben, erstellen Sie ein Issue oder einen Pull Request. Lassen Sie uns gemeinsam den Explorer komfortabler machen.

## 📦 Abhängigkeiten

Die vollständige Liste der Abhängigkeiten finden Sie in `src-tauri/Cargo.toml`, `context-menu-dll/Cargo.toml` und `package.json`.

---


Das Installationspaket befindet sich in `src-tauri/target/release/bundle`.