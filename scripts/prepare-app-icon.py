#!/usr/bin/env python3
"""Genera un icono con padding estilo macOS para que el dock lo muestre del mismo tamaño visual que otras apps."""

from __future__ import annotations

from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "src/assets/MCP MANAGER ICON.jpg"
OUTPUT_PNG = ROOT / "src/assets/app-icon-macos.png"
OUTPUT_JPG = ROOT / "public/app-icon.jpg"

CANVAS = 1024
# ~68% del canvas: inset similar al safe area de iconos de macOS
CONTENT_SCALE = 0.68


def main() -> None:
    if not SOURCE.exists():
        raise SystemExit(f"No se encontró el icono fuente: {SOURCE}")

    img = Image.open(SOURCE).convert("RGB")
    inner = int(CANVAS * CONTENT_SCALE)
    resized = img.resize((inner, inner), Image.Resampling.LANCZOS)

    canvas = Image.new("RGB", (CANVAS, CANVAS), (0, 0, 0))
    offset = ((CANVAS - inner) // 2, (CANVAS - inner) // 2)
    canvas.paste(resized, offset)

    OUTPUT_PNG.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(OUTPUT_PNG, "PNG", optimize=True)

    OUTPUT_JPG.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(OUTPUT_JPG, "JPEG", quality=95)

    print(f"✓ Icono macOS generado: {OUTPUT_PNG} (escala {CONTENT_SCALE:.0%})")


if __name__ == "__main__":
    main()
