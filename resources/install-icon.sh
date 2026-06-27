#!/usr/bin/env bash
set -euo pipefail

APP_ID="org.explorer.app"
DESKTOP_FILE="$APP_ID.desktop"
SVG_FILE="$APP_ID.svg"

# 安装 .desktop 文件
mkdir -p "$HOME/.local/share/applications"
sed "s/Icon=.*/Icon=$APP_ID/" explorer.desktop \
  | sed "s/StartupWMClass=.*/StartupWMClass=$APP_ID/" \
  > "$HOME/.local/share/applications/$DESKTOP_FILE"
echo "  ✓ $HOME/.local/share/applications/$DESKTOP_FILE"

# 安装 SVG 图标
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
mkdir -p "$ICON_DIR"
cp "resources/explorer.svg" "$ICON_DIR/$SVG_FILE"
echo "  ✓ $ICON_DIR/$SVG_FILE"

# 刷新桌面数据库
if command -v update-desktop-database &>/dev/null; then
  update-desktop-database "$HOME/.local/share/applications/" 2>/dev/null || true
  echo "  ✓ update-desktop-database"
fi

if command -v gtk-update-icon-cache &>/dev/null; then
  gtk-update-icon-cache "$HOME/.local/share/icons/hicolor/" 2>/dev/null || true
  echo "  ✓ gtk-update-icon-cache"
fi

echo ""
echo "安装完成！重新运行程序即可看到图标。"
