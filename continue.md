# continue

State of the project at the end of this session. Pick up from here next time.

## What exists

| Tool | Keybinding | Status |
|------|-----------|--------|
| `harbor` | `t` / `ctrl+t` | done |
| `recall` | `ctrl+r` | done |
| `tdo` | `todo` alias | done |
| `jot` | `jot` / `ctrl+n` | done |

All four are built, committed, and wired into `aliases.sh`.

## tdo features

- Browse / Input modes
- Toggle, delete, add todos
- Automatic carryover of unfinished items from the most recent day
- `[[DD-MM-YYYY]]` wiki-links rendered in cyan
- Footer shows forward links (→) and backlinks (←)
- `f` → follow popup (navigate to linked date in-TUI)
- `b` / Backspace → go back (history stack)
- Saves to `~/notes/todos/DD-MM-YYYY.md` (`$TODO_DIR`)

## jot features

- Always-on input bar, Enter to save with HH:MM timestamp
- `[[` → link autocomplete popup: filters existing notes, `+` entry creates new file
- `ctrl+f` → follow popup: forward links (→) + backlinks (←links from other files referencing today)
- Date links navigate in-TUI; named note links open in `$EDITOR`
- `ctrl+b` → go back (history stack)
- `ctrl+j/k` / arrows → scroll today's jots
- Saves to `~/notes/jots/DD-MM-YYYY.md` (`$JOT_DIR`)
- Notes root: `$JOT_DIR/../` (default `~/notes/`)

## Code patterns

- All tools: Rust + ratatui 0.29 + crossterm 0.28 + chrono 0.4
- jot uses an `Action` enum to separate key → action mapping from action dispatch (avoids borrow checker issues with mode state)
- tdo uses direct match on `app.mode` — Follow mode owns its `links: Vec<(String, bool)>` and puts them back in the `_ =>` arm
- Popups: `popup_rect()` anchors to bottom of list chunk, `Clear` widget wipes background
- `[[link]]` rendering: shared `render_text()` in each tool
- Navigation history: `Vec<NaiveDate>` stack, pushed on every `navigate_to` call

## What was planned next but not built

From the original brainstorm, the next candidate was:

**`proc` — interactive process viewer**
- Filter processes by name (fuzzy)
- `Enter` or `k` to send signal (kill)
- Faster than `ps aux | grep … | kill PID`
- Bind to something like `ctrl+p` (check for conflicts with tmux prefix first)

After that: `pick` (file picker) and `env` (.env inspector).

## Repo

```
~/github.com/scripts/   ← git repo root
```

Two commits so far: initial + README.
