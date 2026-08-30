<div align="center">

# QuickSort

**Windows 下一代文件管理器。**

QuickSort 结合了 Shell 扩展的速度和现代文件管理系统的强大功能。在资源管理器中右键点击文件，选择一个文件夹，其余的交给 QuickSort — 支持重复文件检测、操作历史记录和撤销功能。

[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-2-FFC131?style=flat&logo=tauri)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19-61DAFB?style=flat&logo=react)](https://react.dev)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

---

[English](../../README.md) | [Русский](../ru/README.md) | **中文** | [Deutsch](../de/README.md) | [Español](../es/README.md)

---

<div align="center">
 <a target="_blank" href="https://imageban.ru/show/2026/08/25/83fc5efaa96d7d4b84317e957f628e38/png"><img src="https://i8.imageban.ru/thumbs/2026.08.25/83fc5efaa96d7d4b84317e957f628e38.png" border="0" style='border: 1px solid #000000'></a> 
 <a target="_blank" href="https://imageban.ru/show/2026/08/25/b80ebefd51e54a070cc292b0f427fa49/png"><img src="https://i3.imageban.ru/thumbs/2026.08.25/b80ebefd51e54a070cc292b0f427fa49.png" border="0" style='border: 1px solid #000000'></a> 
 <a target="_blank" href="https://imageban.ru/show/2026/08/25/2c0472a156b387eda9d595667fd24381/png"><img src="https://i2.imageban.ru/thumbs/2026.08.25/2c0472a156b387eda9d595667fd24381.png" border="0" style='border: 1px solid #000000'></a> 
 <a target="_blank" href="https://imageban.ru/show/2026/08/25/f2007bd5ce1f0283353c986e2dd5d881/png"><img src="https://i7.imageban.ru/thumbs/2026.08.25/f2007bd5ce1f0283353c986e2dd5d881.png" border="0" style='border: 1px solid #000000'></a> 
 <a target="_blank" href="https://imageban.ru/show/2026/08/25/a784d47f47a701c2f6c01cb4f20544af/png"><img src="https://i7.imageban.ru/thumbs/2026.08.25/a784d47f47a701c2f6c01cb4f20544af.png" border="0" style='border: 1px solid #000000'></a>
</div>



## 功能特性

### 核心功能

- **级联右键菜单** — 收藏文件夹直接显示在资源管理器中（无需管理员权限）
- **即时移动/复制** — 原子文件操作，支持跨磁盘
- **重复文件检测** — 按名称、大小或 SHA-256 内容哈希进行预操作检查
- **操作历史** — 完整的文件操作审计记录，支持撤销
- **可配置默认值** — 默认操作类型、覆盖策略和重复检查模式

### 界面

- **文件夹编辑器** — 在简洁的 GUI 中添加、重命名、切换收藏
- **全部文件夹选择器** — 搜索、筛选并从整个文件夹库中选择
- **事件日志** — 实时后端和前端日志记录与筛选
- **深色主题** — 内置明暗配色方案，琥珀色强调
- **系统托盘** — 后台运行，保持任务栏整洁

### 智能安装

- **零安装部署** — 单个可执行文件，首次启动自动注册 COM 服务器
- **便携模式** — 无需管理员权限，一切在用户空间运行

## 愿景

QuickSort 正在发展为全功能文件管理器：

- **命令行界面** — 支持 Everything 风格搜索语法的交互式文本输入，用于高级文件查询、筛选和排序
- **批量操作** — 基于队列的大规模文件移动处理，带进度跟踪
- **智能文件分析** — 基于内容的重复检测、文件类型识别和元数据索引

## 技术栈

| 层级 | 技术 |
|------|------|
| 核心 | Rust（Clean Architecture + DDD） |
| GUI | Tauri 2 |
| 前端 | React 19 + TypeScript + Ant Design |
| Shell 扩展 | Windows COM（DLL）— 独立组件 |
| IPC | Named Pipes |
| WinAPI | windows-rs |

## 架构

QuickSort 遵循 Clean Architecture 和 Domain-Driven Design：

```
Domain <- Application <- Infrastructure <- Adapters (src-tauri, context-menu-dll)
```

### Cargo 工作空间

| Crate | 角色 | 描述 |
|-------|------|------|
| `quicksort-domain` | Domain | 实体、值对象、事件 |
| `quicksort-application` | Application | 用例、端口、DTO、门面 |
| `quicksort-infrastructure` | Infrastructure | JSON 仓库、文件系统、UUID、Clock |
| `quicksort-ipc-contract` | Contract | Named Pipe 契约 |
| `src-tauri` | Adapter | Tauri 应用、CLI、IPC 服务器、COM 注册 |
| `context-menu-dll` | Adapter | COM Shell 扩展（由资源管理器加载） |

### DLL 作为独立组件

右键菜单 DLL（`context-menu-dll`）是一个**独立组件**，可以与主应用程序分开构建和分发：

- **应用无需 DLL 即可工作** — 优雅降级（无右键菜单，所有其他功能完整）
- **DLL 是可选的** — 将 `context_menu_dll.dll` 放在 `QuickSort.exe` 旁边即可启用右键菜单
- **独立构建** — DLL 有自己的构建流程，与应用无循环依赖
- **锁定文件处理** — 构建脚本通过重命名模式处理 Windows Defender 锁定

```bash
# 仅构建应用（无 DLL 依赖）
npm run tauri build

# 单独构建 DLL
npm run build:dll

# 或使用安全构建脚本（处理锁定文件）
pwsh -NoProfile -File scripts/build-dll.ps1
```

## 构建

### 前提条件

- Windows 10/11（64 位）
- [Rust](https://rustup.rs)（stable，`x86_64-pc-windows-msvc`）
- [Node.js](https://nodejs.org)（LTS）
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)（Desktop development with C++）

### 命令

```bash
git clone https://github.com/azmrv/QuickSort.git
cd QuickSort
npm install
npm run tauri dev        # 开发模式
npm run tauri build      # 生产构建
```

安装程序将在 `target/release/bundle` 中。

## 使用

1. 启动 `QuickSort.exe` — 图标出现在系统托盘
2. 添加文件夹，用星标标记收藏，点击应用
3. 在资源管理器中右键点击文件 — 从 QuickSort 菜单选择文件夹
4. 文件立即移动。重复检测自动运行。
5. 打开历史记录选项卡查看或撤销任何操作。
6. 关闭窗口 — 应用继续在托盘中运行。

### 设置

在设置选项卡中配置默认值：
- **默认操作**：移动或复制
- **覆盖策略**：跳过、覆盖或自动重命名
- **重复检查模式**：名称（快速）、大小（中等）或内容/SHA-256（彻底）

## 项目结构

```
QuickSort/
  src-tauri/             # Tauri 适配器、CLI、IPC 服务器
  context-menu-dll/      # COM Shell 扩展（由资源管理器加载）
  crates/
    quicksort-domain/          # 实体、值对象、事件
    quicksort-application/     # 用例、端口、DTO
    quicksort-infrastructure/  # JSON 仓库、文件系统、UUID
    quicksort-ipc-contract/    # Named Pipe 契约
  src/                   # React 前端
  scripts/               # 构建辅助工具（DLL 安全构建）
```

## 作者

**azmrv** - Telegram [@Fib511](https://t.me/Fib511)

[![GitHub](https://img.shields.io/badge/GitHub-azmrv-181717?style=flat&logo=github)](https://github.com/azmrv)

## 致谢

本项目受到许多优秀开发者和项目的启发：

- [Christian Ghisler](https://www.ghisler.com) — [Total Commander](https://www.ghisler.com)，文件管理器的标杆，启发了 QuickSort 的插件架构（WCX/WDX/WFX/WLX）
- [PaulDance](https://gist.github.com/PaulDance) — Shell 扩展示例，成为我们 COM 服务器的基础
- [ahaoboy](https://github.com/ahaoboy) — [rcm-com](https://github.com/ahaoboy/rcm-com) 和 [windows-contextmenu-manager](https://dev.to/ahaoboy/windows-contextmenu-manager-tauri-and-rust-3l9b)
- [ppound](https://github.com/ppound) — [xmp-reader](https://github.com/ppound/xmp-reader)，另一个 Shell 扩展示例
- [acdvs](https://github.com/acdvs) — [winctx-rs](https://github.com/acdvs/winctx-rs) 库
- [Microsoft](https://github.com/microsoft) — [windows-rs](https://github.com/microsoft/windows-rs) WinAPI 绑定
- [voidtools](https://www.voidtools.com) — Everything 搜索引擎，命令行界面的灵感来源

## 许可证

[MIT](LICENSE) — 自由使用、修改和分发。

---

<div align="center">

**QuickSort** — Windows 下一代文件管理。

</div>



<div align="center">

 <a target="_blank" href="https://imageban.ru/show/2026/08/25/40f71dff67252d956c04edd889115666/png"><img src="https://i5.imageban.ru/thumbs/2026.08.25/40f71dff67252d956c04edd889115666.png" border="0" style='border: 1px solid #000000'></a> 
 <a target="_blank" href="https://imageban.ru/show/2026/08/25/66a4921c12ff6afc4f030dde050b02e2/png"><img src="https://i4.imageban.ru/thumbs/2026.08.25/66a4921c12ff6afc4f030dde050b02e2.png" border="0" style='border: 1px solid #000000'></a>  


</div>
