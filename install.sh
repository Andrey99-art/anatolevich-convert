#!/bin/bash
# ═══════════════════════════════════════════════════════════════
# AnatolevichConvert — Installation Script
# ═══════════════════════════════════════════════════════════════

set -e

echo "🔨 Собираем release-версию..."
cargo build --release

echo ""
echo "📦 Устанавливаем бинарник..."
sudo cp target/release/anatolevich-convert /usr/local/bin/anatolevich-convert
sudo chmod +x /usr/local/bin/anatolevich-convert

echo ""
echo "🎨 Устанавливаем иконку..."
mkdir -p ~/.local/share/icons/hicolor/scalable/apps/
mkdir -p ~/.local/share/icons/hicolor/128x128/apps/
mkdir -p ~/.local/share/icons/hicolor/256x256/apps/
mkdir -p ~/.local/share/icons/hicolor/512x512/apps/

cp icons/anatolevich-convert.svg ~/.local/share/icons/hicolor/scalable/apps/anatolevich-convert.svg
cp icons/anatolevich-convert-128.png ~/.local/share/icons/hicolor/128x128/apps/anatolevich-convert.png
cp icons/anatolevich-convert-256.png ~/.local/share/icons/hicolor/256x256/apps/anatolevich-convert.png
cp icons/anatolevich-convert-512.png ~/.local/share/icons/hicolor/512x512/apps/anatolevich-convert.png

echo ""
echo "📋 Устанавливаем .desktop файл..."
mkdir -p ~/.local/share/applications/
cp anatolevich-convert.desktop ~/.local/share/applications/anatolevich-convert.desktop

echo ""
echo "🔄 Обновляем кэш..."
update-desktop-database ~/.local/share/applications/ 2>/dev/null || true
gtk-update-icon-cache ~/.local/share/icons/hicolor/ 2>/dev/null || true

echo ""
echo "✅ Установка завершена!"
echo ""
echo "   Запуск: anatolevich-convert"
echo "   Или найдите «AnatolevichConvert» в меню приложений GNOME (Super)"
echo ""
