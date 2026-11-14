"""Opens model.blend and writes the Maul's check pictures into renders/.

  flatpak run org.blender.Blender -b --python "<full path>/render.py"

To draw only some pictures, name them after `--`, for example
  ... render.py -- 03 07
The contact sheet is rebuilt every time.
"""
import os
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "scale")))

import bpy
from mathutils import Vector

import scale_lib as sl

NAME = "maul"
OUT = os.path.join(HERE, "renders")
PICTURES = ["01_front", "02_side", "03_threequarter", "04_aerial", "05_size", "06_night",
            "07_detail", "08_dead", "08b_dead_wild", "09_wild", "10_wild_night",
            "11_spike_aerial_4"]
STATES = ["model", "wild", "dead"]


def wanted():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if not args:
        return PICTURES
    return [p for p in PICTURES if any(p.startswith(a + "_") or p == a for a in args)]


def path(name):
    return os.path.join(OUT, name + ".png")


def main():
    todo = wanted()
    sl.reset_scene()
    with bpy.data.libraries.load(os.path.join(HERE, "model.blend"), link=False) as (src, dst):
        dst.collections = list(STATES)
    for c in dst.collections:
        bpy.context.scene.collection.children.link(c)
    colls = {s: bpy.data.collections[s] for s in STATES}
    info = sl.thing(NAME)
    foot = info["footprint"]
    sl.make_ground(footprint=foot)

    root = bpy.data.objects[NAME]
    wroot = bpy.data.objects["wild_" + NAME]
    droot = bpy.data.objects["dead_" + NAME]
    spike = bpy.data.objects["control_spike"]
    dead_spike = [o for o in colls["dead"].objects
                  if o.name.startswith(("dead_control_spike", "dead_collar_arc"))]
    dead_stump = bpy.data.objects["dead_spike_stump"]

    def show(state, dead_wild=False):
        for s in STATES:
            for o in colls[s].objects:
                o.hide_render = (s != state)
        if state == "dead":
            for o in dead_spike:
                o.hide_render = dead_wild
            dead_stump.hide_render = not dead_wild

    def drop(objs):
        for o in objs:
            bpy.data.objects.remove(o)

    show("model")
    sl.day_light()
    if "01_front" in todo:
        sl.cam_front([root])
        sl.render(path("01_front"))
    if "02_side" in todo:
        sl.cam_side([root])
        sl.render(path("02_side"))
    if "03_threequarter" in todo:
        sl.cam_threequarter([root])
        sl.render(path("03_threequarter"))
    if "04_aerial" in todo:
        extra = [sl.footprint_outline(*foot)]
        sl.cam_aerial([root])
        sl.render(path("04_aerial"))
        drop(extra)
    if "05_size" in todo:
        extra = [sl.add_soldier(0.0, -3.0), sl.scale_bar(0.0, -4.2, 1.0)]
        sl.cam_ortho_side([root] + extra)
        sl.render(path("05_size"))
        drop(extra)
    if "07_detail" in todo:
        p = spike.matrix_world.translation
        sl.cam_detail((p.x, p.y, p.z + 0.48), 4.2, elevation_deg=33.0)
        sl.render(path("07_detail"))
    if "11_spike_aerial_4" in todo:
        sc = bpy.context.scene
        old = (sc.render.resolution_x, sc.render.resolution_y)
        sc.render.resolution_x, sc.render.resolution_y = 800, 450
        extra = [sl.footprint_outline(*foot)]
        tiles = []
        for label, az in (("from_front_right", 35.0), ("from_back_right", 125.0),
                          ("from_back_left", 215.0), ("from_front_left", 305.0)):
            sl.cam_aerial([root], azimuth_deg=az)
            tiles.append(sl.render(path("aerial_" + label)))
        sl.contact_sheet(tiles, path("11_spike_aerial_4"), columns=2, cell_width=800)
        for t in tiles:
            os.remove(t)
        drop(extra)
        sc.render.resolution_x, sc.render.resolution_y = old
    if "08_dead" in todo:
        show("dead")
        sl.cam_threequarter([droot], elevation_deg=24.0, margin=1.08)
        sl.render(path("08_dead"))
    if "08b_dead_wild" in todo:
        show("dead", dead_wild=True)
        sl.cam_threequarter([droot], elevation_deg=24.0, margin=1.08)
        sl.render(path("08b_dead_wild"))
    wild_all = [wroot, bpy.data.objects["wild_broken_stake"]]
    if "09_wild" in todo:
        show("wild")
        sl.cam_threequarter(wild_all, elevation_deg=8.0)
        sl.render(path("09_wild"))
    if "06_night" in todo or "10_wild_night" in todo:
        sl.night_light()
        if "06_night" in todo:
            show("model")
            sl.cam_threequarter([root])
            sl.render(path("06_night"))
        if "10_wild_night" in todo:
            show("wild")
            sl.cam_threequarter(wild_all, elevation_deg=8.0)
            sl.render(path("10_wild_night"))
    sl.contact_sheet([path(p) for p in PICTURES], os.path.join(OUT, "contact.png"))


main()
