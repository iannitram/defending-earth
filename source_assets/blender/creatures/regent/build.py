"""Builds the Regent from nothing and saves model.blend.

Run from the shell only:
  flatpak run org.blender.Blender -b --python "<full path>/build.py"

The file holds two collections:
  model  the living Regent, standing, root object "regent"
  dead   the dead form, root object "regent_dead" (hidden from renders)
"""
import sys
import os
import math
import random

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "scale")))

import bpy
import bmesh
from mathutils import Vector, Matrix

import scale_lib as sl

rad = math.radians
I4 = Matrix.Identity(4)


def V(*a):
    return Vector(a)


def unit(*a):
    return Vector(a[0] if len(a) == 1 else a).normalized()


# ---------------------------------------------------------------------------
# Sizes. Whole-figure sizes come from the scale sheet. Part sizes come from
# the job text (the sheet does not hold them).
# ---------------------------------------------------------------------------
SHEET = sl.thing("regent")
HEIGHT = SHEET["height_m"]            # 3.6, top of the dome
FOOTPRINT = tuple(SHEET["footprint"])

DOME_D = 0.55
DOME_STRAIGHT = 0.045                 # short straight wall: "slightly taller than a half sphere"
LOBE_COUNT = 40
LOBE_MIN, LOBE_MAX = 0.04, 0.07
PLATE_LEN = 0.50
RING_D = 0.90
TORSO_LEN, TORSO_W = 1.10, 0.35
TORSO_BOTTOM = 2.0
LEAN = rad(15)
LEG_T = 0.06
KNEE_H = 2.0
FEET_SQUARE = 1.5
ARM_LEN, CLAW_LEN = 1.40, 0.25
CLOTH_L, CLOTH_W = 1.60, 0.25

# ---------------------------------------------------------------------------
# Materials
# ---------------------------------------------------------------------------


class G:
    """Small helper to wire shader nodes."""

    def __init__(self, mat):
        self.mat = mat
        self.nt = mat.node_tree
        self.bsdf = next(n for n in self.nt.nodes if n.bl_idname == "ShaderNodeBsdfPrincipled")
        self.out = next(n for n in self.nt.nodes if n.bl_idname == "ShaderNodeOutputMaterial")

    def new(self, kind):
        return self.nt.nodes.new(kind)

    def put(self, sock, v):
        if isinstance(v, bpy.types.NodeSocket):
            self.nt.links.new(v, sock)
        else:
            sock.default_value = v

    def m(self, op, a, b=None, c=None):
        n = self.new("ShaderNodeMath")
        n.operation = op
        for i, v in enumerate((a, b, c)):
            if v is not None:
                self.put(n.inputs[i], v)
        return n.outputs[0]

    def coord(self, which="Object"):
        return self.new("ShaderNodeTexCoord").outputs[which]

    def uv(self, name):
        n = self.new("ShaderNodeUVMap")
        n.uv_map = name
        return n.outputs[0]

    def sep(self, vec):
        n = self.new("ShaderNodeSeparateXYZ")
        self.put(n.inputs[0], vec)
        return n.outputs[0], n.outputs[1], n.outputs[2]

    def scale_vec(self, vec, s):
        n = self.new("ShaderNodeVectorMath")
        n.operation = "MULTIPLY"
        self.put(n.inputs[0], vec)
        n.inputs[1].default_value = s
        return n.outputs[0]

    def dist(self, vec, point):
        n = self.new("ShaderNodeVectorMath")
        n.operation = "DISTANCE"
        self.put(n.inputs[0], vec)
        n.inputs[1].default_value = point
        return n.outputs["Value"]

    def noise(self, vec, scale, detail=3.0, rough=0.5):
        n = self.new("ShaderNodeTexNoise")
        self.put(n.inputs["Vector"], vec)
        n.inputs["Scale"].default_value = scale
        n.inputs["Detail"].default_value = detail
        n.inputs["Roughness"].default_value = rough
        return n.outputs[0]

    def voro_edge(self, vec, scale):
        n = self.new("ShaderNodeTexVoronoi")
        n.feature = "DISTANCE_TO_EDGE"
        self.put(n.inputs["Vector"], vec)
        n.inputs["Scale"].default_value = scale
        return n.outputs["Distance"]

    def ramp(self, v, a, b, c=0.0, d=1.0):
        n = self.new("ShaderNodeMapRange")
        n.interpolation_type = "SMOOTHSTEP"
        self.put(n.inputs[0], v)
        for i, x in zip((1, 2, 3, 4), (a, b, c, d)):
            n.inputs[i].default_value = x
        return n.outputs[0]

    def mix(self, fac, a, b):
        n = self.new("ShaderNodeMix")
        n.data_type = "RGBA"
        self.put(n.inputs[0], fac)
        self.put(n.inputs[6], a)
        self.put(n.inputs[7], b)
        return n.outputs[2]

    def bump(self, height, strength, distance=0.01):
        n = self.new("ShaderNodeBump")
        self.put(n.inputs["Height"], height)
        n.inputs["Strength"].default_value = strength
        n.inputs["Distance"].default_value = distance
        self.nt.links.new(n.outputs[0], self.bsdf.inputs["Normal"])

    def set(self, name, v):
        self.put(self.bsdf.inputs[name], v)


def make_materials():
    M = {}

    # Flesh: dark wet purple-brown.
    mat, _ = sl.new_material("regent_flesh", "#3a2430", 0.3)
    g = G(mat)
    co = g.coord()
    n1 = g.noise(co, 9.0, 4.0)
    n2 = g.noise(g.scale_vec(co, (1.0, 1.0, 0.25)), 60.0, 2.0)
    g.set("Base Color", g.mix(g.ramp(n1, 0.35, 0.7), sl.rgb("#3a2430"), sl.rgb("#1f1218")))
    g.set("Roughness", g.ramp(n1, 0.3, 0.8, 0.24, 0.38))
    try:
        g.set("Coat Weight", 0.35)
        g.set("Coat Roughness", 0.15)
    except KeyError:
        pass
    g.bump(g.m("ADD", g.m("MULTIPLY", n1, 0.6), g.m("MULTIPLY", n2, 0.4)), 0.35, 0.008)
    M["flesh"] = mat

    # Bone plates: pale, matte, cracked.
    mat, _ = sl.new_material("regent_bone", "#cfc6b4", 0.85)
    g = G(mat)
    co = g.coord()
    warp = g.noise(co, 7.0, 3.0)
    wn = g.new("ShaderNodeTexNoise")
    g.put(wn.inputs["Vector"], co)
    wn.inputs["Scale"].default_value = 6.0
    wn.inputs["Detail"].default_value = 3.0
    addv = g.new("ShaderNodeVectorMath")
    addv.operation = "MULTIPLY_ADD"
    g.put(addv.inputs[0], wn.outputs["Color"])
    addv.inputs[1].default_value = (0.09, 0.09, 0.09)
    g.put(addv.inputs[2], co)
    wco = addv.outputs[0]                                  # warped, so cracks wander
    crack = g.voro_edge(wco, 8.5)
    crack2 = g.voro_edge(wco, 27.0)
    patchy = g.ramp(g.noise(co, 3.5, 2.0), 0.42, 0.62)     # hairline cracks only in patches
    line = g.m("MAXIMUM", g.ramp(crack, 0.0, 0.012, 1.0, 0.0),
               g.m("MULTIPLY", g.m("MULTIPLY", g.ramp(crack2, 0.0, 0.012, 1.0, 0.0), 0.5), patchy))
    stain = g.ramp(g.noise(co, 14.0, 4.0, 0.65), 0.35, 0.8)
    base = g.mix(g.ramp(warp, 0.3, 0.75), sl.rgb("#cfc6b4"), sl.rgb("#b3a993"))
    base = g.mix(g.m("MULTIPLY", stain, 0.35), base, sl.rgb("#8f8470"))
    g.set("Base Color", g.mix(g.m("MULTIPLY", line, 0.8), base, sl.rgb("#5a5145")))
    g.bump(g.m("SUBTRACT", g.m("MULTIPLY", warp, 0.5), g.m("MULTIPLY", line, 0.8)), 0.5, 0.004)
    M["bone"] = mat

    # SPINE: the dark frame, a regular lattice with 6 cm cells.
    mat, _ = sl.new_material("SPINE", "#14171a", 0.3)
    g = G(mat)
    x, y, z = g.sep(g.coord())
    strut = None
    for a in (x, y, z):
        f = g.m("ABSOLUTE", g.m("SUBTRACT", g.m("FRACT", g.m("DIVIDE", a, 0.06)), 0.5))
        s = g.ramp(f, 0.33, 0.40)
        strut = s if strut is None else g.m("MAXIMUM", strut, s)
    g.set("Base Color", g.mix(strut, sl.rgb("#030405"), sl.rgb("#14171a")))
    g.set("Roughness", 0.3)
    g.bump(strut, 0.9, 0.012)
    M["spine"] = mat

    # Registrar hardware.
    M["registrar"], _ = sl.new_material("registrar_hardware", "#0b0b0c", 0.35, 1.0)
    M["cable"], _ = sl.new_material("registrar_cable", "#0b0b0c", 0.55, 0.0)

    # Registrar plate with etched measuring circles (uses the UVMap in metres).
    mat, _ = sl.new_material("registrar_etched", "#0b0b0c", 0.35, 1.0)
    g = G(mat)
    uv = g.uv("UVMap")
    r = g.dist(uv, (0.0, 0.0, 0.0))
    etch = None
    for rr in (0.022, 0.045, 0.062):
        e = g.m("LESS_THAN", g.m("ABSOLUTE", g.m("SUBTRACT", r, rr)), 0.0013)
        etch = e if etch is None else g.m("MAXIMUM", etch, e)
    ux, uy, _z = g.sep(uv)
    ang = g.m("ARCTAN2", uy, ux)
    tick = g.m("LESS_THAN", g.m("FRACT", g.m("MULTIPLY", ang, 24.0 / (2 * math.pi))), 0.12)
    band = g.m("MULTIPLY", g.m("GREATER_THAN", r, 0.045), g.m("LESS_THAN", r, 0.052))
    etch = g.m("MAXIMUM", etch, g.m("MULTIPLY", tick, band))
    g.set("Base Color", g.mix(etch, sl.rgb("#0b0b0c"), sl.rgb("#8a6a2a")))
    g.set("Roughness", g.m("ADD", 0.35, g.m("MULTIPLY", etch, 0.2)))
    M["etch"] = mat

    # Amber seam and lens light. The dead form uses the unlit one.
    M["amber"], _ = sl.new_material("registrar_amber", "#1a0d02", 0.3, 0.0, "#ffa01e", 6.0)
    M["amber_dead"], _ = sl.new_material("registrar_amber_unlit", "#2a1a08", 0.25)

    # Electrical life.
    M["arc"], _ = sl.new_material("electrical_life", "#000000", 0.5, 0.0, "#cfe4ff", 4.0)

    # Lobes: soft warm pink-white light, uneven from lobe to lobe.
    mat, _ = sl.new_material("regent_lobe", "#b98078", 0.32)
    g = G(mat)
    info = g.new("ShaderNodeObjectInfo")
    co = g.coord()
    vein = g.voro_edge(co, 60.0)
    veinm = g.ramp(vein, 0.0, 0.035, 0.45, 1.0)
    per = g.m("ADD", 0.3, g.m("MULTIPLY", info.outputs["Random"], 1.9))
    g.set("Base Color", g.mix(veinm, sl.rgb("#6a3a44"), sl.rgb("#b98078")))
    try:
        g.set("Subsurface Weight", 0.4)
        g.set("Subsurface Radius", (0.02, 0.008, 0.006))
        g.set("Coat Weight", 0.5)
    except KeyError:
        pass
    g.set("Emission Color", sl.rgb("#ffd9c8"))
    g.set("Emission Strength", g.m("MULTIPLY", g.m("MULTIPLY", per, veinm), 2.0 / 1.25))
    g.bump(vein, 0.3, 0.003)
    M["lobe"] = mat

    mat, _ = sl.new_material("regent_lobe_dead", "#77726c", 0.65)
    g = G(mat)
    vein = g.voro_edge(g.coord(), 85.0)
    g.set("Base Color", g.mix(g.ramp(vein, 0.0, 0.06), sl.rgb("#3d3a38"), sl.rgb("#7c7771")))
    g.bump(vein, 0.4, 0.003)
    M["lobe_dead"] = mat

    # Dome: clear, glass-like. Shadow rays pass so daylight reaches the lobes.
    for key, rough, tint in (("glass", 0.03, (0.93, 0.96, 0.95, 1.0)),
                             ("glass_dead", 0.16, (0.62, 0.64, 0.62, 1.0))):
        mat, _ = sl.new_material("regent_dome_" + key, "#ffffff", rough)
        g = G(mat)
        g.set("Base Color", tint)
        g.set("Transmission Weight", 1.0)
        g.set("IOR", 1.35)
        dust = g.noise(g.coord(), 24.0, 4.0)
        g.set("Roughness", g.ramp(dust, 0.45, 0.8, rough, rough + 0.14))
        lp = g.new("ShaderNodeLightPath")
        tr = g.new("ShaderNodeBsdfTransparent")
        tr.inputs[0].default_value = (0.9, 0.93, 0.92, 1.0)
        mx = g.new("ShaderNodeMixShader")
        g.nt.links.new(lp.outputs["Is Shadow Ray"], mx.inputs[0])
        g.nt.links.new(g.bsdf.outputs[0], mx.inputs[1])
        g.nt.links.new(tr.outputs[0], mx.inputs[2])
        g.nt.links.new(mx.outputs[0], g.out.inputs["Surface"])
        M[key] = mat

    # Cloth: dirty off-white with dull gold diagrams of our own.
    mat, _ = sl.new_material("regent_cloth", "#b8ad98", 0.9)
    g = G(mat)
    uv = g.uv("UVMap")               # metres: x across (-0.125..0.125), y along
    u0, v0, _z = g.sep(uv)
    yy = g.m("MULTIPLY", g.m("FRACT", g.m("DIVIDE", v0, CLOTH_L)), CLOTH_L)
    cmb = g.new("ShaderNodeCombineXYZ")
    g.put(cmb.inputs[0], u0)
    g.put(cmb.inputs[1], yy)
    p = cmb.outputs[0]
    ax = g.m("ABSOLUTE", u0)
    marks = []

    def lt(a, b):
        return g.m("LESS_THAN", a, b)

    def between(a, lo, hi):
        return g.m("MULTIPLY", g.m("GREATER_THAN", a, lo), g.m("LESS_THAN", a, hi))

    def ring(cx, cy, r, w=0.008):
        d = g.dist(p, (cx, cy, 0.0))
        marks.append(lt(g.m("ABSOLUTE", g.m("SUBTRACT", d, r)), w / 2))
        return d

    def dot(cx, cy, r):
        marks.append(lt(g.dist(p, (cx, cy, 0.0)), r))

    def ticks(cx, cy, r0, r1, count, duty=0.16):
        d = g.dist(p, (cx, cy, 0.0))
        a = g.m("ARCTAN2", g.m("SUBTRACT", yy, cy), g.m("SUBTRACT", u0, cx))
        t = lt(g.m("FRACT", g.m("MULTIPLY", a, count / (2 * math.pi))), duty)
        marks.append(g.m("MULTIPLY", t, between(d, r0, r1)))

    ring(0.0, 0.24, 0.085)
    ring(0.0, 0.24, 0.052, 0.006)
    dot(0.0, 0.24, 0.008)
    ticks(0.0, 0.24, 0.085, 0.099, 24)
    ring(-0.05, 0.52, 0.028, 0.006)
    ring(0.05, 0.52, 0.028, 0.006)
    dot(-0.05, 0.52, 0.006)
    dot(0.05, 0.52, 0.006)
    ring(0.0, 0.82, 0.098)
    ring(0.0, 0.82, 0.066, 0.006)
    ticks(0.0, 0.82, 0.066, 0.080, 12, 0.2)
    ring(0.0, 0.82, 0.02, 0.006)
    d = ring(0.0, 1.13, 0.06)
    marks[-1] = g.m("MULTIPLY", marks[-1], lt(yy, 1.13))        # half circle
    marks.append(g.m("MULTIPLY", lt(g.m("ABSOLUTE", g.m("SUBTRACT", yy, 1.13)), 0.002), lt(ax, 0.085)))
    ring(0.0, 1.36, 0.042)
    dot(0.0, 1.36, 0.007)
    # Border lines and a ruler down one edge.
    marks.append(lt(g.m("ABSOLUTE", g.m("SUBTRACT", ax, 0.108)), 0.003))
    marks.append(g.m("MULTIPLY", lt(g.m("FRACT", g.m("DIVIDE", yy, 0.025)), 0.14), between(u0, 0.108, 0.118)))
    marks.append(g.m("MULTIPLY", lt(g.m("FRACT", g.m("DIVIDE", yy, 0.125)), 0.035), between(u0, 0.094, 0.118)))
    # Dashed centre line and two crossing construction lines.
    marks.append(g.m("MULTIPLY", lt(ax, 0.0028), lt(g.m("FRACT", g.m("DIVIDE", yy, 0.05)), 0.6)))
    slope = g.m("MULTIPLY", g.m("SUBTRACT", yy, 0.52), 0.42)
    cross = lt(g.m("ABSOLUTE", g.m("SUBTRACT", ax, g.m("ABSOLUTE", slope))), 0.003)
    marks.append(g.m("MULTIPLY", cross, between(yy, 0.325, 0.722)))
    gold = marks[0]
    for mk in marks[1:]:
        gold = g.m("MAXIMUM", gold, mk)

    co = g.coord()
    uv2 = g.uv("len")                # x 0..1 across, y 0 at the top, 1 at the frayed end
    _a, s01, _b = g.sep(uv2)
    dirt = g.noise(co, 5.0, 5.0, 0.6)
    weave = g.noise(g.scale_vec(co, (1.0, 1.0, 1.0)), 420.0, 1.0)
    worn = g.ramp(g.noise(co, 30.0, 3.0), 0.42, 0.62)            # gold rubbed away in places
    low = g.ramp(s01, 0.45, 1.0)                                   # dirtier toward the hem
    base = g.mix(g.ramp(dirt, 0.35, 0.75), sl.rgb("#b8ad98"), sl.rgb("#8a7f6b"))
    base = g.mix(g.m("MULTIPLY", low, 0.75), base, sl.rgb("#51483a"))
    goldf = g.m("MULTIPLY", gold, g.m("SUBTRACT", 1.0, g.m("MULTIPLY", worn, 0.55)))
    g.set("Base Color", g.mix(goldf, base, sl.rgb("#a8842c")))
    g.set("Metallic", g.m("MULTIPLY", goldf, 0.55))
    g.set("Roughness", g.m("SUBTRACT", 0.92, g.m("MULTIPLY", goldf, 0.4)))
    # Frayed end: threads thin out over the last 14 cm. A few small holes.
    thread = g.noise(g.scale_vec(uv, (700.0, 6.0, 1.0)), 1.0, 2.0)
    edge = g.ramp(s01, 1.0 - 0.24 / CLOTH_L, 1.0)
    keep = g.m("GREATER_THAN", g.m("SUBTRACT", thread, g.m("MULTIPLY", edge, 0.62)), 0.12)
    hole = g.m("LESS_THAN", g.noise(co, 11.0, 2.0), 0.74)
    g.set("Alpha", g.m("MULTIPLY", keep, hole))
    g.bump(g.m("ADD", g.m("MULTIPLY", weave, 0.5), g.m("MULTIPLY", dirt, 0.5)), 0.35, 0.002)
    M["cloth"] = mat
    return M


# ---------------------------------------------------------------------------
# Geometry helper: one Part becomes one object
# ---------------------------------------------------------------------------
class Part:
    def __init__(self, name, origin, parent=None, xf=None):
        self.name = name
        self.xf = xf.copy() if xf is not None else I4.copy()
        self.origin = Vector(origin)        # already in the final (posed) place
        self.parent = parent
        self.bm = bmesh.new()
        self.uv = self.bm.loops.layers.uv.new("UVMap")
        self.uv2 = self.bm.loops.layers.uv.new("len")
        self.mats = []

    # -- low level --
    def _p(self, p):
        return self.xf @ Vector(p)

    def _d(self, d):
        return self.xf.to_3x3() @ Vector(d)

    def _tag(self, faces, mat, smooth=True):
        if mat not in self.mats:
            self.mats.append(mat)
        i = self.mats.index(mat)
        for f in faces:
            f.material_index = i
            f.smooth = smooth

    def _face(self, verts, uvs=None, uvs2=None):
        try:
            f = self.bm.faces.new(verts)
        except ValueError:
            return None
        if uvs is not None:
            for lp in f.loops:
                lp[self.uv].uv = uvs[lp.vert]
                if uvs2 is not None:
                    lp[self.uv2].uv = uvs2[lp.vert]
        return f

    # -- shapes --
    def sweep(self, pts, secs, mat, up=None, cap=True, smooth=True):
        pts = [self._p(p) for p in pts]
        n = len(pts)
        tans = [(pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized() for i in range(n)]
        if up is None:
            t = tans[0]
            ref = V(0, 0, 1) if abs(t.z) < 0.9 else V(1, 0, 0)
            nrm = (ref - t * ref.dot(t)).normalized()
        rings = []
        for i, t in enumerate(tans):
            if up is not None:
                u = self._d(up[i] if isinstance(up, list) else up)
                nrm = (u - t * u.dot(t)).normalized()
            else:
                nrm = (nrm - t * nrm.dot(t)).normalized()
            b = t.cross(nrm)
            rings.append([self.bm.verts.new(pts[i] + nrm * a + b * c) for a, c in secs[i]])
        faces = []
        m = len(rings[0])
        for i in range(n - 1):
            for k in range(m):
                k2 = (k + 1) % m
                faces.append(self._face((rings[i][k], rings[i][k2], rings[i + 1][k2], rings[i + 1][k])))
        if cap:
            faces.append(self._face(list(reversed(rings[0]))))
            faces.append(self._face(rings[-1]))
        self._tag([f for f in faces if f], mat, smooth)

    def tube(self, pts, radii, mat, segs=8, cap=True):
        if isinstance(radii, (int, float)):
            radii = [radii] * len(pts)
        secs = [[(r * math.cos(2 * math.pi * k / segs), r * math.sin(2 * math.pi * k / segs))
                 for k in range(segs)] for r in radii]
        self.sweep(pts, secs, mat, cap=cap)

    def limb(self, a, b, r0, r1, mat, bow=None, knob=(0.42, 0.34), n=11, segs=10):
        """A living limb segment: thin in the middle, knobbed at both joints."""
        a, b = Vector(a), Vector(b)
        pts, rr = [], []
        for i in range(n):
            t = i / (n - 1)
            p = a.lerp(b, t)
            if bow is not None:
                p = p + Vector(bow) * math.sin(math.pi * t)
            k = 1.0 + knob[0] * math.exp(-(t / 0.12) ** 2) + knob[1] * math.exp(-((1 - t) / 0.10) ** 2)
            pts.append(p)
            rr.append((r0 + (r1 - r0) * t) * k)
        self.tube(pts, rr, mat, segs=segs)

    def bar(self, pts, w, t, mat, up=None):
        """Square-edged bar (Registrar hardware): flat faces, sharp edges."""
        sec = [(-w / 2, -t / 2), (w / 2, -t / 2), (w / 2, t / 2), (-w / 2, t / 2)]
        self.sweep(pts, [sec] * len(pts), mat, up=up, smooth=False)

    def ball(self, c, r, mat, rot=None, u=12, v=8, smooth=True):
        if isinstance(r, (int, float)):
            r = (r, r, r)
        m = Matrix.Translation(Vector(c)) @ (rot.to_4x4() if rot is not None else I4) \
            @ Matrix.Diagonal((r[0], r[1], r[2], 1.0))
        res = bmesh.ops.create_uvsphere(self.bm, u_segments=u, v_segments=v, radius=1.0,
                                        matrix=self.xf @ m)
        self._tag({f for vv in res["verts"] for f in vv.link_faces}, mat, smooth)

    def box(self, c, size, mat, rot=None):
        m = Matrix.Translation(Vector(c)) @ (rot.to_4x4() if rot is not None else I4) \
            @ Matrix.Diagonal((size[0], size[1], size[2], 1.0))
        res = bmesh.ops.create_cube(self.bm, size=1.0, matrix=self.xf @ m)
        self._tag({f for vv in res["verts"] for f in vv.link_faces}, mat, False)

    def revolve(self, prof, o, ex, ey, ez, n, mat, loop=False, smooth=True):
        o, ex, ey, ez = Vector(o), Vector(ex), Vector(ey), Vector(ez)
        rings, uvs = [], {}
        for r, z in prof:
            if r < 1e-6:
                vv = self.bm.verts.new(self._p(o + ez * z))
                uvs[vv] = (0.0, 0.0)
                rings.append([vv])
            else:
                ring = []
                for k in range(n):
                    a = 2 * math.pi * k / n
                    vv = self.bm.verts.new(self._p(o + ex * (r * math.cos(a)) + ey * (r * math.sin(a)) + ez * z))
                    uvs[vv] = (r * math.cos(a), r * math.sin(a))
                    ring.append(vv)
                rings.append(ring)
        pairs = list(zip(rings, rings[1:])) + ([(rings[-1], rings[0])] if loop else [])
        faces = []
        for a, b in pairs:
            if len(a) == 1 and len(b) == 1:
                continue
            for k in range(n):
                k2 = (k + 1) % n
                if len(a) == 1:
                    faces.append(self._face((a[0], b[k], b[k2]), uvs))
                elif len(b) == 1:
                    faces.append(self._face((a[k], a[k2], b[0]), uvs))
                else:
                    faces.append(self._face((a[k], a[k2], b[k2], b[k]), uvs))
        self._tag([f for f in faces if f], mat, smooth)

    def shell(self, fn, nu, nv, thick, inside, mat):
        """A plate: the surface fn(u, v) given thickness toward the point `inside`."""
        inside = Vector(inside)
        P = [[Vector(fn(i / nu, j / nv)) for j in range(nv + 1)] for i in range(nu + 1)]
        outer = [[self.bm.verts.new(self._p(P[i][j])) for j in range(nv + 1)] for i in range(nu + 1)]
        inner = []
        for i in range(nu + 1):
            row = []
            for j in range(nv + 1):
                a = P[min(i + 1, nu)][j] - P[max(i - 1, 0)][j]
                b = P[i][min(j + 1, nv)] - P[i][max(j - 1, 0)]
                nn = a.cross(b)
                nn = nn.normalized() if nn.length > 1e-9 else (P[i][j] - inside).normalized()
                if nn.dot(P[i][j] - inside) < 0:
                    nn = -nn
                row.append(self.bm.verts.new(self._p(P[i][j] - nn * thick)))
            inner.append(row)
        faces = []
        for i in range(nu):
            for j in range(nv):
                faces.append(self._face((outer[i][j], outer[i + 1][j], outer[i + 1][j + 1], outer[i][j + 1])))
                faces.append(self._face((inner[i][j + 1], inner[i + 1][j + 1], inner[i + 1][j], inner[i][j])))
        for i in range(nu):
            faces.append(self._face((outer[i][0], inner[i][0], inner[i + 1][0], outer[i + 1][0])))
            faces.append(self._face((outer[i + 1][nv], inner[i + 1][nv], inner[i][nv], outer[i][nv])))
        for j in range(nv):
            faces.append(self._face((outer[0][j + 1], inner[0][j + 1], inner[0][j], outer[0][j])))
            faces.append(self._face((outer[nu][j], inner[nu][j], inner[nu][j + 1], outer[nu][j + 1])))
        self._tag([f for f in faces if f], mat, True)

    def limb_plate(self, a, b, t0, t1, r, facing, arc_deg, mat, rng, thick=0.009, flare=0.30):
        a, b = Vector(a), Vector(b)
        ax = (b - a).normalized()
        f = Vector(facing)
        f = (f - ax * f.dot(ax)).normalized()
        gq = ax.cross(f)
        jit = [[rng.uniform(-1, 1) for _ in range(6)] for _ in range(8)]

        def fn(u, v):
            t = t0 + (t1 - t0) * v
            edge = max(abs(u - 0.5), abs(v - 0.5)) * 2
            t += 0.012 * jit[int(u * 7)][int(v * 5)] * (edge > 0.9)
            ang = rad(arc_deg) * (u - 0.5)
            rr = r * (1.0 + 0.10 * math.sin(math.pi * v) + flare * (1 - v) ** 2.5) * (1.0 + 0.16 * math.cos(ang))
            return a + (b - a) * t + (f * math.cos(ang) + gq * math.sin(ang)) * rr
        mid = a + (b - a) * ((t0 + t1) / 2)
        self.shell(fn, 7, 5, thick, mid, mat)

    def cap_plate(self, c, pole, r, cap_deg, mat, rng, thick=0.012, squash=(1.0, 1.0)):
        c = Vector(c)
        pole = Vector(pole).normalized()
        ref = V(0, 0, 1) if abs(pole.z) < 0.9 else V(0, 1, 0)
        e1 = pole.cross(ref).normalized()
        e2 = pole.cross(e1)
        k = math.tan(rad(cap_deg))
        jit = [[rng.uniform(-1, 1) for _ in range(8)] for _ in range(8)]

        def fn(u, v):
            a, b = 2 * u - 1, 2 * v - 1
            a2 = a * math.sqrt(1 - b * b / 2)
            b2 = b * math.sqrt(1 - a * a / 2)
            edge = math.hypot(a2, b2)
            kk = k * (1.0 + 0.10 * jit[int(u * 7)][int(v * 7)] * (edge > 0.93))
            d = (pole + e1 * (a2 * kk * squash[0]) + e2 * (b2 * kk * squash[1])).normalized()
            return c + d * r
        self.shell(fn, 7, 7, thick, c, mat)

    def cloth(self, at, mat, uv_off=0.0, flip=1.0, seed=0, nu=10, nv=30, billow=1.0):
        """at(s) -> (centre, across unit, normal unit) for s in metres along the panel."""
        rng = random.Random(seed)
        ph = rng.uniform(0, 6.28)
        ph2 = rng.uniform(0, 6.28)
        fray = [rng.uniform(0.0, 0.17) for _ in range(nu + 1)]
        grid, uvs, uvs2 = [], {}, {}
        for i in range(nu + 1):
            col = []
            u = i / nu - 0.5
            for j in range(nv + 1):
                f = j / nv
                s = CLOTH_L * f - fray[i] * f ** 8
                c, across, nrm = at(s)
                grow = min(1.0, s / 0.35)
                gather = (0.72 + 0.28 * grow) * (1.0 - 0.10 * f ** 3)
                fold = (0.026 * math.sin(2 * math.pi * (1.3 * u) + ph + 2.6 * f)
                        + 0.011 * math.sin(2 * math.pi * (2.9 * u) + 2 * ph - 3.4 * f)) * grow
                fold += 0.045 * math.sin(math.pi * 1.4 * f + ph2) * f * billow
                vv = self.bm.verts.new(self._p(c + across * (u * CLOTH_W * gather) + nrm * fold))
                uvs[vv] = (flip * u * CLOTH_W, s + uv_off)
                uvs2[vv] = (i / nu, s / CLOTH_L)
                col.append(vv)
            grid.append(col)
        faces = []
        for i in range(nu):
            for j in range(nv):
                faces.append(self._face((grid[i][j], grid[i + 1][j], grid[i + 1][j + 1], grid[i][j + 1]), uvs, uvs2))
        self._tag([f for f in faces if f], mat, True)

    # -- finish --
    def finish(self, col, mats, reg):
        bm = self.bm
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
        bmesh.ops.translate(bm, verts=bm.verts[:], vec=-self.origin)
        me = bpy.data.meshes.new(self.name)
        bm.to_mesh(me)
        bm.free()
        for mname in self.mats:
            me.materials.append(mats[mname])
        if hasattr(me, "set_sharp_from_angle"):
            me.set_sharp_from_angle(angle=rad(42))
        ob = bpy.data.objects.new(self.name, me)
        col.objects.link(ob)
        par = self.parent
        ob.parent = par
        ob.location = self.origin - reg[par.name]
        reg[ob.name] = self.origin.copy()
        return ob


def seg_xf(a_alive, b_alive, a_posed, dir_posed, parent_xf):
    """Rigid move of one limb segment: joint a goes to a_posed, the segment
    then points along dir_posed. With no direction it just follows its parent."""
    if dir_posed is None:
        return parent_xf.copy()
    d0 = parent_xf.to_3x3() @ (Vector(b_alive) - Vector(a_alive)).normalized()
    q = d0.rotation_difference(Vector(dir_posed).normalized())
    rot = q.to_matrix().to_4x4() @ parent_xf.to_3x3().to_4x4()
    return Matrix.Translation(Vector(a_posed)) @ rot @ Matrix.Translation(-Vector(a_alive))


def path_fn(waypoints, across_list):
    """Smooth path through waypoints. Returns at(s) -> (centre, across, normal)."""
    pts = [Vector(p) for p in waypoints]
    acr = [Vector(a).normalized() for a in across_list]
    for _ in range(3):                                   # corner cutting, ends kept
        np_, na = [pts[0]], [acr[0]]
        for i in range(len(pts) - 1):
            np_ += [pts[i] * 0.75 + pts[i + 1] * 0.25, pts[i] * 0.25 + pts[i + 1] * 0.75]
            na += [acr[i] * 0.75 + acr[i + 1] * 0.25, acr[i] * 0.25 + acr[i + 1] * 0.75]
        np_.append(pts[-1])
        na.append(acr[-1])
        pts, acr = np_, na
    cum = [0.0]
    for i in range(len(pts) - 1):
        cum.append(cum[-1] + (pts[i + 1] - pts[i]).length)

    def at(s):
        s = max(0.0, s)
        i = len(pts) - 2
        for k in range(len(pts) - 1):
            if s <= cum[k + 1]:
                i = k
                break
        seg = max(cum[i + 1] - cum[i], 1e-9)
        t = (s - cum[i]) / seg
        c = pts[i].lerp(pts[i + 1], t) if t <= 1 else pts[i + 1] + (pts[i + 1] - pts[i]) / seg * (s - cum[i + 1])
        tan = (pts[i + 1] - pts[i]).normalized()
        a = acr[i].lerp(acr[i + 1], min(t, 1.0))
        a = (a - tan * a.dot(tan)).normalized()
        return c, a, tan.cross(a).normalized()
    return at


# ---------------------------------------------------------------------------
# Poses. A pose only lists the directions that differ from the living stance.
# Directions are in the body's own frame: +Z up the body, -Y its front,
# +X its own left side.
# ---------------------------------------------------------------------------
DEAD_POSE = {
    # right side legs lie on the ground side, folded
    "leg_FR_thigh": (-0.10, -0.72, 0.62), "leg_FR_shin": (0.05, 0.42, -0.90), "leg_FR_foot": (0.05, 0.75, -0.5),
    "leg_RR_thigh": (-0.10, 0.55, 0.80), "leg_RR_shin": (0.03, -0.27, -0.96), "leg_RR_foot": (0.0, -0.7, -0.6),
    # left side legs fall over the body
    "leg_FL_thigh": (0.22, -0.72, 0.55), "leg_FL_shin": (-0.26, 0.22, -0.94), "leg_FL_foot": (-0.3, 0.6, -0.5),
    "leg_RL_thigh": (0.15, 0.62, 0.70), "leg_RL_shin": (-0.235, -0.35, -0.91),
    # upper arms go limp
    "arm_UR_upper": (-0.05, -0.78, -0.60), "arm_UR_fore": (0.0, -0.50, -0.86), "arm_UR_claws": (0.0, -0.4, -0.9),
    "arm_UL_upper": (0.10, -0.80, -0.55), "arm_UL_fore": (-0.75, -0.15, -0.65), "arm_UL_claws": (-0.8, -0.1, -0.6),
}


# ---------------------------------------------------------------------------
# The figure
# ---------------------------------------------------------------------------
def build_figure(col, mats_all, dead):
    sfx = "_dead" if dead else ""
    pose = DEAD_POSE if dead else {}
    mats = dict(mats_all)
    if dead:
        mats["amber"] = mats_all["amber_dead"]
        mats["lobe"] = mats_all["lobe_dead"]
        mats["glass"] = mats_all["glass_dead"]
    rng = random.Random(7)
    reg = {}

    root = bpy.data.objects.new("regent" + sfx, None)
    root.empty_display_size = 0.3
    col.objects.link(root)
    reg[root.name] = V(0, 0, 0)
    objs = []

    def N(name):
        return name + sfx

    def done(part):
        ob = part.finish(col, mats, reg)
        objs.append(ob)
        return ob

    # The dead body lies on its right side, pelvis a little lower, front a
    # little up. R turns body directions into world directions.
    if dead:
        R = Matrix.Rotation(rad(-90 + 5), 3, "Y") @ Matrix.Rotation(rad(9), 3, "Z")
    else:
        R = Matrix.Identity(3)

    # ---- torso ----
    axis = V(0, -math.sin(LEAN), math.cos(LEAN))
    back = V(0, math.cos(LEAN), math.sin(LEAN))
    P0 = V(0, 0.14, TORSO_BOTTOM)
    lean_rot = Matrix.Rotation(LEAN, 3, "X")
    half_w = TORSO_W / 2
    TOR = [(0.0, 0.2, 0.3), (0.05, 0.57, 0.75), (0.18, 0.86, 1.0), (0.30, 0.77, 0.95), (0.46, 0.54, 0.75),
           (0.62, 0.71, 0.85), (0.80, 0.94, 1.0), (0.91, 1.0, 0.95), (0.97, 0.69, 0.8), (1.0, 0.34, 0.55)]

    def tor(s):
        for (s0, w0, d0), (s1, w1, d1) in zip(TOR, TOR[1:]):
            if s <= s1:
                t = (s - s0) / (s1 - s0)
                t = t * t * (3 - 2 * t)
                return (w0 + (w1 - w0) * t) * half_w, (d0 + (d1 - d0) * t) * 0.125
        return TOR[-1][1] * half_w, TOR[-1][2] * 0.125

    def A(s):
        return P0 + axis * (TORSO_LEN * s) + back * (0.06 * math.sin(math.pi * s))

    torso = Part(N("torso"), A(0.18), root)
    stations = [i / 26 for i in range(27)]
    secs = []
    for s in stations:
        w, d = tor(s)
        rip = 1.0 + (0.07 * math.sin(2 * math.pi * 7 * s) if 0.08 < s < 0.88 else 0.0)
        secs.append([(w * rip * math.cos(2 * math.pi * k / 16), d * rip * math.sin(2 * math.pi * k / 16))
                     for k in range(16)])
    torso.sweep([A(s) for s in stations], secs, "flesh", up=V(1, 0, 0))
    # the frame shows down the back and as two girdles under the plates
    ridge = [i / 14 * 0.84 + 0.08 for i in range(15)]
    torso.tube([A(s) + back * (tor(s)[1] + 0.006) for s in ridge], 0.024, "spine", segs=8)
    torso.ball(A(0.89), (half_w + 0.02, 0.135, 0.07), "spine", rot=lean_rot, u=20, v=10)
    torso.ball(A(0.18), (half_w - 0.005, 0.14, 0.075), "spine", rot=lean_rot, u=20, v=10)
    torso_ob = done(torso)

    # ---- joints of the living stance ----
    S_up = {s: A(0.90) + V(s * 0.20, -0.01, 0.0) for s in (1, -1)}
    S_lo = {s: A(0.62) + V(s * 0.135, -0.02, 0.0) for s in (1, -1)}
    half = FEET_SQUARE / 2
    legs = {
        # tag: (sx, sy, hip, knee, ankle, foot)
        "FL": (1, -1, A(0.2) + V(0.15, -0.07, -0.01), V(0.47, -0.45, KNEE_H - 0.04), V(0.62, -0.60, 0.45), V(half, -half, 0)),
        "FR": (-1, -1, A(0.2) + V(-0.15, -0.07, -0.01), V(-0.55, -0.37, KNEE_H + 0.05), V(-0.61, -0.62, 0.40), V(-half, -half, 0)),
        "RL": (1, 1, A(0.2) + V(0.15, 0.08, -0.04), V(0.50, 0.56, KNEE_H), None, V(half, half, 0)),
        "RR": (-1, 1, A(0.2) + V(-0.15, 0.08, -0.04), V(-0.48, 0.58, KNEE_H - 0.02), V(-0.63, 0.61, 0.46), V(-half, half, 0)),
    }

    # ---- shoulder plates ----
    for s, tag in ((1, "L"), (-1, "R")):
        pl = Part(N("plate_shoulder_" + tag), S_up[s], torso_ob)
        pl.cap_plate(S_up[s] + V(-s * 0.02, 0.01, -0.02), (s * 0.62, 0.05, 0.78), 0.135, 66, "bone", rng,
                     squash=(1.0, 0.95))
        done(pl)
        pl = Part(N("plate_shoulder_low_" + tag), S_lo[s], torso_ob)
        pl.cap_plate(S_lo[s] + V(-s * 0.01, 0.0, -0.01), (s * 0.8, -0.1, 0.55), 0.078, 62, "bone", rng, thick=0.009)
        done(pl)

    # ---- legs ----
    r_leg = LEG_T / 2
    knees_posed = {}
    for tag, (sx, sy, hip, knee, ankle, foot) in legs.items():
        strut = ankle is None
        name = "leg_" + tag
        x_th = seg_xf(hip, knee, hip, pose.get(name + "_thigh"), I4)
        th = Part(N(name + "_thigh"), hip, torso_ob, x_th)
        th.ball(hip, 0.058, "spine")
        th.cap_plate(hip + V(-sx * 0.012, -sy * 0.01, -0.012), (sx * 0.72, sy * 0.38, 0.58), 0.128, 70, "bone", rng)
        th.limb(hip, knee, r_leg + 0.018, r_leg + 0.002, "flesh", bow=V(0, 0, rng.uniform(0.03, 0.055)), knob=(0.55, 0.40))
        th_ob = done(th)
        knee_p = x_th @ knee
        knees_posed[tag] = knee_p
        out = V(foot.x, foot.y, 0).normalized()
        if strut:
            # The wrong repair: a straight Registrar strut with a flat pad.
            x_sh = seg_xf(knee, foot, knee_p, pose.get(name + "_shin"), x_th)
            sh = Part(N(name + "_strut"), knee_p, th_ob, x_sh)
            d = (foot - knee).normalized()
            q = V(0, 0, 1).rotation_difference(d).to_matrix()
            sh.ball(knee, 0.040, "spine")
            sh.box(knee + d * 0.105, (0.085, 0.085, 0.13), "registrar", rot=q)      # the plug
            sh.box(knee + d * 0.038, (0.092, 0.092, 0.007), "amber", rot=q)         # the seam
            sh.box(knee + d * 0.19, (0.066, 0.066, 0.04), "registrar", rot=q)
            top = V(foot.x, foot.y, 0.06)
            sh.bar([knee + d * 0.17, top], 0.046, 0.046, "registrar", up=out)
            sh.box(V(foot.x, foot.y, 0.055), (0.075, 0.075, 0.06), "registrar",
                   rot=Matrix.Rotation(rad(45), 3, "Z"))
            sh.box(V(foot.x, foot.y, 0.0125), (0.20, 0.20, 0.025), "registrar",
                   rot=Matrix.Rotation(rad(45), 3, "Z"))
            done(sh)
            continue
        x_sh = seg_xf(knee, ankle, knee_p, pose.get(name + "_shin"), x_th)
        sh = Part(N(name + "_shin"), knee_p, th_ob, x_sh)
        sh.ball(knee, 0.040, "spine")
        sh.limb(knee, ankle, r_leg + 0.002, r_leg - 0.004, "flesh", bow=out * 0.035)
        sh.cap_plate(knee + V(0, 0, -0.01), out + V(0, 0, 0.9), 0.062, 58, "bone", rng, thick=0.009)
        bowed = lambda t: knee.lerp(ankle, t) + out * (0.035 * math.sin(math.pi * t))
        c1, c2 = rng.uniform(0.36, 0.50), rng.uniform(0.68, 0.78)
        for t0, t1 in ((0.06, c1), (c1 + 0.022, c2), (c2 + 0.022, 0.93)):
            sh.limb_plate(bowed(t0), bowed(t1), 0.0, 1.0, r_leg + 0.009, out, 250, "bone", rng)
        for t in ():
            c = knee.lerp(ankle, t)
            dd = (ankle - knee).normalized() * 0.022
            sh.tube([c - dd, c + dd], r_leg + 0.006, "spine")
        sh_ob = done(sh)
        ankle_p = x_sh @ ankle
        x_ft = seg_xf(ankle, foot, ankle_p, pose.get(name + "_foot"), x_sh)
        ft = Part(N(name + "_foot"), ankle_p, sh_ob, x_ft)
        ft.ball(ankle, 0.031, "spine")
        padc = V(foot.x, foot.y, 0.02)
        ft.limb(ankle, padc + V(0, 0, 0.012), r_leg - 0.004, r_leg - 0.002, "flesh", knob=(0.35, 0.5))
        ft.ball(padc, (0.058, 0.058, 0.02), "flesh")
        for a in (-48, 0, 48):
            td = Matrix.Rotation(rad(a), 3, "Z") @ out
            ft.tube([padc + td * 0.03 + V(0, 0, -0.002), padc + td * 0.085 + V(0, 0, -0.003), padc + td * 0.13 + V(0, 0, -0.012)],
                    [0.018, 0.015, 0.008], "flesh", segs=6)
            ft.tube([padc + td * 0.118 + V(0, 0, -0.008), padc + td * 0.15 + V(0, 0, -0.016)], [0.008, 0.003], "bone", segs=6)
        done(ft)

    # ---- arms: four, in two pairs ----
    seg_u = (ARM_LEN - CLAW_LEN) * 0.48
    seg_f = (ARM_LEN - CLAW_LEN) - seg_u
    arm_dirs = {
        # upper pair: held forward, half raised, as if about to reach
        "UL": (S_up[1], unit(0.12, -0.30, -0.95), unit(-0.03, -0.60, 0.80), unit(-0.05, -0.35, -0.94), 0.050),
        "UR": (S_up[-1], unit(-0.14, -0.30, -0.95), unit(0.04, -0.68, 0.73), unit(0.05, -0.35, -0.94), 0.050),
        # lower pair: folded against the torso
        "LL": (S_lo[1], unit(0.0, -0.27, -0.96), unit(-0.07, -0.13, 0.99), unit(-0.25, 0.75, 0.6), 0.040),
        "LR": (S_lo[-1], unit(0.0, -0.27, -0.96), unit(0.07, -0.13, 0.99), unit(0.25, 0.75, 0.6), 0.040),
    }
    shoulders_posed = {}
    for tag, (S, d_up, d_fore, curl, r_joint) in arm_dirs.items():
        sx = 1 if tag[1] == "L" else -1
        name = "arm_" + tag
        E = S + d_up * seg_u
        W = E + d_fore * seg_f
        x_u = seg_xf(S, E, S, pose.get(name + "_upper"), I4)
        up_ = Part(N(name + "_upper"), S, torso_ob, x_u)
        up_.ball(S, r_joint, "spine")
        up_.limb(S, E, 0.031, 0.025, "flesh", knob=(0.5, 0.35))
        up_ob = done(up_)
        shoulders_posed[tag] = S
        E_p = x_u @ E
        x_f = seg_xf(E, W, E_p, pose.get(name + "_fore"), x_u)
        fo = Part(N(name + "_fore"), E_p, up_ob, x_f)
        fo.ball(E, 0.031, "spine")
        fo.limb(E, W, 0.025, 0.020, "flesh")
        facing = V(sx * 0.8, -0.3, 0.5)
        for t0, t1 in ((0.10, 0.50), (0.54, 0.93)):
            fo.limb_plate(E, W, t0, t1, 0.034, facing, 250, "bone", rng, thick=0.007)
        fo_ob = done(fo)
        W_p = x_f @ W
        x_c = seg_xf(W, W + d_fore, W_p, pose.get(name + "_claws"), x_f)
        cl = Part(N(name + "_claws"), W_p, fo_ob, x_c)
        cl.ball(W, 0.024, "spine")
        side = d_fore.cross(curl).normalized()
        for k in (-1, 0, 1):
            base = (d_fore + side * (0.42 * k)).normalized()
            pts, rr = [], []
            for i in range(7):
                t = i / 6
                pts.append(W + base * (CLAW_LEN * 0.92 * t) + curl * (CLAW_LEN * 0.42 * t * t) + side * (0.012 * k))
                rr.append(0.0135 * (1 - t) ** 0.8 + 0.002)
            cl.tube(pts, rr, "bone", segs=6)
        done(cl)

    # ---- head: neck, cradle, dome, lobes, head plate ----
    tilt = rad(10)
    ex = V(1, 0, 0)
    ez = V(0, -math.sin(tilt), math.cos(tilt))            # the dome's own up
    eyb = ez.cross(ex)                                    # toward the back of the head
    hf = -eyb
    R_d = DOME_D / 2
    top_rel = DOME_STRAIGHT * ez.z + R_d                  # highest point of the tilted dome above H
    H = V(0, -0.27, HEIGHT - top_rel)
    P1 = A(1.0)
    neck = Part(N("neck"), P1, torso_ob)
    nb = H - ez * 0.13
    neck.tube([A(0.97), P1 + V(0, -0.02, 0.03), nb.lerp(P1, 0.35), nb], [0.062, 0.055, 0.05, 0.055], "flesh")
    neck.ball(nb + V(0, 0.0, -0.01), (0.085, 0.085, 0.04), "spine")
    neck_ob = done(neck)

    head = Part(N("head"), H, neck_ob)
    head.revolve([(0.0, 0.0), (R_d - 0.012, 0.0), (R_d + 0.010, 0.010), (R_d + 0.018, -0.008),
                  (R_d + 0.012, -0.034), (R_d - 0.004, -0.040)], H, ex, eyb, ez, 32, "bone")
    head.revolve([(R_d - 0.004, -0.036), (0.255, -0.060), (0.215, -0.098), (0.15, -0.130), (0.08, -0.150),
                  (0.0, -0.156)], H, ex, eyb, ez, 32, "flesh")
    head.revolve([(0.0, 0.002), (R_d - 0.014, 0.002)], H, ex, eyb, ez, 32, "flesh")
    head_ob = done(head)

    dome = Part(N("dome_glass"), H, head_ob)
    Ri = R_d - 0.007
    arcn = 9
    prof = [(R_d, 0.0), (R_d, DOME_STRAIGHT)]
    prof += [(R_d * math.cos(rad(90 * i / arcn)), DOME_STRAIGHT + R_d * math.sin(rad(90 * i / arcn))) for i in range(1, arcn)]
    prof += [(0.0, DOME_STRAIGHT + R_d), (0.0, DOME_STRAIGHT + Ri)]
    prof += [(Ri * math.cos(rad(90 * i / arcn)), DOME_STRAIGHT + Ri * math.sin(rad(90 * i / arcn))) for i in range(arcn - 1, 0, -1)]
    prof += [(Ri, DOME_STRAIGHT), (Ri, 0.0)]
    dome.revolve(prof, H, ex, eyb, ez, 40, "glass", loop=True)
    done(dome)

    # lobes: packed like a cluster of pond life; dead, they sink to the low side
    up_local = None
    if dead:
        gw = R.inverted() @ V(0, 0, 1)                    # world up, in body terms
        up_local = V(gw.dot(ex), gw.dot(eyb), gw.dot(ez))
    lrng = random.Random(41)
    radii = sorted([lrng.uniform(LOBE_MIN / 2, LOBE_MAX / 2) for _ in range(LOBE_COUNT)], reverse=True)
    placed = []
    Rin = Ri - 0.004

    def inside(p, r):
        if p.z < r + 0.004:
            return False
        if p.z <= DOME_STRAIGHT:
            return math.hypot(p.x, p.y) <= Rin - r
        return (p - V(0, 0, DOME_STRAIGHT)).length <= Rin - r

    for r in radii:
        best = None
        for _ in range(900):
            p = V(lrng.uniform(-Rin, Rin), lrng.uniform(-Rin, Rin), lrng.uniform(0, DOME_STRAIGHT + Rin))
            if not inside(p, r):
                continue
            if any((p - q).length < (r + rq) * 0.84 for q, rq in placed):
                continue
            if dead:
                sc = p.dot(up_local)
            else:
                sc = p.z + 0.55 * math.hypot(p.x, p.y)
            if best is None or sc < best[0]:
                best = (sc, p)
        placed.append((best[1], r))
    lobes = bpy.data.objects.new(N("dome_lobes"), None)
    lobes.empty_display_size = 0.05
    col.objects.link(lobes)
    lobes.parent = head_ob
    lobes.location = (0, 0, 0)
    reg[lobes.name] = H.copy()
    for i, (p, r) in enumerate(placed):
        c = H + ex * p.x + eyb * p.y + ez * p.z
        lb = Part(N("lobe_%02d" % (i + 1)), c, lobes)
        rot = Matrix.Rotation(lrng.uniform(0, 6.28), 3, "Z") @ Matrix.Rotation(lrng.uniform(0, 3.14), 3, "X")
        k = 0.9 if dead else 1.0
        lb.ball(c, (r * k, r * k * lrng.uniform(0.82, 1.0), r * k * lrng.uniform(0.74, 0.95)), "lobe", rot=rot, u=10, v=6)
        done(lb)

    # head plate: a narrow pale blade, forward and down, five amber lenses
    pd = V(0, -math.cos(rad(50)), -math.sin(rad(50)))
    pout = pd.cross(ex)                                   # the plate's outer face
    proot = H + hf * 0.215 - ez * 0.060
    hp = Part(N("head_plate"), proot, head_ob)

    def plate_pt(t):
        return proot + pd * (PLATE_LEN * t) - pout * (0.065 * t * t)

    def plate_dims(t):
        # a blade: broad at the root, narrowing, then a chisel point past the lenses
        w = 0.20 - 0.078 * t ** 0.8
        if t > 0.9:
            w = w + (0.04 - w) * ((t - 0.9) / 0.1) ** 1.4
        return w, 0.042 + (0.016 - 0.042) * t, 0.030 * (1 - t) ** 1.2

    ts = [-0.12, 0.0, 0.15, 0.3, 0.45, 0.6, 0.75, 0.88, 0.93, 0.97, 1.0]
    secs = []
    for t in ts:
        w, th_, rdg = plate_dims(max(t, 0.0))
        secs.append([(-w / 2, 0.0), (-w / 2 + 0.012, -th_ / 2), (w / 2 - 0.012, -th_ / 2), (w / 2, 0.0),
                     (w * 0.14, th_ / 2 + rdg), (-w * 0.14, th_ / 2 + rdg)])
    hp.sweep([plate_pt(t) for t in ts], secs, "bone", up=ex)
    tl = 0.88
    w, th_, rdg = plate_dims(tl)
    tan = (plate_pt(tl + 0.02) - plate_pt(tl - 0.02)).normalized()
    po = tan.cross(ex).normalized()
    for i in range(5):
        c = plate_pt(tl) + po * (th_ / 2 + rdg - 0.001) + ex * ((i - 2) * 0.0195)
        hp.revolve([(0.0, -0.004), (0.0088, -0.004), (0.0088, 0.004), (0.0066, 0.004)], c, ex, tan, po, 10,
                   "registrar", smooth=False)
        hp.ball(c + po * 0.004, (0.0066, 0.0066, 0.0045), "amber",
                rot=Matrix((ex, tan, po)).transposed(), u=10, v=6)
    done(hp)

    # ---- place the dead body on the ground ----
    if dead:
        lo = V(1e9, 1e9, 1e9)
        hi = -lo
        for ob in objs:
            o = reg[ob.name]
            for v in ob.data.vertices:
                p = R @ (v.co + o)
                for i in range(3):
                    lo[i] = min(lo[i], p[i])
                    hi[i] = max(hi[i], p[i])
        Mw = Matrix.Translation(V(-(lo.x + hi.x) / 2, -(lo.y + hi.y) / 2, -lo.z)) @ R.to_4x4()
    else:
        Mw = I4.copy()
    Mi = Mw.inverted()
    Ri3 = Mi.to_3x3()

    # ---- cloth: four hanging panels ----
    wind = V(0.05, 0.03, 0)
    panels = [
        # name, parent, attach, across, outward, pattern shift, mirror, dead: (direction on the ground, slide)
        ("cloth_shoulder_L", torso_ob, S_up[1] + V(0.115, 0.07, -0.055), V(0.30, 1, 0), V(1, -0.3, 0), 0.0, 1.0, V(0.5, 0.85, 0)),
        ("cloth_shoulder_R", torso_ob, S_up[-1] + V(-0.115, 0.07, -0.055), V(0.30, -1, 0), V(-1, -0.3, 0), 0.55, -1.0, V(0.9, 0.45, 0)),
        ("cloth_hip_front", torso_ob, A(0.19) + V(0, -0.125, 0), V(1, 0, 0), V(0, -1, 0), 1.05, 1.0, V(0.95, -0.3, 0)),
        ("cloth_hip_back", torso_ob, A(0.17) + V(0, 0.15, 0), V(-1, 0, 0), V(0, 1, 0), 0.30, -1.0, V(0.95, 0.3, 0)),
    ]
    for i, (name, par, At, across, outw, off, flip, ddir) in enumerate(panels):
        across = across.normalized()
        if not dead:
            way = [At + V(0, 0, -CLOTH_L * t) + outw.normalized() * (0.05 * t) + wind * (t * t) for t in (0, 0.2, 0.5, 0.8, 1.0, 1.3)]
            acr = [across] * len(way)
        else:
            Aw = Mw @ At
            dw = ddir.normalized()
            aw = V(0, 0, 1).cross(dw).normalized()
            hgt = max(Aw.z - 0.014, 0.02)
            a0 = (R @ across)
            g1 = V(Aw.x, Aw.y, 0.014) + dw * (0.10 + 0.45 * hgt)
            way_w = [Aw, Aw + dw * (0.05 + 0.2 * hgt) + V(0, 0, -0.45 * hgt), g1, g1 + dw * 0.5, g1 + dw * 1.0, g1 + dw * 2.0]
            for k in (3, 4, 5):
                way_w[k].z = 0.014 + 0.012 * ((k + i) % 2)
            acr_w = [a0, a0.lerp(aw, 0.5), aw, aw, aw, aw]
            way = [Mi @ p for p in way_w]
            acr = [Ri3 @ a for a in acr_w]
        cp = Part(N(name), At, par)
        cp.cloth(path_fn(way, acr), "cloth", uv_off=off, flip=flip, seed=20 + i)
        cp.tube([At - across * (CLOTH_W * 0.42), At + across * (CLOTH_W * 0.42)], 0.012, "bone", segs=6)
        done(cp)

    # ---- ring: five black arcs behind the head, one slot empty, one arc loose ----
    rt = rad(12)
    C = V(0, 0.34, 3.08)
    ur = V(0, math.sin(rt), math.cos(rt))                 # up, in the ring's plane
    rn = ex.cross(ur)                                     # the ring's front
    R_r = RING_D / 2
    mount0 = A(0.9) + back * 0.09
    mount = Part(N("ring_mount"), mount0, torso_ob)
    md = (C - ur * 0.27 - mount0).normalized()
    q = V(0, 0, 1).rotation_difference(md).to_matrix()
    mount.box(mount0 + md * 0.01, (0.07, 0.07, 0.07), "registrar", rot=q)
    mount.box(mount0 + md * -0.022, (0.076, 0.076, 0.006), "amber", rot=q)
    end = C - ur * 0.27 - rn * 0.010 if not dead else mount0.lerp(C - ur * 0.27, 0.5)
    mount.bar([mount0, end], 0.034, 0.034, "registrar", up=ex)
    mount_ob = done(mount)

    FA = Matrix((ex, ur, rn)).transposed().to_4x4()
    FA.translation = C
    if dead:
        Hw = Mw @ H
        a = rad(25)
        FD = Matrix((V(math.cos(a), math.sin(a), 0), V(-math.sin(a), math.cos(a), 0), V(0, 0, 1))).transposed().to_4x4()
        FD.translation = V(Hw.x + 0.45, Hw.y + 0.66, 0.052)
        XR = Mi @ FD @ FA.inverted()
    else:
        XR = I4.copy()

    def ring_pt(deg, r=R_r):
        return C + ex * (r * math.cos(rad(deg))) + ur * (r * math.sin(rad(deg)))

    slots = [90, 30, 330, 270, 150, 210]                  # the slot at 210 is empty
    loose = 330
    span = 25.0
    hoop_r = R_r - 0.062
    J = C - ur * 0.27
    hub = Part(N("ring_frame"), XR @ J, mount_ob, XR)
    hub.revolve([(0.0, -0.012), (0.066, -0.012), (0.066, 0.012), (0.0, 0.012)], J, ex, ur, rn, 6, "etch", smooth=False)
    for deg in (243, 297):                                # the fork
        hub.bar([J + (ring_pt(deg, hoop_r) - J).normalized() * 0.06, ring_pt(deg, hoop_r)], 0.016, 0.016, "registrar", up=rn)
    hoop = [ring_pt(a, hoop_r) for a in range(240, 240 + 304, 8)]   # thin hoop, torn open at the empty slot
    hub.bar(hoop, 0.012, 0.012, "registrar", up=rn)
    for deg in slots[:5]:
        if deg == loose:
            hub.bar([ring_pt(deg - 8, hoop_r), ring_pt(deg - 8, hoop_r + 0.022)], 0.012, 0.012, "registrar", up=rn)
            continue
        for da in (-14, 14):
            hub.bar([ring_pt(deg + da, hoop_r), ring_pt(deg + da, R_r - 0.028)], 0.014, 0.014, "registrar", up=rn)
    hub.bar([ring_pt(236, hoop_r), ring_pt(229, hoop_r - 0.02)], 0.012, 0.012, "registrar", up=rn)   # bent, torn end
    hub_ob = done(hub)

    cable_ends = {}
    for n_arc, deg in enumerate(slots[:5]):
        pivot = ring_pt(deg - span)
        xa = XR.copy()
        if deg == loose:
            # hangs from one end, swung down and out of the ring's plane
            pivot = ring_pt(deg + span)
            xa = XR @ Matrix.Translation(pivot) @ Matrix.Rotation(rad(34), 4, rn) @ Matrix.Rotation(rad(-22), 4, ex) \
                @ Matrix.Translation(-pivot)
        elif dead and n_arc in (0, 4):
            # two more arcs have broken away and lie a little apart
            off = V(0.10, 0.16, 0) if n_arc == 0 else V(0.16, -0.10, 0)
            xa = XR @ Matrix.Translation(ex * off.x + ur * off.y) @ Matrix.Translation(pivot) \
                @ Matrix.Rotation(rad(17 if n_arc == 0 else -22), 4, rn) @ Matrix.Translation(-pivot)
        ap = Part(N("ring_arc_%d" % (n_arc + 1)), xa @ pivot, hub_ob, xa)
        angs = [deg - span + 2 * span * i / 5 for i in range(6)]
        ap.bar([ring_pt(a) for a in angs], 0.034, 0.062, "registrar", up=rn)
        for a in (angs[0], angs[-1]):
            dr = (ring_pt(a) - C).normalized()
            rotm = Matrix((rn, dr.cross(rn), dr)).transposed()
            ap.box(ring_pt(a), (0.044, 0.024, 0.076), "registrar", rot=rotm)
        seam_a = [deg - span * 0.8 + 2 * span * 0.8 * i / 5 for i in range(6)]
        sec = [(0.0165, -0.0035), (0.0185, -0.0035), (0.0185, 0.0035), (0.0165, 0.0035)]
        ap.sweep([ring_pt(a) for a in seam_a], [sec] * 6, "amber", up=rn, smooth=False)
        done(ap)
        cable_ends[deg] = xa @ ring_pt(deg, R_r - 0.031)

    # ---- three thin black cables from the ring into the back of the dome ----
    for i, (deg, sxo) in enumerate(((90, 0.0), (150, -0.10), (30, 0.10))):
        sock = H + eyb * (R_d + 0.004) * math.cos(math.asin(min(1, abs(sxo) / R_d))) + ex * sxo + ez * 0.004
        a_w, b_w = Mw @ sock, Mw @ cable_ends[deg]
        sag = 0.10 + 0.03 * i
        pts = []
        for k in range(13):
            t = k / 12
            p = a_w.lerp(b_w, t) + V(0, 0, -sag * 4 * t * (1 - t))
            p.z = max(p.z, 0.012)
            pts.append(Mi @ p)
        cb = Part(N("ring_cable_%d" % (i + 1)), sock, head_ob)
        cb.tube(pts, 0.0065, "cable", segs=6)
        cb.box(sock, (0.034, 0.034, 0.03), "registrar", rot=Matrix((ex, eyb, ez)).transposed())
        done(cb)

    # ---- electrical life: thin pale arcs at the knees and shoulders ----
    if not dead:
        erng = random.Random(99)
        el = Part("electrical_arcs", V(0, 0, 0), root)
        spots = [(p, 0.085) for p in knees_posed.values()] + [(p, 0.10) for p in shoulders_posed.values()]
        for p, reach in spots:
            for _ in range(3):
                a = p + unit(erng.uniform(-1, 1), erng.uniform(-1, 1), erng.uniform(-1, 1)) * reach * 0.75
                b = p + unit(erng.uniform(-1, 1), erng.uniform(-1, 1), erng.uniform(-0.2, 1)) * reach * 1.4
                pts = []
                for k in range(8):
                    t = k / 7
                    j = 0.018 * math.sin(math.pi * t)
                    pts.append(a.lerp(b, t) + V(erng.uniform(-j, j), erng.uniform(-j, j), erng.uniform(-j, j)))
                el.tube(pts, 0.0035, "arc", segs=4)
        el_ob = el.finish(col, mats, reg)
        el_ob.hide_render = True          # shown only in the night picture
        el_ob["night_only"] = True

    # ---- put the whole figure in place ----
    if dead:
        root.matrix_world = Mw
        bpy.context.view_layer.update()
        mm = sl.measure([root])
        root.location.x -= (mm["min"][0] + mm["max"][0]) / 2
        root.location.y -= (mm["min"][1] + mm["max"][1]) / 2
        root.location.z -= mm["min"][2]
    return root


def main():
    sl.reset_scene()
    sc = bpy.context.scene
    mats = make_materials()
    c_model = bpy.data.collections.new("model")
    c_dead = bpy.data.collections.new("dead")
    sc.collection.children.link(c_model)
    sc.collection.children.link(c_dead)
    alive = build_figure(c_model, mats, dead=False)
    dead = build_figure(c_dead, mats, dead=True)
    bpy.context.view_layer.update()

    info = sl.write_size_json(os.path.join(HERE, "size.json"), "regent", [alive])
    md = sl.measure([dead])
    lo, hi = sl.bounds([alive])
    print("REGENT alive:", info, "min z %.3f" % lo.z)
    print("REGENT dead: %.2f by %.2f m, %.2f high, %d triangles, min z %.3f"
          % (md["width_m"], md["depth_m"], md["height_m"], md["triangles"], md["min"][2]))
    feet = [o for o in alive.children_recursive if "_foot" in o.name or "_strut" in o.name]
    flo, fhi = sl.bounds(feet)
    print("REGENT feet box: x %.3f..%.3f  y %.3f..%.3f" % (flo.x, fhi.x, flo.y, fhi.y))

    c_dead.hide_render = True
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(HERE, "model.blend"), compress=True)
    backup = os.path.join(HERE, "model.blend1")
    if os.path.exists(backup):
        os.remove(backup)
    print("REGENT saved")


if __name__ == "__main__":
    main()
