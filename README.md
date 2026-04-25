# RTasks

RTasks is a tiny Windows desktop task capture tool.

It does two things:

1. Capture tasks quickly.
2. Show active tasks in a small panel.

It is not a calendar, GTD system, Notion clone, TickTick clone, sync service, or project manager.

## Status

V1 MVP is functional.

- Quick Add: global hotkey, fast capture, optional status/priority, minimal date parsing.
- Panel: global hotkey, active task list, multiselect, state advance, delete selected.
- Tray: Open Quick Add, Open Panel, Quit.
- Storage: local JSONL file.

## Shortcuts

### Global

| Shortcut | Action |
| --- | --- |
| `Alt+Spacebar` | Open Quick Add |
| `Ctrl+Spacebar` | Toggle task panel |

### Quick Add

| Shortcut | Action |
| --- | --- |
| `Enter` | Save task and close Quick Add |
| `Shift+Enter` | Save task, clear input, keep Quick Add open |
| `Esc` | Close without saving |
| `Alt+1` | Toggle status `TODO` |
| `Alt+2` | Toggle status `DOING` |
| `Alt+3` | Toggle status `DONE` |
| `Alt+Q` | Toggle priority `High` |
| `Alt+W` | Toggle priority `Medium` |
| `Alt+E` | Toggle priority `Low` |

Status and priority are visually shown as placeholders by default, but are not saved unless explicitly toggled on.

### Panel

| Shortcut / Action | Result |
| --- | --- |
| `Click task` | Select/deselect task |
| `Alt+Click task` | Advance task state |
| `Alt+Click selected task` | Advance all selected tasks |
| `Del` | Delete selected tasks |
| `Esc` | Close panel and clear selection |

State advancement:

```text
TODO / no status -> DOING
DOING            -> DONE
DONE             -> TODO (only if visible in future modes)
```

The panel shows active tasks only: `TODO`, no status, and `DOING`. `DONE` tasks are hidden.

## Date parsing

RTasks intentionally has a tiny parser. Supported terms:

- `hoy`
- `mañana`
- `manana`
- `pasado mañana`
- `pasado manana`

Examples:

```text
comprar pan mañana -> title: comprar pan, due: tomorrow
comprar pan manana -> title: comprar pan, due: tomorrow
backup pasado mañana -> title: backup, due: day after tomorrow
```

No LLM, AI, or advanced natural-language parser is used.

## Storage

Tasks are stored locally as JSONL:

```text
%APPDATA%\rtasks\tasks.jsonl
```

Each line is one complete task.

Writes are safe:

1. Load tasks into memory.
2. Modify tasks.
3. Write `tasks.tmp`.
4. Rename `tasks.tmp` to `tasks.jsonl`.

## Task model

```rust
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub due: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub source: TaskSource,
}
```

`status` and `priority` are optional because Quick Add starts with placeholders, not actual saved properties.

## Tech stack

- Rust
- `eframe` / `egui`
- `global-hotkey`
- `serde` / `serde_json`
- `chrono`
- `directories`
- `ulid`
- `anyhow`

## Run

```bash
cargo run
```

RTasks starts with the Quick Add window. After closing/capturing, it stays available through global hotkeys and the tray menu.

## Tray

The tray menu includes:

- Open Quick Add
- Open Panel
- Quit

The tray icon is generated at runtime and uses a simple `R` glyph.

## Validate

```bash
cargo fmt --check
cargo test
cargo check
```

## Local release package

Create a portable Windows release zip:

```powershell
powershell -ExecutionPolicy Bypass -File .\release-local.ps1 -Version 0.1.0 -PackageOnly
```

The script runs formatting/tests/check/build unless `-SkipValidation` is passed, then creates:

```text
dist\rtasks-v<version>-windows-x64.zip
dist\SHA256SUMS.txt
```

The zip includes `rtasks.exe`, `README.md`, `rtasks.md`, and per-file checksums.

## Scope constraints

V1 intentionally excludes:

- sync
- login
- notifications
- recurrence
- subtasks
- projects
- custom tags
- visual calendar
- integrations
- markdown/rich text
- plugin system
- AI/LLM

The product rule is: if a feature does not reduce capture/review friction, it does not belong in V1.
