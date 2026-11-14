"""size_chart.py: the size chart and the pictures that prove the helper.

Run from the project folder:

    flatpak run org.blender.Blender -b --python source_assets/scale/size_chart.py
    flatpak run org.blender.Blender -b --python source_assets/scale/size_chart.py -- chart

With nothing after "--" it writes all four pictures into renders/:
    size_chart.png       the soldier next to every creature, side view
    soldier_standin.png  the soldier stand-in, three-quarter view
    ground_day.png       the game's view of the ground, day light
    ground_night.png     the same, night light
Name one or more of: chart, soldier, ground. "--out <folder>" writes
somewhere else (used for test runs).

A creature is drawn as a plain grey block at its size from the scale sheet
until source_assets/blender/creatures/<name>/model.blend exists. Then the
real model is used.
"""

import math
import os
import sys

import bpy
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.dont_write_bytecode = True      # leave no cache folder in the project
sys.path.insert(0, HERE)
import scale_lib as sl  # noqa: E402

CREATURES_DIR = os.path.normpath(os.path.join(HERE, "..", "blender", "creatures"))

# Chart layout. These are drawing choices, not sizes of things in the game.
CHART_SIZE = (2400, 540)
SLOT_MIN_M = 3.0        # every name gets at least this much room
SLOT_PAD_M = 1.0        # clear space added to a wide creature
BLOCK_HEX = "#6a6a6a"
# Two sizes the sheet does not give. They only shape the grey blocks.
KITE_BLOCK_THICKNESS_M = 0.3      # the sheet gives the Kite no thickness
# The builder has no height in the sheet; its block uses closed_diameter_m.

# Order on the chart, left to right, and the name printed underneath.
# Names follow docs/NAMES.md; "taker" and "builder" are not proper names.
ORDER = [
    ("soldier", "Soldier"), ("tender", "Tender"), ("regent", "Regent"),
    ("lash", "Lash"), ("gorger", "Gorger"), ("kite", "Kite"), ("bloat", "Bloat"),
    ("maul", "Maul"), ("cinder", "Cinder"), ("taker", "taker"), ("builder", "builder"),
]


def blocks_for(name):
    """Grey blocks for one creature: list of (width X, depth Y, height Z,
    height of the underside, offset along the row). Also the note line."""
    t = sl.thing(name)
    if name in ("regent", "tender", "lash", "maul"):
        return [(t["width_m"], t["depth_m"], t["height_m"], 0.0, 0.0)], \
            "%.1f m tall" % t["height_m"]
    if name == "gorger":
        d = t["body_diameter_m"]
        return [(d, t["length_m"], d, 0.0, 0.0)], "%.0f m long, lying down" % t["length_m"]
    if name == "kite":
        return [(t["span_m"], t["length_m"], KITE_BLOCK_THICKNESS_M, t["flight_height_m"], 0.0)], \
            "flies at %.0f m" % t["flight_height_m"]
    if name == "bloat":
        d = t["diameter_m"]
        return [(d, d, d, t["underside_height_m"], 0.0)], \
            "underside at %.1f m" % t["underside_height_m"]
    if name == "cinder":
        out, gap = [], 0.4
        sizes = t["sizes_m"]
        total = sum(sizes) + gap * (len(sizes) - 1)
        u = -total / 2.0
        for s in sizes:
            out.append((s, s, s, 0.0, u + s / 2.0))
            u += s + gap
        return out, " / ".join("%.1f" % s for s in sizes) + " m"
    if name == "taker":
        s = t["closed_size_m"]
        return [(s, s, s, t["hover_height_m"], 0.0)], \
            "hovers at %.1f m" % t["hover_height_m"]
    if name == "builder":
        s = t["spread_m"]
        return [(s, s, t["closed_diameter_m"], 0.0, 0.0)], "%.1f m spread" % s
    raise KeyError(name)


def hover_for(name):
    """Height of the underside for things that hang in the air, else 0."""
    t = sl.thing(name)
    return {"kite": t.get("flight_height_m", 0.0), "bloat": t.get("underside_height_m", 0.0),
            "taker": t.get("hover_height_m", 0.0)}.get(name, 0.0)


def add_block(name, w, d, h, z0, y):
    me = bpy.data.meshes.new("block_%s_mesh" % name)
    x0, x1, y0, y1 = -w / 2, w / 2, y - d / 2, y + d / 2
    v = [(x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0),
         (x0, y0, z0 + h), (x1, y0, z0 + h), (x1, y1, z0 + h), (x0, y1, z0 + h)]
    f = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]
    me.from_pydata(v, [], f)
    mat = bpy.data.materials.get("block_grey")
    if mat is None:
        mat, _ = sl.new_material("block_grey", BLOCK_HEX, roughness=0.8)
    me.materials.append(mat)
    ob = bpy.data.objects.new("block_%s" % name, me)
    bpy.context.scene.collection.objects.link(ob)
    ob.visible_shadow = False     # a block must not shade its neighbour
    return ob


def add_squares_mark(name, y, squares):
    """White bracket under a creature: the squares it takes up along the row."""
    sq = sl.square()
    half = squares * sq / 2.0
    x, z1, z0 = -8.04, -0.10, -0.15
    verts, faces = [], []

    def quad(ya, yb, za, zb):
        n = len(verts)
        verts.extend([(x, ya, za), (x, yb, za), (x, yb, zb), (x, ya, zb)])
        faces.append((n, n + 3, n + 2, n + 1))
    quad(y - half, y + half, z0, z1)
    for i in range(squares + 1):
        yy = y - half + i * sq
        quad(yy - 0.03, yy + 0.03, z0 - 0.12, z1)
    me = bpy.data.meshes.new("squares_%s_mesh" % name)
    me.from_pydata(verts, [], faces)
    mat = bpy.data.materials.get("label_#e9e4da")
    if mat is not None:
        me.materials.append(mat)
    ob = bpy.data.objects.new("squares_%s" % name, me)
    bpy.context.scene.collection.objects.link(ob)
    ob.visible_shadow = False
    return ob


def add_model(name, path, y):
    """Bring in a finished model.blend and stand it at y on the row.

    If the file has a collection called "model", only that is used.
    Cameras, lights and the shared stand-ins are left out.
    """
    with bpy.data.libraries.load(path, link=False) as (src, dst):
        if "model" in src.collections:
            dst.collections = ["model"]
        else:
            dst.objects = list(src.objects)
    objs = list(dst.collections[0].all_objects) if dst.collections else list(dst.objects)
    skip = ("ground", "soldier_standin", "scale_bar", "footprint_outline", "moon", "cam_")
    keep = []
    for o in objs:
        if o is None:
            continue
        if o.type in ("CAMERA", "LIGHT") or o.name.startswith(skip):
            bpy.data.objects.remove(o)
            continue
        bpy.context.scene.collection.objects.link(o)
        keep.append(o)
    root = bpy.data.objects.new("chart_%s" % name, None)
    bpy.context.scene.collection.objects.link(root)
    for o in keep:
        if o.parent is None:
            o.parent = root
    lo, hi = sl.bounds([root])
    lift = hover_for(name) if lo.z < 0.05 else 0.0   # a flyer built on the ground is raised
    root.location = (-(lo.x + hi.x) / 2.0, y - (lo.y + hi.y) / 2.0, lift)
    return root


def add_text(text, y, z, size, x=-8.06, align="CENTER", hex_colour="#e9e4da", name="label"):
    """Upright lettering that faces the side camera (which sits at -X)."""
    cu = bpy.data.curves.new(name, "FONT")
    cu.body = text
    cu.size = size
    cu.align_x = align
    ob = bpy.data.objects.new(name, cu)
    bpy.context.scene.collection.objects.link(ob)
    # Letters run toward -Y (right in the picture), stand up along +Z.
    m = Matrix(((0, 0, -1, x), (-1, 0, 0, y), (0, 1, 0, z), (0, 0, 0, 1)))
    ob.matrix_world = m
    mat = bpy.data.materials.get("label_" + hex_colour)
    if mat is None:
        mat, bsdf = sl.new_material("label_" + hex_colour, hex_colour, roughness=1.0,
                                    emission_hex=hex_colour, emission_strength=0.6)
    cu.materials.append(mat)
    ob.visible_shadow = False
    return ob


def add_panel(name, x, y0, y1, z0, z1, hex_colour, metre_lines=False):
    """Upright flat panel facing the camera: the backdrop and the soil band."""
    me = bpy.data.meshes.new(name + "_mesh")
    me.from_pydata([(x, y0, z0), (x, y1, z0), (x, y1, z1), (x, y0, z1)], [], [(0, 3, 2, 1)])
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    mat, bsdf = sl.new_material(name + "_mat", hex_colour, roughness=1.0)
    bsdf.inputs["Specular IOR Level"].default_value = 0.0
    if metre_lines:
        nt = mat.node_tree
        geo = nt.nodes.new("ShaderNodeNewGeometry")
        sep = nt.nodes.new("ShaderNodeSeparateXYZ")
        nt.links.new(geo.outputs["Position"], sep.inputs["Vector"])
        z = sep.outputs["Z"]
        dist = sl._math(nt, "ABSOLUTE", sl._math(nt, "SUBTRACT", z, sl._math(nt, "ROUND", z)))
        line = sl._math(nt, "LESS_THAN", dist, 0.02)
        mix = nt.nodes.new("ShaderNodeMix")
        mix.data_type = "RGBA"
        mix.inputs["A"].default_value = sl.rgb(hex_colour)
        mix.inputs["B"].default_value = tuple(c * 0.62 for c in sl.rgb(hex_colour)[:3]) + (1.0,)
        nt.links.new(line, mix.inputs["Factor"])
        nt.links.new(mix.outputs["Result"], bsdf.inputs["Base Color"])
    me.materials.append(mat)
    ob.visible_shadow = False
    return ob


def make_chart(out_path):
    sc = sl.reset_scene()
    sc.render.resolution_x, sc.render.resolution_y = CHART_SIZE
    sl.day_light()

    # Slots along the row. u grows to the right of the picture, which is -Y.
    slots = []
    u = 0.0
    for name, label in ORDER:
        if name == "soldier":
            span, note = sl.thing("soldier")["depth_m"], "%.1f m tall" % sl.thing("soldier")["height_m"]
            blocks = None
        else:
            blocks, note = blocks_for(name)
            span = max(b[4] + b[1] / 2 for b in blocks) - min(b[4] - b[1] / 2 for b in blocks)
        width = max(SLOT_MIN_M, span + SLOT_PAD_M)
        slots.append((name, label, note, blocks, u + width / 2.0))
        u += width
    total = u
    shift = total / 2.0     # centre the row on the origin

    ground = sl.make_ground(size_m=math.ceil(total / 10.0) * 10 + 40)
    ground.visible_shadow = False   # lets the sun reach the name band below ground level
    used_models = []
    for name, label, note, blocks, centre in slots:
        y = -(centre - shift)
        path = os.path.join(CREATURES_DIR, name, "model.blend")
        if name == "soldier":
            sl.add_soldier(0.0, y)
        elif os.path.exists(path):
            add_model(name, path, y)
            used_models.append(name)
        else:
            for i, (w, d, h, z0, off) in enumerate(blocks):
                add_block("%s_%d" % (name, i), w, d, h, z0, y - off)
                if z0 > 0.0:    # a thin post shows how high a flyer hangs
                    add_block("%s_post" % name, 0.03, 0.03, z0, 0.0, y - off)
        add_text(label, y, -1.00, 0.56, name="name_" + name)
        add_text(note, y, -1.50, 0.30, hex_colour="#b9b2a4", name="note_" + name)
        add_squares_mark(name, y, sl.thing(name)["footprint"][1])

    left, right = shift + 1.0, -shift - 1.0          # Y at the picture's edges
    view_w = total + 2.0
    view_h = view_w * CHART_SIZE[1] / CHART_SIZE[0]
    z_low = -2.5
    add_panel("backdrop", 30.0, right - 5, left + 5, z_low - 2, z_low + view_h + 2,
              "#d6d2ca", metre_lines=True)
    add_panel("soil_band", -8.0, right - 5, left + 5, z_low - 2, 0.0, "#4b4338")

    # 1 m scale bar, turned to face the camera, bottom left.
    bar = sl.scale_bar(-8.08, left - 1.6, length_m=1.0, width_m=0.24)
    bar.rotation_euler = (0.0, math.radians(90), 0.0)
    bar.location.z = -2.12
    add_text("1 m", left - 2.3, -2.22, 0.30, align="LEFT", name="label_bar")
    add_text("Lines on the wall are 1 m apart. The white marks show squares: one square is %.0f m."
             % sl.square(), right + 0.6, -2.22, 0.26, align="RIGHT",
             hex_colour="#b9b2a4", name="label_key")

    sl._camera("cam_chart", Vector((0.0, 0.0, z_low + view_h / 2.0)),
               sl._direction(-90.0, 0.0), 80.0, ortho_scale=view_w)
    sl.render(out_path)
    print("size_chart: wrote %s (real models used: %s)"
          % (out_path, ", ".join(used_models) or "none"))


def make_soldier(out_path):
    sl.reset_scene()
    sl.make_ground()
    sl.day_light()
    s = sl.add_soldier(0.0, 0.0)
    sl.cam_threequarter([s], elevation_deg=9.0, margin=1.25)
    sl.render(out_path)
    m = sl.measure([s])
    print("size_chart: soldier stand-in measures", m)


def make_ground_pictures(day_path, night_path):
    sl.reset_scene()
    sl.make_ground()
    s = sl.add_soldier(0.0, 0.0)
    sl.cam_aerial([s])
    sl.day_light()
    sl.render(day_path)
    sl.night_light()
    sl.render(night_path)


def main():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    out_dir = os.path.join(HERE, "renders")
    if "--out" in args:
        i = args.index("--out")
        out_dir = args[i + 1]
        del args[i:i + 2]
    what = set(args) or {"chart", "soldier", "ground"}
    bad = what - {"chart", "soldier", "ground"}
    if bad:
        raise SystemExit("size_chart: unknown picture name: %s" % ", ".join(sorted(bad)))
    if "soldier" in what:
        make_soldier(os.path.join(out_dir, "soldier_standin.png"))
    if "ground" in what:
        make_ground_pictures(os.path.join(out_dir, "ground_day.png"),
                             os.path.join(out_dir, "ground_night.png"))
    if "chart" in what:
        make_chart(os.path.join(out_dir, "size_chart.png"))


if __name__ == "__main__":
    main()
