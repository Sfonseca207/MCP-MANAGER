#!/usr/bin/env python3
"""Genera icono macOS según el estándar Apple HIG.

Referencia (Apple / HIG / plantilla oficial):
- Canvas: 1024×1024 px, PNG con transparencia
- Placa de arte: 832×832 px = 13/16 del canvas, centrada
- Corner radius de la placa: 22% de 832 ≈ 183 px
- Margen transparente: 96 px por lado ((1024 - 832) / 2)
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "src/assets/MCP MANAGER ICON.jpg"
OUTPUT_PNG = ROOT / "src/assets/app-icon-macos.png"
OUTPUT_JPG = ROOT / "public/app-icon.jpg"

CANVAS = 1024
# Estándar Apple: arte ocupa 13/16 del canvas (832 px)
PLATE_SIZE = int(CANVAS * 13 / 16)
PLATE_INSET = (CANVAS - PLATE_SIZE) // 2
# Corner radius: 22% del tamaño de la placa
CORNER_RADIUS = int(PLATE_SIZE * 22 / 100)
# Logo dentro de la placa (~66% de la placa, safe zone interna)
LOGO_SIZE = int(PLATE_SIZE * 0.66)


def rounded_rect_mask(size: int, radius: int) -> Image.Image:
    mask = Image.new("L", (size, size), 0)
    draw = ImageDraw.Draw(mask)
    draw.rounded_rectangle([0, 0, size - 1, size - 1], radius=radius, fill=255)
    return mask


def build_icon(source: Path) -> Image.Image:
    src = Image.open(source).convert("RGBA")
    logo = src.resize((LOGO_SIZE, LOGO_SIZE), Image.Resampling.LANCZOS)
    plate_mask = rounded_rect_mask(PLATE_SIZE, CORNER_RADIUS)

    canvas = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))

    # Placa negra 832×832 centrada (estándar macOS)
    black_plate = Image.new("RGBA", (PLATE_SIZE, PLATE_SIZE), (0, 0, 0, 255))
    plate_layer = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    plate_layer.paste(black_plate, (PLATE_INSET, PLATE_INSET), plate_mask)
    canvas = Image.alpha_composite(canvas, plate_layer)

    # Logo centrado
    logo_offset = ((CANVAS - LOGO_SIZE) // 2, (CANVAS - LOGO_SIZE) // 2)
    canvas.alpha_composite(logo, dest=logo_offset)

    return canvas


def main() -> None:
    if not SOURCE.exists():
        raise SystemExit(f"No se encontró el icono fuente: {SOURCE}")

    icon = build_icon(SOURCE)

    OUTPUT_PNG.parent.mkdir(parents=True, exist_ok=True)
    icon.save(OUTPUT_PNG, "PNG", optimize=True)

    OUTPUT_JPG.parent.mkdir(parents=True, exist_ok=True)
    icon.convert("RGB").save(OUTPUT_JPG, "JPEG", quality=95)

    bbox = icon.getbbox()
    print(
        f"✓ Icono macOS (Apple HIG): placa {PLATE_SIZE}px (13/16), "
        f"radius {CORNER_RADIUS}px, logo {LOGO_SIZE}px, bbox {bbox}"
    )


if __name__ == "__main__":
    main()
