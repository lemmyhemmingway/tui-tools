# devtool — Design Spec

**Date:** 2026-05-28

## Overview

A terminal TUI version of DevToys. Sidebar lists all tools; selecting one shows an input/output panel on the right. All transformation happens live on keystroke.

## Layout

```
┌──────────────┬──────────────────────────────────────┐
│ > JSON       │  ╭─ Input ───────────────────────────╮│
│   Base64     │  │                                   ││
│   Hash       │  ╰───────────────────────────────────╯│
│   UUID       │  ╭─ Output ──────────────────────────╮│
│   Regex      │  │                                   ││
│   Timestamp  │  ╰───────────────────────────────────╯│
├──────────────┴──────────────────────────────────────┤
│ Tab: focus input  q: quit  [tool-specific hints]     │
└─────────────────────────────────────────────────────┘
```

- Sidebar: ~18 columns wide, fixed
- Right panel: fills remaining width, split 50/50 top/bottom for input and output
- Footer: one row, shows universal keys + active tool's extra hints
- UUID tool: no input pane; full right panel shows the generated value

## Tools

| Tool | Input | Output | Extra |
|------|-------|--------|-------|
| JSON | multiline text | pretty-printed JSON or error message | — |
| Base64 | multiline text | encoded or decoded string | `Tab` toggles encode/decode mode |
| Hash | multiline text | MD5 / SHA1 / SHA256 lines | — |
| UUID | none | generated UUID | `r` regenerates |
| Regex | pattern (top) + test text (bottom) | test text with matches marked `[match]` | two input areas |
| Timestamp | unix int or date string | both representations | — |

## Focus Model

```
Sidebar  ──Enter/Tab──▶  Input  ──Esc──▶  Sidebar
                          │
                   (Regex only)
                          │
                        Pattern  ──Tab──▶  Input
```

- `Focus::Sidebar` — `j`/`k` or `↑`/`↓` navigate tools; `Enter` or `Tab` moves focus into the tool
- `Focus::Input` — keystrokes go to the main `TextArea`; `Esc` returns to sidebar
- `Focus::Pattern` (Regex only) — single-line field for the regex pattern; `Tab` moves to the test text area
- UUID has no focusable input; `r` works while focus is anywhere

## Architecture

### File structure

```
src/
  main.rs           app loop, layout rendering, sidebar, focus dispatch
  textarea.rs       (exists) multiline editor widget with cursor
  tools/
    mod.rs          Tool trait definition
    json.rs
    base64.rs
    hash.rs
    uuid.rs
    regex.rs
    timestamp.rs
```

### Tool trait

```rust
pub trait Tool {
    fn name(&self) -> &str;
    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus);
    fn handle_key(&mut self, key: KeyEvent) -> Action;
    fn footer_hints(&self) -> &str;
}
```

### App struct

```rust
struct App {
    selected: usize,         // index into tools vec
    focus: Focus,
    tools: Vec<Box<dyn Tool>>,
}
```

### Per-tool structs

Each tool owns its `TextArea`(s) and any mode flags:

- `JsonTool { input: TextArea, output: String, error: Option<String> }`
- `Base64Tool { input: TextArea, output: String, mode: Base64Mode }` — `Base64Mode` is `Encode | Decode`
- `HashTool { input: TextArea, output: String }` — output holds all three hashes
- `UuidTool { current: String }` — no textarea
- `RegexTool { pattern: TextArea, input: TextArea, output: String, focus: RegexFocus }`
- `TimestampTool { input: TextArea, output: String }`

### Transformation

Each tool calls its transform logic inside `handle_key` (or in `render` if stateless). Transformations are pure functions over the current input string — no async, no threads.

## Key Bindings

| Key | Scope | Action |
|-----|-------|--------|
| `j` `k` `↑` `↓` | Sidebar | Navigate tools |
| `Enter` `Tab` | Sidebar | Focus tool input |
| `Esc` | Input | Return to sidebar |
| `Tab` | Base64 input | Toggle encode/decode |
| `Tab` | Regex pattern | Move to test text |
| `r` | UUID (any focus) | Regenerate UUID |
| `q` `ctrl+c` | Any | Quit |

## Error Handling

- JSON: invalid input → output shows the serde_json error string in red
- Regex: invalid pattern → output shows the regex compile error in red; test text untouched
- Base64 decode: invalid input → output shows "invalid base64" in red
- Timestamp: unrecognised format → output shows "unrecognised input" in red

## Out of Scope

- Saving output to file
- Copy-to-clipboard
- Tool configuration persistence
- Any tool not listed above
