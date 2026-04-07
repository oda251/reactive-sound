#!/usr/bin/env bash
set -euo pipefail

# Reactive BGM - 環境セットアップスクリプト
# 前提: WSL2 (Ubuntu) + Windows, mise インストール済み

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[OK]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Reactive BGM 環境セットアップ ==="
echo ""

# -------------------------------------------------------
# 1. OS / WSL チェック
# -------------------------------------------------------
echo "--- 1. OS / WSL チェック ---"

if ! grep -qi microsoft /proc/version 2>/dev/null; then
  error "WSL2 環境ではありません。このスクリプトは WSL2 + Windows を前提としています。"
  exit 1
fi
info "WSL2 検出"

# mirrored networking チェック
WSLCONFIG="/mnt/c/Users/$(cmd.exe /c "echo %USERNAME%" 2>/dev/null | tr -d '\r')/.wslconfig"
if [[ -f "$WSLCONFIG" ]] && grep -qi "networkingMode=mirrored" "$WSLCONFIG" 2>/dev/null; then
  info ".wslconfig: networkingMode=mirrored 設定済み"
else
  warn ".wslconfig に networkingMode=mirrored が未設定"
  echo "  以下を ${WSLCONFIG} に追加してください:"
  echo "    [wsl2]"
  echo "    networkingMode=mirrored"
  echo "  設定後 'wsl --shutdown' で WSL を再起動してください"
fi

# mirrored が実際に有効か確認
if ip addr show loopback0 &>/dev/null; then
  info "mirrored ネットワーキング有効"
else
  warn "mirrored ネットワーキングが有効になっていません（wsl --shutdown が必要かも）"
fi

# -------------------------------------------------------
# 2. Linux 依存パッケージ (apt)
# -------------------------------------------------------
echo ""
echo "--- 2. Linux 依存パッケージ ---"

APT_PACKAGES=(gcc g++ libgmp-dev)
MISSING_APT=()

for pkg in "${APT_PACKAGES[@]}"; do
  if dpkg -s "$pkg" &>/dev/null; then
    info "$pkg インストール済み"
  else
    MISSING_APT+=("$pkg")
    warn "$pkg 未インストール"
  fi
done

if [[ ${#MISSING_APT[@]} -gt 0 ]]; then
  echo "  インストール: sudo apt-get install -y ${MISSING_APT[*]}"
  read -rp "  今すぐインストールしますか？ [y/N] " ans
  if [[ "$ans" =~ ^[Yy] ]]; then
    sudo apt-get update -qq
    sudo apt-get install -y -qq "${MISSING_APT[@]}"
    info "apt パッケージインストール完了"
  fi
fi

# -------------------------------------------------------
# 3. Haskell ツールチェイン (mise)
# -------------------------------------------------------
echo ""
echo "--- 3. Haskell ツールチェイン ---"

if command -v ghcup &>/dev/null; then
  info "ghcup $(ghcup --version 2>&1 | head -1)"
else
  error "ghcup が見つかりません。mise で ghcup をインストールしてください:"
  echo "  mise use -g ghcup@latest"
fi

if command -v ghc &>/dev/null; then
  info "ghc $(ghc --numeric-version)"
else
  error "ghc が見つかりません。mise でインストールしてください:"
  echo "  mise use -g ghc@latest"
fi

if command -v cabal &>/dev/null; then
  info "cabal $(cabal --numeric-version)"
else
  error "cabal が見つかりません。mise でインストールしてください:"
  echo "  mise use -g cabal@latest"
fi

# -------------------------------------------------------
# 4. TidalCycles
# -------------------------------------------------------
echo ""
echo "--- 4. TidalCycles ---"

if ghci -e "import Sound.Tidal.Boot" -e ":quit" &>/dev/null 2>&1; then
  info "TidalCycles インストール済み"
else
  warn "TidalCycles 未インストール"
  echo "  インストール: cabal update && cabal install --lib tidal"
  read -rp "  今すぐインストールしますか？ [y/N] " ans
  if [[ "$ans" =~ ^[Yy] ]]; then
    cabal update
    cabal install --lib tidal
    info "TidalCycles インストール完了"
  fi
fi

# -------------------------------------------------------
# 5. SuperCollider (Windows)
# -------------------------------------------------------
echo ""
echo "--- 5. SuperCollider (Windows) ---"

SCLANG="/mnt/c/Program Files/SuperCollider-3.14.1/sclang.exe"
if [[ -f "$SCLANG" ]]; then
  info "SuperCollider 検出: $SCLANG"
else
  # バージョン違いも探す
  SCLANG_FOUND=$(find "/mnt/c/Program Files/" -name "sclang.exe" -path "*/SuperCollider*" 2>/dev/null | head -1)
  if [[ -n "$SCLANG_FOUND" ]]; then
    info "SuperCollider 検出: $SCLANG_FOUND"
    warn "スクリプト内のパスと異なる可能性があります"
  else
    error "SuperCollider が見つかりません"
    echo "  インストール: winget install SuperCollider.SuperCollider"
  fi
fi

# SuperDirt チェック
SUPERDIRT_DIR="/mnt/c/Users/$(cmd.exe /c "echo %USERNAME%" 2>/dev/null | tr -d '\r')/AppData/Local/SuperCollider/downloaded-quarks/SuperDirt"
if [[ -d "$SUPERDIRT_DIR" ]]; then
  info "SuperDirt インストール済み"
else
  warn "SuperDirt 未インストール"
  echo "  SC IDE で Quarks.install(\"SuperDirt\") を実行してください"
  echo "  その後 Language → Recompile Class Library (Ctrl+Shift+L)"
fi

# -------------------------------------------------------
# 6. Windows ファイアウォール
# -------------------------------------------------------
echo ""
echo "--- 6. Windows ファイアウォール ---"

if cmd.exe /c "netsh advfirewall firewall show rule name=\"SuperDirt UDP\"" &>/dev/null 2>&1; then
  info "ファイアウォールルール 'SuperDirt UDP' 設定済み"
else
  warn "ファイアウォールルール 'SuperDirt UDP' が未設定"
  echo "  管理者権限の cmd で実行:"
  echo "  netsh advfirewall firewall add rule name=\"SuperDirt UDP\" dir=in action=allow protocol=UDP localport=57120"
fi

# -------------------------------------------------------
# 7. Rust ツールチェイン
# -------------------------------------------------------
echo ""
echo "--- 7. Rust ツールチェイン ---"

if command -v rustc &>/dev/null; then
  info "rustc $(rustc --version | awk '{print $2}')"
else
  error "rustc が見つかりません。mise でインストールしてください:"
  echo "  mise use -g rust@stable"
fi

if command -v cargo &>/dev/null; then
  info "cargo $(cargo --version | awk '{print $2}')"
else
  error "cargo が見つかりません"
fi

# -------------------------------------------------------
# 8. 動作確認
# -------------------------------------------------------
echo ""
echo "--- 8. 動作確認 ---"
echo ""
echo "全チェック完了。動作確認の手順:"
echo ""
echo "  1. SuperDirt 起動:"
echo "     \"$SCLANG\" \"\$(wslpath -w $SCRIPT_DIR/start_superdirt.scd)\""
echo ""
echo "  2. Tidal から音を鳴らす（別ターミナル）:"
echo "     echo ':script $SCRIPT_DIR/BootTidal.hs"
echo "     d1 \$ sound \"bd sn\""
echo "     :!sleep 5"
echo "     d1 \$ silence"
echo "     :quit' | ghci"
echo ""
