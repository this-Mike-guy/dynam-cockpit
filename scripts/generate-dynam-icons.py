"""Reproduce the DYNAM app icons with Pillow. Geometry matches dynam-mark.svg."""
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
ICONS = ROOT / "src-tauri" / "icons"
SCALE = 4
canvas = Image.new("RGBA", (512 * SCALE, 512 * SCALE))
draw = ImageDraw.Draw(canvas)
navy = "#101c25"
mint = "#67e8c5"
draw.rounded_rectangle((0, 0, 512 * SCALE - 1, 512 * SCALE - 1), 112 * SCALE, fill=navy)

def polygon(points, fill):
    draw.polygon([(x * SCALE, y * SCALE) for x, y in points], fill=fill)

polygon([(152, 112), (288, 112), (376, 200), (376, 312), (288, 400), (152, 400)], mint)
polygon([(224, 184), (258, 184), (304, 230), (304, 282), (258, 328), (224, 328)], navy)

base = canvas.resize((1024, 1024), Image.Resampling.LANCZOS)
for path in ICONS.glob("*.png"):
    # Retain each platform's required icon dimensions.
    with Image.open(path) as old:
        size = old.size
    base.resize(size, Image.Resampling.LANCZOS).save(path)
base.save(ICONS / "icon.ico", sizes=[(s, s) for s in (16, 24, 32, 48, 64, 128, 256)])
base.save(ICONS / "icon.icns", format="ICNS")
print("DYNAM desktop icons generated.")
