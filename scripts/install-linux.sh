#!/usr/bin/env bash
# QuickSort Linux Installer
# Detects the desktop environment and installs the appropriate file manager
# integration (Nautilus/Nemo, Dolphin, Thunar, PCManFM-Qt).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }

# Detect the desktop environment from environment variables.
detect_de() {
    if [ -n "${XDG_CURRENT_DESKTOP:-}" ]; then
        echo "$XDG_CURRENT_DESKTOP"
    elif [ -n "${DESKTOP_SESSION:-}" ]; then
        echo "$DESKTOP_SESSION"
    else
        echo "unknown"
    fi
}

# Install for Nautilus/Nemo/Caja (GNOME/Cinnamon/MATE).
install_python_ext() {
    local ext_name="$1"   # nautilus, nemo or caja
    info "Installing $ext_name extension..."

    # python3-nautilus provides the gi bindings for nautilus/nemo.
    if ! dpkg -l | grep -q python3-nautilus 2>/dev/null; then
        info "Installing python3-nautilus..."
        sudo apt-get update
        sudo apt-get install -y python3-nautilus
    fi

    # Nautilus uses nautilus-python, Nemo uses nemo-python.
    if [ "$ext_name" = "nemo" ] && ! dpkg -l | grep -q nemo-python 2>/dev/null; then
        info "Installing nemo-python..."
        sudo apt-get install -y nemo-python
    fi

    local extensions_dir="$HOME/.local/share/nautilus-python/extensions"
    mkdir -p "$extensions_dir"

    # Copy the extension and the shared IPC module.
    cp "$REPO_DIR/extensions/nautilus/quicksort-nautilus.py" "$extensions_dir/"
    chmod +x "$extensions_dir/quicksort-nautilus.py"
    mkdir -p "$extensions_dir/shared"
    cp "$REPO_DIR/extensions/shared/ipc_client.py" "$extensions_dir/shared/"

    info "$ext_name extension installed."

    # Restart the file manager if it is running.
    if pgrep -x "$ext_name" > /dev/null; then
        info "Restarting $ext_name..."
        "$ext_name" -q 2>/dev/null || true
    fi
}

# Install for Dolphin (KDE).
install_dolphin() {
    info "Installing Dolphin Service Menu..."
    local services_dir="$HOME/.local/share/kservices5"
    mkdir -p "$services_dir"
    cp "$REPO_DIR/extensions/dolphin/quicksort.desktop" "$services_dir/"

    if command -v update-desktop-database &> /dev/null; then
        update-desktop-database "$services_dir" 2>/dev/null || true
    fi

    info "Dolphin Service Menu installed. Please restart Dolphin."
}

# Install for Thunar (XFCE). Thunar custom actions are configured in GUI,
# so we only print guidance.
install_thunar() {
    info "Thunar integration requires manual configuration."
    info "Open Thunar → Edit → Configure Custom Actions and add a QuickSort action with:"
    info "  Command: quicksort select-folder --file %f"
}

# Install for PCManFM-Qt (LXQt).
install_pcmanfm() {
    info "Installing PCManFM-Qt custom actions..."
    local actions_dir="$HOME/.config/libfm/actions"
    mkdir -p "$actions_dir"
    cp "$REPO_DIR/extensions/thunar/quicksort.desktop" "$actions_dir/"
    info "PCManFM-Qt custom actions installed."
}

# Main installation routine.
main() {
    info "QuickSort Linux Installer"
    info "========================="

    local de
    de=$(detect_de)
    info "Detected DE: $de"

    case "$de" in
        GNOME|gnome|ubuntu|zorin)
            install_python_ext nautilus
            ;;
        KDE|kde|plasma|neon)
            install_dolphin
            ;;
        XFCE|xfce)
            install_thunar
            ;;
        LXQt|lxqt)
            install_pcmanfm
            ;;
        Cinnamon|cinnamon)
            install_python_ext nemo
            ;;
        MATE|mate)
            install_python_ext nautilus
            ;;
        *)
            warn "Unknown DE: $de"
            warn "Installing Nautilus (GNOME) integration by default..."
            install_python_ext nautilus
            ;;
    esac

    info ""
    info "Installation complete!"
    info "Right-click in your file manager to see QuickSort options."
    info "Make sure the QuickSort Tauri app is running for the integration to work."
}

main "$@"
