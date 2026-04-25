#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Install the Clodhopper harness binary from an extracted release package.

Usage:
  ./install.sh [--dir INSTALL_DIR]

Options:
  --dir INSTALL_DIR  Install directory. Defaults to $INSTALL_DIR or ~/.local/bin.
  -h, --help         Show this help text.

Examples:
  ./install.sh
  ./install.sh --dir "$HOME/bin"
  INSTALL_DIR="$HOME/.local/bin" ./install.sh
USAGE
}

install_dir="${INSTALL_DIR:-$HOME/.local/bin}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dir)
      if [[ $# -lt 2 ]]; then
        echo "error: --dir requires a value" >&2
        exit 2
      fi
      install_dir="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
binary="$script_dir/bin/harness"

if [[ ! -f "$binary" ]]; then
  echo "error: expected release binary at $binary" >&2
  echo "extract the release archive, then run install.sh from the extracted directory" >&2
  exit 1
fi

mkdir -p "$install_dir"
install -m 0755 "$binary" "$install_dir/harness"

echo "installed harness to $install_dir/harness"

case ":$PATH:" in
  *":$install_dir:"*) ;;
  *)
    echo "warning: $install_dir is not on PATH"
    echo "add this to your shell profile if needed:"
    echo "  export PATH=\"$install_dir:\$PATH\""
    ;;
esac

"$install_dir/harness" --help >/dev/null
echo "verified harness executable"
echo "next: harness doctor"

