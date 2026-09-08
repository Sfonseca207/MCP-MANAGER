#!/usr/bin/env bash
set -euo pipefail

APP_NAME="MCP Manager.app"
BUILD_DIR="src-tauri/target/release/bundle/macos"
SOURCE="$BUILD_DIR/$APP_NAME"
DEST="/Applications/$APP_NAME"

if [ ! -d "$SOURCE" ]; then
  echo "No se encontró $SOURCE"
  echo "Ejecuta primero: npm run tauri:build"
  exit 1
fi

echo "Instalando $APP_NAME en /Applications..."
rm -rf "$DEST"
cp -R "$SOURCE" "$DEST"

# Firma ad-hoc para evitar bloqueos básicos de Gatekeeper
codesign --force --deep --sign - "$DEST" 2>/dev/null || true

# Evita duplicado en Spotlight (build vs /Applications)
rm -rf "$SOURCE"

echo "✓ Instalado en $DEST"
echo "  Copia de build eliminada para no duplicar en Spotlight."
echo "  Si macOS lo bloquea: clic derecho → Abrir"
