#!/usr/bin/env zsh
# Source this in your ~/.zshrc:
#   source ~/github.com/scripts/aliases.sh

# ── History settings ───────────────────────────────────────────────────────────
HISTFILE="$HOME/.zsh_history"
HISTSIZE=100000
SAVEHIST=100000

setopt EXTENDED_HISTORY        # save timestamp + duration
setopt HIST_EXPIRE_DUPS_FIRST  # expire duplicates before unique entries
setopt HIST_IGNORE_DUPS        # skip consecutive duplicates
setopt HIST_IGNORE_ALL_DUPS    # remove older duplicate entries
setopt HIST_FIND_NO_DUPS       # skip duplicates when searching
setopt HIST_IGNORE_SPACE       # skip commands starting with space
setopt HIST_REDUCE_BLANKS      # remove extra blanks
setopt HIST_VERIFY             # show expanded history before running
setopt SHARE_HISTORY           # share history across sessions, append immediately

# ── harbor — project selector ──────────────────────────────────────────────────
HARBOR_BIN="$HOME/github.com/scripts/harbor/target/release/harbor"

function t() {
    "$HARBOR_BIN"
}

function _t_widget() {
    zle -I
    "$HARBOR_BIN"
    zle reset-prompt
}
zle -N _t_widget
bindkey '^t' _t_widget

# ── recall — history search (ctrl+r) ──────────────────────────────────────────
RECALL_BIN="$HOME/github.com/scripts/recall/target/release/recall"

function recall() {
    local tmp=$(mktemp)
    "$RECALL_BIN" "$tmp"
    if [[ -s "$tmp" ]]; then
        print -z "$(cat "$tmp")"
    fi
    rm -f "$tmp"
}

bindkey -s '^r' '^urecall\n'

# ── tdo — daily todo ───────────────────────────────────────────────────────────
alias todo="$HOME/github.com/scripts/tdo/target/release/tdo"

# ── jot — quick capture ────────────────────────────────────────────────────────
JOT_BIN="$HOME/github.com/scripts/jot/target/release/jot"

alias jot="$JOT_BIN"

function _jot_widget() {
    zle -I
    "$JOT_BIN"
    zle reset-prompt
}
zle -N _jot_widget
bindkey '^n' _jot_widget
