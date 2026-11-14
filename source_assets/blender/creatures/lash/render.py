"""Job C03: check pictures of the Lash. Brings in model.blend and renders.

  flatpak run org.blender.Blender -b --python "<full path>/render.py" -- [names] [--samples N]

names: front side threequarter aerial size night detail dead (default: all).
The contact sheet is always remade from the pictures that exist.
"""
import sys
import os

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "scale")))

import bpy
import scale_lib as sl

OUT = os.path.join(HERE, "renders")
NAMES = ["front", "side", "threequarter", "aerial", "size", "night", "detail", "dead"]
FILES = {"front": "01_front.png", "side": "02_side.png", "threequarter": "03_threequarter.png",
         "aerial": "04_aerial.png", "size": "05_size.png", "night": "06_night.png",
         "detail": "07_detail.png", "dead": "08_dead.png"}


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    samples = None
    if "--samples" in argv:
        i = argv.index("--samples")
        samples = int(argv[i + 1])
        del argv[i:i + 2]
    todo = [n for n in NAMES if n in argv] or NAMES

    sc = sl.reset_scene()
    if samples:
        sc.cycles.samples = samples
    with bpy.data.libraries.load(os.path.join(HERE, "model.blend"), link=False) as (src, dst):
        dst.collections = [c for c in src.collections if c in ("model", "dead")]
    for c in dst.collections:
        sc.collection.children.link(c)
    model = bpy.data.collections["model"]
    dead = bpy.data.collections["dead"]
    root = bpy.data.objects["lash"]
    dead_root = bpy.data.objects["dead_lash_dead"]
    arcs = bpy.data.objects["arcs_spines"]
    foot = sl.thing("lash")["footprint"]

    def show(alive, night=False):
        model.hide_render = not alive
        model.hide_viewport = not alive
        dead.hide_render = alive
        dead.hide_viewport = alive
        arcs.hide_render = not night
        arcs.hide_viewport = not night
        bpy.context.view_layer.update()

    sl.make_ground(footprint=foot)
    extras = []

    def clear_extras():
        for o in extras:
            bpy.data.objects.remove(o)
        del extras[:]

    for name in todo:
        clear_extras()
        path = os.path.join(OUT, FILES[name])
        show(name != "dead", night=(name == "night"))
        if name == "night":
            sl.night_light()
        else:
            sl.day_light()
        if name == "front":
            sl.cam_front([root])
        elif name == "side":
            sl.cam_side([root])
        elif name == "threequarter":
            sl.cam_threequarter([root])
        elif name == "aerial":
            extras.append(sl.footprint_outline(*foot))
            sl.cam_aerial([root])
        elif name == "size":
            extras.append(sl.add_soldier(0.0, 2.9))
            extras.append(sl.scale_bar(-1.3, 4.1))
            sl.cam_ortho_side([root] + extras)
        elif name == "night":
            sl.cam_threequarter([root])
        elif name == "detail":
            # mask, halo, arms, rear of the rifle with bells and strips
            sl.cam_detail((-0.20, -1.45, 1.27), 4.3, azimuth_deg=-40.0, elevation_deg=6.0)
        elif name == "dead":
            sl.cam_threequarter([dead_root], elevation_deg=22.0)
        sl.render(path)
        print("LASH RENDERED", path)
    clear_extras()
    sl.contact_sheet([os.path.join(OUT, FILES[n]) for n in NAMES], os.path.join(OUT, "contact.png"))


main()
