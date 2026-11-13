"""Builds crates/setup/icon/bach.ico (the Windows program icon) from the 1024px
source image next to it. Run: python scripts/make-icon.py"""
from pathlib import Path

from PIL import Image

ICON_DIR = Path(__file__).resolve().parent.parent / "crates" / "app" / "icon"
SIZES = [16, 20, 24, 32, 40, 48, 64, 128, 256]

source = Image.open(ICON_DIR / "bach-icon-1024.png").convert("RGBA")
# Resize each size ourselves with a sharp filter so small sizes stay crisp.
frames = [source.resize((size, size), Image.LANCZOS) for size in SIZES]
frames[-1].save(
    ICON_DIR / "bach.ico",
    format="ICO",
    sizes=[(size, size) for size in SIZES],
    append_images=frames[:-1],
)
print("wrote", ICON_DIR / "bach.ico")
