"""scale_lib: the shared Blender helper for every Defending Earth model job.

It reads the one sheet of sizes (scale_sheet.json, next to this file) and
gives every job the same ground, light, soldier stand-in, cameras, render
settings and measuring tools, so all check pictures match.

Use it inside Blender's own Python only (background mode):

    import sys
    sys.path.insert(0, "<project>/source_assets/scale")
    import scale_lib as sl

See README.md in this folder for a plain-words guide.
"""

import json
import math
import os
import warnings

import bpy
import bmesh
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
SHEET_PATH = os.path.join(HERE, "scale_sheet.json")

# ---------------------------------------------------------------------------
# Look settings shared by every check picture. These are not sizes of things
# in the game, so they live here and not in the sheet.
# ---------------------------------------------------------------------------
RENDER_SIZE = (1600, 900)
RENDER_SAMPLES = 64
SOIL_HEX = "#6b5f4e"
SUN_ELEVATION_DEG = 38.0
# The model faces -Y. Seen from the front, the sun stands to the left: it sits
# over the (-X, -Y) quarter. This angle is how far left of straight ahead.
SUN_LEFT_OF_FRONT_DEG = 40.0
DAY_SKY_STRENGTH = 0.05     # brings the physical sky down to a normal exposure
NIGHT_SKY_COLOUR = (0.020, 0.030, 0.060)
NIGHT_SKY_STRENGTH = 1.0
MOON_COLOUR = (0.62, 0.76, 1.0)
MOON_STRENGTH = 2.0
THREEQUARTER_AZIMUTH_DEG = 35.0   # camera swung this far to the right of front
GROUND_APRON_M = 4000.0          # plain soil runs this far so the horizon is real
GEOMETRY_TYPES = {"MESH", "CURVE", "SURFACE", "META", "FONT"}

_sheet_cache = None


# ---------------------------------------------------------------------------
# The sheet
# ---------------------------------------------------------------------------
def load():
    """Return the whole scale sheet as a dictionary."""
    global _sheet_cache
    if _sheet_cache is None:
        with open(SHEET_PATH, "r", encoding="utf-8") as f:
            _sheet_cache = json.load(f)
    return _sheet_cache


def square():
    """Width of one battlefield square in metres (2.0)."""
    return float(load()["anchor"]["square_m"])


class _Entry(dict):
    """A sheet entry that says clearly which key is missing."""

    def __init__(self, name, data):
        super().__init__(data)
        self._name = name

    def __missing__(self, key):
        raise KeyError(
            "scale sheet: '%s' has no size called '%s'. It has: %s. "
            "A size that is not in the sheet must be decided by the lead, "
            "not typed into a script." % (self._name, key, ", ".join(sorted(self))))


def thing(name):
    """Return one entry of the sheet's "things", for example thing("maul").

    Asking for a name that is not in the sheet, or later for a key the entry
    does not have, raises a KeyError that says what is missing.
    """
    things = load()["things"]
    if name not in things:
        raise KeyError(
            "scale sheet: there is no thing called '%s'. The sheet has: %s."
            % (name, ", ".join(sorted(things))))
    return _Entry(name, things[name])


def derived(key):
    """Return one value from the sheet's "derived" list (doors, cover...)."""
    d = load()["derived"]
    if key not in d:
        raise KeyError("scale sheet: no derived size called '%s'. It has: %s."
                       % (key, ", ".join(sorted(d))))
    return d[key]


# ---------------------------------------------------------------------------
# Small tools
# ---------------------------------------------------------------------------
def rgb(hex_colour, alpha=1.0):
    """Turn "#rrggbb" into the linear colour Blender's nodes expect."""
    h = hex_colour.lstrip("#")
    out = []
    for i in (0, 2, 4):
        c = int(h[i:i + 2], 16) / 255.0
        out.append(c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4)
    return (out[0], out[1], out[2], alpha)


def _node_tree_of(datablock):
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        try:
            datablock.use_nodes = True
        except Exception:
            pass
    return datablock.node_tree


def new_material(name, hex_colour, roughness=0.7, metallic=0.0,
                 emission_hex=None, emission_strength=0.0):
    """A plain Principled BSDF material. Returns (material, bsdf node)."""
    mat = bpy.data.materials.new(name)
    nt = _node_tree_of(mat)
    bsdf = next(n for n in nt.nodes if n.bl_idname == "ShaderNodeBsdfPrincipled")
    bsdf.inputs["Base Color"].default_value = rgb(hex_colour)
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    if emission_hex is not None:
        bsdf.inputs["Emission Color"].default_value = rgb(emission_hex)
        bsdf.inputs["Emission Strength"].default_value = emission_strength
    return mat, bsdf


def _link(obj):
    bpy.context.scene.collection.objects.link(obj)
    return obj


def _math(nt, op, a, b=None, c=None):
    n = nt.nodes.new("ShaderNodeMath")
    n.operation = op
    for i, v in enumerate((a, b, c)):
        if v is None:
            continue
        if isinstance(v, (int, float)):
            n.inputs[i].default_value = v
        else:
            nt.links.new(v, n.inputs[i])
    return n.outputs[0]


def _expand(objects):
    """The given objects plus all their children, geometry only."""
    if isinstance(objects, bpy.types.Object):
        objects = [objects]
    seen, out = set(), []
    for o in objects:
        for x in [o] + list(o.children_recursive):
            if x.name not in seen and x.type in GEOMETRY_TYPES:
                seen.add(x.name)
                out.append(x)
    return out


def bounds(objects):
    """World-space box round the objects (and their children).

    Returns (min corner, max corner) as Vectors, after modifiers.
    """
    bpy.context.view_layer.update()
    dg = bpy.context.evaluated_depsgraph_get()
    lo = Vector((1e9, 1e9, 1e9))
    hi = Vector((-1e9, -1e9, -1e9))
    found = False
    for o in _expand(objects):
        ev = o.evaluated_get(dg)
        for c in ev.bound_box:
            p = ev.matrix_world @ Vector(c)
            for i in range(3):
                lo[i] = min(lo[i], p[i])
                hi[i] = max(hi[i], p[i])
            found = True
    if not found:
        return Vector((0, 0, 0)), Vector((0, 0, 0))
    return lo, hi


# ---------------------------------------------------------------------------
# Scene
# ---------------------------------------------------------------------------
def reset_scene():
    """Empty scene in metres, set up for the standard check picture.

    Cycles on the CPU, 1600 by 900, 64 samples, denoising, AgX.
    """
    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene
    sc.unit_settings.system = "METRIC"
    sc.unit_settings.scale_length = 1.0
    sc.unit_settings.length_unit = "METERS"
    r = sc.render
    r.engine = "CYCLES"
    r.resolution_x, r.resolution_y = RENDER_SIZE
    r.resolution_percentage = 100
    r.film_transparent = False
    r.image_settings.file_format = "PNG"
    r.image_settings.color_mode = "RGB"
    r.image_settings.color_depth = "8"
    # Nothing that changes from run to run goes into the picture file.
    r.use_stamp = False
    for p in ("date", "time", "render_time", "memory", "hostname", "filename",
              "frame", "camera", "scene", "note", "marker", "lens",
              "sequencer_strip", "frame_range"):
        if hasattr(r, "use_stamp_" + p):
            setattr(r, "use_stamp_" + p, False)
    c = sc.cycles
    c.device = "CPU"
    c.samples = RENDER_SAMPLES
    c.seed = 0
    c.use_animated_seed = False
    c.use_denoising = True
    try:
        c.denoiser = "OPENIMAGEDENOISE"
    except TypeError:
        pass
    sc.view_settings.view_transform = "AgX"
    sc.view_settings.look = "None"
    sc.view_settings.exposure = 0.0
    sc.view_settings.gamma = 1.0
    if sc.world is None:
        sc.world = bpy.data.worlds.new("world")
    return sc


# ---------------------------------------------------------------------------
# Ground, footprint outline, scale bar
# ---------------------------------------------------------------------------
def make_ground(size_m=60, footprint=(1, 1), line_strength=0.28):
    """Flat dry soil with a faint darker line every square.

    footprint: the footprint (w, d in squares) of the model that stands on
    the origin. The lines are shifted so they run along that footprint's
    edges: a model 1 square wide sits in the middle of a square, a model 2
    squares wide sits on a line.

    The square lines cover size_m by size_m. The same soil, without lines,
    carries on far beyond that so level cameras see a true horizon.
    """
    sq = square()
    inner = size_m / 2.0
    half = max(inner, GROUND_APRON_M / 2.0)
    me = bpy.data.meshes.new("ground_mesh")
    me.from_pydata([(-half, -half, 0), (half, -half, 0), (half, half, 0), (-half, half, 0)],
                   [], [(0, 1, 2, 3)])
    ob = _link(bpy.data.objects.new("ground", me))

    mat = bpy.data.materials.new("ground_soil")
    nt = _node_tree_of(mat)
    N = nt.nodes
    L = nt.links
    bsdf = next(n for n in N if n.bl_idname == "ShaderNodeBsdfPrincipled")
    bsdf.inputs["Roughness"].default_value = 0.95
    bsdf.inputs["Specular IOR Level"].default_value = 0.15

    geo = N.new("ShaderNodeNewGeometry")
    pos = geo.outputs["Position"]

    def noise(scale, detail, rough=0.55):
        n = N.new("ShaderNodeTexNoise")
        n.inputs["Scale"].default_value = scale
        n.inputs["Detail"].default_value = detail
        n.inputs["Roughness"].default_value = rough
        L.new(pos, n.inputs["Vector"])
        return n.outputs["Fac"]

    big = noise(0.17, 6.0, 0.6)      # patches many metres across
    mid = noise(0.9, 5.0)       # patches about a metre across
    fine = noise(14.0, 4.0, 0.7)  # grit

    # Two soil tones either side of the base colour, mixed by the big noise.
    base = rgb(SOIL_HEX)
    dark = tuple(c * 0.80 for c in base[:3]) + (1.0,)
    light = (base[0] * 1.20, base[1] * 1.19, base[2] * 1.17, 1.0)
    ramp = N.new("ShaderNodeMapRange")
    ramp.inputs["From Min"].default_value = 0.25
    ramp.inputs["From Max"].default_value = 0.75
    L.new(big, ramp.inputs["Value"])
    mix1 = N.new("ShaderNodeMix")
    mix1.data_type = "RGBA"
    mix1.inputs["A"].default_value = dark
    mix1.inputs["B"].default_value = light
    L.new(ramp.outputs["Result"], mix1.inputs["Factor"])
    # Metre-scale and grit variation as a brightness factor round 1.0.
    v_mid = _math(nt, "MULTIPLY_ADD", mid, 0.50, 0.75)
    v_fine = _math(nt, "MULTIPLY_ADD", fine, 0.36, 0.82)
    v = _math(nt, "MULTIPLY", v_mid, v_fine)

    # Square lines: distance in metres to the nearest line, on X and on Y.
    sep = N.new("ShaderNodeSeparateXYZ")
    L.new(pos, sep.inputs["Vector"])
    masks = []
    for axis, count in (("X", footprint[0]), ("Y", footprint[1])):
        off = sq / 2.0 if count % 2 == 1 else 0.0
        t = _math(nt, "DIVIDE", _math(nt, "SUBTRACT", sep.outputs[axis], off), sq)
        dist = _math(nt, "MULTIPLY", _math(nt, "ABSOLUTE",
                     _math(nt, "SUBTRACT", t, _math(nt, "ROUND", t))), sq)
        mr = N.new("ShaderNodeMapRange")
        mr.interpolation_type = "SMOOTHSTEP"
        mr.inputs["From Min"].default_value = 0.007   # line is about 2 cm wide
        mr.inputs["From Max"].default_value = 0.013
        mr.inputs["To Min"].default_value = 1.0
        mr.inputs["To Max"].default_value = 0.0
        L.new(dist, mr.inputs["Value"])
        masks.append(mr.outputs["Result"])
    line = _math(nt, "MAXIMUM", masks[0], masks[1])
    # The lines fade in and out a little, like scuffed marks, and stop at
    # the edge of the squared area.
    line = _math(nt, "MULTIPLY", line, _math(nt, "MULTIPLY_ADD", mid, 0.9, 0.55))
    for axis in ("X", "Y"):
        inside = _math(nt, "LESS_THAN", _math(nt, "ABSOLUTE", sep.outputs[axis]), inner + 0.02)
        line = _math(nt, "MULTIPLY", line, inside)
    v = _math(nt, "MULTIPLY", v, _math(nt, "MULTIPLY_ADD", line, -line_strength, 1.0))

    mul = N.new("ShaderNodeMix")
    mul.data_type = "RGBA"
    mul.blend_type = "MULTIPLY"
    mul.inputs["Factor"].default_value = 1.0
    L.new(mix1.outputs["Result"], mul.inputs["A"])
    L.new(v, mul.inputs["B"])
    L.new(mul.outputs["Result"], bsdf.inputs["Base Color"])

    bump = N.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = 0.35
    bump.inputs["Distance"].default_value = 0.02
    L.new(_math(nt, "ADD", fine, _math(nt, "MULTIPLY", mid, 2.0)), bump.inputs["Height"])
    L.new(bump.outputs["Normal"], bsdf.inputs["Normal"])

    me.materials.append(mat)
    return ob


def footprint_outline(w, d, line_m=0.035):
    """Thin white line on the ground round a footprint of w by d squares."""
    sq = square()
    hx, hy = w * sq / 2.0, d * sq / 2.0
    t = line_m / 2.0
    z = 0.004
    outer = [(-hx - t, -hy - t), (hx + t, -hy - t), (hx + t, hy + t), (-hx - t, hy + t)]
    inner = [(-hx + t, -hy + t), (hx - t, -hy + t), (hx - t, hy - t), (-hx + t, hy - t)]
    verts = [(x, y, z) for x, y in outer + inner]
    faces = [(i, (i + 1) % 4, 4 + (i + 1) % 4, 4 + i) for i in range(4)]
    me = bpy.data.meshes.new("footprint_outline_mesh")
    me.from_pydata(verts, [], faces)
    ob = _link(bpy.data.objects.new("footprint_outline", me))
    mat = bpy.data.materials.get("footprint_white")
    if mat is None:
        mat, _ = new_material("footprint_white", "#ffffff", roughness=1.0)
    me.materials.append(mat)
    ob.visible_shadow = False
    return ob


def scale_bar(x, y, length_m=1.0, axis="Y", segment_m=0.25, width_m=0.12, height_m=0.05):
    """Flat black-and-white bar on the ground, centred on (x, y).

    It runs along Y by default so the side cameras see its full length.
    Pass axis="X" for the front camera. Each segment is 0.25 m.
    """
    bm = bmesh.new()
    n = max(1, int(math.ceil(length_m / segment_m - 1e-6)))
    a = -length_m / 2.0
    for i in range(n):
        b = min(a + segment_m, length_m / 2.0)
        res = bmesh.ops.create_cube(bm, size=1.0)
        for v in res["verts"]:
            v.co = Vector((v.co.x * width_m, a + (v.co.y + 0.5) * (b - a),
                           (v.co.z + 0.5) * height_m))
        for f in {f for v in res["verts"] for f in v.link_faces}:
            f.material_index = i % 2
        a = b
    me = bpy.data.meshes.new("scale_bar_mesh")
    bm.to_mesh(me)
    bm.free()
    for name, hexc in (("scale_bar_black", "#0a0a0a"), ("scale_bar_white", "#f2f2f2")):
        mat = bpy.data.materials.get(name)
        if mat is None:
            mat, _ = new_material(name, hexc, roughness=0.8)
        me.materials.append(mat)
    ob = _link(bpy.data.objects.new("scale_bar", me))
    ob.location = (x, y, 0.0)
    if axis.upper() == "X":
        ob.rotation_euler = (0, 0, math.radians(90))
    return ob


# ---------------------------------------------------------------------------
# Light
# ---------------------------------------------------------------------------
def sun_direction(elevation_deg=SUN_ELEVATION_DEG):
    """Unit vector from the scene toward the sun (and the moon)."""
    el = math.radians(elevation_deg)
    az = math.radians(SUN_LEFT_OF_FRONT_DEG)
    return Vector((-math.sin(az) * math.cos(el), -math.cos(az) * math.cos(el), math.sin(el)))


def _clear_world():
    sc = bpy.context.scene
    if sc.world is None:
        sc.world = bpy.data.worlds.new("world")
    nt = _node_tree_of(sc.world)
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputWorld")
    bg = nt.nodes.new("ShaderNodeBackground")
    nt.links.new(bg.outputs["Background"], out.inputs["Surface"])
    old = bpy.data.objects.get("moon")
    if old is not None:
        data = old.data
        bpy.data.objects.remove(old)
        bpy.data.lights.remove(data)
    return nt, bg


def day_light(strength=DAY_SKY_STRENGTH):
    """Physical sky. Sun 38 degrees up, from the front left of the model."""
    nt, bg = _clear_world()
    sky = nt.nodes.new("ShaderNodeTexSky")
    kinds = [e.identifier for e in sky.bl_rna.properties["sky_type"].enum_items]
    for kind in ("MULTIPLE_SCATTERING", "NISHITA", "SINGLE_SCATTERING"):
        if kind in kinds:
            sky.sky_type = kind
            break
    d = sun_direction()
    sky.sun_disc = True
    sky.sun_elevation = math.radians(SUN_ELEVATION_DEG)
    # The sky's sun sits toward +Y at rotation 0 and turns toward +X.
    sky.sun_rotation = math.atan2(d.x, d.y) % (2 * math.pi)
    nt.links.new(sky.outputs[0], bg.inputs["Color"])
    bg.inputs["Strength"].default_value = strength
    return sky


def night_light(moon_strength=MOON_STRENGTH):
    """Nearly black sky and a weak cool moon from the same side as the sun."""
    nt, bg = _clear_world()
    bg.inputs["Color"].default_value = NIGHT_SKY_COLOUR + (1.0,)
    bg.inputs["Strength"].default_value = NIGHT_SKY_STRENGTH
    li = bpy.data.lights.new("moon", "SUN")
    li.color = MOON_COLOUR
    li.energy = moon_strength
    li.angle = math.radians(1.0)
    ob = _link(bpy.data.objects.new("moon", li))
    ob.rotation_euler = sun_direction().to_track_quat("Z", "Y").to_euler()
    ob.location = sun_direction() * 30.0
    return ob


# ---------------------------------------------------------------------------
# Soldier stand-in
# ---------------------------------------------------------------------------
def _capsule(bm, p0, p1, r0, r1=None, sx=1.0, sy=1.0, segments=20, rings=10):
    """Rounded limb from p0 to p1, radius r0 at p0 and r1 at p1.

    sx, sy flatten the round section (used for the torso and the hands).
    """
    if r1 is None:
        r1 = r0
    p0, p1 = Vector(p0), Vector(p1)
    axis = p1 - p0
    rot = (axis.to_track_quat("Z", "Y").to_matrix() if axis.length > 1e-6
           else Matrix.Identity(3))
    res = bmesh.ops.create_uvsphere(bm, u_segments=segments, v_segments=rings, radius=1.0)
    for v in res["verts"]:
        up = v.co.z > 0.0
        r = r1 if up else r0
        local = Vector((v.co.x * r * sx, v.co.y * r * sy, v.co.z * r))
        v.co = (p1 if up else p0) + rot @ local


def _build_soldier_mesh():
    height = float(load()["anchor"]["soldier_height_m"])
    shoulders = float(load()["derived"]["soldier_shoulder_width_m"])

    bm = bmesh.new()
    c = lambda *a, **k: _capsule(bm, *a, **k)
    # Proportions are laid out for a figure 1.80 tall and 0.55 across the
    # shoulders, then fitted to the sheet's numbers at the end.
    # Head, neck
    c((0, -0.012, 1.672), (0, -0.004, 1.700), 0.098, 0.100, sx=0.80, sy=0.98)
    c((0, 0.014, 1.47), (0, 0.000, 1.61), 0.062, 0.054)
    # Torso: chest, upper back, waist, hips
    c((0, -0.004, 1.19), (0, 0.008, 1.385), 0.124, 0.120, sx=1.34, sy=0.98)
    c((0, -0.004, 1.03), (0, -0.004, 1.19), 0.108, 0.118, sx=1.32, sy=0.92)
    c((-0.066, 0.006, 0.955), (0.066, 0.006, 0.955), 0.104)
    for s in (-1, 1):
        lean = 0.012 * s          # a little more weight on one leg
        # Sloping shoulder from the neck out to the arm
        c((s * 0.040, 0.012, 1.490), (s * 0.185, 0.008, 1.440), 0.058, 0.066)
        # Arm: shoulder cap, upper arm, forearm, hand
        c((s * 0.207, 0.006, 1.438), (s * 0.207, 0.006, 1.438), 0.068)
        c((s * 0.214, 0.006, 1.42), (s * 0.266, 0.024, 1.150), 0.058, 0.046)
        c((s * 0.266, 0.024, 1.150), (s * 0.280, -0.050 - 0.012 * s, 0.900), 0.047, 0.035)
        c((s * 0.280, -0.052 - 0.012 * s, 0.890), (s * 0.276, -0.068 - 0.012 * s, 0.790),
          0.042, 0.032, sx=0.55, sy=1.0)
        # Leg: thigh, shin, calf, foot
        c((s * 0.088, 0.002, 0.925), (s * 0.104 + lean, -0.022, 0.505), 0.092, 0.064)
        c((s * 0.104 + lean, -0.022, 0.505), (s * 0.112 + lean, 0.030, 0.095), 0.063, 0.045)
        c((s * 0.106 + lean, 0.004, 0.37), (s * 0.108 + lean, 0.022, 0.29), 0.062, 0.054)
        c((s * 0.112 + lean, 0.055, 0.052), (s * 0.142 + lean, -0.150, 0.042), 0.052, 0.042)
    raw = bpy.data.meshes.new("soldier_standin_raw")
    bm.to_mesh(raw)
    bm.free()

    # Melt the separate limbs into one smooth mannequin skin.
    tmp = bpy.data.objects.new("soldier_standin_tmp", raw)
    _link(tmp)
    rem = tmp.modifiers.new("remesh", "REMESH")
    rem.mode = "VOXEL"
    rem.voxel_size = 0.012
    rem.use_smooth_shade = True
    sm = tmp.modifiers.new("smooth", "SMOOTH")
    sm.factor = 0.5
    sm.iterations = 36
    dec = tmp.modifiers.new("decimate", "DECIMATE")
    dec.ratio = 0.35
    bpy.context.view_layer.update()
    dg = bpy.context.evaluated_depsgraph_get()
    me = bpy.data.meshes.new_from_object(tmp.evaluated_get(dg))
    me.name = "soldier_standin_mesh"
    bpy.data.objects.remove(tmp)
    bpy.data.meshes.remove(raw)

    # Fit exactly: feet on Z = 0, height and shoulder width from the sheet.
    zs = [v.co.z for v in me.vertices]
    z0, z1 = min(zs), max(zs)
    kz = height / (z1 - z0)
    band = [abs(v.co.x) for v in me.vertices
            if 0.775 < (v.co.z - z0) / (z1 - z0) < 0.83]
    kx = shoulders / (2.0 * max(band))
    for v in me.vertices:
        v.co = Vector((v.co.x * kx, v.co.y * kx, (v.co.z - z0) * kz))
    for p in me.polygons:
        p.use_smooth = True
    mat, _ = new_material("standin_grey", "#787878", roughness=0.65)
    me.materials.append(mat)
    return me


def add_soldier(x=0.0, y=0.0, facing_deg=0.0):
    """Soldier stand-in: a plain grey mannequin, 1.8 m tall, no face.

    It stands on the ground at (x, y). facing_deg 0 faces -Y like the models;
    positive turns it to its left (anticlockwise seen from above).
    """
    me = bpy.data.meshes.get("soldier_standin_mesh")
    if me is None:
        me = _build_soldier_mesh()
    ob = _link(bpy.data.objects.new("soldier_standin", me))
    ob.location = (x, y, 0.0)
    ob.rotation_euler = (0.0, 0.0, math.radians(facing_deg))
    return ob


# ---------------------------------------------------------------------------
# Cameras
# ---------------------------------------------------------------------------
def _direction(azimuth_deg, elevation_deg):
    """Unit vector from the target toward the camera.

    Azimuth 0 is straight in front of the model (the -Y side); positive
    swings the camera to the right as seen from the front (+X side);
    -90 is the lit left side. Elevation is degrees above level.
    """
    az, el = math.radians(azimuth_deg), math.radians(elevation_deg)
    return Vector((math.sin(az) * math.cos(el), -math.cos(az) * math.cos(el), math.sin(el)))


def _camera(name, target, direction, distance, lens=85.0, ortho_scale=None):
    sc = bpy.context.scene
    old = bpy.data.objects.get(name)
    if old is not None:
        data = old.data
        bpy.data.objects.remove(old)
        bpy.data.cameras.remove(data)
    cam = bpy.data.cameras.new(name)
    cam.sensor_width = 36.0
    cam.lens = lens
    cam.clip_start = 0.05
    cam.clip_end = 2000.0
    if ortho_scale is not None:
        cam.type = "ORTHO"
        cam.ortho_scale = ortho_scale
    ob = _link(bpy.data.objects.new(name, cam))
    ob.location = Vector(target) + direction * distance
    ob.rotation_euler = direction.to_track_quat("Z", "Y").to_euler()
    sc.camera = ob
    return ob


def _fit_distance(lo, hi, target, direction, lens, margin):
    """Smallest camera distance that keeps the whole box in the picture."""
    sc = bpy.context.scene
    aspect = sc.render.resolution_x / sc.render.resolution_y
    tan_h = 18.0 / lens
    tan_v = tan_h / aspect
    q = direction.to_track_quat("Z", "Y").to_matrix()
    right, up = q.col[0], q.col[1]
    need = 0.1
    for cx in (lo.x, hi.x):
        for cy in (lo.y, hi.y):
            for cz in (lo.z, hi.z):
                v = Vector((cx, cy, cz)) - target
                depth = v.dot(direction)
                need = max(need, depth + margin * abs(v.dot(right)) / tan_h,
                           depth + margin * abs(v.dot(up)) / tan_v)
    return need


def _framed(name, objects, azimuth_deg, elevation_deg, lens, margin):
    lo, hi = bounds(objects)
    target = (lo + hi) / 2.0
    d = _direction(azimuth_deg, elevation_deg)
    return _camera(name, target, d, _fit_distance(lo, hi, target, d, lens, margin), lens)


def cam_front(objects, lens=85.0, margin=1.18):
    """Camera straight in front, at the model's mid height, all in frame."""
    return _framed("cam_front", objects, 0.0, 0.0, lens, margin)


def cam_side(objects, lens=85.0, margin=1.18):
    """Camera on the model's lit side (-X), at mid height. The model faces
    right in the picture."""
    return _framed("cam_side", objects, -90.0, 0.0, lens, margin)


def cam_threequarter(objects, lens=85.0, margin=1.18,
                     azimuth_deg=THREEQUARTER_AZIMUTH_DEG, elevation_deg=0.0):
    """Camera to the front right, at mid height: shows the lit front and
    the shadowed side together."""
    return _framed("cam_threequarter", objects, azimuth_deg, elevation_deg, lens, margin)


def cam_aerial(objects=None, distance_m=45.0, down_deg=40.0, lens=135.0,
               azimuth_deg=THREEQUARTER_AZIMUTH_DEG):
    """The game's view: 45 m away, looking down at 40 degrees, 135 mm lens.

    It aims at the middle of the objects. With no objects it aims at the
    origin on the ground.
    """
    if objects:
        lo, hi = bounds(objects)
        target = (lo + hi) / 2.0
    else:
        target = Vector((0.0, 0.0, 0.0))
    return _camera("cam_aerial", target, _direction(azimuth_deg, down_deg), distance_m, lens)


def cam_ortho_side(objects, margin=1.2):
    """Straight side view with no perspective, from the lit side (-X)."""
    sc = bpy.context.scene
    aspect = sc.render.resolution_x / sc.render.resolution_y
    lo, hi = bounds(objects)
    target = (lo + hi) / 2.0
    scale = max((hi.y - lo.y) * margin, (hi.z - lo.z) * margin * aspect, 0.5)
    dist = (hi.x - lo.x) / 2.0 + 60.0
    return _camera("cam_ortho_side", target, _direction(-90.0, 0.0), dist, ortho_scale=scale)


def cam_detail(target, distance, lens=85.0,
               azimuth_deg=THREEQUARTER_AZIMUTH_DEG, elevation_deg=10.0):
    """Close view. target is an object or a point (x, y, z)."""
    if isinstance(target, bpy.types.Object):
        lo, hi = bounds([target])
        point = (lo + hi) / 2.0 if _expand([target]) else target.matrix_world.translation.copy()
    else:
        point = Vector(target)
    return _camera("cam_detail", point, _direction(azimuth_deg, elevation_deg), distance, lens)


# ---------------------------------------------------------------------------
# Render, contact sheet
# ---------------------------------------------------------------------------
def render(path):
    """Render the active camera to a PNG at path."""
    path = os.path.abspath(path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    sc = bpy.context.scene
    sc.render.filepath = path
    bpy.ops.render.render(write_still=True)
    return path


def contact_sheet(paths, out_path, columns=3, cell_width=640):
    """Tile pictures on one sheet with their file names underneath."""
    import numpy as np
    import blf
    import imbuf

    paths = [p for p in paths if os.path.exists(p)]
    if not paths:
        raise ValueError("contact_sheet: none of the pictures exist")
    cell_h = int(round(cell_width * 9 / 16))
    label_h, pad = 30, 10
    columns = min(columns, len(paths))
    rows = int(math.ceil(len(paths) / columns))
    W = pad + columns * (cell_width + pad)
    H = pad + rows * (cell_h + label_h + pad)
    sheet = np.empty((H, W, 4), dtype=np.float32)
    sheet[:, :, :3] = 0.09
    sheet[:, :, 3] = 1.0
    spots = []
    for i, p in enumerate(paths):
        img = bpy.data.images.load(p, check_existing=False)
        w, h = img.size
        k = min(cell_width / w, cell_h / h)
        tw, th = max(1, int(round(w * k))), max(1, int(round(h * k)))
        img.scale(tw, th)
        px = np.empty(tw * th * 4, dtype=np.float32)
        img.pixels.foreach_get(px)
        px = px.reshape(th, tw, 4)
        bpy.data.images.remove(img)
        col, row = i % columns, i // columns
        x0 = pad + col * (cell_width + pad)
        top = pad + row * (cell_h + label_h + pad)      # counted from the top
        y0 = H - top - cell_h                             # rows run bottom-up
        ox, oy = x0 + (cell_width - tw) // 2, y0 + (cell_h - th) // 2
        sheet[oy:oy + th, ox:ox + tw, :3] = px[:, :, :3]
        spots.append((x0 + 2, y0 - label_h + 9, os.path.basename(p)))
    out_path = os.path.abspath(out_path)
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    out = bpy.data.images.new("contact_sheet", W, H, alpha=False)
    out.pixels.foreach_set(sheet.ravel())
    out.filepath_raw = out_path
    out.file_format = "PNG"
    out.save()
    bpy.data.images.remove(out)
    # Names: drawn with Blender's own font straight into the saved picture.
    buf = imbuf.load(out_path)
    with blf.bind_imbuf(0, buf):
        blf.size(0, 19)
        blf.color(0, 0.88, 0.88, 0.88, 1.0)
        for x, y, text in spots:
            blf.position(0, x, y, 0)
            blf.draw_buffer(0, text)
    imbuf.write(buf, filepath=out_path)
    return out_path


# ---------------------------------------------------------------------------
# Measuring
# ---------------------------------------------------------------------------
def measure(objects):
    """Width (X), depth (Y), height (Z) in metres and the triangle count.

    Measured on the objects and their children after modifiers.
    """
    import numpy as np
    bpy.context.view_layer.update()
    dg = bpy.context.evaluated_depsgraph_get()
    lo = np.full(3, 1e9)
    hi = np.full(3, -1e9)
    tris = 0
    for o in _expand(objects):
        ev = o.evaluated_get(dg)
        try:
            me = ev.to_mesh()
        except RuntimeError:
            continue
        if me is None or len(me.vertices) == 0:
            ev.to_mesh_clear()
            continue
        co = np.empty(len(me.vertices) * 3, dtype=np.float64)
        me.vertices.foreach_get("co", co)
        co = co.reshape(-1, 3)
        m = np.array(ev.matrix_world)
        world = co @ m[:3, :3].T + m[:3, 3]
        lo = np.minimum(lo, world.min(axis=0))
        hi = np.maximum(hi, world.max(axis=0))
        me.calc_loop_triangles()
        tris += len(me.loop_triangles)
        ev.to_mesh_clear()
    if tris == 0:
        lo = hi = np.zeros(3)
    size = hi - lo
    return {
        "width_m": round(float(size[0]), 3),
        "depth_m": round(float(size[1]), 3),
        "height_m": round(float(size[2]), 3),
        "triangles": int(tris),
        "min": [round(float(v), 3) for v in lo],
        "max": [round(float(v), 3) for v in hi],
    }


def write_size_json(path, name, objects):
    """Measure the finished model and write its size.json.

    footprint_squares is the footprint the sheet declares for this name.
    If the name is not in the sheet it is worked out from the measured size.
    A warning is printed when the model is larger than its declared squares.
    """
    m = measure(objects)
    sq = square()
    fit = [max(1, math.ceil(m["width_m"] / sq - 1e-6)),
           max(1, math.ceil(m["depth_m"] / sq - 1e-6))]
    things = load()["things"]
    if name in things and "footprint" in things[name]:
        foot = list(things[name]["footprint"])
        if fit[0] > foot[0] or fit[1] > foot[1]:
            print("scale_lib WARNING: %s measures %.2f by %.2f m, which needs %s squares; "
                  "the sheet declares %s. Check the sheet's 'cheats' list."
                  % (name, m["width_m"], m["depth_m"], fit, foot))
    else:
        foot = fit
    data = {
        "name": name,
        "width_m": m["width_m"],
        "depth_m": m["depth_m"],
        "height_m": m["height_m"],
        "footprint_squares": foot,
        "triangles": m["triangles"],
    }
    path = os.path.abspath(path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
        f.write("\n")
    return data
