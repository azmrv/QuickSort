<div align="center">

# QuickSort

**Un gestor de archivos de próxima generación para Windows.**

QuickSort combina la velocidad de una extensión de shell con la potencia de un sistema de gestión de archivos moderno. Haz clic derecho en cualquier archivo en el Explorador, elige una carpeta y deja que QuickSort haga el resto — con detección de duplicados, historial de operaciones y soporte de deshacer.

[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-2-FFC131?style=flat&logo=tauri)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19-61DAFB?style=flat&logo=react)](https://react.dev)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

---

[English](../../README.md) | [Русский](../ru/README.md) | [中文](../cn/README.md) | [Deutsch](../de/README.md) | **Español**

---

## Características

### Funciones principales

- **Menú contextual en cascada** — carpetas favoritas directamente en el Explorador (sin UAC)
- **Mover/copiar instantáneo** — operaciones atómicas con soporte entre discos
- **Detección de duplicados** — verificaciones previas por nombre, tamaño o hash SHA-256
- **Historial de operaciones** — registro completo de todas las operaciones con soporte de deshacer
- **Valores por defecto configurables** — tipo de operación, política de sobrescritura y modo de verificación de duplicados

### Interfaz

- **Editor de carpetas** — agregar, renombrar, marcar favoritos en una GUI limpia
- **Selector de todas las carpetas** — buscar, filtrar y elegir de toda tu biblioteca de carpetas
- **Registro de eventos** — logging en tiempo real de backend y frontend con filtración
- **Tema oscuro** — esquemas de colores claro y oscuro con acentos ámbar
- **Bandeja del sistema** — ejecuta en segundo plano, mantiene la barra de tareas limpia

### Instalación inteligente

- **Despliegue sin instalación** — ejecutable único, auto-registra el servidor COM en el primer inicio
- **Modo portátil** — no requiere derechos de administrador, todo ejecuta en espacio de usuario

## Visión

QuickSort evoluciona hacia un gestor de archivos completo con:

- **Línea de comandos** — entrada de texto interactivo con sintaxis de búsqueda estilo Everything
- **Operaciones por lotes** — procesamiento basado en colas para movimientos a gran escala con seguimiento de progreso
- **Análisis inteligente de archivos** — detección de duplicados por contenido, reconocimiento de tipos e indexación de metadatos

## Stack Tecnológico

| Capa | Tecnología |
|------|-----------|
| Núcleo | Rust (Clean Architecture + DDD) |
| GUI | Tauri 2 |
| Frontend | React 19 + TypeScript + Ant Design |
| Extensión Shell | Windows COM (DLL) — componente independiente |
| IPC | Named Pipes |
| WinAPI | windows-rs |

## Arquitectura

QuickSort sigue Clean Architecture con Domain-Driven Design:

```
Domain <- Application <- Infrastructure <- Adapters (src-tauri, context-menu-dll)
```

### Cargo Workspace

| Crate | Rol | Descripción |
|-------|-----|-------------|
| `quicksort-domain` | Domain | Entidades, objetos de valor, eventos |
| `quicksort-application` | Application | Casos de uso, puertos, DTOs, fachada |
| `quicksort-infrastructure` | Infrastructure | Repositorios JSON, FileSystem, UUID |
| `quicksort-ipc-contract` | Contract | Contratos Named Pipe |
| `src-tauri` | Adapter | App Tauri, CLI, servidor IPC, registro COM |
| `context-menu-dll` | Adapter | Extensión COM Shell (cargada por el Explorador) |

### DLL como componente independiente

La DLL del menú contextual (`context-menu-dll`) es un **componente independiente** que se puede construir y distribuir por separado de la aplicación principal:

- **La app funciona sin DLL** — degradación elegante (sin menú contextual, todas las demás funciones intactas)
- **La DLL es opcional** — coloca `context_menu_dll.dll` junto a `QuickSort.exe` para habilitar el menú contextual
- **Build separado** — la DLL tiene su propio proceso de build, sin dependencia circular
- **Manejo de archivos bloqueados** — el script de build maneja bloqueos de Windows Defender mediante patrón de renombrado

```bash
# Construir solo la app (sin dependencia de DLL)
npm run tauri build

# Construir DLL por separado
npm run build:dll

# O usar el script de build seguro (maneja archivos bloqueados)
pwsh -NoProfile -File scripts/build-dll.ps1
```

## Build

### Prerrequisitos

- Windows 10/11 (64-bit)
- [Rust](https://rustup.rs) (stable, `x86_64-pc-windows-msvc`)
- [Node.js](https://nodejs.org) (LTS)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Desktop development with C++)

### Comandos

```bash
git clone https://github.com/azmrv/QuickSort.git
cd QuickSort
npm install
npm run tauri dev        # desarrollo
npm run tauri build      # build de producción
```

El instalador estará en `target/release/bundle`.

## Uso

1. Ejecuta `QuickSort.exe` — el ícono aparece en la bandeja del sistema
2. Agrega carpetas, marca favoritos con una estrella, haz clic en Aplicar
3. Haz clic derecho en un archivo en el Explorador — elige una carpeta del menú QuickSort
4. Los archivos se mueven instantáneamente. La detección de duplicados funciona automáticamente.
5. Abre la pestaña de Historial para revisar o deshacer operaciones.
6. Cierra la ventana — la app sigue ejecutándose en la bandeja.

### Configuración

Configura los valores por defecto en la pestaña de Configuración:
- **Operación por defecto**: Mover o Copiar
- **Política de sobrescritura**: Saltar, Sobrescribir o Auto-renombrar
- **Modo de verificación de duplicados**: Nombre (rápido), Tamaño (medio) o Contenido/SHA-256 (completo)

## Estructura del Proyecto

```
QuickSort/
  src-tauri/             # Adaptador Tauri, CLI, servidor IPC
  context-menu-dll/      # Extensión COM Shell (cargada por el Explorador)
  crates/
    quicksort-domain/          # Entidades, objetos de valor, eventos
    quicksort-application/     # Casos de uso, puertos, DTOs
    quicksort-infrastructure/  # Repositorios JSON, FileSystem, UUID
    quicksort-ipc-contract/    # Contratos Named Pipe
  src/                   # Frontend React
  scripts/               # Ayudantes de build (build seguro de DLL)
```

## Autor

**azmrv** - [@Fib511](https://t.me/Fib511) en Telegram

[![GitHub](https://img.shields.io/badge/GitHub-azmrv-181717?style=flat&logo=github)](https://github.com/azmrv)

## Agradecimientos

Este proyecto está inspirado en el trabajo de muchos desarrolladores Rust talentosos:

- [PaulDance](https://gist.github.com/PaulDance) — ejemplo de Shell Extension que se convirtió en la base de nuestro servidor COM
- [ahaoboy](https://github.com/ahaoboy) — [rcm-com](https://github.com/ahaoboy/rcm-com) y [windows-contextmenu-manager](https://dev.to/ahaoboy/windows-contextmenu-manager-tauri-and-rust-3l9b)
- [ppound](https://github.com/ppound) — [xmp-reader](https://github.com/ppound/xmp-reader), otro ejemplo de Shell Extension
- [acdvs](https://github.com/acdvs) — biblioteca [winctx-rs](https://github.com/acdvs/winctx-rs)
- [Microsoft](https://github.com/microsoft) — [windows-rs](https://github.com/microsoft/windows-rs) bindings de WinAPI
- [voidtools](https://www.voidtools.com) — Everything, inspiración para la línea de comandos

## Licencia

[MIT](LICENSE) — libre para usar, modificar y distribuir.

---

<div align="center">

**QuickSort** — gestión de archivos de próxima generación para Windows.

</div>
