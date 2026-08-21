# ADR-014: Interactive Command Line Interface

## Status

Proposed

## Context

QuickSort is evolving from a context-menu utility into a next-generation file manager. A key differentiator is the addition of an interactive command line interface — a text-based input that provides Everything-style search syntax for advanced file queries, filtering, and sorting.

This feature is inspired by:
- **Everything** (voidtools) — the gold standard for instant file search on Windows, with a powerful search syntax including modifiers, functions, wildcards, and regex
- **VS Code Command Palette** — `Ctrl+Shift+P` pattern for quick actions
- **Lister** (Total Commander) — keyboard-driven file navigation
- **fzf** — fuzzy finder pattern for terminal-based selection

The command line will serve as the primary power-user interface, while the GUI remains the primary interface for casual users.

## Decision

### Architecture

The command line will be implemented as a **modal overlay** within the Tauri frontend, triggered by:
- Menu item "Command Line" in the context menu
- Keyboard shortcut `Ctrl+Shift+Space` (when QuickSort window is focused)
- Future: `Ctrl+Space` from Explorer (via IPC)

### Search Syntax (Phase 1)

We will implement a subset of Everything's search syntax, adapted for our use case:

#### Basic Syntax
```
<text>                    # Search filenames containing text
"path:<text>"             # Search full paths
"ext:<ext>"               # Filter by extension
"size:>10mb"              # Filter by size
"date-modified:today"     # Filter by modification date
```

#### Operators
```
<term1> <term2>           # AND (implicit)
<term1> | <term2>         # OR
!<term>                   # NOT
<term1> <term2>           # Group: <term1 AND term2>
```

#### Wildcards
```
*                         # Matches zero or more characters
?                         # Matches one character
```

#### Search Functions (Phase 1 subset)
```
ext:                      # File extension
size:                     # File size (supports >, <, >=, <=, ==, ranges)
date-modified:            # Modification date (supports relative: "2days", "today")
date-created:             # Creation date
name:                     # Filename (default)
path:                     # Full path
content:                  # File content (slow, opt-in)
```

#### Search Modifiers (Phase 1 subset)
```
case:                     # Case-sensitive matching
regex:                    # Regular expression mode
whole:                    # Exact filename match
```

#### QuickSort-specific Commands
```
sort:<field>              # Sort results by field (name, size, date, ext)
limit:<n>                 # Limit result count
folders:                  # Show only folders
files:                    # Show only files
duplicates:               # Find duplicate files
move <folder>             # Move selected files to folder
copy <folder>             # Copy selected files to folder
undo                      # Undo last operation
history                   # Show operation history
```

### UI Design

```
+------------------------------------------+
|  QuickSort - Command Line                |
+------------------------------------------+
|  > ext:pdf size:>10mb date-modified:today |
+------------------------------------------+
|  Found 23 files (145 MB)                 |
|                                          |
|  1. report.pdf          2.3 MB  today    |
|  2. presentation.pdf   15.7 MB  today    |
|  3. invoice.pdf         0.8 MB  today    |
|  ...                                     |
+------------------------------------------+
|  Enter: select | Tab: autocomplete       |
|  Ctrl+Enter: execute action | Esc: close |
+------------------------------------------+
```

### Autocomplete

The command line will provide tab-completion for:
- Search function names (`ext:`, `size:`, `date-modified:`)
- File extensions (`.pdf`, `.jpg`, `.docx`)
- Folder paths (from configured folders)
- QuickSort commands (`sort:`, `move`, `copy`, `undo`)

### Integration Points

1. **Domain Layer**: New `SearchQuery` value object that parses search syntax into a structured query
2. **Application Layer**: New `SearchFiles` use case that executes queries against the file system index
3. **Infrastructure Layer**: `FileIndexer` service that maintains an in-memory index of indexed folders (future: persistent index)
4. **Adapter Layer**: Tauri command `search_files(query: String)` that returns results

### Implementation Phases

#### Phase 1: Basic Search (v0.3.0)
- Command line UI overlay
- Basic syntax parser (text search, ext, size, date-modified)
- Results display with sorting
- Integration with existing folder list

#### Phase 2: Advanced Search (v0.4.0)
- Full Everything-compatible syntax subset
- Autocomplete and suggestions
- Content search (slow, opt-in)
- Regex support

#### Phase 3: Actions (v0.5.0)
- `move`, `copy`, `undo` commands
- Batch operations from search results
- Progress tracking for large operations

#### Phase 4: Indexing (v0.6.0)
- Persistent file index
- Real-time updates
- Metadata extraction (EXIF, tags, etc.)

## Consequences

### Positive
- Power users get Everything-level search capability
- Unified interface for file management and search
- Extensible command system for future operations
- Consistent with our Clean Architecture (new use case, new port)

### Negative
- Search syntax parser adds complexity to the Domain layer
- File indexing requires background processing and storage
- Content search is inherently slow without an index
- Must maintain compatibility with Everything's syntax subset

### Risks
- **Parser bugs**: Mitigated by comprehensive test suite for the search parser
- **Index performance**: Mitigated by lazy indexing (only index configured folders)
- **Memory usage**: Mitigated by configurable index size limits

## References

- [Everything Search Syntax](https://www.voidtools.com/support/everything/search_syntax)
- [Everything Search Modifiers](https://www.voidtools.com/support/everything/search_modifiers)
- [Everything Search Functions](https://www.voidtools.com/support/everything/search_functions)
- [Everything SDK](https://www.voidtools.com/support/everything/sdk)
- ADR-001: Architectural Style
- ADR-007: Application Facade
