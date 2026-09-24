"""Generate the deterministic Aster monogram raster/icon pair from source.svg geometry.

Engineering utility only; Pillow is not a runtime dependency of the app.
"""

from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent
SIZE = 256
SCALE = 4
CANVAS = SIZE * SCALE


def point(v):
    return round(v * SCALE)


def mix(a, b, ratio):
    return tuple(round(x + (y - x) * ratio) for x, y in zip(a, b))


def gradient(at, stops):
    for (start, a), (end, b) in zip(stops, stops[1:]):
        if at <= end:
            return mix(a, b, max(0, min(1, (at - start) / (end - start))))
    return stops[-1][1]


img = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)
disc = [(0, (53, 76, 122)), (0.52, (13, 20, 35)),
        (0.83, (16, 44, 96)), (1, (6, 19, 43))]

def circle(radius, color, width=0):
    box = (point(128 - radius), point(128 - radius), point(128 + radius), point(128 + radius))
    if width:
        draw.ellipse(box, outline=color + (255,), width=point(width))
    else:
        draw.ellipse(box, fill=color + (255,))


circle(119, (10, 19, 40))
circle(119, (31, 69, 140), 5)
circle(111, (6, 19, 43))
for radius in range(110, -1, -1):
    color = gradient(1 - radius / 110, disc)
    circle(radius, color)
circle(111, (84, 167, 230), 5)
circle(102, (47, 91, 147), 2)

outer = [(73, 192), (112, 66), (118, 55), (128, 53), (138, 55),
         (144, 66), (183, 192), (152, 192), (143, 160),
         (112, 160), (103, 192)]
hole = [(120, 136), (128, 105), (136, 136)]
mask = Image.new("L", (CANVAS, CANVAS))
ink = ImageDraw.Draw(mask)
ink.polygon([(point(x), point(y)) for x, y in outer], fill=255)
ink.polygon([(point(x), point(y)) for x, y in hole], fill=0)
letter = Image.new("RGBA", img.size, (0, 0, 0, 0))
shade = ImageDraw.Draw(letter)
for y in range(point(52), point(194)):
    f = (y / SCALE - 52) / 142
    color = gradient(f, [(0, (255, 255, 255)), (0.55, (231, 244, 255)),
                         (1, (58, 185, 250))])
    shade.line((0, y, CANVAS, y), fill=color + (255,))
img.paste(letter, (0, 0), mask)
draw = ImageDraw.Draw(img)
draw.line([(point(x), point(y)) for x, y in outer + [outer[0]]],
          fill=(194, 237, 255, 255), width=point(2), joint="curve")
draw.line([(point(x), point(y)) for x, y in hole + [hole[0]]],
          fill=(194, 237, 255, 255), width=point(2), joint="curve")
arc = []
for step in range(101):
    t = step / 100
    x = 44 * (1 - t) ** 2 + 2 * 128 * t * (1 - t) + 212 * t ** 2
    y = 195 * (1 - t) ** 2 + 2 * 253 * t * (1 - t) + 195 * t ** 2
    arc.append((point(x), point(y)))
draw.line(arc, fill=(15, 212, 238, 185), width=point(4), joint="curve")

icon = img.resize((SIZE, SIZE), Image.Resampling.LANCZOS)
icon.save(ROOT / "icon.png", format="PNG", optimize=False)
icon.save(ROOT / "icon.ico", format="ICO",
          sizes=[(16, 16), (24, 24), (32, 32), (48, 48),
                 (64, 64), (128, 128), (256, 256)])
