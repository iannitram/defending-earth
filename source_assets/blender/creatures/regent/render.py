"""Opens model.blend and writes the Regent's check pictures into renders/.

  flatpak run org.blender.Blender -b --python "<full path>/render.py"

After `--` you may name pictures to draw only those (01 02 ... 08, contact)
and `--samples N` for a quick look. With nothing named, all are drawn.
"""
import sys
import os

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "scale")))

import bpy
from mathutils import Vector

import scale_lib as sl

OUT = os.path.join(HERE, "renders")
NAMES = ["01_front", "02_side", "03_threequarter", "04_aerial", "05_size", "06_night", "07_detail", "08_dead"]


def main():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    samples = None
    if "--samples" in args:
        i = args.index("--samples")
        samples = int(args[i + 1])
        del args[i:i + 2]
    want = [n for n in NAMES if not args or n[:2] in args]
    do_contact = not args or "contact" in args

    bpy.ops.wm.open_mainfile(filepath=os.path.join(HERE, "model.blend"))
    sc = bpy.context.scene
    if samples:
        sc.cycles.samples = samples
    sheet = sl.thing("regent")
    foot = tuple(sheet["footprint"])
    alive = bpy.data.objects["regent"]
    dead = bpy.data.objects["regent_dead"]
    c_model = bpy.data.collections["model"]
    c_dead = bpy.data.collections["dead"]
    night_only = [o for o in bpy.data.objects if o.get("night_only")]

    def show(which, night=False):
        c_model.hide_render = which != "alive"
        c_dead.hide_render = which != "dead"
        for o in night_only:
            o.hide_render = not night

    def path(n):
        return os.path.join(OUT, n + ".png")

    sl.make_ground(footprint=foot)
    sl.day_light()
    show("alive")

    if "01_front" in want:
        sl.cam_front([alive])
        sl.render(path("01_front"))
    if "02_side" in want:
        sl.cam_side([alive])
        sl.render(path("02_side"))
    if "03_threequarter" in want:
        sl.cam_threequarter([alive])
        sl.render(path("03_threequarter"))
    if "04_aerial" in want:
        before = set(bpy.data.objects)
        sl.footprint_outline(*foot)
        added = [o for o in bpy.data.objects if o not in before]
        sl.cam_aerial([alive])
        sl.render(path("04_aerial"))
        for o in added:
            bpy.data.objects.remove(o)
    if "05_size" in want:
        soldier = sl.add_soldier(0.0, -1.9)
        before = set(bpy.data.objects)
        sl.scale_bar(0.0, 1.8, 1.0, "Y")
        added = [o for o in bpy.data.objects if o not in before]
        sl.cam_ortho_side([alive, soldier] + added)
        sl.render(path("05_size"))
        for o in [soldier] + added:
            bpy.data.objects.remove(o)
    if "07_detail" in want:
        # the dome, head plate and ring, from the front three-quarter, close
        sl.cam_detail((0.0, -0.10, 3.14), 4.0, azimuth_deg=30.0, elevation_deg=3.0)
        sl.render(path("07_detail"))
    if "08_dead" in want:
        show("dead")
        sl.cam_threequarter([dead], azimuth_deg=-48.0, elevation_deg=16.0)
        sl.render(path("08_dead"))
        show("alive")
    if "06_night" in want:
        sl.night_light()
        show("alive", night=True)
        sl.cam_threequarter([alive])
        sl.render(path("06_night"))

    if do_contact:
        sl.contact_sheet([path(n) for n in NAMES], os.path.join(OUT, "contact.png"))
    print("REGENT pictures done:", want)


if __name__ == "__main__":
    main()
