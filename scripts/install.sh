#!/usr/bin/env sh
set -eu

REPO="https://github.com/paternosterrack/pater.git"
BIN_DIR="${HOME}/.local/bin"

need_cmd() {
  command -v "$1" >/dev/null 2>&1
}

echo "[pater] starting install..."

if ! need_cmd cargo; then
  echo "[pater] cargo not found. installing rustup toolchain..."
  if need_cmd curl; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  elif need_cmd wget; then
    wget -qO- https://sh.rustup.rs | sh -s -- -y
  else
    echo "[pater] error: need curl or wget to install rustup" >&2
    exit 1
  fi
  # shellcheck disable=SC1090
  . "$HOME/.cargo/env"
fi

echo "[pater] installing latest pater from git..."
cargo install --locked --git "$REPO" pater --force

mkdir -p "$BIN_DIR"
if [ -f "$HOME/.cargo/bin/pater" ]; then
  ln -sf "$HOME/.cargo/bin/pater" "$BIN_DIR/pater"
fi

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *)
    echo "[pater] add this to your shell profile if needed:"
    echo "export PATH=\"$BIN_DIR:\$PATH\""
    ;;
esac

echo "[pater] installed: $("$HOME/.cargo/bin/pater" --version)"
