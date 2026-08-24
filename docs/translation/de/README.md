<div align="center">

# QuickSort

**Ein Dateimanager der nächsten Generation für Windows.**

QuickSort kombiniert die Geschwindigkeit einer Shell-Erweiterung mit der Leistung eines modernen Dateiverwaltungssystems. Klicken Sie mit der rechten Maustaste auf eine Datei im Explorer, wählen Sie einen Ordner aus und QuickSort erledigt den Rest — mit Duplikaterkennung, Verlauf der Operationen und Undo-Unterstützung.

[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-2-FFC131?style=flat&logo=tauri)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19-61DAFB?style=flat&logo=react)](https://react.dev)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

---

[English](../../README.md) | [Русский](../ru/README.md) | [中文](../cn/README.md) | **Deutsch** | [Español](../es/README.md)

---

## Funktionen

### Kernfunktionen

- **Kaskaden-Kontextmenü** — Lieblingsordner direkt im Explorer (kein UAC)
- **Sofortiges Verschieben/Kopieren** — atomare Dateioperationen mit Cross-Drive-Unterstützung
- **Duplikaterkennung** — Vorab-Prüfungen nach Name, Größe oder SHA-256-Inhalts-Hash
- **Operationsverlauf** — vollständige Audit-Trail aller Dateioperationen mit Undo-Unterstützung
- **Konfigurierbare Standardwerte** — Standard-Operationstyp, Überschreibungsrichtlinie und Duplikat-Check-Modus

### Benutzeroberfläche

- **Ordner-Editor** — Hinzufügen, Umbennen, Favoriten umschalten in einer sauberen GUI
- **Alle Ordner auswählen** — Suche, Filter und Auswahl aus Ihrer gesamten Ordnerbibliothek
- **Ereignisprotokoll** — Echtzeit-Backend- und Frontend-Protokollierung mit Filterung
- **Dunkles Thema** — Integrierte helle und dunkle Farbschemen mit Bernstein-Akzenten
- **Systemtray** — Läuft im Hintergrund, hält die Taskbar sauber

### Smarte Installation

- **Null-Installations-Deployment** — Einzelne ausführbare Datei, automatische COM-Server-Registrierung beim ersten Start
- **Portabler Modus** — Keine Administratorrechte erforderlich, alles läuft im Benutzerraum

## Vision

QuickSort entwickelt sich zu einem vollwertigen Dateimanager mit:

- **Kommandozeile** — Interaktive Texteingabe mit Everything-Stil Suchsyntax für erweiterte Dateianfragen
- **Batch-Operationen** — Warteschlangenbasierte Verarbeitung für groß angelegte Dateibewegungen mit Fortschrittsverfolgung
- **Intelligente Dateianalyse** — Inhaltsbasierte Duplikaterkennung, Dateityperkennung und Metadaten-Indexierung

## Tech-Stack

| Schicht | Technologie |
|---------|------------|
| Kern | Rust (Clean Architecture + DDD) |
| GUI | Tauri 2 |
| Frontend | React 19 + TypeScript + Ant Design |
| Shell-Erweiterung | Windows COM (DLL) — unabhängige Komponente |
| IPC | Named Pipes |
| WinAPI | windows-rs |

## Architektur

QuickSort folgt Clean Architecture mit Domain-Driven Design:

```
Domain <- Application <- Infrastructure <- Adapters (src-tauri, context-menu-dll)
```

### Cargo Workspace

| Crate | Rolle | Beschreibung |
|-------|-------|-------------|
| `quicksort-domain` | Domain | Entitäten, Value Objects, Ereignisse |
| `quicksort-application` | Application | Use Cases, Ports, DTOs, Fassade |
| `quicksort-infrastructure` | Infrastructure | JSON-Repositories, FileSystem, UUID |
| `quicksort-ipc-contract` | Contract | Named Pipe Verträge |
| `src-tauri` | Adapter | Tauri-App, CLI, IPC-Server, COM-Registrierung |
| `context-menu-dll` | Adapter | COM Shell-Erweiterung (vom Explorer geladen) |

### DLL als unabhängige Komponente

Die Kontextmenü-DLL (`context-menu-dll`) ist eine **unabhängige Komponente**, die separat von der Hauptanwendung gebaut und verteilt werden kann:

- **App funktioniert ohne DLL** — Graceful Degradation (kein Kontextmenü, alle anderen Funktionen intakt)
- **DLL ist optional** — Legen Sie `context_menu_dll.dll` neben `QuickSort.exe`, um das Kontextmenü zu aktivieren
- **Separater Build** — DLL hat ihren eigenen Build-Prozess, keine zirkuläre Abhängigkeit
- **Locked-File-Handling** — Build-Skript umgeht Windows Defender Sperren durch Rename-Muster

```bash
# Nur App bauen (keine DLL-Abhängigkeit)
npm run tauri build

# DLL separat bauen
npm run build:dll

# Oder sicheres Build-Skript (behandelt gesperrte Dateien)
pwsh -NoProfile -File scripts/build-dll.ps1
```

## Build

### Voraussetzungen

- Windows 10/11 (64-Bit)
- [Rust](https://rustup.rs) (stable, `x86_64-pc-windows-msvc`)
- [Node.js](https://nodejs.org) (LTS)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Desktop development with C++)

### Befehle

```bash
git clone https://github.com/azmrv/QuickSort.git
cd QuickSort
npm install
npm run tauri dev        # Entwicklung
npm run tauri build      # Produktions-Build
```

Der Installer befindet sich in `target/release/bundle`.

## Verwendung

1. Starten Sie `QuickSort.exe` — Symbol erscheint im Systemtray
2. Fügen Sie Ordner hinzu, markieren Sie Favoriten mit einem Stern, klicken Sie auf Anwenden
3. Klicken Sie mit der rechten Maustaste auf eine Datei im Explorer — wählen Sie einen Ordner aus dem QuickSort-Menü
4. Dateien werden sofort verschoben. Duplikaterkennung läuft automatisch.
5. Öffnen Sie den Verlauf-Tab, um Operationen zu überprüfen oder rückgängig zu machen.
6. Schließen Sie das Fenster — App läuft weiter im Tray.

### Einstellungen

Konfigurieren Sie Standardwerte im Einstellungen-Tab:
- **Standard-Operation**: Verschieben oder Kopieren
- **Überschreibungsrichtlinie**: Überspringen, Überschreiben oder Auto-Umbenennen
- **Duplikat-Check-Modus**: Name (schnell), Größe (mittel) oder Inhalt/SHA-256 (gründlich)

## Projektstruktur

```
QuickSort/
  src-tauri/             # Tauri-Adapter, CLI, IPC-Server
  context-menu-dll/      # COM Shell-Erweiterung (vom Explorer geladen)
  crates/
    quicksort-domain/          # Entitäten, Value Objects, Ereignisse
    quicksort-application/     # Use Cases, Ports, DTOs
    quicksort-infrastructure/  # JSON-Repositories, FileSystem, UUID
    quicksort-ipc-contract/    # Named Pipe Verträge
  src/                   # React-Frontend
  scripts/               # Build-Helfer (DLL-sicheres Build)
```

## Autor

**azmrv** - [@Fib511](https://t.me/Fib511) auf Telegram

[![GitHub](https://img.shields.io/badge/GitHub-azmrv-181717?style=flat&logo=github)](https://github.com/azmrv)

## Danksagung

Dieses Projekt ist inspiriert von der Arbeit vieler talentierter Rust-Entwickler:

- [PaulDance](https://gist.github.com/PaulDance) — Shell-Erweiterungs-Beispiel, das die Grundlage unseres COM-Servers bildete
- [ahaoboy](https://github.com/ahaoboy) — [rcm-com](https://github.com/ahaoboy/rcm-com) und [windows-contextmenu-manager](https://dev.to/ahaoboy/windows-contextmenu-manager-tauri-and-rust-3l9b)
- [ppound](https://github.com/ppound) — [xmp-reader](https://github.com/ppound/xmp-reader), ein weiteres Shell-Erweiterungs-Beispiel
- [acdvs](https://github.com/acdvs) — [winctx-rs](https://github.com/acdvs/winctx-rs) Bibliothek
- [Microsoft](https://github.com/microsoft) — [windows-rs](https://github.com/microsoft/windows-rs) WinAPI-Bindungen
- [voidtools](https://www.voidtools.com) — Everything Suchmaschine, Inspiration für die Kommandozeile

## Lizenz

[MIT](LICENSE) — frei verwendbar, modifizierbar und verteilbar.

---

<div align="center">

**QuickSort** — Dateiverwaltung der nächsten Generation für Windows.

</div>
