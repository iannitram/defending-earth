"""Job C03: builds the Lash from nothing and saves model.blend.

Run from the project folder:
  flatpak run org.blender.Blender -b --python "<full path>/build.py"

The file holds two collections:
  model  the living Lash, standing, at true size, facing -Y
  dead   the dead form (hidden from render until render.py shows it)
"""
import sys
import os
import math
import json
import random

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "scale")))

import bpy
import bmesh
from mathutils import Vector, Matrix

import scale_lib as sl

# ---------------------------------------------------------------------------
# Sizes. The first block comes from the scale sheet. The second block comes
# from the job brief. The third block holds the small shape choices the
# brief leaves to the modeller ("thin", "short", "narrow").
# ---------------------------------------------------------------------------
T = sl.thing("lash")
TRUNK_LEN = T["trunk_length_m"]
BACK_H = T["back_height_m"]
TOP_H = T["height_m"]
STANCE_W = T["width_m"]
RIFLE_LEN = T["rifle_length_m"]
FOOT = T["footprint"]

TRUNK_W = 0.45          # brief
N_SEG = 9               # brief
N_LEGS = 7              # brief: seven pairs
ARM_LEN = 1.0           # brief
MASK_H = 0.30           # brief
HALO_D = 0.50           # brief
SPINE_LEN_R = [1.10, 1.00, 0.90, 0.85, 0.75, 0.70, 0.60]   # brief: 0.6 to 1.1
SPINE_LEN_L = [1.05, 1.00, 0.95, 0.80, 0.80, 0.65, 0.60]
SAC_SIZES = [0.12, 0.09, 0.11, 0.08, 0.10, 0.12, 0.09, 0.08]  # brief: 8 to 12 cm
STAVE_LEN = 1.2         # brief
STRIP_LEN = 0.25        # brief

HALF_H = 0.18           # half the trunk's depth, top to belly
RIDGE = 0.012           # low ridge along the back
SPIKE = 0.07            # halo spikes ("short")
ARC_T = 0.028           # halo arc thickness
HEAD_FWD, HEAD_UP = 0.16, 0.05   # head centre from the neck joint
MASK_D = 0.075          # deepest point of the mask's front
GAP = 0.012             # half the gap between two trunk plates

HIDE = "#b89a94"
RED = "#8c1f1a"
WEED = "#3c4630"
BRASS = "#9c7a32"
STRIP = "#d8d2c2"
BONE = "#cfc6b0"        # the "pale" mask and shoulder plate
FRAME = "#14171a"
REG = "#0b0b0c"
GOLD = "#8a6a2a"
AMBER = "#ffa01e"
ELEC = "#cfe4ff"
GUN = "#26282a"         # the rifle's "dark metal"
STAVE = "#4a3d2c"       # bipod staves
CORD = "#30291f"

X = Vector((1, 0, 0))
Y = Vector((0, 1, 0))
Z = Vector((0, 0, 1))


def lerp(a, b, k):
    return a + (b - a) * k


def pl(x, pts):
    """Smooth curve through (x, y) knots."""
    if x <= pts[0][0]:
        return pts[0][1]
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        if x <= x1:
            k = (x - x0) / (x1 - x0)
            k = k * k * (3 - 2 * k)
            return y0 + (y1 - y0) * k
    return pts[-1][1]


# ---------------------------------------------------------------------------
# Materials
# ---------------------------------------------------------------------------
def sock(node, name, kind=None, out=False):
    coll = node.outputs if out else node.inputs
    hits = [s for s in coll if s.name == name and (kind is None or s.type == kind)]
    if not hits:
        raise KeyError("%s has no socket %s" % (node.bl_idname, name))
    return hits[0]


def setp(bsdf, **kw):
    for k, v in kw.items():
        name = k.replace("_", " ")
        if name in bsdf.inputs:
            bsdf.inputs[name].default_value = v
        else:
            print("lash build: Principled BSDF has no input '%s'" % name)


def mth(nt, op, a, b=None, c=None):
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


def mix(nt, fac, a, b):
    n = nt.nodes.new("ShaderNodeMix")
    n.data_type = "RGBA"
    for s, v in ((sock(n, "Factor", "VALUE"), fac), (sock(n, "A", "RGBA"), a),
                 (sock(n, "B", "RGBA"), b)):
        if isinstance(v, (int, float, tuple)):
            s.default_value = v
        else:
            nt.links.new(v, s)
    return sock(n, "Result", "RGBA", out=True)


def noise(nt, vec, scale, detail=4.0, rough=0.55):
    n = nt.nodes.new("ShaderNodeTexNoise")
    n.inputs["Scale"].default_value = scale
    n.inputs["Detail"].default_value = detail
    n.inputs["Roughness"].default_value = rough
    nt.links.new(vec, n.inputs["Vector"])
    return n.outputs["Fac"]


def bump(nt, bsdf, height, strength, distance):
    n = nt.nodes.new("ShaderNodeBump")
    n.inputs["Strength"].default_value = strength
    n.inputs["Distance"].default_value = distance
    nt.links.new(height, n.inputs["Height"])
    nt.links.new(n.outputs["Normal"], bsdf.inputs["Normal"])


def obj_coords(nt):
    return nt.nodes.new("ShaderNodeTexCoord").outputs["Object"]


def uv_split(nt):
    uv = nt.nodes.new("ShaderNodeUVMap")
    sep = nt.nodes.new("ShaderNodeSeparateXYZ")
    nt.links.new(uv.outputs["UV"], sep.inputs["Vector"])
    return sep.outputs["X"], sep.outputs["Y"]


def m_hide(name, mode):
    """Hide: pale flesh-pink to grey, matte, slightly damp.
    mode 'trunk' adds thin red lines along the plate, 'limb' thin red rings."""
    mat, bsdf = sl.new_material(name, HIDE, roughness=0.62)
    nt = mat.node_tree
    oc = obj_coords(nt)
    big = noise(nt, oc, 4.0, 4.0)
    fine = noise(nt, oc, 60.0, 3.0, 0.7)
    k = nt.nodes.new("ShaderNodeMapRange")
    k.inputs["From Min"].default_value = 0.35
    k.inputs["From Max"].default_value = 0.70
    nt.links.new(big, k.inputs["Value"])
    col = mix(nt, k.outputs["Result"], sl.rgb(HIDE), sl.rgb("#8f8988"))
    # darker damp blotches and fine speckle
    mid = noise(nt, oc, 17.0, 5.0, 0.65)
    b2 = nt.nodes.new("ShaderNodeMapRange")
    b2.inputs["From Min"].default_value = 0.52
    b2.inputs["From Max"].default_value = 0.72
    nt.links.new(mid, b2.inputs["Value"])
    col = mix(nt, mth(nt, "MULTIPLY", b2.outputs["Result"], 0.8), col, sl.rgb("#7c625e"))
    col = mix(nt, mth(nt, "MULTIPLY", mth(nt, "GREATER_THAN", fine, 0.64), 0.35),
              col, sl.rgb("#6f5d5a"))
    u, v = uv_split(nt)
    if mode == "trunk":
        au = mth(nt, "ABSOLUTE", u)
        # the flanks and belly are greyer and darker, where the wet sits
        low = nt.nodes.new("ShaderNodeMapRange")
        low.inputs["From Min"].default_value = 0.17
        low.inputs["From Max"].default_value = 0.36
        nt.links.new(au, low.inputs["Value"])
        col = mix(nt, mth(nt, "MULTIPLY", low.outputs["Result"], 0.7), col, sl.rgb("#6b6261"))
        f = mth(nt, "SUBTRACT", mth(nt, "FRACT", mth(nt, "ADD",
                mth(nt, "DIVIDE", mth(nt, "SUBTRACT", au, 0.045), 0.07), 0.5)), 0.5)
        line = mth(nt, "LESS_THAN", mth(nt, "ABSOLUTE", f), 0.06)
        line = mth(nt, "MULTIPLY", line, mth(nt, "LESS_THAN", au, 0.215))
        line = mth(nt, "MULTIPLY", line, mth(nt, "GREATER_THAN", v, 0.16))
        line = mth(nt, "MULTIPLY", line, mth(nt, "LESS_THAN", v, 0.90))
        band = mth(nt, "LESS_THAN", mth(nt, "ABSOLUTE", mth(nt, "SUBTRACT", v, 0.10)), 0.011)
        band = mth(nt, "MULTIPLY", band, mth(nt, "LESS_THAN", au, 0.27))
        col = mix(nt, mth(nt, "MULTIPLY", mth(nt, "MAXIMUM", line, band), 0.92),
                  col, sl.rgb(RED))
    elif mode == "limb":
        r1 = mth(nt, "LESS_THAN", mth(nt, "ABSOLUTE", mth(nt, "SUBTRACT", v, 0.20)), 0.006)
        r2 = mth(nt, "LESS_THAN", mth(nt, "ABSOLUTE", mth(nt, "SUBTRACT", v, 0.80)), 0.006)
        col = mix(nt, mth(nt, "MULTIPLY", mth(nt, "MAXIMUM", r1, r2), 0.9), col, sl.rgb(RED))
    nt.links.new(col, bsdf.inputs["Base Color"])
    bump(nt, bsdf, mth(nt, "ADD", fine, mth(nt, "MULTIPLY", mid, 2.0)), 0.35, 0.008)
    setp(bsdf, Coat_Weight=0.06, Coat_Roughness=0.35)
    bsdf.inputs["Roughness"].default_value = 0.7
    return mat


def m_spine_reed():
    """The reed-like spines: hide colour at the root, grey-brown at the tip."""
    mat, bsdf = sl.new_material("lash_spine_reed", HIDE, roughness=0.5)
    nt = mat.node_tree
    u, v = uv_split(nt)
    col = mix(nt, v, sl.rgb("#94786f"), sl.rgb("#4a403a"))
    ring = mth(nt, "LESS_THAN", mth(nt, "ABSOLUTE", mth(nt, "SUBTRACT",
               mth(nt, "FRACT", mth(nt, "MULTIPLY", v, 5.0)), 0.5)), 0.02)
    col = mix(nt, mth(nt, "MULTIPLY", ring, 0.8), col, sl.rgb("#3a2c28"))
    nt.links.new(col, bsdf.inputs["Base Color"])
    return mat


def m_frame():
    """SPINE: the frame under the flesh. Regular lattice, 6 cm cells, wet."""
    mat, bsdf = sl.new_material("lash_SPINE_frame", FRAME, roughness=0.3)
    nt = mat.node_tree
    oc = obj_coords(nt)
    sc = nt.nodes.new("ShaderNodeVectorMath")
    sc.operation = "SCALE"
    sc.inputs["Scale"].default_value = 1.0 / 0.06
    nt.links.new(oc, sc.inputs[0])
    fr = nt.nodes.new("ShaderNodeVectorMath")
    fr.operation = "FRACTION"
    nt.links.new(sc.outputs["Vector"], fr.inputs[0])
    sep = nt.nodes.new("ShaderNodeSeparateXYZ")
    nt.links.new(fr.outputs["Vector"], sep.inputs["Vector"])
    bar = None
    for a in "XYZ":
        d = mth(nt, "ABSOLUTE", mth(nt, "SUBTRACT", sep.outputs[a], 0.5))
        m = mth(nt, "GREATER_THAN", d, 0.36)
        bar = m if bar is None else mth(nt, "MAXIMUM", bar, m)
    col = mix(nt, bar, sl.rgb("#050607"), sl.rgb(FRAME))
    nt.links.new(col, bsdf.inputs["Base Color"])
    bump(nt, bsdf, bar, 0.8, 0.012)
    return mat


def m_weed():
    mat, bsdf = sl.new_material("lash_weed", WEED, roughness=0.45)
    nt = mat.node_tree
    geo = nt.nodes.new("ShaderNodeNewGeometry")
    k = geo.outputs["Random Per Island"]
    col = mix(nt, k, sl.rgb("#262d1f"), sl.rgb("#474f36"))
    nt.links.new(col, bsdf.inputs["Base Color"])
    return mat


def m_gun():
    mat, bsdf = sl.new_material("lash_rifle_metal", GUN, roughness=0.48, metallic=0.85)
    nt = mat.node_tree
    n = noise(nt, obj_coords(nt), 90.0, 3.0, 0.7)
    bump(nt, bsdf, n, 0.15, 0.002)
    r = nt.nodes.new("ShaderNodeMapRange")
    r.inputs["To Min"].default_value = 0.38
    r.inputs["To Max"].default_value = 0.62
    nt.links.new(n, r.inputs["Value"])
    nt.links.new(r.outputs["Result"], bsdf.inputs["Roughness"])
    return mat


def m_stave():
    mat, bsdf = sl.new_material("lash_stave", STAVE, roughness=0.75)
    nt = mat.node_tree
    w = nt.nodes.new("ShaderNodeTexWave")
    w.inputs["Scale"].default_value = 14.0
    w.inputs["Distortion"].default_value = 6.0
    w.inputs["Detail"].default_value = 2.0
    nt.links.new(obj_coords(nt), w.inputs["Vector"])
    col = mix(nt, w.outputs["Fac"], sl.rgb("#3a2f22"), sl.rgb("#57493a"))
    nt.links.new(col, bsdf.inputs["Base Color"])
    return mat


def make_materials():
    M = {}
    M["hide"] = m_hide("lash_hide_trunk", "trunk")
    M["limb"] = m_hide("lash_hide_limb", "limb")
    M["plain"] = m_hide("lash_hide_plain", "none")
    M["spine"] = m_spine_reed()
    M["frame"] = m_frame()
    M["reg"], _ = sl.new_material("lash_registrar_black", REG, roughness=0.35, metallic=1.0)
    M["gold"], _ = sl.new_material("lash_registrar_etch", GOLD, roughness=0.55, metallic=1.0)
    M["lens"], b = sl.new_material("lash_lens_amber", AMBER, roughness=0.2,
                                   emission_hex=AMBER, emission_strength=6.0)
    M["lens_dark"], _ = sl.new_material("lash_lens_dark", "#1c1408", roughness=0.15)
    M["arc"], b = sl.new_material("lash_electrical", ELEC, roughness=0.5,
                                  emission_hex=ELEC, emission_strength=4.0)
    M["bone"], b = sl.new_material("lash_mask_bone", BONE, roughness=0.5)
    nt = M["bone"].node_tree
    n = noise(nt, obj_coords(nt), 25.0, 4.0, 0.6)
    col = mix(nt, n, sl.rgb("#b3a78d"), sl.rgb("#d6cdb8"))
    n2 = noise(nt, obj_coords(nt), 9.0, 6.0, 0.7)
    st = nt.nodes.new("ShaderNodeMapRange")
    st.inputs["From Min"].default_value = 0.5
    st.inputs["From Max"].default_value = 0.75
    nt.links.new(n2, st.inputs["Value"])
    col = mix(nt, mth(nt, "MULTIPLY", st.outputs["Result"], 0.6), col, sl.rgb("#8d7c62"))
    vor = nt.nodes.new("ShaderNodeTexVoronoi")
    vor.feature = "DISTANCE_TO_EDGE"
    vor.inputs["Scale"].default_value = 22.0
    nt.links.new(obj_coords(nt), vor.inputs["Vector"])
    crack = mth(nt, "LESS_THAN", vor.outputs["Distance"], 0.012)
    crack = mth(nt, "MULTIPLY", crack, mth(nt, "GREATER_THAN", n2, 0.45))
    col = mix(nt, mth(nt, "MULTIPLY", crack, 0.7), col, sl.rgb("#5a4c3a"))
    nt.links.new(col, b.inputs["Base Color"])
    bump(nt, b, mth(nt, "SUBTRACT", mth(nt, "ADD", n, n2), crack), 0.3, 0.003)
    M["red"], _ = sl.new_material("lash_red_sign", RED, roughness=0.75)
    M["sac"], b = sl.new_material("lash_sac", RED, roughness=0.42)
    setp(b, Subsurface_Weight=0.12, Subsurface_Scale=0.02,
         Coat_Weight=0.15, Coat_Roughness=0.3)
    if "Subsurface Radius" in b.inputs:
        b.inputs["Subsurface Radius"].default_value = (1.0, 0.25, 0.15)
    M["weed"] = m_weed()
    M["gun"] = m_gun()
    M["glass"], _ = sl.new_material("lash_scope_glass", "#0a0d10", roughness=0.06)
    M["stave"] = m_stave()
    M["cord"], _ = sl.new_material("lash_cord", CORD, roughness=0.9)
    M["brass"], _ = sl.new_material("lash_brass", BRASS, roughness=0.58, metallic=1.0)
    M["strip"], _ = sl.new_material("lash_prayer_strip", STRIP, roughness=0.85)
    return M


# ---------------------------------------------------------------------------
# Mesh tools
# ---------------------------------------------------------------------------
class B:
    """One mesh being built."""

    def __init__(self):
        self.bm = bmesh.new()
        self.mats = []
        self.uv = self.bm.loops.layers.uv.new("uv")

    def mi(self, mat):
        if mat not in self.mats:
            self.mats.append(mat)
        return self.mats.index(mat)

    def v(self, p):
        return self.bm.verts.new(p)

    def face(self, vs, mat, smooth=True, uvs=None):
        try:
            f = self.bm.faces.new(vs)
        except ValueError:
            return None
        f.material_index = self.mi(mat)
        f.smooth = smooth
        if uvs:
            for l, uv in zip(f.loops, uvs):
                l[self.uv].uv = uv
        return f


def tube(b, pts, radii, mat, sides=8, smooth=True, caps=True, hint=None, rot=0.0,
         sx=1.0, sy=1.0):
    """Sweeps a ring along pts. Works as a lathe, a limb, a bar (sides=4)."""
    pts = [Vector(p) for p in pts]
    n = len(pts)
    if not isinstance(radii, (list, tuple)):
        radii = [radii] * n
    rings, prev = [], None
    for i, p in enumerate(pts):
        t = pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]
        t.normalize()
        if prev is not None:
            ref = prev
        elif hint is not None:
            ref = Vector(hint)
        else:
            ref = Z if abs(t.z) < 0.9 else X
        nr = ref - t * ref.dot(t)
        if nr.length < 1e-6:
            nr = t.orthogonal()
        nr.normalize()
        bn = t.cross(nr)
        prev = nr
        rings.append([b.v(p + (nr * math.cos(rot + 2 * math.pi * k / sides) * sx
                               + bn * math.sin(rot + 2 * math.pi * k / sides) * sy) * radii[i])
                      for k in range(sides)])
    for i in range(n - 1):
        v0, v1 = i / (n - 1), (i + 1) / (n - 1)
        for k in range(sides):
            k2 = (k + 1) % sides
            b.face([rings[i][k], rings[i][k2], rings[i + 1][k2], rings[i + 1][k]], mat, smooth,
                   uvs=[(k / sides, v0), ((k + 1) / sides, v0), ((k + 1) / sides, v1), (k / sides, v1)])
    if caps:
        b.face(rings[0][::-1], mat, False)
        b.face(rings[-1], mat, False)
    return rings


def ball(b, c, r, mat, seg=10, rings=6, scale=(1, 1, 1), axes=(X, Y, Z), smooth=True, top=1.0):
    res = bmesh.ops.create_uvsphere(b.bm, u_segments=seg, v_segments=rings, radius=1.0)
    c = Vector(c)
    faces = []
    for v in res["verts"]:
        for f in v.link_faces:
            if f not in faces:
                faces.append(f)
        k = top if v.co.z > 0 else 1.0
        q = v.co.copy()
        shrink = 1.0 - (1.0 - 1.0 / top) * max(q.z, 0.0) if top != 1.0 else 1.0
        v.co = c + (axes[0] * q.x * scale[0] * shrink + axes[1] * q.y * scale[1] * shrink
                    + axes[2] * q.z * scale[2] * k) * r
    mi = b.mi(mat)
    for f in faces:
        f.material_index = mi
        f.smooth = smooth


def box(b, c, size, mat, axes=(X, Y, Z)):
    c = Vector(c)
    vs = {}
    for i in (-1, 1):
        for j in (-1, 1):
            for k in (-1, 1):
                vs[(i, j, k)] = b.v(c + axes[0] * i * size[0] / 2 + axes[1] * j * size[1] / 2
                                    + axes[2] * k * size[2] / 2)
    q = [((-1, -1, -1), (-1, -1, 1), (-1, 1, 1), (-1, 1, -1)),
         ((1, -1, -1), (1, 1, -1), (1, 1, 1), (1, -1, 1)),
         ((-1, -1, -1), (1, -1, -1), (1, -1, 1), (-1, -1, 1)),
         ((-1, 1, -1), (-1, 1, 1), (1, 1, 1), (1, 1, -1)),
         ((-1, -1, -1), (-1, 1, -1), (1, 1, -1), (1, -1, -1)),
         ((-1, -1, 1), (1, -1, 1), (1, 1, 1), (-1, 1, 1))]
    for f in q:
        b.face([vs[k] for k in f], mat, False)


def ribbon(b, pts, nrms, width, mat, taper=0.5):
    """Flat strand along pts. Returns its frames for decals."""
    n = len(pts)
    frames, left, right = [], [], []
    for i, p in enumerate(pts):
        t = pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]
        if t.length < 1e-6:
            t = Z.copy()
        t.normalize()
        side = t.cross(nrms[i])
        if side.length < 1e-3:
            side = t.orthogonal()
        side.normalize()
        nr = side.cross(t).normalized()
        w = width * (1 - (1 - taper) * (i / max(1, n - 1)) ** 2) / 2
        left.append(b.v(p - side * w))
        right.append(b.v(p + side * w))
        frames.append((p.copy(), side, nr))
    for i in range(n - 1):
        b.face([left[i], right[i], right[i + 1], left[i + 1]], mat, True)
    return frames


def limb(b, p0, p1, r0, r1, mat, bow=None, sides=8):
    """A living limb: swollen at both joints, lean between, a little bowed."""
    p0, p1 = Vector(p0), Vector(p1)
    L = (p1 - p0).length
    bow = Vector(bow) if bow is not None else Vector((0, 0, 0))
    ts = [0.0, 0.05, 0.14, 0.32, 0.55, 0.78, 0.90, 0.96, 1.0]
    ks = [1.0, 1.22, 1.18, 0.92, 0.84, 0.88, 1.12, 1.24, 1.12]
    pts = [p0.lerp(p1, t) + bow * (L * math.sin(math.pi * t)) for t in ts]
    radii = [lerp(r0, r1, t) * k for t, k in zip(ts, ks)]
    tube(b, pts, radii, mat, sides=sides)


def plane_place(c, ex, ey, en, lift=0.0015):
    c = Vector(c)
    return lambda x, y: c + ex * x + ey * y + en * lift


def annulus(b, place, r0, r1, mat, seg=28):
    a = [b.v(place(r0 * math.cos(2 * math.pi * k / seg), r0 * math.sin(2 * math.pi * k / seg)))
         for k in range(seg)]
    c = [b.v(place(r1 * math.cos(2 * math.pi * k / seg), r1 * math.sin(2 * math.pi * k / seg)))
         for k in range(seg)]
    for k in range(seg):
        k2 = (k + 1) % seg
        b.face([a[k], a[k2], c[k2], c[k]], mat, False)


def disc(b, place, r, mat, seg=12):
    mid = b.v(place(0, 0))
    ring = [b.v(place(r * math.cos(2 * math.pi * k / seg), r * math.sin(2 * math.pi * k / seg)))
            for k in range(seg)]
    for k in range(seg):
        b.face([mid, ring[k], ring[(k + 1) % seg]], mat, False)


def stroke(b, place, p0, p1, w, mat, n=5):
    """Straight line of width w from p0 to p1, in the sign's own (x, y)."""
    p0, p1 = Vector(p0), Vector(p1)
    d = (p1 - p0).normalized()
    s = Vector((-d.y, d.x)) * (w / 2)
    prev = None
    for i in range(n + 1):
        q = p0.lerp(p1, i / n)
        cur = (b.v(place(q.x - s.x, q.y - s.y)), b.v(place(q.x + s.x, q.y + s.y)))
        if prev:
            b.face([prev[0], prev[1], cur[1], cur[0]], mat, False)
        prev = cur


def target_sign(b, place, R, mat):
    """The Lash's own sign: two thin rings, a centre dot, one reed stroke
    that crosses the rings' lower edge and runs on below."""
    annulus(b, place, R, R * 0.84, mat)
    annulus(b, place, R * 0.60, R * 0.45, mat)
    disc(b, place, R * 0.17, mat)
    stroke(b, place, (0, -R * 0.32), (0, -R * 1.9), R * 0.15, mat)


def reed_sign(b, place, R, mat):
    """A reed: one upright stroke with two short leaves."""
    stroke(b, place, (0, -R), (0, R), R * 0.16, mat, n=3)
    stroke(b, place, (0, -R * 0.1), (R * 0.5, R * 0.55), R * 0.13, mat, n=1)
    stroke(b, place, (0, -R * 0.5), (-R * 0.45, R * 0.1), R * 0.13, mat, n=1)


# ---------------------------------------------------------------------------
# Things that hang
# ---------------------------------------------------------------------------
class Env:
    """Which way is down, and where the ground is, in the space being built."""

    def __init__(self, g, ground, collider=None, x_limit=None):
        self.G = Vector(g).normalized()
        self.UP = -self.G
        self.GROUND = ground
        self.collider = collider
        self.x_limit = x_limit

    def height(self, p):
        return p.dot(self.UP) - self.GROUND


def rand_vec(rng):
    return Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)))


def hang_path(env, p0, n_out, length, rng, step=0.05, sway=0.25, lift=0.012):
    """Path of a limp strand from p0. Falls, slides round the body, lies on
    the ground. Returns (points, normals)."""
    p = Vector(p0)
    n_out = Vector(n_out).normalized()
    d = env.G - n_out * env.G.dot(n_out)
    if d.length < 0.2:
        d = n_out.cross(Y if abs(n_out.y) < 0.9 else X) * rng.choice((-1, 1))
    d.normalize()
    cur_n = n_out
    pts, nrms = [p.copy()], [cur_n.copy()]
    lay = None
    for _ in range(max(1, int(round(length / step)))):
        if env.height(p) <= lift + 1e-4:
            if lay is None:
                lay = d - env.UP * d.dot(env.UP)
                if lay.length < 0.3:
                    lay = n_out - env.UP * n_out.dot(env.UP)
                if lay.length < 0.3:
                    lay = env.UP.orthogonal()
                lay.normalize()
            j = rand_vec(rng) * 0.35
            lay = lay + j - env.UP * j.dot(env.UP)
            lay.normalize()
            p = p + lay * step
            p += env.UP * (lift - env.height(p))
            cur_n = env.UP.copy()
        else:
            d = (d * 0.45 + env.G * 0.55 + rand_vec(rng) * sway * 0.3).normalized()
            p = p + d * step
            if env.collider is not None:
                p, nn = env.collider(p)
                if nn is not None:
                    cur_n = nn
            if env.height(p) < lift:
                p += env.UP * (lift - env.height(p))
                cur_n = env.UP.copy()
        if env.x_limit is not None and abs(p.x) > env.x_limit:
            p.x = math.copysign(env.x_limit, p.x)
        pts.append(p.copy())
        nrms.append(cur_n.copy())
    return pts, nrms


def strip_place(frames, s_c, flip):
    """Maps a sign's (x, y) onto a hanging strip, centred s_c along it."""
    acc = [0.0]
    for (p0, _, _), (p1, _, _) in zip(frames, frames[1:]):
        acc.append(acc[-1] + (p1 - p0).length)

    def place(x, y):
        s = min(max(s_c - y, 0.0), acc[-1] - 1e-6)
        i = 0
        while i < len(acc) - 2 and acc[i + 1] < s:
            i += 1
        k = (s - acc[i]) / max(acc[i + 1] - acc[i], 1e-9)
        p = frames[i][0].lerp(frames[i + 1][0], k)
        side = frames[i][1].lerp(frames[i + 1][1], k).normalized()
        nr = frames[i][2].lerp(frames[i + 1][2], k).normalized()
        return p + side * x * flip + nr * 0.0012 * flip
    return place


# ---------------------------------------------------------------------------
# The builder
# ---------------------------------------------------------------------------
class Lash:
    def __init__(self, M, coll, prefix, dead):
        self.M = M
        self.coll = coll
        self.prefix = prefix
        self.dead = dead
        self.origin = {}
        self.objs = {}
        self.rng = random.Random(7 if not dead else 11)

    # -- objects ----------------------------------------------------------
    def empty(self, name, origin, parent, size=0.1):
        ob = bpy.data.objects.new(self.prefix + name, None)
        ob.empty_display_size = size
        return self._place(ob, name, origin, parent)

    def mesh(self, name, b, origin, parent):
        origin = Vector(origin)
        bmesh.ops.recalc_face_normals(b.bm, faces=b.bm.faces[:])
        for v in b.bm.verts:
            v.co -= origin
        me = bpy.data.meshes.new(self.prefix + name)
        b.bm.to_mesh(me)
        b.bm.free()
        for m in b.mats:
            me.materials.append(m)
        ob = bpy.data.objects.new(self.prefix + name, me)
        return self._place(ob, name, origin, parent)

    def _place(self, ob, name, origin, parent):
        origin = Vector(origin)
        self.coll.objects.link(ob)
        self.origin[ob.name] = origin
        self.objs[name] = ob
        if parent is not None:
            ob.parent = parent
            ob.location = origin - self.origin[parent.name]
        else:
            ob.location = origin
        return ob

    # -- the trunk's centre line and plates --------------------------------
    def layout(self):
        dead = self.dead
        zc = BACK_H - HALF_H
        a2, a1 = math.radians(45), math.radians(70)
        self.Lf = (TOP_H - HALO_D / 2 - SPIKE - HEAD_UP - zc) / (math.sin(a2) + math.sin(a1))
        front = self.Lf * (math.cos(a2) + math.cos(a1)) + HEAD_FWD + MASK_D
        Fy = -TRUNK_LEN / 2 + front
        self.Lh = (TRUNK_LEN / 2 + GAP - Fy) / (N_SEG - 2)
        if dead:
            a2, a1 = math.radians(-9), math.radians(-24)
        J = {2: Vector((0, Fy, zc))}
        J[1] = J[2] + Vector((0, -math.cos(a2), math.sin(a2))) * self.Lf
        J[0] = J[1] + Vector((0, -math.cos(a1), math.sin(a1))) * self.Lf
        for s in range(3, N_SEG + 1):
            p = math.radians(-8.0 * (s - 3)) if dead else 0.0
            J[s] = J[s - 1] + Vector((0, math.cos(p), math.sin(p))) * self.Lh
        self.J = J
        # width at each joint, neck to tail tip
        self.WJ = [0.56, 0.78, 0.97, 1.0, 1.0, 1.0, 0.95, 0.85, 0.68, 0.42]
        self.frames = {}
        for s in range(1, N_SEG + 1):
            A, Bf = J[s], J[s - 1]
            t = (Bf - A).normalized()
            up = Vector((0, t.z, -t.y))
            self.frames[s] = (A, t, up, (Bf - A).length)

    PROF = [(0.0, 0.84), (0.06, 0.97), (0.3, 1.0), (0.75, 0.95), (0.93, 0.84), (1.0, 0.70)]
    PROF_TAIL = [(0.0, 0.30), (0.05, 0.62), (0.16, 0.86), (0.4, 0.98), (0.8, 0.97),
                 (0.93, 0.88), (1.0, 0.74)]

    def dims(self, s, tau):
        w = lerp(self.WJ[s], self.WJ[s - 1], tau)
        w *= pl(tau, self.PROF_TAIL if s == N_SEG else self.PROF)
        return (TRUNK_W / 2) * w, (HALF_H - RIDGE) * w, RIDGE * w

    def surf(self, s, tau, phi, off=0.0):
        """Point on plate s. tau 0 rear to 1 front, phi angle from the back's
        middle. Returns (point, outward normal)."""
        A, t, up, L = self.frames[s]
        a, b, ridge = self.dims(s, tau)
        C = A + t * (GAP + tau * (L - 2 * GAP))
        sx, cz = math.sin(phi), math.cos(phi)
        z = b * math.copysign(abs(cz) ** 0.85, cz) * (0.85 if cz < 0 else 1.0) + ridge * max(0.0, cz) ** 8
        xx = a * math.copysign(abs(sx) ** 0.8, sx)
        n = (X * (sx / a) + up * (cz / b)).normalized()
        return C + X * xx + up * z + n * off, n

    def collider(self, p):
        m = 0.014
        hit = None
        for s in range(1, N_SEG + 1):
            A, t, up, L = self.frames[s]
            y = (p - A).dot(t)
            if -0.04 <= y <= L + 0.04:
                tau = min(max(y / L, 0.0), 1.0)
                a, b, _ = self.dims(s, tau)
                a = max(a, 0.17 * self.WJ[s]) + m
                C = A + t * y
                x = (p - C).dot(X)
                z = (p - C).dot(up)
                zb = max(b, 0.13 * self.WJ[s]) * (0.85 if z < 0 else 1.0) + m
                q = (x / a) ** 2 + (z / zb) ** 2
                if q < 1.0:
                    k = 1.0 / math.sqrt(max(q, 1e-4))
                    p = C + X * x * k + up * z * k
                    hit = (X * (x * k / a ** 2) + up * (z * k / zb ** 2)).normalized()
        return p, hit

    def build_trunk(self, root):
        M = self.M
        taus = [0.0, 0.04, 0.12, 0.22, 0.35, 0.5, 0.65, 0.8, 0.88, 0.94, 0.98, 1.0]
        NA = 24
        segs = {}
        order = [5, 4, 3, 2, 1, 6, 7, 8, 9]
        for s in order:
            A, t, up, L = self.frames[s]
            b = B()
            rings = []
            for tau in taus:
                ring = []
                for k in range(NA):
                    phi = 2 * math.pi * k / NA
                    p, _ = self.surf(s, tau, phi)
                    ring.append(b.v(p))
                rings.append(ring)
            for i in range(len(taus) - 1):
                for k in range(NA):
                    k2 = k + 1
                    u0 = k / NA if k < NA / 2 else k / NA - 1.0
                    u1 = k2 / NA if k2 <= NA / 2 else k2 / NA - 1.0
                    if k == NA // 2:
                        u0 = -0.5
                    b.face([rings[i][k], rings[i][k2 % NA], rings[i + 1][k2 % NA], rings[i + 1][k]],
                           M["hide"], True,
                           uvs=[(u0, taus[i]), (u1, taus[i]), (u1, taus[i + 1]), (u0, taus[i + 1])])
            b.face(rings[0][::-1], M["hide"], False)
            b.face(rings[-1], M["hide"], False)
            # the dark frame, seen in the gap at this plate's rear joint
            if s < N_SEG:
                w = self.WJ[s]
                ball(b, A, 1.0, M["frame"], seg=14, rings=8,
                     scale=(0.2 * w, 0.11 if s > 2 else 0.14, 0.16 * w), axes=(X, t, up))
            if s == 5:
                origin, parent = A + t * (L / 2), root
            elif s < 5:
                origin, parent = A, segs[s + 1]
            else:
                origin, parent = self.J[s - 1], segs[s - 1]
            segs[s] = self.mesh("trunk_seg_%d" % s, b, origin, parent)
        self.segs = segs

    # -- head: mask and halo -------------------------------------------------
    WF = [(-1.0, 0.004), (-0.92, 0.022), (-0.6, 0.036), (-0.2, 0.045), (0.15, 0.057),
          (0.5, 0.070), (0.8, 0.058), (0.95, 0.030), (1.0, 0.004)]
    DF = [(-1.0, 0.004), (-0.9, 0.030), (-0.5, 0.048), (0.0, 0.055), (0.5, MASK_D),
          (0.85, 0.052), (1.0, 0.004)]

    def mask_front(self, x, v):
        vn = v / (MASK_H / 2)
        W, D = pl(vn, self.WF), pl(vn, self.DF)
        q = min(abs(x) / W, 0.985)
        return D * (1 - q * q) ** 0.4

    def build_head(self):
        M = self.M
        N = self.J[0]
        A1, t1, up1, L1 = self.frames[1]
        if not self.dead:
            f, u = -Y, Z.copy()
            c = N + Vector((0, -HEAD_FWD, HEAD_UP))
        else:
            f = X.copy()                      # mask turned up to the sky
            u = (t1 - X * t1.dot(X)).normalized()
            c = Vector((-TRUNK_W / 2 + 0.07, N.y, N.z)) + u * 0.17
        r = u.cross(f).normalized()
        self.head = (c, r, f, u)
        head = self.empty("head", N, self.segs[1], 0.15)
        # neck: the frame, bare
        b = B()
        tube(b, [N - t1 * 0.03, N.lerp(c, 0.5) - f * 0.03, c - f * 0.03], [0.07, 0.05, 0.045],
             M["frame"], sides=10)
        self.mesh("neck", b, N, head)

        # mask
        b = B()
        nv, na = 18, 28
        rings = []
        for i in range(nv + 1):
            vn = -1 + 2 * i / nv
            W, D = pl(vn, self.WF), pl(vn, self.DF)
            ring = []
            for k in range(na):
                th = 2 * math.pi * k / na
                ct, st = math.cos(th), math.sin(th)
                dep = D * abs(st) ** 0.8 if st > 0 else 0.022 * st * min(1.0, D / 0.03)
                ring.append(b.v(c + r * (W * ct) + f * dep + u * (vn * MASK_H / 2)))
            rings.append(ring)
        for i in range(nv):
            for k in range(na):
                k2 = (k + 1) % na
                b.face([rings[i][k], rings[i][k2], rings[i + 1][k2], rings[i + 1][k]], M["bone"], True)
        b.face(rings[0][::-1], M["bone"], True)
        b.face(rings[-1], M["bone"], True)
        v0 = 0.062
        target_sign(b, lambda x, y: c + r * x + u * (v0 + y)
                    + f * (self.mask_front(x, v0 + y) + 0.002), 0.030, M["red"])
        self.mesh("mask", b, c, head)

        # halo: Registrar hardware, bolted on over the people's own mask
        b = B()
        hc = c - f * 0.037
        Rc = HALO_D / 2 - ARC_T / 2
        half = ARC_T / math.sqrt(2)
        for mid in (90, 0, 180, 270):
            pts = []
            for k in range(9):
                a = math.radians(mid - 33 + 66 * k / 8)
                pts.append(hc + r * (Rc * math.cos(a)) + u * (Rc * math.sin(a)))
            tube(b, pts, half, M["reg"], sides=4, smooth=False, hint=f, rot=math.pi / 4)
            for da in (-22, 0, 22):
                a = math.radians(mid + da)
                d = r * math.cos(a) + u * math.sin(a)
                tube(b, [hc + d * (HALO_D / 2 - 0.004), hc + d * (HALO_D / 2 + SPIKE)],
                     [0.016, 0.0035], M["reg"], sides=4, smooth=False, hint=f, rot=math.pi / 4)
            # flat spoke from the hub to the arc
            a = math.radians(mid)
            d = r * math.cos(a) + u * math.sin(a)
            side = f.cross(d)
            box(b, hc + d * (Rc / 2 + 0.01), (Rc - 0.03, 0.022, 0.008), M["reg"], axes=(d, side, f))
        tube(b, [hc - f * 0.012, hc + f * 0.012], 0.05, M["reg"], sides=8, smooth=False, hint=u)
        # brow clamp with two bolts, and the tab that carries the lenses
        vc = 0.128
        fc = self.mask_front(0, vc) + 0.007
        box(b, c + u * vc + f * fc, (0.118, 0.012, 0.030), M["reg"], axes=(r, f, u))
        for sgn in (-1, 1):
            box(b, c + u * vc + r * (sgn * 0.062) + f * (fc - 0.045) / 2, (0.012, fc + 0.05, 0.030),
                M["reg"], axes=(r, f, u))
            tube(b, [c + u * vc + r * (sgn * 0.040) + f * (fc + 0.005),
                     c + u * vc + r * (sgn * 0.040) + f * (fc + 0.013)], 0.0075, M["reg"],
                 sides=6, smooth=False, hint=u)
        tab_f = fc - 0.004
        box(b, c + u * 0.187 + f * tab_f, (0.044, 0.012, 0.092), M["reg"], axes=(r, f, u))
        box(b, c + u * 0.187 + f * (tab_f - 0.03), (0.016, 0.06, 0.03), M["reg"], axes=(r, f, u))
        halo = self.mesh("halo", b, hc, head)
        lens_mat = M["lens_dark"] if self.dead else M["lens"]
        for i, vv in enumerate((0.168, 0.207)):
            b = B()
            p = c + u * vv + f * (tab_f + 0.006)
            tube(b, [p, p + f * 0.004], 0.0155, M["reg"], sides=12, smooth=False, hint=u)
            tube(b, [p + f * 0.004, p + f * 0.0075], [0.0125, 0.009], lens_mat, sides=12, hint=u)
            self.mesh("halo_lens_%d" % (i + 1), b, p, halo)

    # -- legs ---------------------------------------------------------------
    KZ = [1.00, 1.03, 1.00, 0.98, 0.95, 0.90, 0.84]
    FX = [1.0, 1.0, 1.0, 1.0, 1.0, 0.93, 0.83]     # share of half the stance
    STRUTS = {("R", 3), ("L", 5)}

    def leg_points(self, i, side):
        s = 3 + i
        A, t, up, L = self.frames[s]
        w = lerp(self.WJ[s], self.WJ[s - 1], 0.5)
        mid = A + t * (L * 0.5)
        r_tip = 0.008
        rng = random.Random(100 + 10 * i + side)
        phi = math.radians(lerp(26, -26, i / (N_LEGS - 1)) + rng.uniform(-5, 5))
        dirh = Vector((side * math.cos(phi), -math.sin(phi), 0))
        hip0 = Vector((side * 0.19 * w, mid.y, BACK_H - HALF_H - 0.05))
        r_foot = (self.FX[i] * STANCE_W / 2 - r_tip - 0.19 * w) / math.cos(phi)
        knee0 = hip0 + dirh * (0.34 + rng.uniform(-0.04, 0.04))
        knee0.z = self.KZ[i] + rng.uniform(-0.03, 0.04)
        ankle0 = hip0 + dirh * (r_foot - 0.10)
        ankle0.z = 0.20
        tip0 = hip0 + dirh * r_foot
        tip0.z = r_tip
        if not self.dead:
            return hip0, knee0, ankle0, tip0
        lf, lt, ls = (knee0 - hip0).length, (ankle0 - knee0).length, (tip0 - ankle0).length
        j = lambda k: rng.uniform(-k, k)
        if side > 0:      # the upper side: legs curl in tight over the belly
            hip = mid + X * (0.19 * w) - up * 0.05
            sg = 1 if i % 2 else -1
            knee = hip + (X * (0.12 + j(0.12)) - up * 1.0 + t * j(0.25)).normalized() * lf
            ankle = knee + (-X * (0.30 + j(0.1)) + up * (0.62 + j(0.1)) + t * sg * (0.55 + j(0.2))).normalized() * lt
            tip = ankle + (-X * 0.3 - up * (0.75 + j(0.2)) - t * sg * 0.5).normalized() * ls
        else:             # the side it lies on: legs folded flat along the ground
            hip = mid - X * (0.165 * w) - up * 0.05
            sg = rng.choice((-1, 1))
            knee = hip + (-X * 0.02 - up * 1.0 + t * j(0.3)).normalized() * lf
            ankle = knee + (X * 0.04 + up * (0.62 + j(0.15)) + t * sg * 0.7).normalized() * lt
            tip = ankle + (-up * 0.7 - t * sg * 0.6).normalized() * ls
        for p in (knee, ankle, tip):
            p.x = max(p.x, -TRUNK_W / 2 + 0.035)
        return hip, knee, ankle, tip

    def build_legs(self):
        M = self.M
        self.legs = {}
        for i in range(N_LEGS):
            for side, tag in ((-1, "R"), (1, "L")):
                name = "leg_%s%d" % (tag, i + 1)
                seg = self.segs[3 + i]
                hip, knee, ankle, tip = self.leg_points(i, side)
                if (tag, i + 1) in self.STRUTS:
                    self.build_strut(name, seg, i, side, hip, tip)
                    continue
                lr = random.Random(300 + 10 * i + side)
                bow = Vector((0, lr.uniform(-0.03, 0.03), lr.uniform(0.01, 0.035)))
                b = B()
                ball(b, hip, 0.05, M["frame"], seg=8, rings=6)
                limb(b, hip, knee, 0.037, 0.029, M["limb"], bow)
                up_o = self.mesh(name + "_upper", b, hip, seg)
                b = B()
                ball(b, knee, 0.025, M["frame"], seg=8, rings=6)
                limb(b, knee, ankle, 0.027, 0.018, M["limb"],
                     Vector((side * 0.02, lr.uniform(-0.02, 0.02), 0.0)))
                lo_o = self.mesh(name + "_lower", b, knee, up_o)
                b = B()
                ball(b, ankle, 0.019, M["frame"], seg=8, rings=5)
                tube(b, [ankle, ankle.lerp(tip, 0.25), ankle.lerp(tip, 0.6), tip],
                     [0.017, 0.019, 0.012, 0.008], M["plain"], sides=6)
                ft_o = self.mesh(name + "_foot", b, ankle, lo_o)
                self.legs[(tag, i + 1)] = dict(hip=hip, knee=knee, ankle=ankle, tip=tip,
                                               upper=up_o, lower=lo_o, foot=ft_o)
                # weed caught on the leg
                b = B()
                p0 = knee.lerp(ankle, self.rng.uniform(0.0, 0.35))
                n0 = Vector((self.rng.uniform(-1, 1), self.rng.uniform(-1, 1), 0.2)).normalized()
                for k in range(self.rng.choice((3, 4, 5))):
                    n = (n0 + rand_vec(self.rng) * 0.5).normalized()
                    pts, nr = hang_path(self.env, p0 + n * 0.03, n, self.rng.uniform(0.12, 0.55),
                                        self.rng, sway=0.6)
                    ribbon(b, pts, nr, self.rng.uniform(0.007, 0.02), M["weed"], taper=0.15)
                self.mesh("weed_" + name, b, knee, lo_o)

    def build_strut(self, name, seg, i, side, hip, tip):
        """Wrong repair: a straight black Registrar strut with a flat pad."""
        M = self.M
        _, t, up, _ = self.frames[3 + i]
        pad_half = 0.08
        foot0 = Vector((side * (STANCE_W / 2 - pad_half), tip.y, 0.022))
        hip0 = Vector((hip.x if not self.dead else side * abs(hip.x), 0, 0))
        if not self.dead:
            foot = foot0
            d = (foot - hip).normalized()
            ex, ez = X.copy(), Z.copy()
        else:
            alive_hip = self.alive_hips[(side, i)]
            length = (foot0 - alive_hip).length
            d_alive = (foot0 - alive_hip).normalized()
            if side < 0:   # under the body: lies straight out along the ground
                d = (-X * 0.0 - up * 1.0 + t * 0.12).normalized()
                hip = hip.copy()
                hip.x = -TRUNK_W / 2 + 0.09
            else:          # upper side: sticks straight up and out, not curled
                d = (X * 0.72 - up * 0.66 - t * 0.1).normalized()
            foot = hip + d * length
            q = d_alive.rotation_difference(d)
            ex, ez = q @ X, q @ Z
        ey = ez.cross(ex).normalized()
        b = B()
        tube(b, [hip + d * 0.03, foot], 0.028, M["reg"], sides=4, smooth=False, rot=math.pi / 4,
             hint=ez)
        # the plug: a block set into the hip, with four pins into the frame
        side_v = d.cross(ez).normalized()
        nrm = side_v.cross(d).normalized()
        box(b, hip + d * 0.05, (0.13, 0.10, 0.10), M["reg"], axes=(d, side_v, nrm))
        for sa in (-1, 1):
            for sb in (-1, 1):
                p = hip + side_v * (sa * 0.032) + nrm * (sb * 0.032)
                tube(b, [p - d * 0.07, p + d * 0.0], 0.011, M["reg"], sides=6, smooth=False)
        ball(b, hip - d * 0.03, 0.075, M["frame"], seg=8, rings=6)
        annulus(b, plane_place(hip + d * 0.05 + nrm * 0.05, d, side_v, nrm, 0.001), 0.036, 0.031, M["gold"], seg=20)
        strut = self.mesh(name + "_strut", b, hip, seg)
        b = B()
        box(b, foot - ez * 0.011, (pad_half * 2, pad_half * 2, 0.022), M["reg"], axes=(ex, ey, ez))
        pp = plane_place(foot, ex, ey, ez, 0.001)
        annulus(b, pp, 0.066, 0.061, M["gold"], seg=24)
        annulus(b, pp, 0.046, 0.043, M["gold"], seg=24)
        self.mesh(name + "_pad", b, foot, strut)

    # -- back: spines, sacs, weed --------------------------------------------
    def spine_pts(self, i, side):
        s = 3 + i
        A, t, up, L = self.frames[s]
        w = lerp(self.WJ[s], self.WJ[s - 1], 0.45)
        length = (SPINE_LEN_R if side < 0 else SPINE_LEN_L)[i]
        sr = random.Random(500 + 10 * i + side)
        lean = math.radians(lerp(52, 40, i / 6) + sr.uniform(-6, 5))
        splay = math.radians(sr.uniform(5, 17) if not (self.dead and side < 0) else -2)
        base = A + t * (L * (0.42 if side < 0 else 0.52)) + X * (side * 0.075 * w) + up * (0.145 * w)
        d = (X * (side * math.sin(splay)) - t * (math.sin(lean) * math.cos(splay))
             + up * (math.cos(lean) * math.cos(splay))).normalized()
        droop = -t * 0.75 - up * 0.65
        pts = []
        for k in range(7):
            q = k / 6
            pts.append(base + d * (length * q * 0.985) + droop * (0.10 * length * q * q))
        # keep the true length
        tip_dir = (pts[-1] - pts[0])
        scale = length / sum((b_ - a_).length for a_, b_ in zip(pts, pts[1:]))
        pts = [base + (p - base) * scale for p in pts]
        return pts

    def build_back(self):
        M = self.M
        self.spines = {}
        for i in range(7):
            for side, tag in ((-1, "R"), (1, "L")):
                pts = self.spine_pts(i, side)
                b = B()
                radii = [lerp(0.021, 0.004, (k / 6) ** 0.7) for k in range(7)]
                tube(b, pts, radii, M["spine"], sides=6)
                ball(b, pts[0], 0.026, M["frame"], seg=8, rings=5)
                self.mesh("spine_%s%d" % (tag, i + 1), b, pts[0], self.segs[3 + i])
                self.spines[(tag, i + 1)] = pts
        # egg-like sacs in the grooves between the plates
        spots = [(3, 0.05, -0.035), (4, 0.03, 0.045), (4, 0.05, -0.05), (5, 0.04, 0.03),
                 (6, 0.03, -0.045), (6, 0.06, 0.05), (7, 0.04, -0.02), (8, 0.05, 0.03)]
        for n, (s, tau, xo) in enumerate(spots):
            A, t, up, L = self.frames[s]
            size = SAC_SIZES[n]
            a, bb, ridge = self.dims(s, tau)
            base = A + t * (GAP + tau * L) + X * xo + up * (bb * 0.93)
            axis = (up * 1.0 - t * 0.25 + X * (xo * 2)).normalized()
            e1 = axis.orthogonal().normalized()
            e2 = axis.cross(e1)
            b = B()
            ball(b, base + axis * (size * 0.36), size / 2, M["sac"], seg=12, rings=8,
                 scale=(0.74, 0.74, 1.0), axes=(e1, e2, axis), top=1.25)
            # the top half is stretched by 1.25: bring the whole egg back to its size
            self.mesh("sac_%d" % (n + 1), b, base, self.segs[s])
        # wet grass and weed over the trunk
        for s in range(1, N_SEG + 1):
            b = B()
            clumps = {1: 3, 2: 4}.get(s, 5 if s < 9 else 3)
            for c in range(clumps):
                tau0 = self.rng.uniform(0.1, 0.9)
                phi0 = math.radians(self.rng.uniform(5, 95)) * (1 if c % 2 else -1)
                for k in range(self.rng.choice((4, 5, 6, 7))):
                    tau = min(max(tau0 + self.rng.uniform(-0.08, 0.08), 0.03), 0.97)
                    phi = phi0 + math.radians(self.rng.uniform(-9, 9))
                    p, n = self.surf(s, tau, phi, 0.012)
                    pts, nr = hang_path(self.env, p, n, self.rng.uniform(0.15, 0.85), self.rng,
                                        sway=0.55)
                    ribbon(b, pts, nr, self.rng.uniform(0.007, 0.022), M["weed"], taper=0.15)
            self.mesh("weed_seg_%d" % s, b, self.frames[s][0], self.segs[s])

    def build_arcs(self):
        """Electrical life: faint pale arcs along the spines. Night only."""
        b = B()
        rng = random.Random(3)
        for tag in ("R", "L"):
            for i in ((1, 4) if tag == "R" else (2, 5)):
                a, c = self.spines[(tag, i)], self.spines[(tag, i + 1)]
                k0 = rng.choice((2, 3))
                p0, p1 = a[k0], c[k0 - 1]
                pts = [p0]
                for k in range(1, 6):
                    pts.append(p0.lerp(p1, k / 6) + rand_vec(rng) * 0.02)
                pts.append(p1)
                tube(b, pts, 0.0016, self.M["arc"], sides=3)
        for tag, i in (("R", 2), ("L", 3), ("R", 6), ("L", 7)):
            a = self.spines[(tag, i)]
            pts = [a[3].lerp(a[5], k / 5) + rand_vec(rng) * (0.012 if 0 < k < 5 else 0) for k in range(6)]
            tube(b, pts, 0.0014, self.M["arc"], sides=3)
        ob = self.mesh("arcs_spines", b, self.frames[5][0], self.segs[5])
        ob.hide_render = True
        ob.hide_viewport = True
        ob.visible_shadow = False

    # -- arms and shoulder plate ---------------------------------------------
    def shoulder(self, side):
        A1, t1, up1, L1 = self.frames[1]
        a, _, _ = self.dims(1, 0.78)
        return A1 + t1 * (0.78 * L1) + X * (side * a * 0.9) - up1 * 0.03

    def build_arms(self, hands, poles):
        M = self.M
        for side, tag in ((-1, "R"), (1, "L")):
            S = self.shoulder(side)
            H = Vector(hands[tag])
            l = ARM_LEN / 2
            d = H - S
            D = min(d.length, 2 * l - 1e-3)
            dn = d.normalized()
            H = S + dn * D
            pole = Vector(poles[tag])
            pp = (pole - dn * pole.dot(dn)).normalized()
            E = S + dn * (D / 2) + pp * math.sqrt(max(l * l - D * D / 4, 0.0))
            b = B()
            ball(b, S, 0.042, M["frame"], seg=8, rings=6)
            limb(b, S, E, 0.031, 0.024, M["limb"])
            up_o = self.mesh("arm_%s_upper" % tag, b, S, self.segs[1])
            b = B()
            ball(b, E, 0.021, M["frame"], seg=8, rings=6)
            limb(b, E, H, 0.022, 0.015, M["limb"])
            lo_o = self.mesh("arm_%s_fore" % tag, b, E, up_o)
            b = B()
            fwd = (H - E).normalized()
            sv = fwd.cross(self.env.UP)
            if sv.length < 0.2:
                sv = fwd.orthogonal()
            sv.normalize()
            curl = sv.cross(fwd).normalized()
            ball(b, H, 0.016, M["frame"], seg=6, rings=4)
            for k in (-1, 0, 1):
                p1 = H + fwd * 0.045 + sv * (k * 0.022) + curl * 0.012
                p2 = p1 + fwd * 0.02 - curl * 0.045
                p3 = p2 - fwd * 0.03 - curl * 0.02
                p1, p2, p3 = [q + self.env.UP * max(0.0, 0.012 - self.env.height(q)) for q in (p1, p2, p3)]
                tube(b, [H, p1, p2, p3], [0.009, 0.0075, 0.006, 0.003], M["plain"], sides=5)
            self.mesh("hand_%s" % tag, b, H, lo_o)

        # plate on the left shoulder with the target sign
        A1, t1, up1, L1 = self.frames[1]
        a, _, _ = self.dims(1, 0.72)
        en = (X * 0.86 + t1 * 0.30 - up1 * 0.40).normalized()
        ev = (t1 - en * t1.dot(en)).normalized()
        eh = ev.cross(en)
        pc = A1 + t1 * (0.80 * L1) + X * (a + 0.075) - up1 * 0.03
        b = B()
        outline = []
        for k in range(20):
            th = 2 * math.pi * k / 20
            ct, st = math.cos(th), math.sin(th)
            x = 0.085 * math.copysign(abs(ct) ** 0.7, ct)
            y = 0.115 * math.copysign(abs(st) ** 0.7, st) - (0.02 if st < 0 else 0) * abs(st)
            outline.append((x, y))
        bulge = lambda x, y: 0.012 * (1 - (x / 0.085) ** 2)
        front = [b.v(pc + eh * x + ev * y + en * bulge(x, y)) for x, y in outline]
        back = [b.v(pc + eh * x + ev * y + en * (bulge(x, y) - 0.012)) for x, y in outline]
        mid = b.v(pc + en * 0.012)
        for k in range(20):
            k2 = (k + 1) % 20
            b.face([mid, front[k], front[k2]], M["bone"], True)
            b.face([front[k], back[k], back[k2], front[k2]], M["bone"], False)
        b.face(back[::-1], M["bone"], False)
        target_sign(b, lambda x, y: pc + eh * x + ev * (y + 0.03) + en * (bulge(x, y) * 0.9 + 0.004),
                    0.045, M["red"])
        for sg in (-1, 1):   # two cords tie it to the limb
            tube(b, [pc + ev * (sg * 0.07) + eh * 0.07 + en * 0.006, pc + ev * (sg * 0.07) - en * 0.06,
                     pc + ev * (sg * 0.07) - eh * 0.07 + en * 0.006], 0.005, M["cord"], sides=5)
        self.mesh("shoulder_plate_L", b, pc, self.segs[1])

    # -- devotion: bells and prayer strips -----------------------------------
    def bell(self, name, parent, env, p0, n_out, rng, cord=0.06):
        M = self.M
        pts, _ = hang_path(env, p0, n_out, cord, rng, step=cord / 3, sway=0.1, lift=0.027)
        b = B()
        tube(b, pts, 0.0025, M["cord"], sides=4)
        d = (pts[-1] - pts[-2]).normalized()
        q = pts[-1]
        prof = [(0.0, 0.006), (0.004, 0.012), (0.014, 0.016), (0.028, 0.0175), (0.036, 0.023), (0.040, 0.0245)]
        tube(b, [q + d * z for z, _ in prof], [r for _, r in prof], M["brass"], sides=10, caps=False)
        tube(b, [q, q + d * 0.004], [0.004, 0.006], M["brass"], sides=6)
        ball(b, q + d * 0.036, 0.006, M["brass"], seg=6, rings=4)
        return self.mesh(name, b, p0, parent)

    def strip(self, name, parent, env, p0, n_out, rng, signs=("target", "reed")):
        M = self.M
        pts, nr = hang_path(env, p0, n_out, STRIP_LEN, rng, step=STRIP_LEN / 8, sway=0.12, lift=0.004)
        total = sum((b_ - a_).length for a_, b_ in zip(pts, pts[1:]))
        pts = [pts[0] + (p - pts[0]) * (STRIP_LEN / total) for p in pts]
        b = B()
        frames = ribbon(b, pts, nr, 0.03, M["strip"], taper=1.0)
        for flip in (1, -1):
            for kind, s_c in zip(signs, (0.075, 0.175)):
                place = strip_place(frames, s_c, flip)
                if kind == "target":
                    target_sign(b, place, 0.0085, M["red"])
                else:
                    reed_sign(b, place, 0.016, M["red"])
            stroke(b, strip_place(frames, 0.232, flip), (-0.011, 0), (0.011, 0), 0.004, M["red"], n=1)
        return self.mesh(name, b, p0, parent)

    def build_leg_devotion(self):
        rng = random.Random(21)
        n_b, n_s = 4, 5
        for tag, i, what in (("R", 1, "bell"), ("L", 1, "bell"), ("L", 2, "bell"),
                             ("R", 1, "strip"), ("L", 1, "strip"), ("R", 2, "strip"), ("L", 2, "strip")):
            leg = self.legs[(tag, i)]
            side = -1 if tag == "R" else 1
            n = Vector((side * 0.8, -0.6, 0)).normalized()
            if what == "bell":
                p = leg["knee"].lerp(leg["ankle"], 0.12) + n * 0.024
                self.bell("bell_%d" % n_b, leg["lower"], self.env, p, n, rng)
                n_b += 1
            else:
                p = leg["knee"].lerp(leg["ankle"], 0.32 if i == 1 else 0.2) + n * 0.022
                self.strip("strip_%d" % n_s, leg["lower"], self.env, p, n, rng,
                           ("target", "reed") if n_s % 2 else ("reed", "target"))
                n_s += 1

    # -- the rifle -----------------------------------------------------------
    def build_rifle(self, root, location, rotation, env, stave_dirs):
        M = self.M
        save_origin = self.origin
        self.origin = {}
        rifle = self.empty("rifle", (0, 0, 0), None, 0.2)
        y_rear = 0.29 * RIFLE_LEN
        y_muz = y_rear - RIFLE_LEN
        b = B()
        # receiver: a slim six-sided bar
        ys = [0.19, 0.13, -0.30, -0.42, -0.50]
        ks = [0.55, 1.0, 1.0, 0.8, 0.5]
        tube(b, [(0, y, 0) for y in ys], ks, M["gun"], sides=6, smooth=False, hint=Z,
             sx=0.032, sy=0.021, rot=math.pi / 6)
        # barrel with small barbs
        tube(b, [(0, -0.48, 0), (0, y_muz + 0.16, 0), (0, y_muz + 0.10, 0), (0, y_muz + 0.05, 0),
                 (0, y_muz, 0)], [0.0125, 0.0100, 0.0100, 0.0165, 0.0125], M["gun"], sides=8)
        k = 0
        y = -0.62
        while y > y_muz + 0.2:
            a = math.radians(90 + 120 * k + 17 * k)
            rad = Vector((math.cos(a), 0, math.sin(a)))
            base = Vector((0, y, 0)) + rad * 0.008
            tube(b, [base, base + rad * 0.024 + Y * 0.034], [0.0055, 0.0008], M["gun"], sides=4,
                 smooth=False)
            y -= 0.105
            k += 1
        for k in range(3):   # three long barbs behind the muzzle, like a reed-spear
            a = math.radians(90 + 120 * k)
            rad = Vector((math.cos(a), 0, math.sin(a)))
            base = Vector((0, y_muz + 0.06, 0)) + rad * 0.012
            tube(b, [base, base + rad * 0.03 + Y * 0.10], [0.006, 0.001], M["gun"], sides=4, smooth=False)
        # skeleton stock: two thin rods and a crescent butt
        tube(b, [(0, 0.17, 0.012), (0, 0.45, 0.022), (0, y_rear - 0.035, 0.040)], 0.008, M["gun"], sides=6)
        tube(b, [(0, 0.17, -0.016), (0, 0.42, -0.050), (0, y_rear - 0.045, -0.095)], 0.008, M["gun"], sides=6)
        tube(b, [(0, y_rear - 0.045, 0.062), (0, y_rear - 0.018, 0.02), (0, y_rear - 0.009, -0.03),
                 (0, y_rear - 0.022, -0.085), (0, y_rear - 0.06, -0.125)],
             [0.006, 0.009, 0.009, 0.009, 0.005], M["gun"], sides=6)
        # grip: a hooked rod. Fore-grip: cord wound round the barrel.
        tube(b, [(0, 0.0, -0.025), (0, 0.035, -0.075), (0, 0.075, -0.105)], [0.012, 0.011, 0.008],
             M["gun"], sides=6)
        tube(b, [(0, -0.56, 0), (0, -0.70, 0)], 0.019, M["cord"], sides=8)
        # the long scope
        sz = 0.078
        prof = [(0.06, 0.021), (0.03, 0.021), (0.0, 0.0135), (-0.70, 0.0125), (-0.80, 0.019), (-0.86, 0.021)]
        tube(b, [(0, y, sz) for y, _ in prof], [r for _, r in prof], M["gun"], sides=10, caps=False)
        tube(b, [(0, 0.058, sz), (0, 0.054, sz)], 0.0195, M["glass"], sides=10)
        tube(b, [(0, -0.858, sz), (0, -0.854, sz)], 0.0195, M["glass"], sides=10)
        for y, h0 in ((-0.06, 0.028), (-0.40, 0.028), (-0.66, 0.008)):
            box(b, (0, y, (h0 + sz - 0.012) / 2), (0.008, 0.022, sz - 0.012 - h0), M["gun"])
        rifle_mesh = self.mesh("rifle_body", b, (0, 0, 0), rifle)

        # bipod: two crossed staves, lashed; the barrel rests in the crotch
        y_b = -1.35
        C = Vector((0, y_b, -0.026))
        b = B()
        ball(b, C + Z * 0.01, 0.03, M["cord"], seg=8, rings=6, scale=(1.0, 1.25, 1.0))
        tube(b, [(0, y_b - 0.035, 0), (0, y_b + 0.035, 0)], 0.016, M["cord"], sides=8)
        self.mesh("bipod_lashing", b, C, rifle)
        self.stave_feet = []
        for tag, d in zip(("R", "L"), stave_dirs):
            d = Vector(d).normalized()
            top = C - d * 0.12
            footp = C + d * (STAVE_LEN - 0.12)
            b = B()
            tube(b, [top, C, top.lerp(footp, 0.55), footp - d * 0.05, footp],
                 [0.011, 0.013, 0.0135, 0.012, 0.004], M["stave"], sides=6)
            tube(b, [footp - d * 0.20, footp - d * 0.16], 0.016, M["cord"], sides=6)
            self.mesh("bipod_stave_%s" % tag, b, C, rifle)
            self.stave_feet.append(footp)

        # devotion on the rifle: three bells, four prayer strips
        rng = random.Random(5)
        self.bell("bell_1", rifle, env, (0, 0.46, -0.060), (-1, 0, 0), rng)
        self.bell("bell_2", rifle, env, (0, -0.46, -0.030), (-1, 0, 0), rng, cord=0.08)
        self.bell("bell_3", rifle, env, (0, -1.20, -0.012), (-1, 0, 0), rng, cord=0.07)
        self.strip("strip_1", rifle, env, (0, 0.63, -0.085), (-1, 0.2, 0), rng, ("target", "reed"))
        self.strip("strip_2", rifle, env, (-0.022, -0.22, -0.02), (-1, 0, 0), rng, ("reed", "target"))
        self.strip("strip_3", rifle, env, (0, -0.95, -0.012), (-1, -0.2, 0), rng, ("target", "reed"))
        self.strip("strip_4", rifle, env, (0, -1.55, -0.012), (-1, 0.1, 0), rng, ("reed", "target"))

        rifle.parent = root
        rifle.location = location
        rifle.rotation_euler = rotation
        self.origin = save_origin
        self.origin[rifle.name] = Vector(location)
        return rifle

    # -- everything -----------------------------------------------------------
    def build(self):
        self.layout()
        root = bpy.data.objects.new(self.prefix + ("lash" if not self.dead else "lash_dead"), None)
        root.empty_display_size = 0.5
        self.coll.objects.link(root)
        self.origin[root.name] = Vector((0, 0, 0))
        self.root = root

        hz = math.sqrt((STAVE_LEN - 0.12) ** 2 - 0.36 ** 2 - 0.06 ** 2)
        rifle_z = hz + 0.026 + 0.003
        if not self.dead:
            body = root
            self.env = Env((0, 0, -1), 0.0, self.collider, x_limit=STANCE_W / 2 - 0.012)
            rifle_loc = Vector((-0.34, -1.30, rifle_z))
            rifle_rot = (0, 0, 0)
            rifle_env = Env((0, 0, -1), -rifle_z)
            staves = [(-0.36, -0.06, -hz), (0.36, -0.06, -hz)]
        else:
            body = bpy.data.objects.new(self.prefix + "body", None)
            body.empty_display_size = 0.3
            self.coll.objects.link(body)
            body.parent = root
            self.origin[body.name] = Vector((0, 0, 0))
            rot = Matrix.Rotation(math.radians(-70), 4, "Z") @ Matrix.Rotation(math.radians(-90), 4, "Y")
            body.matrix_world = Matrix.Translation((-0.35, 0.75, TRUNK_W / 2)) @ rot
            self.env = Env((-1, 0, 0), -TRUNK_W / 2, self.collider)
            rifle_loc = Vector((-0.55, -0.55, 0.036))
            rr = Matrix.Rotation(math.radians(-83), 4, "Z") @ Matrix.Rotation(math.radians(90), 4, "Y")
            rifle_rot = rr.to_euler()
            rifle_env = Env((1, 0, 0), -0.036)
            staves = [(0.0, -0.42, -0.9), (0.0, 0.62, -0.78)]
        self.body = body

        self.build_trunk(body)
        self.build_head()
        self.build_legs()
        self.build_back()
        rifle = self.build_rifle(root, rifle_loc, rifle_rot, rifle_env, staves)
        if not self.dead:
            self.build_arcs()
            hands = {"R": rifle_loc + Vector((0.0, 0.03, -0.065)),
                     "L": rifle_loc + Vector((0.0, -0.63, -0.02))}
            poles = {"R": (-0.75, 0.65, 0.35), "L": (0.45, -0.3, -0.85)}
        else:
            SR, SL = self.shoulder(-1), self.shoulder(1)
            lo = -TRUNK_W / 2 + 0.035
            hands = {"R": Vector((lo, SR.y - 0.72, SR.z - 0.38)),
                     "L": Vector((lo, SL.y - 0.30, SL.z - 0.80))}
            poles = {"R": (0.0, 0.2, -1.0), "L": (0.3, -1.0, 0.3)}
        self.build_arms(hands, poles)
        self.build_leg_devotion()
        return root


def world_points(objs):
    bpy.context.view_layer.update()
    for o in objs:
        if o.type != "MESH":
            continue
        mw = o.matrix_world
        for v in o.data.vertices:
            yield mw @ v.co


def extent(objs):
    pts = list(world_points(objs))
    lo = [min(p[i] for p in pts) for i in range(3)]
    hi = [max(p[i] for p in pts) for i in range(3)]
    return lo, hi


def main():
    sl.reset_scene()
    bpy.context.preferences.filepaths.save_version = 0
    sc = bpy.context.scene
    M = make_materials()

    model = bpy.data.collections.new("model")
    sc.collection.children.link(model)
    alive = Lash(M, model, "", dead=False)
    root = alive.build()

    deadc = bpy.data.collections.new("dead")
    sc.collection.children.link(deadc)
    dead = Lash(M, deadc, "dead_", dead=True)
    dead.alive_hips = {}
    for i in range(N_LEGS):
        for side in (-1, 1):
            dead.alive_hips[(side, i)] = alive.leg_points(i, side)[0]
    dead_root = dead.build()
    bpy.context.view_layer.update()
    dm = sl.measure([dead_root])
    deadc.hide_render = True
    deadc.hide_viewport = True

    # measure
    path = os.path.join(HERE, "size.json")
    sl.write_size_json(path, "lash", [root])
    trunk = [alive.segs[s] for s in alive.segs] + [alive.objs["mask"], alive.objs["halo"],
                                                    alive.objs["halo_lens_1"]]
    lo, hi = extent(trunk)
    legs = [o for n, o in alive.objs.items() if n.startswith("leg_")]
    llo, lhi = extent(legs)
    rl, rh = extent([alive.objs["rifle_body"]])
    alo, ahi = extent([o for o in alive.objs.values()])
    back = max(p.z for p in world_points([alive.segs[s] for s in range(3, 10)]))
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    data["trunk_length_m"] = round(hi[1] - lo[1], 3)
    data["back_height_m"] = round(back, 3)
    data["stance_width_m"] = round(lhi[0] - llo[0], 3)
    data["rifle_length_m"] = round(rh[1] - rl[1], 3)
    data["depth_without_rifle_m"] = round(max(hi[1], lhi[1]) - min(lo[1], llo[1]), 3)
    data["lowest_point_m"] = round(alo[2], 3)
    data["dead_form"] = {"width_m": dm["width_m"], "depth_m": dm["depth_m"],
                         "height_m": dm["height_m"], "triangles": dm["triangles"],
                         "lowest_point_m": dm["min"][2]}
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
        f.write("\n")
    print("LASH SIZE", json.dumps(data))
    print("LASH OBJECTS", len(model.objects), "dead", len(deadc.objects))
    deadc.hide_viewport = False
    bpy.context.view_layer.update()
    low = sorted((min((o.matrix_world @ v.co).z for v in o.data.vertices), o.name)
                 for o in deadc.objects if o.type == "MESH" and len(o.data.vertices))[:6]
    print("LASH DEAD LOWEST", low)
    deadc.hide_viewport = True

    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(HERE, "model.blend"), compress=True)


main()
