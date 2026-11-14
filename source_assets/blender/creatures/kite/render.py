"""Opens model.blend and writes the Kite's check pictures into renders/.

  flatpak run org.blender.Blender -b --python "<full path>/render.py"

To draw only some pictures, name them after `--`, for example
  ... render.py -- 03 07
The contact sheet is always rebuilt from the pictures that exist.
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

NAME = "kite"
OUT = os.path.join(HERE, "renders")
PICTURES = ["01_front", "02_side", "03_threequarter", "04_aerial", "05_size", "06_night",
            "07_detail", "08_dead", "09_top", "10_close"]


def wanted():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if not args:
        return PICTURES
    return [p for p in PICTURES if any(p.startswith(a) for a in args)]


def load_model():
    sl.reset_scene()
    with bpy.data.libraries.load(os.path.join(HERE, "model.blend"), link=False) as (src, dst):
        dst.collections = ["model", "dead"]
    for c in dst.collections:
        bpy.context.scene.collection.children.link(c)
    return bpy.data.collections["model"], bpy.data.collections["dead"]


def path(name):
    return os.path.join(OUT, name + ".png")


def dashed_line(x, y, top, dash=0.2, gap=0.15, width=0.03):
    """Thin dashed line straight down from `top` to the ground."""
    import bmesh
    from mathutils import Matrix
    bm = bmesh.new()
    z = 0.0
    while z < top - 1e-6:
        z1 = min(z + dash, top)
        bmesh.ops.create_cube(bm, size=1.0, matrix=Matrix.Translation((x, y, (z + z1) / 2))
                              @ Matrix.Diagonal((width, width, z1 - z, 1.0)))
        z += dash + gap
    me = bpy.data.meshes.new("height_line_mesh")
    bm.to_mesh(me)
    bm.free()
    me.materials.append(sl.new_material("height_line_black", "#0a0a0a", roughness=0.8)[0])
    ob = bpy.data.objects.new("height_line", me)
    bpy.context.scene.collection.objects.link(ob)
    ob.visible_shadow = False
    return ob


def cam_top(centre, width_m):
    """Straight down, no perspective. The nose (-Y) points up the picture."""
    cam = bpy.data.cameras.new("cam_top")
    cam.type = "ORTHO"
    cam.ortho_scale = width_m
    cam.clip_start, cam.clip_end = 0.05, 200.0
    ob = bpy.data.objects.new("cam_top", cam)
    bpy.context.scene.collection.objects.link(ob)
    ob.location = (centre[0], centre[1], centre[2] + 20.0)
    ob.rotation_euler = (0.0, 0.0, math.pi)
    bpy.context.scene.camera = ob
    return ob


def main():
    todo = wanted()
    model, dead = load_model()
    root = bpy.data.objects[NAME]
    droot = bpy.data.objects["dead_" + NAME]
    info = sl.thing(NAME)
    foot = info["footprint"]
    sl.make_ground(footprint=foot)
    body = [o for o in root.children_recursive if "thread" not in o.name]
    lo, hi = sl.bounds(body)
    cage = bpy.data.objects["cage"]

    def show(live=True):
        model.hide_render = not live
        dead.hide_render = live

    def drop(objs):
        for o in objs:
            bpy.data.objects.remove(o)

    show()
    sl.day_light()
    soldier = sl.add_soldier(0.0, 0.0)
    if "01_front" in todo:
        sl.cam_front([root, soldier])
        sl.render(path("01_front"))
    if "02_side" in todo:
        sl.cam_side([root, soldier])
        sl.render(path("02_side"))
    if "03_threequarter" in todo:
        sl.cam_threequarter([root, soldier])
        sl.render(path("03_threequarter"))
    if "04_aerial" in todo:
        extra = [sl.footprint_outline(*foot)]
        # Aim between the Kite and its shadow so both are in the picture.
        sun = sl.sun_direction()
        mid = (lo + hi) / 2.0
        shadow = mid - sun * (mid.z / sun.z)
        aim = (mid + shadow) / 2.0
        cam = sl.cam_aerial(None)
        bpy.context.view_layer.update()
        back = cam.matrix_world.to_3x3() @ Vector((0.0, 0.0, 1.0))
        cam.location = aim + back * 45.0
        sl.render(path("04_aerial"))
        drop(extra)
    if "05_size" in todo:
        soldier.location = (0.0, 1.3, 0.0)
        cw = cage.matrix_world.translation
        bpy.context.view_layer.update()
        low = min((o.matrix_world @ v.co).z for o in body if o.type == "MESH"
                  for v in o.data.vertices)          # lowest point of the body, not the threads
        extra = [sl.scale_bar(0.0, -1.4, 1.0), dashed_line(cw.x, cw.y, low)]
        sl.cam_ortho_side([root, soldier] + extra)
        sl.render(path("05_size"))
        drop(extra)
        soldier.location = (0.0, 0.0, 0.0)
    if "09_top" in todo:
        soldier.location = (-0.40, 0.30, 0.0)    # half under the right wing: seen through the skin
        cam = cam_top(((lo.x + hi.x) / 2, 0.3, hi.z), 4.2)
        sl.render(path("09_top"))
        bpy.data.objects.remove(cam)
        soldier.location = (0.0, 0.0, 0.0)
    if "10_close" in todo:
        sl.cam_threequarter([root], elevation_deg=32.0, margin=1.08)
        sl.render(path("10_close"))
    if "07_detail" in todo:
        cw = cage.matrix_world @ Vector((0.0, 0.0, -0.09))
        sl.cam_detail((cw.x, cw.y, cw.z), 1.25, elevation_deg=-52.0)
        sl.render(path("07_detail"))
    if "08_dead" in todo:
        show(live=False)
        soldier.hide_render = True
        sl.cam_threequarter([droot], elevation_deg=38.0, margin=1.12)
        sl.render(path("08_dead"))
        soldier.hide_render = False
        show()
    if "06_night" in todo:
        sl.night_light()
        soldier.hide_render = True
        sl.cam_threequarter([root], elevation_deg=-24.0, margin=1.1)
        sl.render(path("06_night"))
    sl.contact_sheet([path(p) for p in PICTURES], os.path.join(OUT, "contact.png"), columns=4,
                     cell_width=560)


main()
