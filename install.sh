#!/bin/sh
# install.sh — Install aip (AI Providers) from GitHub Releases
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/Albert556/ai-providers/main/install.sh | sh
#   VERSION=1.1.0 curl -fsSL ... | sh
#   INSTALL_DIR=/opt/bin curl -fsSL ... | sh
#   UNINSTALL=1 curl -fsSL ... | sh

set -u

GITHUB_API_BASE="https://api.github.com"
GITHUB_BASE="https://github.com"
REPO="Albert556/ai-providers"
DEFAULT_INSTALL_DIR="$HOME/.local/bin"
PATH_EXPORT_COMMENT="# Added by aip installer"

say() {
    printf 'aip: %s\n' "$*"
}

err() {
    say "ERROR: $*" >&2
    exit 1
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        err "need '$1' (command not found)"
    fi
}

detect_downloader() {
    if command -v curl > /dev/null 2>&1; then
        _downloader="curl"
    elif command -v wget > /dev/null 2>&1; then
        _downloader="wget"
    else
        err "need 'curl' or 'wget' to download files"
    fi
}

download() {
    _url="$1"
    _output="$2"
    if [ "$_downloader" = "curl" ]; then
        curl -fsSL -o "$_output" "$_url"
    else
        wget -qO "$_output" "$_url"
    fi
}

download_to_stdout() {
    _url="$1"
    if [ "$_downloader" = "curl" ]; then
        curl -fsSL "$_url"
    else
        wget -qO- "$_url"
    fi
}

detect_platform() {
    _os="$(uname -s)"
    _arch="$(uname -m)"

    case "$_os" in
        Linux)
            case "$_arch" in
                x86_64) _target="x86_64-unknown-linux-gnu" ;;
                *) err "unsupported architecture: $_arch (Linux only supports x86_64)" ;;
            esac
            ;;
        Darwin)
            case "$_arch" in
                arm64) _target="aarch64-apple-darwin" ;;
                *) err "unsupported architecture: $_arch (macOS only supports arm64)" ;;
            esac
            ;;
        *) err "unsupported OS: $_os" ;;
    esac
}

get_latest_version() {
    # Try /releases/latest first
    _json="$(download_to_stdout "${GITHUB_API_BASE}/repos/${REPO}/releases/latest" 2>/dev/null)" || _json=""

    if [ -n "$_json" ]; then
        _version="$(printf '%s' "$_json" | grep -o '"tag_name"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v\{0,1\}\([^"]*\)".*/\1/')"
        if [ -n "$_version" ]; then
            return
        fi
    fi

    # Fallback: list releases
    _json="$(download_to_stdout "${GITHUB_API_BASE}/repos/${REPO}/releases?per_page=1" 2>/dev/null)" || _json=""
    if [ -n "$_json" ]; then
        _version="$(printf '%s' "$_json" | grep -o '"tag_name"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v\{0,1\}\([^"]*\)".*/\1/')"
        if [ -n "$_version" ]; then
            return
        fi
    fi

    err "could not determine latest version from GitHub API"
}

configure_path() {
    _install_dir="$1"

    # Check if already in PATH
    case ":$PATH:" in
        *":${_install_dir}:"*) return ;;
    esac

    _shell_name="$(basename "${SHELL:-}")"
    _rc_file=""
    _path_line="export PATH=\"${_install_dir}:\$PATH\" ${PATH_EXPORT_COMMENT}"

    case "$_shell_name" in
        bash) _rc_file="$HOME/.bashrc" ;;
        zsh)  _rc_file="$HOME/.zshrc" ;;
        fish)
            # Fish uses a different syntax
            _fish_dir="$HOME/.config/fish/conf.d"
            _fish_file="${_fish_dir}/aip.fish"
            if [ -f "$_fish_file" ] && grep -q "aip installer" "$_fish_file" 2>/dev/null; then
                say "PATH already configured in $_fish_file"
                return
            fi
            mkdir -p "$_fish_dir"
            printf '# Added by aip installer\nfish_add_path %s\n' "$_install_dir" > "$_fish_file"
            say "PATH configured in $_fish_file"
            say "restart your terminal or run: source $_fish_file"
            return
            ;;
        *)
            say "add the following to your shell rc file:"
            say "  export PATH=\"${_install_dir}:\$PATH\""
            return
            ;;
    esac

    # For bash/zsh: check if already added
    if [ -f "$_rc_file" ] && grep -qF "$PATH_EXPORT_COMMENT" "$_rc_file" 2>/dev/null; then
        say "PATH already configured in $_rc_file"
        return
    fi

    # Append to rc file
    printf '\n%s\n' "$_path_line" >> "$_rc_file"
    say "PATH configured in $_rc_file"
    say "run: source $_rc_file  (or restart your terminal)"
}

remove_path_config() {
    _install_dir="$1"

    # Remove from bash/zsh rc files
    for _rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
        if [ -f "$_rc" ] && grep -qF "$PATH_EXPORT_COMMENT" "$_rc" 2>/dev/null; then
            # Create temp file without the PATH line
            _tmp="$(mktemp)"
            grep -vF "$PATH_EXPORT_COMMENT" "$_rc" > "$_tmp"
            mv "$_tmp" "$_rc"
            say "removed PATH config from $_rc"
        fi
    done

    # Remove fish config
    _fish_file="$HOME/.config/fish/conf.d/aip.fish"
    if [ -f "$_fish_file" ]; then
        rm -f "$_fish_file"
        say "removed $_fish_file"
    fi
}

do_install() {
    detect_downloader
    detect_platform

    _install_dir="${INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"

    if [ -z "${VERSION:-}" ]; then
        get_latest_version
    else
        _version="$VERSION"
    fi

    say "installing aip v${_version} (${_target})"

    # Create temp dir with cleanup trap
    _tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$_tmp_dir"' EXIT

    # Construct download URL
    _url="${GITHUB_BASE}/${REPO}/releases/download/v${_version}/aip-v${_version}-${_target}"
    _tmp_file="${_tmp_dir}/aip"

    say "downloading from ${_url}"
    download "$_url" "$_tmp_file" || err "download failed — check that version v${_version} exists"

    chmod +x "$_tmp_file"
    mkdir -p "$_install_dir"
    mv "$_tmp_file" "${_install_dir}/aip"

    # Verify
    if "${_install_dir}/aip" --version > /dev/null 2>&1; then
        say "installed $("${_install_dir}/aip" --version) to ${_install_dir}/aip"
    else
        say "installed to ${_install_dir}/aip (could not verify version)"
    fi

    configure_path "$_install_dir"
}

do_uninstall() {
    _install_dir="${INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
    _bin="${_install_dir}/aip"

    if [ -f "$_bin" ]; then
        rm -f "$_bin"
        say "removed $_bin"
    else
        say "$_bin not found, nothing to remove"
    fi

    remove_path_config "$_install_dir"
    say "uninstall complete"
}

parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --version)
                shift
                VERSION="${1:-}"
                [ -z "$VERSION" ] && err "--version requires a value"
                ;;
            --uninstall)
                UNINSTALL=1
                ;;
            --install-dir)
                shift
                INSTALL_DIR="${1:-}"
                [ -z "$INSTALL_DIR" ] && err "--install-dir requires a value"
                ;;
            -h|--help)
                cat <<'EOF'
install.sh — Install aip (AI Providers)

Usage:
  curl -fsSL <url>/install.sh | sh
  ./install.sh [OPTIONS]

Options:
  --version <VER>       Install specific version (default: latest)
  --install-dir <DIR>   Custom install directory (default: ~/.local/bin)
  --uninstall           Remove aip and PATH configuration
  -h, --help            Show this help

Environment variables:
  VERSION               Same as --version
  INSTALL_DIR           Same as --install-dir
  UNINSTALL             Set to 1 to uninstall
EOF
                exit 0
                ;;
            *)
                err "unknown option: $1 (use --help for usage)"
                ;;
        esac
        shift
    done
}

main() {
    parse_args "$@"

    if [ "${UNINSTALL:-0}" = "1" ]; then
        do_uninstall
    else
        do_install
    fi
}

main "$@"
