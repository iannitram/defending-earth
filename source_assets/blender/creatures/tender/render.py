"""Opens model.blend and writes the Tender's check pictures into renders/.

  flatpak run org.blender.Blender -b --python "<full path>/render.py"

To draw only some pictures, name them after `--`, for example
  ... render.py -- 03 07
`contact` rebuilds only the contact sheet.
"""
import math
import os
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "scale")))

import bpy
from mathutils import Vector

import scale_lib as sl

NAME = "tender"
OUT = os.path.join(HERE, "renders")
PICTURES = ["01_front", "02_side", "03_threequarter", "04_aerial", "05_size", "06_night",
            "07_detail", "08_dead", "09_with_regent_block"]


def wanted():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if not args:
        return PICTURES, True
    return [p for p in PICTURES if any(p.startswith(a) for a in args)], True


def load_model():
    sl.reset_scene()
    path = os.path.join(HERE, "model.blend")
    with bpy.data.libraries.load(path, link=False) as (src, dst):
        dst.collections = ["model", "dead"]
    for c in dst.collections:
        bpy.context.scene.collection.children.link(c)
    return bpy.data.collections["model"], bpy.data.collections["dead"]


def path(name):
    return os.path.join(OUT, name + ".png")


def main():
    todo, sheet = wanted()
    model, dead = load_model()
    root = bpy.data.objects[NAME]
    droot = bpy.data.objects["dead_" + NAME]
    arcs = [o for o in model.objects if "knee_arcs" in o.name]
    info = sl.thing(NAME)
    foot = info["footprint"]
    sl.make_ground(footprint=foot)

    def show(live=True, night=False):
        model.hide_render = not live
        dead.hide_render = live
        for o in arcs:
            o.hide_render = not night

    def drop(objs):
        for o in objs:
            bpy.data.objects.remove(o)

    show()
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
        extra = [sl.add_soldier(0.0, 1.0), sl.scale_bar(0.0, -1.1, 1.0)]
        sl.cam_ortho_side([root] + extra)
        sl.render(path("05_size"))
        drop(extra)
    if "07_detail" in todo:
        head = bpy.data.objects["dome"].matrix_world.translation
        sl.cam_detail((head.x, head.y - 0.02, head.z - 0.10), 2.3)
        sl.render(path("07_detail"))
    if "09_with_regent_block" in todo:
        regent = sl.thing("regent")
        bpy.ops.mesh.primitive_cube_add(size=1.0)
        block = bpy.context.active_object
        block.name = "regent_block"
        block.scale = (regent["width_m"], regent["depth_m"], regent["height_m"])
        block.location = (0.0, 4.0, regent["height_m"] / 2.0)
        block.data.materials.append(sl.new_material("plain_grey", "#8a8a8a", roughness=0.8)[0])
        extra = [block, sl.add_soldier(0.0, 1.6)]
        sl.cam_ortho_side([root] + extra)
        sl.render(path("09_with_regent_block"))
        drop(extra)
    if "08_dead" in todo:
        show(live=False)
        sl.cam_threequarter([droot], elevation_deg=38.0, margin=1.15)
        sl.render(path("08_dead"))
        show()
    if "06_night" in todo:
        show(night=True)
        sl.night_light()
        sl.cam_threequarter([root])
        sl.render(path("06_night"))
    if sheet:
        sl.contact_sheet([path(p) for p in PICTURES], os.path.join(OUT, "contact.png"))


main()
