#!/usr/bin/env bash
set -e

SCRIPTS_DIR="$(cd "$(dirname "$0")" && pwd)"
CARGO=$(which cargo 2>/dev/null || echo "$HOME/.cargo/bin/cargo")

echo "→ Building harbor..."
cd "$SCRIPTS_DIR/harbor"
"$CARGO" build --release
echo "✓ Built: $SCRIPTS_DIR/harbor/target/release/harbor"

echo "→ Building recall..."
cd "$SCRIPTS_DIR/recall"
"$CARGO" build --release
echo "✓ Built: $SCRIPTS_DIR/recall/target/release/recall"

echo "→ Building tdo..."
cd "$SCRIPTS_DIR/tdo"
"$CARGO" build --release
echo "✓ Built: $SCRIPTS_DIR/tdo/target/release/tdo"

echo "→ Building jot..."
cd "$SCRIPTS_DIR/jot"
"$CARGO" build --release
echo "✓ Built: $SCRIPTS_DIR/jot/target/release/jot"

ZSHRC="$HOME/.zshrc"
SOURCE_LINE="source $SCRIPTS_DIR/aliases.sh"

if grep -qF "$SOURCE_LINE" "$ZSHRC" 2>/dev/null; then
    echo "✓ aliases.sh already sourced in $ZSHRC"
else
    echo "" >> "$ZSHRC"
    echo "# github.com scripts" >> "$ZSHRC"
    echo "$SOURCE_LINE" >> "$ZSHRC"
    echo "✓ Added source line to $ZSHRC"
fi

echo ""
echo "Done. Open a new shell or run:"
echo "  source ~/.zshrc"
echo ""
echo "Then use:"
echo "  t / ctrl+t   → project selector"
echo "  ctrl+r       → history search"
echo "  jot / ctrl+n → quick capture"
