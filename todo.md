# todo

## devtool — in progress

Subagent-driven implementation. Plan: `docs/superpowers/plans/2026-05-28-devtool.md`

### Status

| Task | Status |
|------|--------|
| Task 1: Tool trait + App scaffold | ✅ done |
| Task 2: JSON tool | 🔄 implemented, awaiting spec review |
| Task 3: Base64 tool | ⏳ pending |
| Task 4: Hash tool | ⏳ pending |
| Task 5: UUID tool | ⏳ pending |
| Task 6: Regex tool | ⏳ pending |
| Task 7: Timestamp + release build | ⏳ pending |

### Resume from

Task 2 is implemented and committed. Resume by running spec + code quality review for Task 2, then continue Tasks 3–7.

Last git log:
```
git -C ~/github.com/scripts log --oneline -5
```

### Notes

- Task 1 code review flagged two Important issues — both were fixed in Task 2:
  - Sidebar `_ =>` arm now propagates Action::Quit
  - Terminal cleanup refactored into run()/run_app() for panic safety
- `textarea.rs` had a pre-existing compile bug (`.as_str()` on `&str`) — fixed in Task 1
- `set_cursor` is deprecated in ratatui 0.29 (pre-existing warning, not yet fixed)
- Action enum will need FocusPattern + FocusInput added in Task 6 (Regex tool)
