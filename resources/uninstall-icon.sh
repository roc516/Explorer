#!/usr/bin/env bash
set -euo pipefail

APP_ID="org.explorer.app"

rm -f "$HOME/.local/share/applications/$APP_ID.desktop"
rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/$APP_ID.svg"
rm -f "$HOME/.local/share/icons/hicolor/16x16/apps/$APP_ID.png"
rm -f "$HOME/.local/share/icons/hicolor/32x32/apps/$APP_ID.png"
rm -f "$HOME/.local/share/icons/hicolor/64x64/apps/$APP_ID.png"
rm -f "$HOME/.local/share/icons/hicolor/128x128/apps/$APP_ID.png"

if command -v update-desktop-database &>/dev/null; then
  update-desktop-database "$HOME/.local/share/applications/" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache &>/dev/null; then
  gtk-update-icon-cache "$HOME/.local/share/icons/hicolor/" 2>/dev/null || true
fi

echo "已卸载图标和 .desktop 文件。"
