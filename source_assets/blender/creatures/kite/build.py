"""Builds the Kite from nothing and saves model.blend next to this file.

  flatpak run org.blender.Blender -b --python "<full path>/build.py"

The file holds two collections:
  model  the living Kite in the air (root object "kite"),
  dead   the dead form, crumpled belly-up on the ground (root "dead_kite").

Sizes that the scale sheet gives (span, length, flight height) are read
from the sheet. Sizes that only the job text gives are named below.
"""
import json
import math
import os
import random
import sys
import warnings

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "scale")))

import bpy
import bmesh
from mathutils import Matrix, Vector

import scale_lib as sl

NAME = "kite"
K = sl.thing(NAME)
SPAN = float(K["span_m"])                  # wing point to wing point
LENGTH = float(K["length_m"])              # nose point to tail point
FLY = float(K["flight_height_m"])          # lowest point of the body

# ---- sizes and facts from the job text (not in the sheet) -----------------
WIDEST_BACK = 1.0 / 3.0     # the widest part, as a share of the length from the nose
SPAR_THICK = 0.02           # spar thickness at the centre
SIGN_ACROSS = 0.25          # the target sign
CAGE_ACROSS = 0.2           # the Registrar cage
THREAD_LEN = 1.5
THREAD_COUNT = 12
REPAIR_SHARE = 1.0 / 3.0    # outer share of the right wing spar that is a black rod
BANK_DEG = 6.0              # "tilted a few degrees into a turn"

# ---- colours ---------------------------------------------------------------
AMBER_SKIN = "#c98a2a"
RED = "#8c1f1a"
BONE = "#cfc6b0"            # the same pale bone as the Lash's mask
REG = "#0b0b0c"
GOLD = "#8a6a2a"
AMBER = "#ffa01e"
ELEC = "#cfe4ff"

# ---- layout, flat, seen from above. x is across, yr is along, measured from
# the crossing of the spars. The nose is at -Y. The Kite's own left is +X. ----
HALF = SPAN / 2.0
LN = LENGTH * WIDEST_BACK           # crossing to nose
LT = LENGTH - LN                    # crossing to tail
CY = -LENGTH / 2.0 + LN             # y of the crossing, footprint centred on 0
TIPZ = {"N": 0.0, "T": 0.012, "WL": 0.05, "WR": 0.05}     # the tips' heights
BOW = {"N": 0.006, "T": 0.022, "WL": 0.02, "WR": 0.0}       # how much each spar arches
BILLOW = {("WL", "N"): 0.03, ("WR", "N"): 0.03, ("WL", "T"): 0.07, ("WR", "T"): 0.062}
EDGE_PULL = 0.10            # the skin's pull bows the thin edge spar inward between the points
R_C = SPAR_THICK / 2.0              # spar radius at the centre
R_TIP = 0.004
R_EDGE = 0.004
SIGN_AT = (0.38, 0.19)              # centre of the sign on the left wing (x, yr)
SIGN_R = SIGN_ACROSS / 2.0
N_SKIN = 64
ROD_X0 = -HALF * (1.0 - REPAIR_SHARE)      # where the black rod takes over

# dead form
SHRINK = 0.95
YK = 0.47                   # the main spar snaps here (yr)
KINK = math.radians(30.0)
XF = 0.44                   # the left wing spar snaps here (x)
FOLD = math.radians(54.0)
DEAD_SHIFT = Vector((0.12, -0.22, 0.0))


def smoothstep(a, b, x):
    t = min(1.0, max(0.0, (x - a) / (b - a)))
    return t * t * (3.0 - 2.0 * t)


def weights(x, yr):
    wx = abs(x) / HALF
    wy = abs(yr) / (LN if yr < 0 else LT)
    return wx, wy, 1.0 - wx - wy


def flat_h(x, yr):
    """Height of the stretched surface above the crossing."""
    wx, wy, wc = weights(x, yr)
    wc = max(0.0, wc)
    kx = "WL" if x >= 0 else "WR"
    ky = "N" if yr < 0 else "T"
    return (wx * TIPZ[kx] + wy * TIPZ[ky] + 4.0 * wc * (wx * BOW[kx] + wy * BOW[ky])
            + BILLOW[(kx, ky)] * 27.0 * wx * wy * wc)


def pulled(x, yr):
    """The flat layout with the edges drawn inward between the four points."""
    wx, wy, _ = weights(x, yr)
    s = wx + wy
    if s < 1e-9:
        return x, yr
    t = wy / s
    k = 1.0 - EDGE_PULL * 4.0 * t * (1.0 - t) * s * s
    return x * k, yr * k


def place_live(x, yr, lift=0.0):
    px, py = pulled(x, yr)
    return Vector((px, CY + py, flat_h(x, yr) + lift))


def place_dead(x, yr, lift=0.0):
    """The same point on the dead form: belly-up, crumpled, snapped twice."""
    cz = (0.006
          + 0.024 * (0.5 + 0.5 * math.sin(17 * x + 2.1 + 3 * math.sin(9 * yr)))
          * (0.5 + 0.5 * math.sin(21 * yr + 0.7 + 2 * math.sin(11 * x)))
          + 0.008 * abs(math.sin(47 * x + 31 * yr)))
    px, py = pulled(x, yr)
    p = Vector((-px * SHRINK + 0.006 * math.sin(13 * yr + 1.0),
                py * SHRINK + 0.006 * math.sin(11 * x + 0.4),
                cz + lift))
    if x > 0:       # the skin buckles on the inside of the kink
        p.z += 0.05 * math.exp(-((yr - YK - 0.015) / 0.045) ** 2) * min(1.0, x / 0.15)
    t = smoothstep(YK, YK + 0.03, yr)
    if t > 0:
        h = Vector((0.0, YK * SHRINK, 0.0))
        p = h + Matrix.Rotation(KINK * t, 3, "Z") @ (p - h)
    t = smoothstep(XF, XF + 0.03, x)
    if t > 0:
        a = Vector((-XF * SHRINK, 0.0, 0.012))
        p = a + Matrix.Rotation(FOLD * t, 3, Vector((0.14, 1.0, 0.0)).normalized()) @ (p - a)
    p.z = max(p.z, 0.004 + lift)        # nothing sinks into the ground
    return p + DEAD_SHIFT


# ---------------------------------------------------------------------------
# Mesh tools
# ---------------------------------------------------------------------------
def tube(bm, pts, radii, sides=8, mat=0, smooth=True, up=None, cap=True):
    """A tube along pts. radii is one number or one per point."""
    pts = [Vector(p) for p in pts]
    n = len(pts)
    if isinstance(radii, (int, float)):
        radii = [radii] * n
    rings = []
    u = None
    for i in range(n):
        t = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized()
        if u is None:
            u = Vector(up) if up is not None else t.orthogonal()
        u = (u - t * u.dot(t)).normalized()
        v = t.cross(u)
        off = math.pi / sides if sides == 4 else 0.0
        ring = []
        for k in range(sides):
            a = off + 2 * math.pi * k / sides
            ring.append(bm.verts.new(pts[i] + (u * math.cos(a) + v * math.sin(a)) * radii[i]))
        rings.append(ring)
    for i in range(n - 1):
        for k in range(sides):
            f = bm.faces.new((rings[i][k], rings[i][(k + 1) % sides],
                              rings[i + 1][(k + 1) % sides], rings[i + 1][k]))
            f.material_index = mat
            f.smooth = smooth
    if cap:
        f = bm.faces.new(list(reversed(rings[0])))
        f.material_index = mat
        f = bm.faces.new(rings[-1])
        f.material_index = mat


def box(bm, centre, size, rot=None, mat=0):
    m = Matrix.Translation(Vector(centre))
    if rot is not None:
        m = m @ rot.to_4x4()
    m = m @ Matrix.Diagonal((size[0], size[1], size[2], 1.0))
    res = bmesh.ops.create_cube(bm, size=1.0, matrix=m)
    for f in {f for v in res["verts"] for f in v.link_faces}:
        f.material_index = mat
        f.smooth = False


def cylinder(bm, centre, radius, depth, segments=24, mat=0, caps=True, rot=None):
    m = Matrix.Translation(Vector(centre))
    if rot is not None:
        m = m @ rot.to_4x4()
    res = bmesh.ops.create_cone(bm, cap_ends=caps, cap_tris=False, segments=segments,
                                radius1=radius, radius2=radius, depth=depth, matrix=m)
    for f in {f for v in res["verts"] for f in v.link_faces}:
        f.material_index = mat
        f.smooth = len(f.verts) == 4


def dome(bm, centre, radius, sag, segments=24, rings=4, mat=0):
    """A shallow lens that bulges toward -Z from a rim at centre."""
    c = Vector(centre)
    rs = (radius * radius + sag * sag) / (2.0 * sag)
    amax = math.asin(radius / rs)
    prev = None
    tip = bm.verts.new(c + Vector((0, 0, -sag)))
    for j in range(1, rings + 1):
        a = amax * j / rings
        r, z = rs * math.sin(a), -sag + rs * (1 - math.cos(a))
        ring = [bm.verts.new(c + Vector((r * math.cos(2 * math.pi * k / segments),
                                         r * math.sin(2 * math.pi * k / segments), z)))
                for k in range(segments)]
        for k in range(segments):
            k2 = (k + 1) % segments
            if prev is None:
                f = bm.faces.new((tip, ring[k2], ring[k]))
            else:
                f = bm.faces.new((prev[k], prev[k2], ring[k2], ring[k]))
            f.material_index = mat
            f.smooth = True
        prev = ring


def finish(bm, name, coll, parent, mats, origin=(0, 0, 0), sharp_deg=None):
    """Turn a bmesh into an object whose origin is at `origin` (its joint)."""
    origin = Vector(origin)
    for v in bm.verts:
        v.co -= origin
    me = bpy.data.meshes.new(name + "_mesh")
    bm.to_mesh(me)
    bm.free()
    for m in mats:
        me.materials.append(m)
    if sharp_deg is not None and hasattr(me, "set_sharp_from_angle"):
        me.set_sharp_from_angle(angle=math.radians(sharp_deg))
    ob = bpy.data.objects.new(name, me)
    coll.objects.link(ob)
    ob.parent = parent
    ob.location = origin
    return ob


def new_collection(name):
    c = bpy.data.collections.new(name)
    bpy.context.scene.collection.children.link(c)
    return c


# ---------------------------------------------------------------------------
# Materials
# ---------------------------------------------------------------------------
class Nodes:
    def __init__(self, name):
        self.mat = bpy.data.materials.new(name)
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")
            try:
                self.mat.use_nodes = True
            except Exception:
                pass
        self.nt = self.mat.node_tree
        self.nt.nodes.clear()
        self.N, self.L = self.nt.nodes, self.nt.links
        self.out = self.N.new("ShaderNodeOutputMaterial")

    def _set(self, sock, v):
        if isinstance(v, (int, float, tuple, list)):
            sock.default_value = v
        else:
            self.L.new(v, sock)

    def math(self, op, a, b=None, c=None, clamp=False):
        n = self.N.new("ShaderNodeMath")
        n.operation = op
        n.use_clamp = clamp
        for i, v in enumerate((a, b, c)):
            if v is not None:
                self._set(n.inputs[i], v)
        return n.outputs[0]

    def mix(self, fac, a, b):
        n = self.N.new("ShaderNodeMix")
        n.data_type = "RGBA"
        ins = [s for s in n.inputs if s.enabled]
        self._set(next(s for s in ins if s.name == "Factor"), fac)
        self._set(next(s for s in ins if s.name == "A"), a)
        self._set(next(s for s in ins if s.name == "B"), b)
        return next(s for s in n.outputs if s.enabled)

    def ramp(self, x, lo, hi, out0=0.0, out1=1.0):
        n = self.N.new("ShaderNodeMapRange")
        n.interpolation_type = "SMOOTHSTEP"
        self._set(n.inputs["Value"], x)
        n.inputs["From Min"].default_value = lo
        n.inputs["From Max"].default_value = hi
        n.inputs["To Min"].default_value = out0
        n.inputs["To Max"].default_value = out1
        return n.outputs["Result"]

    def between(self, x, lo, hi, soft=0.002):
        return self.math("MULTIPLY", self.ramp(x, lo - soft, lo + soft),
                         self.ramp(x, hi - soft, hi + soft, 1.0, 0.0))

    def noise(self, vec, scale, detail=2.0, rough=0.5, colour=False):
        n = self.N.new("ShaderNodeTexNoise")
        if hasattr(n, "noise_dimensions"):
            n.noise_dimensions = "3D"
        if vec is not None:
            self.L.new(vec, n.inputs["Vector"])
        n.inputs["Scale"].default_value = scale
        n.inputs["Detail"].default_value = detail
        n.inputs["Roughness"].default_value = rough
        return n.outputs["Color" if colour else "Fac"]

    def cells(self, vec, scale):
        """Distance to the nearest cell wall: a net of lines, like wing veins."""
        n = self.N.new("ShaderNodeTexVoronoi")
        n.voronoi_dimensions = "2D"
        n.feature = "DISTANCE_TO_EDGE"
        self.L.new(vec, n.inputs["Vector"])
        n.inputs["Scale"].default_value = scale
        return n.outputs["Distance"]

    def shader(self, kind, **inputs):
        n = self.N.new(kind)
        for k, v in inputs.items():
            self._set(n.inputs[k.replace("_", " ")], v)
        return n

    def surface(self, sock):
        self.L.new(sock, self.out.inputs["Surface"])
        return self.mat


def skin_material(name, dead=False):
    """Thin clear amber skin, veined in red, with the Lash's sign on the left wing.

    All patterns are drawn in the skin's flat layout (the UV map holds metres
    from the crossing), so the living and the dead skin carry the same marks.
    """
    g = Nodes(name)
    amber = sl.rgb(AMBER_SKIN)
    red = sl.rgb(RED)
    if dead:    # the colour drains: duller and greyer
        grey = sl.rgb("#6f6248")
        amber = tuple(a * 0.55 + b * 0.45 for a, b in zip(amber, grey))
        red = tuple(a * 0.6 + b * 0.4 * 0.5 for a, b in zip(red, grey))
    dark = tuple(c * 0.30 for c in red[:3]) + (1.0,)

    uv = g.N.new("ShaderNodeTexCoord").outputs["UV"]
    sep = g.N.new("ShaderNodeSeparateXYZ")
    g.L.new(uv, sep.inputs[0])
    u, v = sep.outputs["X"], sep.outputs["Y"]

    # the vein net, bent a little so it does not look ruled
    wob = g.N.new("ShaderNodeVectorMath")
    wob.operation = "MULTIPLY_ADD"
    g.L.new(g.noise(uv, 4.0, 2.0, colour=True), wob.inputs[0])
    wob.inputs[1].default_value = (0.11, 0.11, 0.0)
    g.L.new(uv, wob.inputs[2])
    w = wob.outputs[0]
    long_w = g.N.new("ShaderNodeVectorMath")
    long_w.operation = "MULTIPLY"
    g.L.new(w, long_w.inputs[0])
    long_w.inputs[1].default_value = (0.42, 1.0, 1.0)
    big = g.ramp(g.cells(long_w.outputs[0], 8.0), 0.0, 0.05, 1.0, 0.0)
    fine = g.math("MULTIPLY", g.ramp(g.cells(w, 24.0), 0.0, 0.07, 1.0, 0.0), 0.6)
    hair = g.math("MULTIPLY", g.ramp(g.cells(w, 70.0), 0.0, 0.10, 1.0, 0.0), 0.16)
    veins = g.math("MAXIMUM", g.math("MAXIMUM", big, fine), hair)

    # thick dark tissue where the skin is rooted on the cross spars. Over the
    # black repair rod nothing has grown back: no band there.
    au = g.math("ABSOLUTE", u)
    av = g.math("ABSOLUTE", v)
    on_rod = g.ramp(u, ROD_X0 - 0.02, ROD_X0 + 0.02, 0.0, 1.0)
    band_wing = g.math("MULTIPLY", g.ramp(av, 0.012, 0.034, 1.0, 0.0), on_rod)
    band = g.math("MAXIMUM", g.ramp(au, 0.012, 0.034, 1.0, 0.0), band_wing)
    near = g.math("MAXIMUM", g.ramp(au, 0.03, 0.16, 1.0, 0.0),
                  g.math("MULTIPLY", g.ramp(av, 0.03, 0.16, 1.0, 0.0), on_rod))
    rim_attr = g.N.new("ShaderNodeAttribute")
    rim_attr.attribute_name = "rim"
    rim = rim_attr.outputs["Fac"]
    band = g.math("MAXIMUM", band, rim)

    # the target sign: two thin rings, a centre dot, one stroke toward the tail
    du = g.math("SUBTRACT", u, SIGN_AT[0])
    dv = g.math("SUBTRACT", v, SIGN_AT[1])
    r = g.math("SQRT", g.math("ADD", g.math("MULTIPLY", du, du), g.math("MULTIPLY", dv, dv)))
    R = SIGN_R
    sign = g.between(r, R * 0.86, R)
    sign = g.math("MAXIMUM", sign, g.between(r, R * 0.47, R * 0.60))
    sign = g.math("MAXIMUM", sign, g.ramp(r, R * 0.17 - 0.002, R * 0.17 + 0.002, 1.0, 0.0))
    stroke = g.math("MULTIPLY", g.between(dv, R * 0.32, R * 1.9),
                    g.between(du, -R * 0.075, R * 0.075))
    sign = g.math("MAXIMUM", sign, stroke)
    worn = g.ramp(g.noise(uv, 38.0, 3.0, 0.7), 0.30, 0.62, 0.45, 1.0)
    sign = g.math("MULTIPLY", sign, worn)
    back = g.N.new("ShaderNodeNewGeometry").outputs["Backfacing"]
    sign = g.math("MULTIPLY", sign, g.math("MULTIPLY_ADD", back, -0.55, 1.0))

    thick = g.noise(uv, 2.6, 3.0, 0.6)
    col = g.mix(g.math("MULTIPLY", veins, 0.92), amber, red)
    col = g.mix(g.math("MULTIPLY", near, 0.35), col, red)
    col = g.mix(band, col, dark)
    col = g.mix(sign, col, tuple(c * 0.7 for c in red[:3]) + (1.0,))

    opaque = g.math("MULTIPLY_ADD", thick, 0.22, 0.40 if dead else 0.10)
    opaque = g.math("MAXIMUM", opaque, g.math("MULTIPLY", veins, 0.9))
    opaque = g.math("MAXIMUM", opaque, g.math("MULTIPLY", near, 0.42))
    opaque = g.math("MAXIMUM", opaque, g.math("MULTIPLY", band, 0.97))
    opaque = g.math("MAXIMUM", opaque, g.math("MULTIPLY", sign, 0.9), clamp=True)

    bump = g.N.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = 0.22
    bump.inputs["Distance"].default_value = 0.002
    g.L.new(g.math("ADD", g.math("MAXIMUM", big, fine), g.math("MULTIPLY", g.noise(uv, 30.0, 3.0), 0.5)),
            bump.inputs["Height"])

    pr = g.shader("ShaderNodeBsdfPrincipled", Base_Color=col,
                  Roughness=g.math("MULTIPLY_ADD", thick, 0.25, 0.55 if dead else 0.36))
    pr.inputs["Specular IOR Level"].default_value = 0.35
    g.L.new(bump.outputs["Normal"], pr.inputs["Normal"])
    tl = g.shader("ShaderNodeBsdfTranslucent", Color=col)
    g.L.new(bump.outputs["Normal"], tl.inputs["Normal"])
    body = g.N.new("ShaderNodeMixShader")
    body.inputs[0].default_value = 0.55
    g.L.new(pr.outputs[0], body.inputs[1])
    g.L.new(tl.outputs[0], body.inputs[2])
    tint = (1.0, 0.80, 0.50, 1.0) if not dead else (0.80, 0.66, 0.44, 1.0)
    clear = g.shader("ShaderNodeBsdfTransparent", Color=tint)
    fin = g.N.new("ShaderNodeMixShader")
    g.L.new(opaque, fin.inputs[0])
    g.L.new(clear.outputs[0], fin.inputs[1])
    g.L.new(body.outputs[0], fin.inputs[2])
    return g.surface(fin.outputs[0])


def bone_material(name, dead=False):
    g = Nodes(name)
    pos = g.N.new("ShaderNodeTexCoord").outputs["Object"]
    base = sl.rgb(BONE)
    stain = sl.rgb("#7c6a4c")
    grain = g.noise(pos, 60.0, 4.0, 0.7)
    blot = g.noise(pos, 7.0, 3.0)
    if dead:
        base = tuple(c * 0.62 for c in base[:3]) + (1.0,)
    col = g.mix(g.ramp(grain, 0.45, 0.75, 0.0, 0.55 if not dead else 0.75), base, stain)
    col = g.mix(g.ramp(blot, 0.5, 0.8, 0.0, 0.35), col, sl.rgb(RED))
    bump = g.N.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = 0.4
    bump.inputs["Distance"].default_value = 0.002
    g.L.new(grain, bump.inputs["Height"])
    pr = g.shader("ShaderNodeBsdfPrincipled", Base_Color=col, Roughness=0.6)
    g.L.new(bump.outputs["Normal"], pr.inputs["Normal"])
    return g.surface(pr.outputs[0])


def lens_glass(name, tint, clear_share):
    """Dark smoked glass: it dims the amber core so it reads amber, not white."""
    g = Nodes(name)
    pr = g.shader("ShaderNodeBsdfPrincipled", Base_Color=(0.05, 0.022, 0.004, 1.0), Roughness=0.22)
    cl = g.shader("ShaderNodeBsdfTransparent", Color=tint)
    mx = g.N.new("ShaderNodeMixShader")
    mx.inputs[0].default_value = clear_share
    g.L.new(pr.outputs[0], mx.inputs[1])
    g.L.new(cl.outputs[0], mx.inputs[2])
    return g.surface(mx.outputs[0])


def make_materials():
    m = {}
    m["skin"] = skin_material("kite_skin")
    m["skin_dead"] = skin_material("kite_skin_dead", dead=True)
    m["bone"] = bone_material("kite_bone")
    m["bone_dead"] = bone_material("kite_bone_dead", dead=True)
    m["reg"] = sl.new_material("registrar_black", REG, roughness=0.35, metallic=1.0)[0]
    m["gold"] = sl.new_material("registrar_etched_gold", GOLD, roughness=0.5, metallic=1.0)[0]
    m["lens"] = sl.new_material("registrar_amber_core", "#3a2206", roughness=0.2,
                                emission_hex=AMBER, emission_strength=6.0)[0]
    m["glass"] = lens_glass("registrar_lens_glass", (0.20, 0.17, 0.11, 1.0), 0.8)
    m["glass_dead"] = lens_glass("registrar_lens_glass_dead", (0.10, 0.06, 0.02, 1.0), 0.25)
    m["lens_dead"] = sl.new_material("registrar_amber_dead", "#1c1206", roughness=0.25)[0]
    m["thread"] = sl.new_material("kite_thread", RED, roughness=0.6)[0]
    m["thread_dead"] = sl.new_material("kite_thread_dead", "#5a1a16", roughness=0.8)[0]
    m["bead"] = sl.new_material("kite_bead", "#7d8794", roughness=0.35,
                                emission_hex=ELEC, emission_strength=1.0)[0]
    m["bead_dead"] = sl.new_material("kite_bead_dead", "#6a6862", roughness=0.6)[0]
    return m


# ---------------------------------------------------------------------------
# Skin
# ---------------------------------------------------------------------------
def notches(seed, count, deep, sign_side):
    rng = random.Random(seed)
    out = []
    for i in range(count):
        t = 0.07 + 0.86 * (i + rng.uniform(0.15, 0.85)) / count
        d = rng.uniform(0.035, 0.12) * deep
        if i == count // 3:
            d = 0.21 * deep                       # one big bite
        if sign_side and 0.22 < t < 0.6:
            d = min(d, 0.085)                     # the sign stays readable
        out.append((t, d, rng.uniform(0.022, 0.06)))
    return out


def tear_depth(nlist, t, seed):
    best = 0.0
    for tk, d, wdt in nlist:
        e = abs(t - tk) / wdt
        if e < 1.0:
            best = max(best, d * (1.0 - e * e) ** 0.8)
    if best > 0:
        best += 0.022 * math.sin(t * 120.0 + seed) * math.sin(t * 47.0 + 2 * seed) \
            + 0.012 * math.sin(t * 260.0 + 3 * seed)
    return max(0.0, best)


def in_hole(x, yr, holes):
    for hx, hy, hr, ph in holes:
        dx, dy = x - hx, yr - hy
        a = math.atan2(dy, dx)
        rr = hr * (1.0 + 0.42 * math.cos(2 * (a - ph)) + 0.14 * math.sin(3 * a + ph)
                   + 0.08 * math.sin(7 * a + 2 * ph))
        if dx * dx + dy * dy < rr * rr:
            return True
    return False


def build_skin(side, dead):
    """One half of the skin (side +1 is the Kite's left, +X). Returns a bmesh
    in flat layout: x, yr, and the height the living skin has."""
    n = N_SKIN
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.new("flat")
    rim = bm.verts.layers.float.new("rim")
    deep = 1.35 if dead else 1.0
    nl = notches(11 if side > 0 else 23, 11 if dead else 9, deep, side > 0)
    seed = 1.3 if side > 0 else 4.1
    holes = [(-0.34, 0.14, 0.05, 0.7), (0.15, 0.50, 0.034, 2.2)]
    if dead:
        holes += [(-0.2, 0.62, 0.06, 1.1), (0.2, -0.2, 0.04, 1.9)]
    verts = {}

    def vert(i, j, sy):
        key = (i, j * sy)
        if key not in verts:
            wx, wy = i / n, j / n
            wc = 1.0 - wx - wy
            if i > 0 and j > 0 and i + j < n:      # inner points: break up the grid
                h1 = math.sin(i * 12.9898 + j * sy * 78.233 + side) * 43758.5453
                h2 = math.sin(i * 39.346 + j * sy * 11.135 + side) * 24634.6345
                wx += 0.36 / n * ((h1 - math.floor(h1)) - 0.5)
                wy += 0.36 / n * ((h2 - math.floor(h2)) - 0.5)
                wc = 1.0 - wx - wy
            x, yr = side * wx * HALF, sy * wy * (LN if sy < 0 else LT)
            z = flat_h(x, yr) + 0.005 + 0.006 * wc
            if sy > 0 and wx + wy > 0:
                t = wy / (wx + wy)
                td = tear_depth(nl, t, seed)
                e = wc - td
                if td > 0.01 and e < 0.09:      # loose tatters hang a little
                    z -= 0.035 * (1.0 - max(0.0, e) / 0.09) ** 2 * (0.55 + 0.45 * math.sin(t * 95.0))
            v = bm.verts.new((x, yr, z))
            v[rim] = max(0.0, 1.0 - wc / 0.03)
            verts[key] = v
        return verts[key]

    for sy in (-1, 1):
        for i in range(n):
            for j in range(n - i):
                tris = [((i, j), (i + 1, j), (i, j + 1))]
                if i + j < n - 1:
                    tris.append(((i + 1, j), (i + 1, j + 1), (i, j + 1)))
                for tri in tris:
                    wx = sum(a for a, _ in tri) / (3.0 * n)
                    wy = sum(b for _, b in tri) / (3.0 * n)
                    wc = 1.0 - wx - wy
                    x, yr = side * wx * HALF, sy * wy * (LN if sy < 0 else LT)
                    if sy > 0 and wc < tear_depth(nl, wy / (wx + wy), seed):
                        continue
                    if in_hole(x, yr, holes):
                        continue
                    if dead:
                        # the skin rips open on the outside of the kink ...
                        if x < -0.012 and YK - 0.008 < yr < YK + 0.04 + 0.03 * abs(math.sin(x * 40)):
                            continue
                        # ... and along the back of the wing fold
                        if XF - 0.01 < x < XF + 0.035 + 0.02 * math.sin(yr * 50) and yr > 0.1:
                            continue
                    vs = [vert(a, b, sy) for a, b in tri]
                    e1, e2 = vs[1].co - vs[0].co, vs[2].co - vs[0].co
                    if e1.x * e2.y - e1.y * e2.x < 0:
                        vs.reverse()
                    f = bm.faces.new(vs)
                    f.smooth = True
                    for lp in f.loops:
                        lp[uvl].uv = (lp.vert.co.x, lp.vert.co.y)
    return bm


def skin_objects(coll, root, mats, dead):
    out = []
    for side, nm in ((1, "skin_left"), (-1, "skin_right")):
        bm = build_skin(side, dead)
        for v in bm.verts:
            if dead:
                v.co = place_dead(v.co.x, v.co.y, 0.0)
            else:
                px, py = pulled(v.co.x, v.co.y)
                v.co = Vector((px, CY + py, v.co.z))
        if dead:
            bmesh.ops.reverse_faces(bm, faces=bm.faces[:])   # the mirror turned them
        origin = place_dead(0, 0) if dead else place_live(0, 0)
        out.append(finish(bm, ("dead_" if dead else "") + nm, coll, root,
                          [mats["skin_dead" if dead else "skin"]], origin))
    return out


# ---------------------------------------------------------------------------
# Spars
# ---------------------------------------------------------------------------
def spar_radius(share):
    """share: 0 at the crossing, 1 at the tip."""
    r = R_TIP + (R_C - R_TIP) * (1.0 - share)
    return r


def knots(dist, r):
    k = (dist % 0.17) - 0.085
    return r * (1.0 + 0.22 * math.exp(-(k / 0.012) ** 2))


def spar_run(place, x0, y0, x1, y1, step=0.02, lift=0.0, point=True):
    """Points and radii of a cross spar from (x0, y0) to (x1, y1), flat."""
    a, b = Vector((x0, y0)), Vector((x1, y1))
    n = max(2, int(round((b - a).length / step)))
    pts, rad = [], []
    for i in range(n + 1):
        q = a.lerp(b, i / n)
        wx, wy, wc = weights(q.x, q.y)
        r = knots(q.length, spar_radius(1.0 - max(0.0, wc)))
        pts.append(place(q.x, q.y, lift))
        rad.append(r)
    if point:      # the tip thins to a point over the last few centimetres
        for i in range(n + 1):
            d = (b - a).length * (1 - i / n)
            if d < 0.04:
                rad[i] *= 0.3 + 0.7 * d / 0.04
    return pts, rad


def splinters(bm, pts, rad, at_end, seed):
    """A broken end: a few thin spikes."""
    rng = random.Random(seed)
    i0, i1 = (-1, -2) if at_end else (0, 1)
    d = (pts[i0] - pts[i1]).normalized()
    side = d.orthogonal()
    for k in range(4):
        a = k * 1.7 + rng.uniform(0, 0.6)
        off = (side * math.cos(a) + d.cross(side) * math.sin(a)) * rad[i0] * 0.55
        tip = pts[i0] + off * 1.6 + d * rng.uniform(0.012, 0.03)
        tube(bm, [pts[i0] + off - d * 0.004, tip], [rad[i0] * 0.45, 0.0006], sides=4, cap=False)


def spars(coll, root, mats, dead):
    place = place_dead if dead else place_live
    pre = "dead_" if dead else ""
    bone = [mats["bone_dead" if dead else "bone"]]
    lift = 0.012 if dead else 0.0
    c = place(0, 0, lift)
    out = []

    # main spar: nose to tail
    bm = bmesh.new()
    if dead:
        for seg, (ya, yb) in enumerate(((-LN, YK - 0.012), (YK + 0.034, LT))):
            pts, rad = spar_run(place, 0, ya, 0, yb, lift=lift, point=False)
            tube(bm, pts, rad)
            splinters(bm, pts, rad, seg == 0, 5 + seg)
    else:
        pts, rad = spar_run(place, 0, 0, 0, -LN)
        pts2, rad2 = spar_run(place, 0, 0, 0, LT)
        tube(bm, list(reversed(pts)) + pts2[1:], list(reversed(rad)) + rad2[1:])
    # a flat knuckle where the spars cross
    res = bmesh.ops.create_icosphere(bm, subdivisions=2, radius=1.0,
                                     matrix=Matrix.Translation(c) @ Matrix.Diagonal((0.021, 0.021, 0.0105, 1)))
    for f in {f for v in res["verts"] for f in v.link_faces}:
        f.smooth = True
    out.append(finish(bm, pre + "spar_main", coll, root, bone, c))

    # left wing spar
    bm = bmesh.new()
    if dead:
        for seg, (xa, xb) in enumerate(((0.0, XF - 0.012), (XF + 0.034, HALF))):
            pts, rad = spar_run(place, xa, 0, xb, 0, lift=lift, point=seg == 1)
            tube(bm, pts, rad)
            splinters(bm, pts, rad, seg == 0, 9 + seg)
    else:
        tube(bm, *spar_run(place, 0, 0, HALF, 0))
    out.append(finish(bm, pre + "spar_wing_left", coll, root, bone, c))

    # right wing spar: bone for the inner part, then a broken stub
    bm = bmesh.new()
    pts, rad = spar_run(place, 0, 0, ROD_X0 - 0.02, 0, lift=lift, point=False)
    tube(bm, pts, rad)
    splinters(bm, pts, rad, True, 3)
    wing_r = finish(bm, pre + "spar_wing_right", coll, root, bone, c)
    out.append(wing_r)

    # the wrong repair: a straight black Registrar rod for the outer third
    bm = bmesh.new()
    p0 = place(ROD_X0 + 0.10, 0.014, lift)
    p1 = place(-HALF + 0.0005, 0.0, lift)
    axis = (p1 - p0).normalized()
    bone_pt = place(ROD_X0 + 0.05, 0.0, lift)
    side = bone_pt - (p0 + axis * (bone_pt - p0).dot(axis))
    gap = side.length
    side.normalize()
    upv = axis.cross(side).normalized()
    rot = Matrix((axis, side, upv)).transposed()
    half = 0.006
    tube(bm, [p0, p1], half * math.sqrt(2.0), sides=4, smooth=False, up=side + upv)
    j0 = p0 + axis * 0.02 + side * gap * 0.5       # joint centre, inner collar
    j1 = p0 + axis * 0.10 + side * gap * 0.5       # outer collar
    for j in (j0, j1):
        box(bm, j, (0.022, gap + 0.034, 0.034), rot)
    wire = []
    turns, per = 9, 14
    for i in range(turns * per + 1):
        a = 2 * math.pi * i / per
        s = i / (turns * per)
        wire.append(j0.lerp(j1, 0.16 + 0.68 * s)
                    + side * math.cos(a) * (gap * 0.5 + 0.0135) + upv * math.sin(a) * 0.0135)
    tube(bm, wire, 0.0017, sides=4, smooth=True, cap=False)
    rod = finish(bm, pre + "repair_rod", coll, wing_r, [mats["reg"]], j0)
    rod.location = j0 - wing_r.location     # a child: placed from the wing spar's origin
    out.append(rod)
    # edge spar round the outline
    bm = bmesh.new()
    tips = [(0, -LN), (HALF, 0), (0, LT), (-HALF, 0)]
    for k in range(4):
        (xa, ya), (xb, yb) = tips[k], tips[(k + 1) % 4]
        pts, rad = [], []
        for i in range(25):
            s = i / 24
            q = 0.004 + (1.0 - 0.008) * s      # stops just short of the tip points
            pts.append(place(xa + (xb - xa) * q, ya + (yb - ya) * q, lift))
            rad.append(R_EDGE * (1.0 - 0.3 * math.sin(math.pi * s)) * (0.45 + 0.55 * min(1.0, 8 * s, 8 * (1 - s))))
        tube(bm, pts, rad, sides=6)
    out.append(finish(bm, pre + "spar_edge", coll, root, bone, c))
    return out


# ---------------------------------------------------------------------------
# Cage (Registrar hardware) and lenses
# ---------------------------------------------------------------------------
CAGE_A = CAGE_ACROSS / 2.0
CAGE_BAR = 0.008
CAGE_TOP = -0.036
CAGE_BOT = -0.136
LENS_AT = 0.046
BARREL_R = 0.030


def cage(coll, root, mats, at, dead):
    """Built hanging below its own origin (the crossing). Returns the cage
    object and the points on it where threads are tied."""
    pre = "dead_" if dead else ""
    bm = bmesh.new()
    a, b = CAGE_A - CAGE_BAR / 2, CAGE_BAR
    box(bm, (0, 0, (CAGE_TOP - 0.002) / 2), (0.052, 0.052, -CAGE_TOP - 0.002))      # clamp on the crossing
    for z in (CAGE_TOP, CAGE_BOT):
        for s in (-1, 1):
            box(bm, (0, s * a, z), (CAGE_ACROSS, b, b))
            box(bm, (s * a, 0, z), (b, CAGE_ACROSS, b))
    for sx in (-1, 1):
        for sy in (-1, 1):
            box(bm, (sx * a, sy * a, (CAGE_TOP + CAGE_BOT) / 2), (b, b, CAGE_TOP - CAGE_BOT))
    for ang in (45, -45):       # two flat straps from the corners to the clamp
        box(bm, (0, 0, CAGE_TOP), (CAGE_ACROSS * math.sqrt(2) - 0.02, b, b * 0.6),
            Matrix.Rotation(math.radians(ang), 3, "Z"))
    zg = -0.088
    for s in (-1, 1):           # the grid that carries the four lens barrels
        box(bm, (0, s * LENS_AT, zg), (CAGE_ACROSS - b, b * 0.7, b * 0.7))
        box(bm, (s * LENS_AT, 0, zg), (b * 0.7, CAGE_ACROSS - b, b * 0.7))
    ob = finish(bm, pre + "cage", coll, root, [mats["reg"]], (0, 0, 0))
    ob.location = at

    k = 0
    for sx in (-1, 1):
        for sy in (-1, 1):
            k += 1
            bm = bmesh.new()
            cylinder(bm, (0, 0, -0.012), BARREL_R, 0.064, mat=0, caps=False)
            cylinder(bm, (0, 0, 0.018), BARREL_R, 0.004, mat=0)            # closed top
            cylinder(bm, (0, 0, -0.012), BARREL_R - 0.002, 0.003, mat=0)   # dark floor inside
            cylinder(bm, (0, 0, -0.0145), 0.011, 0.002, segments=16, mat=2)  # the amber core
            for z in (-0.036, -0.040):          # two etched measuring circles
                cylinder(bm, (0, 0, z), BARREL_R + 0.0005, 0.0014, mat=1, caps=False)
            dome(bm, (0, 0, -0.044), BARREL_R - 0.002, 0.007, mat=3)
            lens = finish(bm, pre + "lens_%d" % k, coll, ob,
                          [mats["reg"], mats["gold"], mats["lens_dead" if dead else "lens"],
                           mats["glass_dead" if dead else "glass"]],
                          (0, 0, 0), sharp_deg=40)
            lens.location = (sx * LENS_AT, sy * LENS_AT, zg)
            lens.rotation_euler = (math.radians(-6 * sy), math.radians(6 * sx), 0)
    ties = [Vector((x, a, CAGE_BOT)) for x in (-0.08, -0.04, 0.0, 0.04, 0.08)]
    ties += [Vector((-a, 0.05, CAGE_BOT)), Vector((a, 0.05, CAGE_BOT))]
    return ob, ties


# ---------------------------------------------------------------------------
# Threads and beads
# ---------------------------------------------------------------------------
SEG = 0.03


def thread_object(name, coll, parent, mats, pts, rng, dead):
    bm = bmesh.new()
    n = len(pts)
    rad = [0.0017 - 0.0009 * i / (n - 1) for i in range(n)]
    tube(bm, pts, rad, sides=4, mat=0, cap=False)
    count = rng.randint(6, 8)
    for k in range(count):
        s = (k + rng.uniform(0.25, 0.75)) / count
        i = min(n - 1, max(2, int(s * (n - 1))))
        r = rng.uniform(0.004, 0.006) if k < count - 1 else 0.0075
        at = Vector(pts[i])
        if dead:
            at.z = max(at.z, r)             # a bead rests on the ground, not in it
        res = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=r,
                                         matrix=Matrix.Translation(at))
        for f in {f for v in res["verts"] for f in v.link_faces}:
            f.material_index = 1
            f.smooth = True
    return finish(bm, name, coll, parent,
                  [mats["thread_dead" if dead else "thread"], mats["bead_dead" if dead else "bead"]],
                  pts[0])


def hanging_path(rng, start, unbank, spread):
    """A thread that hangs and trails back (+Y), exactly THREAD_LEN long."""
    n = int(round(THREAD_LEN / SEG))
    phi0 = math.radians(rng.uniform(6, 16))
    phi1 = math.radians(rng.uniform(34, 58))
    yaw0 = spread + rng.uniform(-0.12, 0.12)
    ph, amp, fr = rng.uniform(0, 6.28), rng.uniform(0.12, 0.3), rng.uniform(4.0, 7.0)
    pts = [Vector(start)]
    for k in range(n):
        s = (k + 0.5) / n
        phi = phi0 + (phi1 - phi0) * s ** 1.3 + 0.13 * math.sin(fr * s * 2.2 + ph)
        yaw = yaw0 * (1.0 - 0.5 * s) + amp * math.sin(fr * s * 1.6 + 2 * ph) * s
        d = Vector((math.sin(phi) * math.sin(yaw), math.sin(phi) * math.cos(yaw), -math.cos(phi)))
        pts.append(pts[-1] + (unbank @ d) * SEG)
    return pts


def over_body(p):
    """0 on the dead body, 1 on open ground (worked out in the flat layout)."""
    q = p - DEAD_SHIFT
    x, yr = -q.x / SHRINK, q.y / SHRINK
    f = abs(x) / HALF + (-yr / LN if yr < 0 else yr / LT)
    return smoothstep(0.9, 1.06, f)


def tangled_path(rng, start, heading, layer):
    """A thread lying where it fell: loops on the body and on the ground."""
    n = int(round(THREAD_LEN / SEG))
    pts = [Vector(start)]
    th = heading
    f1, f2 = rng.uniform(2.0, 5.0), rng.uniform(6.0, 11.0)
    p1, p2 = rng.uniform(0, 6.28), rng.uniform(0, 6.28)
    a1, a2 = rng.uniform(6.0, 11.0), rng.uniform(4.0, 8.0)
    for k in range(n):
        s = k * SEG
        th += (a1 * math.sin(f1 * s + p1) + a2 * math.sin(f2 * s + p2)) * SEG
        cur = pts[-1]
        probe = cur + Vector((math.cos(th), math.sin(th), 0)) * SEG
        rest = 0.045 * (1.0 - over_body(probe)) + 0.0035 + 0.0032 * layer
        z = max(rest, cur.z - SEG * 0.75)
        dz = z - cur.z
        flat = math.sqrt(max(1e-8, SEG * SEG - dz * dz))
        pts.append(Vector((cur.x + math.cos(th) * flat, cur.y + math.sin(th) * flat, z)))
    return pts


# ---------------------------------------------------------------------------
# Assembly
# ---------------------------------------------------------------------------
def build_live(coll, mats):
    root = bpy.data.objects.new(NAME, None)
    root.empty_display_size = 0.3
    coll.objects.link(root)
    bank = Matrix.Rotation(math.radians(BANK_DEG), 3, "Y")     # left wing down
    unbank = bank.inverted()
    body = skin_objects(coll, root, mats, False) + spars(coll, root, mats, False)
    cg, ties = cage(coll, root, mats, place_live(0, 0), False)
    body.append(cg)
    rng = random.Random(77)
    k = 0
    for i, t in enumerate(ties):
        k += 1
        start = Vector(cg.location) + t
        pts = hanging_path(rng, start, unbank, (t.x / CAGE_A) * 0.35)
        ob = thread_object("thread_cage_%d" % (i + 1), coll, cg, mats, pts, rng, False)
        ob.location = t
    for i in range(THREAD_COUNT - len(ties)):
        start = place_live(0, LT - 0.012 - 0.02 * i, -0.004)
        pts = hanging_path(rng, start, unbank, (i - 2) * 0.16)
        thread_object("thread_tail_%d" % (i + 1), coll, root, mats, pts, rng, False)
    root.rotation_euler = (0, math.radians(BANK_DEG), 0)
    bpy.context.view_layer.update()
    root.location = (0, 0, FLY - body_low(body))
    bpy.context.view_layer.update()
    return root, body


def body_low(body):
    """Lowest point of the body: skin, spars, cage and lenses, not the threads."""
    bpy.context.view_layer.update()
    objs = list(body) + [c for o in body for c in o.children if "thread" not in c.name]
    return min((o.matrix_world @ v.co).z for o in objs for v in o.data.vertices)


def build_dead(coll, mats):
    root = bpy.data.objects.new("dead_" + NAME, None)
    root.empty_display_size = 0.3
    coll.objects.link(root)
    skin_objects(coll, root, mats, True)
    spars(coll, root, mats, True)
    at = place_dead(0, 0, 0.024)
    cg, ties = cage(coll, root, mats, at, True)
    cg.rotation_euler = (math.radians(11), math.radians(180 - 8), math.radians(17))
    for ch in cg.children:      # the lenses were knocked out of line
        ch.rotation_euler.x += math.radians(5 if "1" in ch.name or "4" in ch.name else -7)
    m = Matrix.Translation(at) @ cg.rotation_euler.to_matrix().to_4x4()
    rng = random.Random(123)
    for i, t in enumerate(ties):
        start = m @ t
        pts = tangled_path(rng, start, math.radians(40 + 38 * i + rng.uniform(-15, 15)), i)
        thread_object("dead_thread_cage_%d" % (i + 1), coll, root, mats, pts, rng, True)
    for i in range(THREAD_COUNT - len(ties)):
        start = place_dead(0, LT - 0.012 - 0.02 * i, 0.016)
        pts = tangled_path(rng, start, math.radians(120 + 30 * i + rng.uniform(-20, 20)), 7 + i)
        thread_object("dead_thread_tail_%d" % (i + 1), coll, root, mats, pts, rng, True)
    root.rotation_euler = (0, 0, math.radians(-14))
    return root


def main():
    sl.reset_scene()
    mats = make_materials()
    model = new_collection("model")
    dead = new_collection("dead")
    root, body = build_live(model, mats)
    droot = build_dead(dead, mats)
    dead.hide_render = True

    data = sl.write_size_json(os.path.join(HERE, "size.json"), NAME, [root])
    # extra facts, measured: the true span and length (without the bank and
    # the threads), the height of the lowest point of the body, thread length.
    flat = [o for o in body if "spar" in o.name or "skin" in o.name]
    keep = root.rotation_euler.copy()
    root.rotation_euler = (0, 0, 0)
    m = sl.measure(flat)
    root.rotation_euler = keep
    bpy.context.view_layer.update()
    data["span_m"] = m["width_m"]
    data["length_m"] = m["depth_m"]
    data["flight_height_m"] = round(body_low(body), 3)
    th = bpy.data.objects["thread_tail_1"].data
    data["thread_length_m"] = round(sum(
        (th.vertices[4 * (i + 1)].co - th.vertices[4 * i].co).length
        for i in range(int(round(THREAD_LEN / SEG)))), 3)
    data["dead_triangles"] = sl.measure([droot])["triangles"]
    with open(os.path.join(HERE, "size.json"), "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
        f.write("\n")
    print("KITE live:", data)
    print("KITE dead:", sl.measure([droot]))
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(HERE, "model.blend"), compress=True)


main()
