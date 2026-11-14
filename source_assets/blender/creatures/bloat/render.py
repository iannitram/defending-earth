"""Opens model.blend and writes the Bloat's check pictures into renders/.

  flatpak run org.blender.Blender -b --python "<full path>/render.py"

To draw only some pictures, name them after `--`, for example
  ... render.py -- 03 07
`contact` rebuilds only the contact sheet.
"""
import os
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "scale")))

import bpy

import scale_lib as sl

NAME = "bloat"
OUT = os.path.join(HERE, "renders")
PICTURES = ["01_front", "02_side", "03_threequarter", "04_aerial", "05_size", "06_night",
            "07_detail", "08_dead"]


def wanted():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if not args:
        return PICTURES
    return [p for p in PICTURES if any(p.startswith(a) for a in args)]


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


def ground_mark():
    """A speck on the ground under the model that does not render. Listing
    it makes the cameras keep the ground, and so the shadow, in frame."""
    me = bpy.data.meshes.new("ground_mark")
    me.from_pydata([(-0.01, 0, 0), (0.01, 0, 0), (0, 0.01, 0)], [], [(0, 1, 2)])
    ob = bpy.data.objects.new("ground_mark", me)
    bpy.context.scene.collection.objects.link(ob)
    ob.hide_render = True
    return ob


def main():
    todo = wanted()
    model, dead = load_model()
    root = bpy.data.objects[NAME]
    droot = bpy.data.objects["dead_" + NAME]
    arcs = bpy.data.objects["rib_arcs"]
    info = sl.thing(NAME)
    foot = info["footprint"]
    sl.make_ground(footprint=foot)
    mark = ground_mark()
    sc = bpy.context.scene
    sc.cycles.volume_bounces = 2
    sc.cycles.transparent_max_bounces = 16

    def show(live=True, night=False):
        model.hide_render = not live
        dead.hide_render = live
        arcs.hide_render = not night

    def drop(objs):
        for o in objs:
            bpy.data.objects.remove(o)

    show()
    sl.day_light()
    if "01_front" in todo:
        sl.cam_front([root, mark], margin=1.12)
        sl.render(path("01_front"))
    if "02_side" in todo:
        sl.cam_side([root, mark], margin=1.12)
        sl.render(path("02_side"))
    if "03_threequarter" in todo:
        sl.cam_threequarter([root, mark], margin=1.12)
        sl.render(path("03_threequarter"))
    if "04_aerial" in todo:
        extra = [sl.footprint_outline(*foot)]
        sl.cam_aerial([mark])
        sl.render(path("04_aerial"))
        drop(extra)
    if "05_size" in todo:
        extra = [sl.add_soldier(0.0, 1.7), sl.scale_bar(0.0, -1.9, 1.0)]
        sl.cam_ortho_side([root, mark] + extra)
        sl.render(path("05_size"))
        drop(extra)
    if "07_detail" in todo:
        # the lower half, seen from the side away from the sun, so the
        # daylight comes through the skin toward the camera
        under = info["underside_height_m"]
        sl.cam_detail((0.0, 0.0, under + 0.02), 6.4, azimuth_deg=140.0, elevation_deg=-4.0)
        sl.render(path("07_detail"))
    if "08_dead" in todo:
        show(live=False)
        sl.cam_threequarter([droot], elevation_deg=34.0, margin=1.1)
        sl.render(path("08_dead"))
        show()
    if "06_night" in todo:
        show(night=True)
        sl.night_light()
        sl.cam_threequarter([root, mark], margin=1.12)
        sl.render(path("06_night"))
    sl.contact_sheet([path(p) for p in PICTURES], os.path.join(OUT, "contact.png"))


main()
