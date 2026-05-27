# scripts

Personal terminal tools built with Rust + [ratatui](https://github.com/ratatui-org/ratatui).

## Tools

### `harbor` — project selector (`t` / `ctrl+t`)

Fuzzy project switcher. Lists all directories under `~/github.com`, filters on keystroke, and creates or switches to a tmux session on Enter.

```
t          open from shell
ctrl+t     open as zsh widget (stays in current terminal)
```

| Key | Action |
|-----|--------|
| type | filter |
| `↑` `↓` | navigate |
| `Enter` | switch to / create tmux session |
| `Esc` `ctrl+c` | quit |

Session names replace `.`, ` `, `:` with `_`. Respects `$PROJECTS_DIR`.

---

### `recall` — history search (`ctrl+r`)

Replaces the default zsh `ctrl+r`. Reads `~/.zsh_history`, deduplicates, and pastes the selected command into the prompt without executing it.

```
ctrl+r     open
```

| Key | Action |
|-----|--------|
| type | filter |
| `↑` `↓` | navigate |
| `Enter` | paste into prompt |
| `Esc` `ctrl+c` | quit |

---

### `tdo` — daily todo (`todo`)

Date-based todo list saved to `~/notes/todos/DD-MM-YYYY.md`. Unfinished tasks carry over automatically from the most recent day that had open items.

```
todo       open
```

| Key | Action |
|-----|--------|
| `j` `k` `↑` `↓` | navigate |
| `a` | add todo |
| `space` `Enter` | toggle done |
| `d` | delete |
| `f` | open follow popup (forward links + backlinks) |
| `b` `Backspace` | go back (navigation history) |
| `q` `Esc` | quit |

Supports `[[DD-MM-YYYY]]` wiki-links in todo text. Links are shown in the footer; `f` lets you jump to a linked date directly. Respects `$TODO_DIR`.

---

### `jot` — quick capture (`jot` / `ctrl+n`)

Scratch pad for capturing thoughts during the day. Saves timestamped entries to `~/notes/jots/DD-MM-YYYY.md`. Always open and ready to type.

```
jot        open from shell
ctrl+n     open as zsh widget
```

| Key | Action |
|-----|--------|
| type | compose note |
| `Enter` | save with HH:MM timestamp |
| `[[` | open link autocomplete (search or create note) |
| `ctrl+f` | open follow popup (forward links + backlinks) |
| `ctrl+b` | go back (navigation history) |
| `↑` `↓` | scroll today's jots |
| `Esc` `ctrl+c` | quit |

**Link autocomplete** (`[[`): filters existing notes as you type. Select an existing note or navigate to the `+` entry to create a new one at `~/notes/<name>.md`.

**Follow** (`ctrl+f`): shows all `[[links]]` in today's jots (→) and any notes that link back to today (←). Date links navigate within jot; named notes open in `$EDITOR`.

Respects `$JOT_DIR`. Notes are stored in `$JOT_DIR/../` (default `~/notes/`).

---

## File layout

```
harbor/        fuzzy project selector
recall/        zsh history search
tdo/           daily todo list
jot/           quick capture
aliases.sh     zsh bindings — source this in ~/.zshrc
install.sh     build all tools + wire up ~/.zshrc
```

## Install

```bash
./install.sh
source ~/.zshrc
```

Requires Rust (`~/.cargo/bin/cargo`). Binaries land at `<tool>/target/release/<tool>`.

## Note storage

| Path | Used by |
|------|---------|
| `~/notes/todos/DD-MM-YYYY.md` | tdo |
| `~/notes/jots/DD-MM-YYYY.md` | jot |
| `~/notes/<name>.md` | named notes created via `[[` |

All files are plain Markdown — readable and editable outside the TUIs.
