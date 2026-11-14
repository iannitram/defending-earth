"""Opens model.blend and writes the Gorger's check pictures into renders/.

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

NAME = "gorger"
OUT = os.path.join(HERE, "renders")
PICTURES = ["01_front", "02_side", "03_threequarter", "04_aerial", "05_size", "06_night",
            "07_detail", "08_dead", "09_head_out", "10_sign_aerial", "11_sign_close",
            "12_sign_compare"]
STATES = ("model", "head_out", "sign", "dead")


def wanted():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if not args:
        return PICTURES
    return [p for p in PICTURES if any(p.startswith(a) for a in args)]


def path(name):
    return os.path.join(OUT, name + ".png")


def main():
    todo = wanted()
    sl.reset_scene()
    with bpy.data.libraries.load(os.path.join(HERE, "model.blend"), link=False) as (src, dst):
        dst.collections = list(STATES)
    colls = {}
    for c in dst.collections:
        bpy.context.scene.collection.children.link(c)
        colls[c.name] = c
    info = sl.thing(NAME)
    root = bpy.data.objects[NAME]
    hroot = bpy.data.objects["head_out_" + NAME]
    droot = bpy.data.objects["dead_" + NAME]
    sroot = bpy.data.objects["sign_" + NAME]
    arcs = [o for o in bpy.data.objects if o.get("night_only")]

    def show(state, night=False):
        for name, c in colls.items():
            c.hide_render = name != state
        for o in arcs:
            o.hide_render = not night

    def solid(r):
        return [o for o in r.children_recursive if not o.get("night_only")]

    def ground(footprint):
        old = bpy.data.objects.get("ground")
        if old is not None:
            me = old.data
            mats = list(me.materials)
            bpy.data.objects.remove(old)
            bpy.data.meshes.remove(me)
            for mt in mats:
                if mt.users == 0:
                    bpy.data.materials.remove(mt)
        sl.make_ground(footprint=footprint)

    def drop(objs):
        for o in objs:
            bpy.data.objects.remove(o)

    def red_circle(radius):
        import bmesh
        bm = bmesh.new()
        n, w, z = 96, 0.03, 0.06
        vs = []
        for i in range(n):
            a = 2 * math.pi * i / n
            vs.append((bm.verts.new((math.cos(a) * (radius - w / 2), math.sin(a) * (radius - w / 2), z)),
                       bm.verts.new((math.cos(a) * (radius + w / 2), math.sin(a) * (radius + w / 2), z))))
        for i in range(n):
            a, b = vs[i], vs[(i + 1) % n]
            bm.faces.new((a[0], a[1], b[1], b[0]))
        me = bpy.data.meshes.new("red_circle_mesh")
        bm.to_mesh(me)
        bm.free()
        mat, _ = sl.new_material("marker_red", "#d01010", 1.0, 0.0, "#ff1010", 1.5)
        me.materials.append(mat)
        ob = bpy.data.objects.new("red_circle", me)
        bpy.context.scene.collection.objects.link(ob)
        ob.visible_shadow = False
        return ob

    sl.day_light()

    # ---- state 1: on the surface ------------------------------------------
    if any(p in todo for p in ("01_front", "02_side", "03_threequarter", "04_aerial", "05_size", "08_dead")):
        ground(info["footprint"])
        show("model")
        parts = solid(root)
        if "01_front" in todo:
            sl.cam_front(parts)
            sl.render(path("01_front"))
        if "02_side" in todo:
            sl.cam_side(parts)
            sl.render(path("02_side"))
        if "03_threequarter" in todo:
            sl.cam_threequarter(parts, elevation_deg=10.0)
            sl.render(path("03_threequarter"))
        if "04_aerial" in todo:
            extra = [sl.footprint_outline(*info["footprint"])]
            sl.cam_aerial(parts)
            sl.render(path("04_aerial"))
            drop(extra)
        if "05_size" in todo:
            lo, hi = sl.bounds(parts)
            extra = [sl.add_soldier(0.0, lo.y - 1.0), sl.scale_bar(0.0, lo.y - 2.2, 1.0)]
            sl.cam_ortho_side(parts + extra)
            sl.render(path("05_size"))
            drop(extra)
        if "08_dead" in todo:
            show("dead")
            sl.cam_threequarter(solid(droot), elevation_deg=18.0)
            sl.render(path("08_dead"))

    # ---- state 2: head out -------------------------------------------------
    if any(p in todo for p in ("07_detail", "09_head_out", "06_night")):
        ground(info["footprint_head_out"])
        show("head_out")
        parts = solid(hroot)
        hinge = bpy.data.objects["head_out_jaw_1"].matrix_world.translation
        if "07_detail" in todo:
            sl.cam_detail((0.0, -0.15, hinge.z - 0.10), 6.2, elevation_deg=30.0)
            sl.render(path("07_detail"))
        if "09_head_out" in todo:
            extra = [sl.add_soldier(-1.5, -0.2, facing_deg=90.0)]
            sl.cam_threequarter(parts + extra, elevation_deg=6.0)
            sl.render(path("09_head_out"))
            drop(extra)

    # ---- state 3: the buried sign -----------------------------------------
    if any(p in todo for p in ("10_sign_aerial", "11_sign_close", "12_sign_compare")):
        ground(info["footprint_head_out"])
        show("sign")
        sq = sl.square()
        soldier = sl.add_soldier(-3 * sq, 0.0, facing_deg=90.0)
        if "10_sign_aerial" in todo:
            sl.cam_aerial([sroot, soldier])
            sl.render(path("10_sign_aerial"))
        if "12_sign_compare" in todo:
            extra = [red_circle(info["sign_patch_m"] / 2.0 + 0.2)]
            sl.cam_aerial([sroot, soldier])
            sl.render(path("12_sign_compare"))
            drop(extra)
        if "11_sign_close" in todo:
            eye = 0.93 * sl.load()["anchor"]["soldier_height_m"]
            sl.cam_detail((0.0, 0.0, 0.0), math.hypot(6.0, eye), lens=50.0,
                          elevation_deg=math.degrees(math.atan2(eye, 6.0)))
            sl.render(path("11_sign_close"))
        drop([soldier])

    # ---- night: state 2 ----------------------------------------------------
    if "06_night" in todo:
        ground(info["footprint_head_out"])
        show("head_out", night=True)
        sl.night_light()
        sl.cam_threequarter(solid(hroot), elevation_deg=6.0)
        sl.render(path("06_night"))

    sl.contact_sheet([path(p) for p in PICTURES], os.path.join(OUT, "contact.png"))


main()
