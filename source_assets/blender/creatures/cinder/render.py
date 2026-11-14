"""Opens model.blend and writes the Cinder's check pictures into renders/.

  flatpak run org.blender.Blender -b --python "<full path>/render.py"

To draw only some pictures, name them after `--`, for example
  ... render.py -- 03 07
`contact` rebuilds only the contact sheet.

The three sizes stand in a row 1.2 m apart in every picture. For the two
side views the row is turned to run front to back, so no size hides
another. Each Cinder keeps its own facing.
"""
import os
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "scale")))

import bpy

import scale_lib as sl

NAME = "cinder"
OUT = os.path.join(HERE, "renders")
KEYS = ("small", "middle", "large")
ROW_STEP_M = 1.2
PICTURES = ["01_front", "02_side", "03_threequarter", "04_aerial", "05_size", "06_night",
            "07_detail", "08_dead", "09_not_a_registrar_cube"]


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


def main():
    todo = wanted()
    model, dead = load_model()
    tops = {k: bpy.data.objects["%s_%s" % (NAME, k)] for k in KEYS}
    bodies = [bpy.data.objects["%s_%s_body" % (NAME, k)] for k in KEYS]
    droot = bpy.data.objects["dead_" + NAME]
    foot = sl.thing(NAME)["footprint"]
    sq = sl.square()
    sl.make_ground(footprint=foot)

    def row(along):
        for i, k in enumerate(KEYS):
            step = (i - 1) * ROW_STEP_M
            tops[k].location = (step, 0.0, 0.0) if along == "X" else (0.0, step, 0.0)

    def show(live=True, only=None):
        model.hide_render = not live
        dead.hide_render = live
        for k in KEYS:
            hide = only is not None and k != only
            for o in [tops[k]] + list(tops[k].children_recursive):
                o.hide_render = hide

    def drop(objs):
        for o in objs:
            bpy.data.objects.remove(o)

    show()
    row("X")
    sl.day_light()
    if "01_front" in todo:
        sl.cam_front(bodies, margin=1.3)
        sl.render(path("01_front"))
    if "03_threequarter" in todo:
        sl.cam_threequarter(bodies, margin=1.25)
        sl.render(path("03_threequarter"))
    if "04_aerial" in todo:
        # One white square for each battlefield square the row touches.
        extra = []
        for i in (-1, 0, 1):
            o = sl.footprint_outline(*foot)
            o.location.x = i * sq * foot[0]
            extra.append(o)
        sl.cam_aerial(bodies)
        sl.render(path("04_aerial"))
        drop(extra)
    if "07_detail" in todo:
        b = bodies[0].matrix_world.translation
        sl.cam_detail((b.x, b.y, b.z), 1.75, elevation_deg=16.0)
        sl.render(path("07_detail"))
    if "09_not_a_registrar_cube" in todo:
        show(only="middle")
        side = sl.thing("taker")["core_cube_m"]
        bpy.ops.mesh.primitive_cube_add(size=side)
        cube = bpy.context.active_object
        cube.name = "registrar_cube_for_comparison"
        cube.location = (0.95, 0.0, side / 2.0)
        cube.data.materials.append(
            sl.new_material("registrar_hardware", "#0b0b0c", roughness=0.35, metallic=1.0)[0])
        sl.cam_threequarter([bodies[1], cube], margin=1.5, elevation_deg=14.0)
        sl.render(path("09_not_a_registrar_cube"))
        drop([cube])
        show()
    if "02_side" in todo or "05_size" in todo:
        row("Y")
        if "02_side" in todo:
            sl.cam_side(bodies, margin=1.3)
            sl.render(path("02_side"))
        if "05_size" in todo:
            extra = [sl.add_soldier(0.0, -ROW_STEP_M - 1.0),
                     sl.scale_bar(-0.9, ROW_STEP_M + 1.5, 1.0)]
            sl.cam_ortho_side(bodies + extra)
            sl.render(path("05_size"))
            drop(extra)
        row("X")
    if "08_dead" in todo:
        show(live=False)
        sl.cam_threequarter([droot], elevation_deg=30.0, margin=1.1)
        sl.render(path("08_dead"))
        show()
    if "06_night" in todo:
        sl.night_light()
        sl.cam_threequarter(bodies, margin=1.45, elevation_deg=14.0)
        sl.render(path("06_night"))
    sl.contact_sheet([path(p) for p in PICTURES], os.path.join(OUT, "contact.png"))


main()
