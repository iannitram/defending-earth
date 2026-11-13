"""Pack the Poly Haven ground textures into game-ready files in assets/ground/.
Run from app/:  python3 -P source_assets/polyhaven/make_ground.py
One RGBA file per surface (the engine has few texture slots):
  R = brightness grain around mid grey (linear brightness x occlusion / its average * 0.5)
  G,B = normal x,y (OpenGL convention)     A = roughness
The field colour itself comes from the site's ground map; these add the grain."""
import os
from PIL import Image, ImageStat, ImageChops
here = os.path.dirname(os.path.abspath(__file__))
out = os.path.join(here, "../../assets/ground")
SIZE = 512
for name in ["leafy_grass", "sparse_grass", "farm_soil", "dry_mud_field_001", "gravel_road"]:
    d = os.path.join(here, name)
    op = lambda kind: Image.open(os.path.join(d, f"{name}_{kind}_1k.jpg")).convert("RGB").resize((SIZE, SIZE), Image.LANCZOS)
    diff, nor, arm = op("diff"), op("nor_gl"), op("arm")
    ao, rough, _ = arm.split()
    lum = diff.convert("L").point(lambda v: int(((v / 255.0) ** 2.2) * 255 + 0.5))
    lum = ImageChops.multiply(lum, ao)
    mean = max(ImageStat.Stat(lum).mean[0], 1.0)
    grain = lum.point(lambda v: max(0, min(255, int(v / mean * 127.5 + 0.5))))
    nx, ny, _ = nor.split()
    if ImageStat.Stat(rough).mean[0] < 20:  # this source ships an empty roughness channel: dry stone
        rough = Image.new("L", (SIZE, SIZE), 225)
    Image.merge("RGBA", (grain, nx, ny, rough)).save(os.path.join(out, name + ".png"), optimize=True)
    print(name, "mean", round(mean), "grain range", grain.getextrema(), "rough mean", round(ImageStat.Stat(rough).mean[0]))
