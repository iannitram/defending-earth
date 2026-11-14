"""Builds the Tender from nothing and saves model.blend.

Run from anywhere:
  flatpak run org.blender.Blender -b --python "<full path>/build.py"

The file holds two collections:
  model  the living Tender, standing (root object "tender")
  dead   the dead form, curled on its side (root object "dead_tender"),
         hidden from rendering until render.py asks for it
"""
import math
import os
import random
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "scale")))

import bpy
import bmesh
from mathutils import Matrix, Vector

import scale_lib as sl

NAME = "tender"

# ---------------------------------------------------------------------------
# Sizes. The overall size comes from the scale sheet. The part sizes are the
# ones the job text gives (the sheet has no part sizes for the Tender).
# ---------------------------------------------------------------------------
SHEET = sl.thing(NAME)
HEIGHT = SHEET["height_m"]

DOME_D = 0.28          # clear dome, across
LOBE_D = 0.09          # the single lobe, across
HEAD_PLATE_L = 0.20    # narrow pale plate
RING_D = 0.40          # broken ring behind the head
TORSO_L = 0.40
TORSO_W = 0.18
FEET_APART = 0.30
ARM_L = 0.50           # shoulder to claw tip
CLAW_L = 0.09
CLOTH_L = 0.30
PLUG_D = 0.05          # the wrong repair

# Shared colours (from the job rules)
C_FLESH = "#3a2430"
C_PLATE = "#cfc6b4"
C_SPINE = "#14171a"
C_HARDWARE = "#0b0b0c"
C_AMBER = "#ffa01e"
C_LOBE = "#ffd9c8"
C_CLOTH = "#b8ad98"
C_GOLD = "#a8842c"
C_ARC = "#cfe4ff"

# Proportions of the leg before it is scaled to reach the sheet height.
THIGH_L, SHIN_L, META_L = 0.20, 0.20, 0.13
UPPER_ARM_L = 0.20
FOREARM_L = ARM_L - UPPER_ARM_L - CLAW_L

R_DOME = DOME_D / 2.0
DOME_CUT = 0.080        # the dome is cut this far below its centre
DOME_Z = 0.100          # dome centre above the neck joint
RING_R = RING_D / 2.0


# ---------------------------------------------------------------------------
# Poses. Angles in degrees, each relative to the parent part.
# X swings a hanging limb back (+) or forward (-).
# ---------------------------------------------------------------------------
POSE_LIVE = {
    "root": (0, 0, 0),
    "pelvis": (0, 0, 0),
    "torso": (20, 0, 0),
    "neck": (12, 0, 0),
    "head": (-22, 9, 8),
    "head_plate": (-55, 0, 0),
    "ring_mount": (0, 0, 0),
    "thigh_l": (-42, -3, 0), "thigh_r": (-42, 3, 0),
    "shin_l": (85, -31, 0), "shin_r": (85, 31, 0),
    "meta_l": (-58, 0, 0), "meta_r": (-58, 0, 0),
    "foot_l": (15, 34, -16), "foot_r": (15, -34, 16),
    "upper_arm_l": (8, -8, 0), "upper_arm_r": (3, 8, 0),
    "forearm_l": (-152, -28, 0), "forearm_r": (-143, 30, 0),
    "hand_l": (112, 0, -10), "hand_r": (104, 0, 10),
}

POSE_DEAD = {
    "root": (0, -90, 25),
    "pelvis": (0, 0, 0),
    "torso": (46, 0, 0),
    "neck": (28, 0, 0),
    "head": (12, 0, 0),
    "head_plate": (-66, 0, 0),
    "ring_mount": (0, 0, 0),
    "thigh_l": (-128, -24, 0), "thigh_r": (-136, 0, 0),
    "shin_l": (150, 0, 0), "shin_r": (154, 0, 0),
    "meta_l": (-44, 0, 0), "meta_r": (-36, 0, 0),
    "foot_l": (30, 0, 0), "foot_r": (25, 0, 0),
    "upper_arm_l": (-48, 26, 0), "upper_arm_r": (-36, 0, 0),
    "forearm_l": (-140, 0, 0), "forearm_r": (-150, 0, 0),
    "hand_l": (45, 0, 0), "hand_r": (35, 0, 0),
}


# ---------------------------------------------------------------------------
# Materials
# ---------------------------------------------------------------------------
def _sock(node, name, kind=None, out=False):
    socks = node.outputs if out else node.inputs
    for s in socks:
        if s.name == name and (kind is None or s.type == kind):
            return s
    raise KeyError("%s has no socket %s" % (node.bl_idname, name))


class NT:
    """Small helper round one material's node tree."""

    def __init__(self, mat, bsdf):
        self.mat, self.bsdf = mat, bsdf
        self.nt = mat.node_tree
        self.coord = self.nt.nodes.new("ShaderNodeTexCoord")

    def link(self, a, b):
        self.nt.links.new(a, b)

    def obj(self):
        return self.coord.outputs["Object"]

    def scaled(self, vec, scale):
        n = self.nt.nodes.new("ShaderNodeMapping")
        n.inputs["Scale"].default_value = scale
        self.link(vec, n.inputs["Vector"])
        return n.outputs[0]

    def noise(self, scale, detail=3.0, rough=0.55, vec=None):
        n = self.nt.nodes.new("ShaderNodeTexNoise")
        n.inputs["Scale"].default_value = scale
        n.inputs["Detail"].default_value = detail
        n.inputs["Roughness"].default_value = rough
        self.link(vec or self.obj(), n.inputs["Vector"])
        return n.outputs[0]

    def voronoi_edge(self, scale, randomness=1.0, vec=None):
        n = self.nt.nodes.new("ShaderNodeTexVoronoi")
        n.feature = "DISTANCE_TO_EDGE"
        n.inputs["Scale"].default_value = scale
        n.inputs["Randomness"].default_value = randomness
        self.link(vec or self.obj(), n.inputs["Vector"])
        return n.outputs["Distance"]

    def math(self, op, a, b=None, clamp=False):
        n = self.nt.nodes.new("ShaderNodeMath")
        n.operation = op
        n.use_clamp = clamp
        for i, v in enumerate((a, b)):
            if v is None:
                continue
            if isinstance(v, (int, float)):
                n.inputs[i].default_value = v
            else:
                self.link(v, n.inputs[i])
        return n.outputs[0]

    def remap(self, v, lo, hi, to_lo=0.0, to_hi=1.0):
        n = self.nt.nodes.new("ShaderNodeMapRange")
        n.clamp = True
        self.link(v, n.inputs[0])
        for i, x in enumerate((lo, hi, to_lo, to_hi)):
            n.inputs[i + 1].default_value = x
        return n.outputs[0]

    def mix(self, fac, a, b):
        n = self.nt.nodes.new("ShaderNodeMix")
        n.data_type = "RGBA"
        ia, ib = _sock(n, "A", "RGBA"), _sock(n, "B", "RGBA")
        for s, v in ((ia, a), (ib, b)):
            if isinstance(v, (tuple, list)):
                s.default_value = v
            else:
                self.link(v, s)
        if isinstance(fac, (int, float)):
            n.inputs[0].default_value = fac
        else:
            self.link(fac, n.inputs[0])
        return _sock(n, "Result", "RGBA", out=True)

    def bump(self, height, strength, distance):
        n = self.nt.nodes.new("ShaderNodeBump")
        n.inputs["Strength"].default_value = strength
        n.inputs["Distance"].default_value = distance
        self.link(height, n.inputs["Height"])
        self.link(n.outputs[0], self.bsdf.inputs["Normal"])

    def base(self, colour):
        self.link(colour, self.bsdf.inputs["Base Color"])


def _shade(hex_colour, k):
    c = sl.rgb(hex_colour)
    return (c[0] * k, c[1] * k, c[2] * k, 1.0)


def make_materials():
    m = {}

    # Dark wet flesh
    t = NT(*sl.new_material("tender_flesh", C_FLESH, roughness=0.3))
    big = t.noise(38.0, 4.0)
    t.base(t.mix(t.remap(big, 0.3, 0.75), _shade(C_FLESH, 0.6), _shade(C_FLESH, 1.0)))
    strands = t.noise(120.0, 2.0, vec=t.scaled(t.obj(), (1.0, 1.0, 0.18)))
    t.bump(t.math("ADD", strands, t.math("MULTIPLY", big, 0.6)), 0.5, 0.004)
    m["flesh"] = t.mat

    # Pale cracked bone-like plates
    t = NT(*sl.new_material("tender_plate", C_PLATE, roughness=0.85))
    wob = t.nt.nodes.new("ShaderNodeVectorMath")
    wob.operation = "ADD"
    t.link(t.obj(), wob.inputs[0])
    wn = t.nt.nodes.new("ShaderNodeTexNoise")
    wn.inputs["Scale"].default_value = 18.0
    t.link(t.obj(), wn.inputs["Vector"])
    ws = t.nt.nodes.new("ShaderNodeVectorMath")
    ws.operation = "SCALE"
    ws.inputs["Scale"].default_value = 0.035
    t.link(_sock(wn, "Color", out=True), ws.inputs[0])
    t.link(ws.outputs[0], wob.inputs[1])
    crack = t.voronoi_edge(24.0, vec=wob.outputs[0])
    fine = t.voronoi_edge(70.0, vec=wob.outputs[0])
    line = t.math("MINIMUM", t.remap(crack, 0.0, 0.014), t.remap(fine, 0.0, 0.02, 0.72, 1.0))
    stain = t.remap(t.noise(22.0, 4.0), 0.3, 0.8, 0.62, 1.05)
    pale = t.mix(stain, _shade(C_PLATE, 0.0), sl.rgb(C_PLATE))
    t.base(t.mix(line, _shade("#2c241f", 1.0), pale))
    t.bump(line, 0.8, 0.003)
    m["plate"] = t.mat

    # The frame under the flesh: regular lattice with 6 cm cells
    t = NT(*sl.new_material("SPINE", C_SPINE, roughness=0.3))
    cell = t.voronoi_edge(1.0 / 0.06, randomness=0.0)
    strut = t.remap(cell, 0.0, 0.16, 1.0, 0.0)
    t.base(t.mix(strut, _shade(C_SPINE, 0.7), _shade(C_SPINE, 2.2)))
    t.bump(strut, 1.0, 0.004)
    m["spine"] = t.mat

    # Registrar hardware
    m["hardware"] = sl.new_material("registrar_hardware", C_HARDWARE, roughness=0.35,
                                    metallic=1.0)[0]
    m["amber"] = sl.new_material("registrar_amber", "#3a2206", roughness=0.25,
                                 emission_hex=C_AMBER, emission_strength=6.0)[0]
    m["amber_dead"] = sl.new_material("registrar_amber_dead", "#1c1206", roughness=0.25)[0]

    # The lobe: soft warm pink-white light, darker in the folds
    for key, dead in (("lobe", False), ("lobe_dead", True)):
        t = NT(*sl.new_material("tender_" + key, "#6f6a66" if dead else "#e8b7a4",
                                roughness=0.75 if dead else 0.45))
        warp = t.nt.nodes.new("ShaderNodeVectorMath")
        warp.operation = "ADD"
        t.link(t.obj(), warp.inputs[0])
        wn = t.nt.nodes.new("ShaderNodeTexNoise")
        wn.inputs["Scale"].default_value = 30.0
        t.link(t.obj(), wn.inputs["Vector"])
        sc = t.nt.nodes.new("ShaderNodeVectorMath")
        sc.operation = "SCALE"
        t.link(_sock(wn, "Color", out=True), sc.inputs[0])
        sc.inputs["Scale"].default_value = 0.02
        t.link(sc.outputs[0], warp.inputs[1])
        worm = t.noise(46.0, 0.0, vec=warp.outputs[0])
        fold = t.remap(t.math("ABSOLUTE", t.math("SUBTRACT", worm, 0.5)), 0.0, 0.035)
        t.bump(fold, 1.0, 0.004)
        if dead:
            t.base(t.mix(fold, _shade("#3b3836", 1.0), _shade("#77726d", 1.0)))
        else:
            t.base(t.mix(fold, _shade("#7c3a34", 1.0), _shade("#f0c4b2", 1.0)))
            t.bsdf.inputs["Emission Color"].default_value = sl.rgb(C_LOBE)
            t.link(t.remap(fold, 0.0, 1.0, 0.25, 2.0), t.bsdf.inputs["Emission Strength"])
        m[key] = t.mat

    # The clear dome. Shadow rays pass through so the lobe lights the ground.
    for key, dead in (("glass", False), ("glass_dead", True)):
        mat, bsdf = sl.new_material("tender_dome_" + key, "#f7ece6", roughness=0.04)
        bsdf.inputs["Transmission Weight"].default_value = 1.0
        bsdf.inputs["IOR"].default_value = 1.33
        t = NT(mat, bsdf)
        if dead:
            # Dark: a film has dried on the inside of the glass.
            film = t.remap(t.noise(9.0, 3.0), 0.35, 0.7)
            t.base(t.mix(film, _shade("#cfc8c0", 1.0), _shade("#f0ebe6", 1.0)))
            bsdf.inputs["Roughness"].default_value = 0.06
        nt = mat.node_tree
        out = next(n for n in nt.nodes if n.bl_idname == "ShaderNodeOutputMaterial")
        mixs = nt.nodes.new("ShaderNodeMixShader")
        clear = nt.nodes.new("ShaderNodeBsdfTransparent")
        clear.inputs["Color"].default_value = (0.8, 0.78, 0.76, 1.0) if dead else (0.93, 0.88, 0.86, 1.0)
        path = nt.nodes.new("ShaderNodeLightPath")
        nt.links.new(path.outputs["Is Shadow Ray"], mixs.inputs[0])
        nt.links.new(bsdf.outputs[0], mixs.inputs[1])
        nt.links.new(clear.outputs[0], mixs.inputs[2])
        nt.links.new(mixs.outputs[0], out.inputs["Surface"])
        m[key] = mat

    # Cloth: dirty off-white with a few dull gold marks, drawn from the UVs
    t = NT(*sl.new_material("tender_cloth", C_CLOTH, roughness=0.92))
    uv = t.coord.outputs["UV"]
    sep = t.nt.nodes.new("ShaderNodeSeparateXYZ")
    t.link(uv, sep.inputs[0])
    u, v = sep.outputs[0], sep.outputs[1]
    aspect = CLOTH_L / 0.07

    def dist_to(u0, v0):
        du = t.math("SUBTRACT", u, u0)
        dv = t.math("MULTIPLY", t.math("SUBTRACT", v, v0), aspect)
        return t.math("SQRT", t.math("ADD", t.math("POWER", du, 2.0), t.math("POWER", dv, 2.0)))

    def ring(d, r, w):
        return t.math("LESS_THAN", t.math("ABSOLUTE", t.math("SUBTRACT", d, r)), w)

    d1, d2 = dist_to(0.5, 0.30), dist_to(0.5, 0.62)
    marks = t.math("MAXIMUM", ring(d1, 0.30, 0.035), t.math("LESS_THAN", d1, 0.07))
    marks = t.math("MAXIMUM", marks, ring(d2, 0.17, 0.03))
    marks = t.math("MAXIMUM", marks, t.math("LESS_THAN", t.math("ABSOLUTE", t.math("SUBTRACT", v, 0.07)), 0.012))
    worn = t.remap(t.noise(60.0, 3.0), 0.42, 0.6)
    marks = t.math("MULTIPLY", marks, worn)
    dirt = t.math("MULTIPLY", t.remap(t.noise(34.0, 5.0), 0.3, 0.75, 0.25, 1.0),
                  t.remap(v, 0.2, 1.0, 1.0, 0.3))
    cloth = t.mix(dirt, _shade("#4a4034", 1.0), sl.rgb(C_CLOTH))
    t.base(t.mix(marks, cloth, sl.rgb(C_GOLD)))
    t.link(t.math("MULTIPLY", marks, 0.6), t.bsdf.inputs["Metallic"])
    t.link(t.remap(marks, 0.0, 1.0, 0.92, 0.55), t.bsdf.inputs["Roughness"])
    weave = t.math("ADD", t.noise(700.0, 1.0), t.math("MULTIPLY", t.noise(25.0, 2.0), 2.0))
    t.bump(weave, 0.5, 0.002)
    m["cloth"] = t.mat
    m["cord"] = sl.new_material("tender_cord", "#5a5144", roughness=0.9)[0]

    # Electrical life: night picture only
    m["arc"] = sl.new_material("electrical_life", "#0a0d12", roughness=0.5,
                               emission_hex=C_ARC, emission_strength=4.0)[0]
    return m


# ---------------------------------------------------------------------------
# Mesh building
# ---------------------------------------------------------------------------
def lerp(a, b, t):
    return a + (b - a) * t


def bezier(p0, p1, p2, n):
    p0, p1, p2 = Vector(p0), Vector(p1), Vector(p2)
    return [(1 - t) ** 2 * p0 + 2 * (1 - t) * t * p1 + t * t * p2
            for t in (i / n for i in range(n + 1))]


def resample(points, n):
    points = [Vector(p) for p in points]
    lens = [0.0]
    for a, b in zip(points, points[1:]):
        lens.append(lens[-1] + (b - a).length)
    out = []
    for i in range(n + 1):
        d = lens[-1] * i / n
        k = 0
        while k < len(points) - 2 and lens[k + 1] < d:
            k += 1
        seg = lens[k + 1] - lens[k]
        out.append(points[k].lerp(points[k + 1], (d - lens[k]) / seg if seg > 0 else 0.0))
    return out


class MB:
    """Collects geometry for one object."""

    def __init__(self):
        self.bm = bmesh.new()
        self.uv = self.bm.loops.layers.uv.new("UVMap")
        self.mats = []

    def mi(self, mat):
        if mat not in self.mats:
            self.mats.append(mat)
        return self.mats.index(mat)

    def _face(self, verts, mi):
        try:
            f = self.bm.faces.new(verts)
        except ValueError:
            return None
        f.material_index = mi
        return f

    def _skin(self, rings, mi, seg_mis=None):
        for k, (a, b) in enumerate(zip(rings, rings[1:])):
            m = seg_mis[k] if seg_mis else mi
            n = len(a)
            for j in range(n):
                self._face((a[j], a[(j + 1) % n], b[(j + 1) % n], b[j]), m)

    def _cap(self, ring, centre, mi):
        c = self.bm.verts.new(centre)
        n = len(ring)
        for j in range(n):
            self._face((ring[j], ring[(j + 1) % n], c), mi)

    def tube(self, pts, radii, mat, segs=10, sx=1.0, sy=1.0, hint=(1, 0, 0), caps=True):
        pts = [Vector(p) for p in pts]
        n = len(pts)
        if isinstance(radii, (int, float)):
            radii = [radii] * n
        mi = self.mi(mat)
        nrm = Vector(hint)
        t0 = (pts[1] - pts[0]).normalized()
        if abs(nrm.dot(t0)) > 0.95:
            nrm = Vector((0, 1, 0))
        rings = []
        for i in range(n):
            t = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized()
            nrm = (nrm - t * nrm.dot(t)).normalized()
            b = t.cross(nrm)
            rings.append([self.bm.verts.new(
                pts[i] + (nrm * math.cos(a) * sx + b * math.sin(a) * sy) * radii[i])
                for a in (2 * math.pi * j / segs for j in range(segs))])
        self._skin(rings, mi)
        if caps:
            self._cap(rings[0], pts[0], mi)
            self._cap(rings[-1], pts[-1], mi)

    def loft_z(self, rings, mat, segs=16, seg_mats=None, caps=True):
        """rings: (z, half width on X, half depth on Y, offset on Y)."""
        mi = self.mi(mat)
        seg_mis = [self.mi(x) for x in seg_mats] if seg_mats else None
        out = []
        for z, rx, ry, oy in rings:
            out.append([self.bm.verts.new((rx * math.cos(a), oy + ry * math.sin(a), z))
                        for a in (2 * math.pi * j / segs for j in range(segs))])
        self._skin(out, mi, seg_mis)
        if caps:
            self._cap(out[0], (0, rings[0][3], rings[0][0]), seg_mis[0] if seg_mis else mi)
            self._cap(out[-1], (0, rings[-1][3], rings[-1][0]), seg_mis[-1] if seg_mis else mi)

    def ellipsoid(self, c, rx, ry, rz, mat, segs=14, rings=8, lat0=-90.0, lat1=90.0):
        c = Vector(c)
        mi = self.mi(mat)
        out = []
        poles = {}
        for k in range(rings + 1):
            lat = math.radians(lerp(lat0, lat1, k / rings))
            if abs(abs(math.degrees(lat)) - 90.0) < 1e-4:
                poles[k] = c + Vector((0, 0, rz * math.sin(lat)))
                continue
            out.append((k, [self.bm.verts.new(c + Vector((rx * math.cos(lat) * math.cos(a),
                                                          ry * math.cos(lat) * math.sin(a),
                                                          rz * math.sin(lat))))
                            for a in (2 * math.pi * j / segs for j in range(segs))]))
        self._skin([r for _, r in out], mi)
        if 0 in poles:
            self._cap(out[0][1], poles[0], mi)
        if rings in poles:
            self._cap(out[-1][1], poles[rings], mi)

    def plate(self, rx, ry, rz, su, sv, mat, seed, orient=None, offset=(0, 0, 0), K=3, M=13,
              point=0.0):
        """A chipped patch of an ellipsoid shell. Its middle faces -Y before
        `orient` (a 3x3 matrix) turns it. su and sv are half spans in radians."""
        orient = orient or Matrix.Identity(3)
        offset = Vector(offset)
        rnd = random.Random(seed)
        jit = [rnd.uniform(-0.22, 0.06) for _ in range(M)]
        for _ in range(2):
            jit[rnd.randrange(M)] = -0.38
        mi = self.mi(mat)

        def pt(u, v):
            return offset + orient @ Vector((rx * math.sin(u) * math.cos(v),
                                             -ry * math.cos(u) * math.cos(v),
                                             rz * math.sin(v)))
        centre = self.bm.verts.new(pt(0, 0))
        rings = []
        for k in range(1, K + 1):
            rho = k / K
            ring = []
            for j in range(M):
                a = 2 * math.pi * j / M
                e = 1.0 + (jit[j] + point * max(0.0, -math.sin(a)) ** 3) * rho * rho
                ring.append(self.bm.verts.new(pt(rho * e * math.cos(a) * su,
                                                 rho * e * math.sin(a) * sv)))
            rings.append(ring)
        for j in range(M):
            self._face((centre, rings[0][j], rings[0][(j + 1) % M]), mi)
        self._skin(rings, mi)

    def arc_box(self, r_out, r_in, thick, a0, a1, mat, step_deg=8.0):
        """A flat-faced arc in the XZ plane, centred on the origin."""
        mi = self.mi(mat)
        n = max(2, int(round((a1 - a0) / step_deg)))
        rings = []
        for i in range(n + 1):
            a = math.radians(lerp(a0, a1, i / n))
            ca, sa = math.cos(a), math.sin(a)
            rings.append([self.bm.verts.new((r * ca, y, r * sa)) for r, y in
                          ((r_out, -thick / 2), (r_out, thick / 2),
                           (r_in, thick / 2), (r_in, -thick / 2))])
        self._skin(rings, mi)
        self._face(rings[0], mi)
        self._face(rings[-1], mi)

    def strip(self, path, wdir, w0, w1, mat, seed, n=12, cols=3, wave=0.006):
        """A hanging cloth strip along `path`, `wdir` across it."""
        rnd = random.Random(seed)
        mi = self.mi(mat)
        pts = resample(path, n)
        wdir = Vector(wdir).normalized()
        ph = rnd.uniform(0, 6.28)
        rows, uvs = [], {}
        for i, p in enumerate(pts):
            t = i / n
            tan = (pts[min(i + 1, n)] - pts[max(i - 1, 0)]).normalized()
            nrm = tan.cross(wdir).normalized()
            width = lerp(w0, w1, t)
            row = []
            for c in range(cols + 1):
                s = c / cols - 0.5
                q = (p + wdir * s * width
                     + nrm * (wave * t * math.sin(t * 8.0 + ph) + 0.005 * math.sin(s * 5.0 + ph * 2)))
                if i == n:
                    q += tan * (rnd.uniform(-0.03, 0.0) - 0.05 * abs(s))
                elif i > n * 0.25 and c in (0, cols):
                    q -= wdir * s * width * rnd.uniform(0.0, 0.45)
                vert = self.bm.verts.new(q)
                uvs[vert] = (c / cols, t)
                row.append(vert)
            rows.append(row)
        for a, b in zip(rows, rows[1:]):
            for c in range(cols):
                f = self._face((a[c], a[c + 1], b[c + 1], b[c]), mi)
                if f:
                    for lp in f.loops:
                        lp[self.uv].uv = uvs[lp.vert]


def limb_pts(length, n=5):
    return [(0, 0, -length * i / (n - 1)) for i in range(n)]


def hoop(rx, ry, oy, z, dip=0.0, n=16):
    """A closed loop round the body at height z, dipping at the front."""
    return [(rx * math.cos(a), oy + ry * math.sin(a), z + dip * math.sin(a))
            for a in (2 * math.pi * j / n for j in range(n + 1))]


class Figure:
    """One Tender, built part by part into a collection."""

    def __init__(self, coll, mats, pose, prefix="", dead=False, leg_scale=1.0):
        self.coll, self.m, self.pose = coll, mats, pose
        self.prefix, self.dead, self.leg_scale = prefix, dead, leg_scale
        self.parts = {}

    def finish(self, mb, name, parent, loc, pose_key=None, subsurf=1, smooth=True,
               solidify=0.0):
        full = self.prefix + name
        if mb is None:
            ob = bpy.data.objects.new(full, None)
            ob.empty_display_size = 0.1
        else:
            bmesh.ops.recalc_face_normals(mb.bm, faces=mb.bm.faces[:])
            me = bpy.data.meshes.new(full + "_mesh")
            mb.bm.to_mesh(me)
            mb.bm.free()
            for mat in mb.mats:
                me.materials.append(mat)
            me.polygons.foreach_set("use_smooth", [smooth] * len(me.polygons))
            ob = bpy.data.objects.new(full, me)
            if solidify:
                md = ob.modifiers.new("thickness", "SOLIDIFY")
                md.thickness = solidify
                md.offset = 0.0
            if subsurf:
                md = ob.modifiers.new("smooth", "SUBSURF")
                md.levels = md.render_levels = subsurf
        self.coll.objects.link(ob)
        ob.parent = parent
        ob.location = loc
        rot = self.pose.get(pose_key or name, (0, 0, 0))
        ob.rotation_euler = [math.radians(a) for a in rot]
        self.parts[name] = ob
        return ob

    def sinew(self, mb, length, r0, r1, seed, strands=3, n=7, knuckle=1.0):
        """A thin limb made of a core and a few twisted strands of flesh,
        with a bony knuckle at the top joint."""
        m = self.m
        rnd = random.Random(seed)

        def rad(t):
            return lerp(r0, r1, t) * (1.0 + 0.30 * math.sin(math.pi * min(1.0, t * 1.6)) ** 2)
        ts = [i / (n - 1) for i in range(n)]
        mb.tube([(0, 0, -length * t) for t in ts], [rad(t) * 0.62 for t in ts], m["spine"], segs=8)
        ph0 = rnd.uniform(0, 6.28)
        for k in range(strands):
            ph = ph0 + 2 * math.pi * k / strands
            tw = rnd.uniform(0.5, 1.1)
            pts, radii = [], []
            for t in ts:
                off = rad(t) * 0.52
                a = ph + tw * t * math.pi
                pts.append((off * math.cos(a), off * math.sin(a), -length * t))
                radii.append(rad(t) * rnd.uniform(0.50, 0.62) * (0.55 if t in (0.0, 1.0) else 1.0))
            mb.tube(pts, radii, m["flesh"], segs=7)
        # knuckle: two flesh condyles either side of a dark hinge
        k = r0 * 0.98 * knuckle
        mb.ellipsoid((0, 0, 0), k * 0.8, k * 0.8, k * 0.8, m["spine"], segs=8, rings=6)
        for sx in (-1, 1):
            mb.ellipsoid((sx * k * 0.62, 0, -k * 0.2), k * 0.5, k * 0.85, k * 1.5, m["flesh"],
                         segs=8, rings=6)

    # -- the body -----------------------------------------------------------
    def build(self):
        m = self.m
        root = self.finish(None, NAME, None, (0, 0, 0), "root")
        hip_h = (THIGH_L + SHIN_L + META_L) * self.leg_scale
        hw = TORSO_W / 2.0
        L = TORSO_L

        # Pelvis, with the cord the cloth hangs from
        mb = MB()
        mb.ellipsoid((0, 0, 0), 0.070, 0.048, 0.046, m["flesh"])
        for sx in (-1, 1):       # hip bones
            mb.ellipsoid((sx * 0.052, 0.004, 0.012), 0.03, 0.04, 0.042, m["flesh"], segs=10, rings=6)
        mb.tube(hoop(0.074, 0.056, 0.0, 0.034, n=20), 0.0055, m["cord"], segs=6, hint=(0, 0, 1))
        pelvis = self.finish(mb, "pelvis", root, (0, 0, hip_h))

        # Torso: the dark frame, hunched forward, with ribs and bands of
        # flesh over it so the frame shows between them.
        def curve(z):
            return -0.055 * (z / L) ** 2
        core = [(0.00, 0.048, 0.038), (0.06, 0.038, 0.031), (0.12, 0.033, 0.028),
                (0.19, 0.046, 0.037), (0.26, 0.064, 0.048), (L * 0.85, hw * 0.82, 0.048),
                (L * 0.95, 0.050, 0.036), (L, 0.026, 0.024)]
        self.torso_prof = [(z, rx, ry, curve(z)) for z, rx, ry in core]
        mb = MB()
        mb.loft_z(self.torso_prof, m["spine"], segs=14)
        for i, z in enumerate((0.185, 0.220, 0.255, 0.290, 0.325)):
            rx, ry, oy = self._torso_at(z)
            mb.tube(hoop(rx + 0.003, ry + 0.003, oy, z, dip=0.020 + 0.004 * i, n=12),
                    0.0085 - 0.0006 * (4 - i), m["flesh"], segs=6, sx=1.55, sy=0.75, hint=(0, 0, 1))
        # shoulder yoke and breastbone
        rx, ry, oy = self._torso_at(L * 0.85)
        mb.ellipsoid((0, oy, L * 0.86), hw, 0.044, 0.034, m["flesh"], segs=14, rings=8)
        mb.tube([(0, self._front(z) - 0.002, z) for z in (0.37, 0.31, 0.25, 0.18)],
                [0.016, 0.014, 0.011, 0.007], m["flesh"], segs=8)
        # four bands of flesh across the waist
        for sx in (-1, 1):
            for sy in (-1, 1):
                top = self._torso_at(0.20)
                mb.tube([(sx * top[0] * 0.62, top[2] + sy * top[1] * 0.80, 0.21),
                         (sx * 0.024, curve(0.12) + sy * 0.026, 0.12),
                         (sx * 0.030, sy * 0.030, 0.045),
                         (sx * 0.040, sy * 0.030, -0.01)],
                        [0.012, 0.0105, 0.012, 0.014], m["flesh"], segs=6)
        # a pad of flesh on the left side, where the wrong repair sits
        zp = 0.085
        pad_c = Vector((0.020, curve(zp) - 0.004, zp))
        mb.ellipsoid(pad_c, 0.030, 0.042, 0.058, m["flesh"], segs=12, rings=8)
        torso = self.finish(mb, "torso", pelvis, (0, 0, 0))

        # The frame along the back: a knobbed ridge
        mb = MB()
        pts, radii = [], []
        steps = 19
        for i in range(steps + 1):
            z = lerp(0.02, L - 0.015, i / steps)
            _, ry, oy = self._torso_at(z)
            pts.append((0, oy + ry + 0.006, z))
            radii.append(0.014 if i % 2 == 0 else 0.007)
        mb.tube(pts, radii, m["spine"], segs=8)
        self.finish(mb, "back_frame", torso, (0, 0, 0))

        # The wrong repair: a round black plug in the left side, with a seam
        ang = math.radians(-22.0)      # a little toward the front
        nrm = Vector((math.cos(ang), math.sin(ang), 0))
        surf = pad_c + Vector((0.030 * nrm.x, 0.042 * nrm.y * 0.55, 0))
        mb = MB()
        mb.tube([-nrm * 0.03, nrm * 0.011], PLUG_D / 2.0, m["hardware"], segs=24, hint=(0, 0, 1))
        self.finish(mb, "wrong_repair_plug", torso, surf, subsurf=0, smooth=False)
        mb = MB()
        side = Vector((0, 0, 1)).cross(nrm).normalized()
        seam = [(side * math.cos(a) + Vector((0, 0, 1)) * math.sin(a)) * (PLUG_D / 2.0 + 0.007)
                + nrm * 0.001 for a in (2 * math.pi * j / 20 for j in range(21))]
        mb.tube(seam, [0.0075 + 0.002 * math.sin(j * 2.3) for j in range(21)], m["flesh"],
                segs=6, hint=nrm)
        self.finish(mb, "wrong_repair_seam", torso, surf, subsurf=0)

        # Neck: bare frame with two cords of flesh
        mb = MB()
        mb.tube([(0, 0, -0.02), (0, 0, 0.03), (0, 0, 0.075)], [0.019, 0.015, 0.017],
                m["spine"], segs=8)
        for sx in (-1, 1):
            mb.tube([(sx * 0.02, 0.004, -0.02), (sx * 0.014, 0.006, 0.03), (sx * 0.022, 0.004, 0.075)],
                    [0.008, 0.006, 0.008], m["flesh"], segs=6)
        neck = self.finish(mb, "neck", torso, (0, curve(L), L))

        self._head(neck)
        for s, tag in ((1, "l"), (-1, "r")):
            self._leg(pelvis, s, tag)
            self._arm(torso, s, tag)
        if not self.dead:
            self._cloth_live(pelvis)
        return root

    def _torso_at(self, z):
        p = self.torso_prof
        for a, b in zip(p, p[1:]):
            if a[0] <= z <= b[0]:
                t = (z - a[0]) / (b[0] - a[0])
                return lerp(a[1], b[1], t), lerp(a[2], b[2], t), lerp(a[3], b[3], t)
        return p[-1][1:]

    def _front(self, z):
        _, ry, oy = self._torso_at(z)
        return oy - ry

    def _head(self, neck):
        m = self.m
        amber = m["amber_dead"] if self.dead else m["amber"]
        cut_z = DOME_Z - DOME_CUT
        rim = math.sqrt(R_DOME ** 2 - DOME_CUT ** 2)

        # The dark cup the dome sits in. Its floor rises in a mound to the
        # stem of the lobe, with roots running down it.
        lobe_z = DOME_Z - 0.004
        stem_z = lobe_z - LOBE_D * 0.34
        mb = MB()
        mound = [(cut_z + 0.006, rim * 0.74), (cut_z + 0.018, rim * 0.50),
                 (cut_z + 0.040, 0.034), (stem_z, 0.017), (stem_z + 0.02, 0.012)]
        mb.loft_z([(-0.018, 0.020, 0.020, 0.0), (0.0, 0.042, 0.046, 0.0),
                   (cut_z - 0.014, rim * 0.80, rim * 0.84, 0.0),
                   (cut_z + 0.002, rim + 0.004, rim + 0.004, 0.0),
                   (cut_z + 0.010, rim - 0.006, rim - 0.006, 0.0)]
                  + [(z, r, r, 0.0) for z, r in mound], m["flesh"], segs=20)
        rnd = random.Random(7)
        for j in range(7):
            a = 2 * math.pi * j / 7 + rnd.uniform(-0.25, 0.25)
            reach = rnd.uniform(0.75, 1.0)
            pts = [(r * math.cos(a + 0.25 * i), r * math.sin(a + 0.25 * i), z + 0.003)
                   for i, (z, r) in enumerate(reversed(mound[:4]))]
            pts[-1] = (pts[-1][0] * reach, pts[-1][1] * reach, pts[-1][2])
            mb.tube(pts, [0.008, 0.0065, 0.005, 0.0035], m["flesh"], segs=5)
        head = self.finish(mb, "head", neck, (0, 0, 0.07))

        # Dome
        mb = MB()
        lat0 = -math.degrees(math.asin(DOME_CUT / R_DOME))
        mb.ellipsoid((0, 0, 0), R_DOME, R_DOME, R_DOME, m["glass_dead" if self.dead else "glass"],
                     segs=36, rings=12, lat0=lat0, lat1=90.0)
        dome = self.finish(mb, "dome", head, (0, 0, DOME_Z), subsurf=0, solidify=0.004)

        # The one lobe
        mb = MB()
        r = LOBE_D / 2.0
        mb.ellipsoid((0, 0, 0), r, r * 1.14, r * 0.84, m["lobe_dead" if self.dead else "lobe"],
                     segs=24, rings=14)
        lobe = self.finish(mb, "lobe", dome, (0, -0.004, -0.004), subsurf=1)
        tex = bpy.data.textures.get("lobe_lumps") or bpy.data.textures.new("lobe_lumps", "VORONOI")
        tex.noise_scale = 0.022
        md = lobe.modifiers.new("lumps", "DISPLACE")
        md.texture = tex
        md.texture_coords = "LOCAL"
        md.strength = 0.011
        md.mid_level = 0.25

        # Head plate: narrow, pale, pointing forward and down
        Lp = HEAD_PLATE_L
        mb = MB()
        mb.loft_z([(0.02, 0.024, 0.006, 0.004), (0.0, 0.040, 0.011, 0.0),
                   (-Lp * 0.25, 0.036, 0.013, -0.006), (-Lp * 0.55, 0.026, 0.011, -0.011),
                   (-Lp * 0.85, 0.013, 0.007, -0.007), (-Lp, 0.003, 0.002, 0.0)],
                  m["plate"], segs=6)
        plate = self.finish(mb, "head_plate", head, (0, -rim + 0.022, cut_z - 0.006), subsurf=0,
                            smooth=False)
        # One small amber lens in a black housing, low on the plate
        mb = MB()
        mb.tube([(0, 0.006, 0), (0, -0.004, 0)], 0.0095, m["hardware"], segs=16, hint=(1, 0, 0))
        mb.tube([(0, -0.003, 0), (0, -0.0055, 0)], 0.0072, amber, segs=16, hint=(1, 0, 0))
        self.finish(mb, "lens", plate, (0, -0.0165, -Lp * 0.70), subsurf=0, smooth=False)

        # The broken ring behind the head
        mount_y = rim - 0.012
        cy = R_DOME + 0.022 - mount_y        # ring plane clears the back of the dome
        cz = 0.02
        mb = MB()
        mb.tube([(0, 0, 0), (0, cy, cz)], 0.008, m["hardware"], segs=4, hint=(1, 0, 1))
        mb.tube([(0, cy, cz), (0, cy, cz - RING_R + 0.012)], 0.0055, m["hardware"],
                segs=4, hint=(1, 1, 0))
        mb.tube([(0, cy - 0.008, cz), (0, cy + 0.008, cz)], 0.013, m["hardware"],
                segs=6, hint=(1, 0, 0))
        mount = self.finish(mb, "ring_mount", head, (0, mount_y, cut_z - 0.02),
                            subsurf=0, smooth=False)
        span = 94.0
        for i, mid in enumerate((270.0, 30.0, 150.0)):
            mb = MB()
            mb.arc_box(RING_R, RING_R - 0.022, 0.012, mid - span / 2, mid + span / 2,
                       m["hardware"])
            for end in (mid - span / 2 + 3, mid + span / 2 - 3):      # end blocks
                mb.arc_box(RING_R + 0.004, RING_R - 0.026, 0.018, end - 3, end + 3,
                           m["hardware"], step_deg=3.0)
            # amber seam light: a thin slot along the middle of the arc
            mb.arc_box(RING_R - 0.009, RING_R - 0.0135, 0.014, mid - 20, mid + 20, amber,
                       step_deg=5.0)
            self.finish(mb, "ring_arc_%d" % (i + 1), mount, (0, cy, cz), subsurf=0, smooth=False)
        # One thin cable from the ring into the dome
        a = math.radians(30.0 + span / 2)
        start = Vector(((RING_R - 0.011) * math.cos(a), cy, cz + (RING_R - 0.011) * math.sin(a)))
        end = Vector((0.03, -0.014, 0.024))
        mid_pt = (start + end) / 2 + Vector((0.05, 0.06, -0.05))
        mb = MB()
        mb.tube(bezier(start, mid_pt, end, 12), 0.003, m["hardware"], segs=6)
        self.finish(mb, "ring_cable", mount, (0, 0, 0), subsurf=0)

    def _leg(self, pelvis, s, tag):
        m = self.m
        k = self.leg_scale
        lt, ls, lm = THIGH_L * k, SHIN_L * k, META_L * k
        hip_x = 0.060

        mb = MB()
        self.sinew(mb, lt, 0.032, 0.022, seed=100 + s, knuckle=1.0)
        thigh = self.finish(mb, "thigh_" + tag, pelvis, (s * hip_x, 0, -0.014), subsurf=0)
        # hip plate on the outside of the thigh
        mb = MB()
        mb.plate(0.046, 0.046, 0.095, 1.15, 1.0, m["plate"], seed=11 + s, point=0.45,
                 orient=Matrix.Rotation(math.radians(s * 60.0), 3, "Z"))
        self.finish(mb, "hip_plate_" + tag, thigh, (0, 0, -0.035), subsurf=1, solidify=0.006)

        mb = MB()
        self.sinew(mb, ls, 0.022, 0.014, seed=110 + s, knuckle=1.25)
        shin = self.finish(mb, "shin_" + tag, thigh, (0, 0, -lt), subsurf=0)
        mb = MB()
        mb.plate(0.030, 0.031, ls * 0.54, 1.15, 1.3, m["plate"], seed=21 + s, point=0.3)
        self.finish(mb, "shin_plate_" + tag, shin, (0, 0, -ls * 0.52), subsurf=1, solidify=0.005)
        if not self.dead:
            # Electrical life: faint arcs across the knee. Night picture only.
            rnd = random.Random(40 + s)
            mb = MB()
            for _ in range(3):
                a0 = rnd.uniform(0, 6.28)
                pts = []
                for j in range(7):
                    t = j / 6.0
                    a = a0 + t * rnd.uniform(1.2, 2.2)
                    r = 0.030 + rnd.uniform(-0.004, 0.008)
                    pts.append((r * math.cos(a), r * math.sin(a), lerp(0.04, -0.045, t)
                                + rnd.uniform(-0.007, 0.007)))
                mb.tube(pts, 0.0014, m["arc"], segs=4)
            arcs = self.finish(mb, "knee_arcs_" + tag, shin, (0, 0, 0), subsurf=0)
            arcs.hide_render = True
            arcs.visible_shadow = False

        mb = MB()
        self.sinew(mb, lm, 0.014, 0.011, seed=120 + s, strands=2, n=5, knuckle=1.4)
        meta = self.finish(mb, "meta_" + tag, shin, (0, 0, -ls), subsurf=0)

        # small three-toed foot
        mb = MB()
        mb.ellipsoid((0, 0, 0), 0.014, 0.016, 0.013, m["flesh"], segs=8, rings=6)
        for a in (-32.0, 0.0, 32.0):
            ra = math.radians(a)
            d = Vector((math.sin(ra), -math.cos(ra), 0))
            ln = 0.085 if a == 0 else 0.07
            mb.tube([Vector((0, 0, -0.002)), d * ln * 0.4 + Vector((0, 0, -0.002)),
                     d * ln * 0.75 + Vector((0, 0, -0.008))],
                    [0.0095, 0.0085, 0.007], m["flesh"], segs=6)
            mb.tube([d * ln * 0.72 + Vector((0, 0, -0.008)), d * ln * 0.9 + Vector((0, 0, -0.010)),
                     d * ln + Vector((0, 0, -0.0155))], [0.0065, 0.005, 0.0015], m["plate"], segs=6)
        mb.tube([(0, 0, -0.004), (0, 0.026, -0.011)], [0.010, 0.005], m["flesh"], segs=6)
        self.finish(mb, "foot_" + tag, meta, (0, 0, -lm), subsurf=0)

    def _arm(self, torso, s, tag):
        m = self.m
        z = TORSO_L * 0.85
        sh = (s * (TORSO_W / 2.0 + 0.002), self._torso_at(z)[2], z)
        mb = MB()
        self.sinew(mb, UPPER_ARM_L, 0.021, 0.015, seed=130 + s, knuckle=1.2)
        upper = self.finish(mb, "upper_arm_" + tag, torso, sh, subsurf=0)
        # shoulder plate
        mb = MB()
        orient = (Matrix.Rotation(math.radians(-s * 42.0), 3, "Y")
                  @ Matrix.Rotation(math.radians(s * 78.0), 3, "Z"))
        mb.plate(0.052, 0.040, 0.058, 1.25, 1.2, m["plate"], seed=31 + s, orient=orient, point=0.4)
        self.finish(mb, "shoulder_plate_" + tag, torso, (sh[0] - s * 0.004, sh[1], sh[2] + 0.002),
                    subsurf=1, solidify=0.006)

        mb = MB()
        self.sinew(mb, FOREARM_L, 0.016, 0.011, seed=140 + s, knuckle=1.3)
        fore = self.finish(mb, "forearm_" + tag, upper, (0, 0, -UPPER_ARM_L), subsurf=0)

        # hand: three claws
        mb = MB()
        mb.ellipsoid((0, 0, -0.006), 0.015, 0.010, 0.013, m["flesh"], segs=8, rings=6)
        for i, a in enumerate((-22.0, 0.0, 22.0)):
            side = math.sin(math.radians(a))
            ln = CLAW_L * (1.0 if i == 1 else 0.88)
            mb.tube([(side * 0.018, 0, -0.006), (side * 0.040, -0.003, -ln * 0.40)],
                    [0.0065, 0.0055], m["flesh"], segs=6)
            mb.tube([(side * 0.040, -0.003, -ln * 0.40), (side * 0.054, -0.012, -ln * 0.75),
                     (side * 0.056, -0.030, -ln)], [0.0052, 0.004, 0.001], m["plate"], segs=6)
        self.finish(mb, "hand_" + tag, fore, (0, 0, -FOREARM_L))

    # -- cloth ---------------------------------------------------------------
    CLOTH = (  # name, where it hangs from (seen from the pelvis), kind
        ("cloth_front", (-0.006, -0.056, 0.034), "front"),
        ("cloth_side_l", (0.076, 0.004, 0.034), "side"),
        ("cloth_back_r", (-0.030, 0.052, 0.034), "back"),
    )

    def _cloth_live(self, pelvis):
        m = self.m
        for i, (name, at, kind) in enumerate(self.CLOTH):
            mb = MB()
            if kind == "front":
                path = [(0, 0, 0), (0, -0.016, -0.05), (0.004, -0.022, -0.16), (0.008, -0.012, -0.31)]
                wdir = (1, 0, 0)
            elif kind == "side":
                path = [(0, 0, 0), (0.026, 0, -0.03), (0.036, 0.004, -0.12), (0.028, 0.012, -0.31)]
                wdir = (0, 1, 0)
            else:
                path = [(0, 0, 0), (0, 0.014, -0.06), (-0.004, 0.024, -0.17), (-0.008, 0.016, -0.31)]
                wdir = (1, 0, 0.0)
            path = self._cut(path, CLOTH_L)
            mb.strip(path, wdir, 0.068, 0.036, m["cloth"], seed=50 + i)
            self.finish(mb, name, pelvis, at, subsurf=1, solidify=0.003)

    @staticmethod
    def _cut(path, length):
        pts = [Vector(p) for p in path]
        out, run = [pts[0]], 0.0
        for a, b in zip(pts, pts[1:]):
            seg = (b - a).length
            if run + seg >= length:
                out.append(a.lerp(b, (length - run) / seg))
                return out
            out.append(b)
            run += seg
        return out

    def cloth_dead(self):
        """Cloth of the dead form: it has fallen and lies on the ground."""
        m = self.m
        pelvis = self.parts["pelvis"]
        bpy.context.view_layer.update()
        mw = pelvis.matrix_world
        feet_dir = (mw.to_3x3() @ Vector((0, 0, -1)))
        back_dir = (mw.to_3x3() @ Vector((0, 1, 0)))
        mix = {"front": (1.0, -0.15), "side": (0.8, 0.5), "back": (0.45, 1.0)}
        for i, (name, at, kind) in enumerate(self.CLOTH):
            p0 = mw @ Vector(at)
            d = feet_dir * mix[kind][0] + back_dir * mix[kind][1]
            d.z = 0
            d.normalize()
            h = max(p0.z - 0.008, 0.0)
            reach = h * 0.55 + 0.03
            g = Vector((p0.x + d.x * reach, p0.y + d.y * reach, 0.008))
            ctrl = Vector((p0.x + d.x * reach * 0.9, p0.y + d.y * reach * 0.9, p0.z * 0.75))
            path = bezier(p0, ctrl, g, 8) + [g + d * 0.15, g + d * 0.40]
            path = [p - p0 for p in self._cut(path, CLOTH_L)]
            mb = MB()
            mb.strip(path, (-d.y, d.x, 0), 0.068, 0.036, m["cloth"], seed=50 + i, wave=0.003)
            ob = self.finish(mb, name, None, p0, subsurf=1, solidify=0.003)
            ob.parent = pelvis
            ob.matrix_parent_inverse = mw.inverted()


def new_collection(name):
    c = bpy.data.collections.new(name)
    bpy.context.scene.collection.children.link(c)
    return c


def drop_to_ground(root, objects=None):
    lo, _ = sl.bounds(objects or [root])
    root.location.z -= lo.z
    bpy.context.view_layer.update()


def lowest(objects):
    """Lowest point of these objects alone (children are not added)."""
    bpy.context.view_layer.update()
    dg = bpy.context.evaluated_depsgraph_get()
    z = 1e9
    for o in objects:
        ev = o.evaluated_get(dg)
        me = ev.to_mesh()
        mw = ev.matrix_world
        for v in me.vertices:
            z = min(z, (mw @ v.co).z)
        ev.to_mesh_clear()
    return z


def clear_collection(coll):
    for ob in list(coll.objects):
        data = ob.data
        bpy.data.objects.remove(ob)
        if data is not None and data.users == 0:
            bpy.data.meshes.remove(data)


def main():
    sl.reset_scene()
    mats = make_materials()
    model = new_collection("model")
    dead = new_collection("dead")

    # The living Tender. Built twice: the first pass measures how tall it
    # comes out, the second sets the leg length so it stands at the sheet height.
    leg = 1.0
    for _ in range(3):
        clear_collection(model)
        fig = Figure(model, mats, POSE_LIVE, leg_scale=leg)
        root = fig.build()
        drop_to_ground(root)
        h = sl.measure([root])["height_m"]
        if abs(h - HEIGHT) < 0.002:
            break
        hip = (THIGH_L + SHIN_L + META_L) * leg
        hip_world = fig.parts["pelvis"].matrix_world.translation.z
        leg *= (hip_world + (HEIGHT - h)) / hip_world
    # centre the feet on the origin
    feet = [fig.parts["foot_l"].matrix_world.translation, fig.parts["foot_r"].matrix_world.translation]
    root.location.x -= (feet[0].x + feet[1].x) / 2
    root.location.y -= (feet[0].y + feet[1].y) / 2 - 0.02
    bpy.context.view_layer.update()

    # The dead form
    dfig = Figure(dead, mats, POSE_DEAD, prefix="dead_", dead=True, leg_scale=leg)
    droot = dfig.build()
    mount = dfig.parts["ring_mount"]
    ring = [mount] + list(mount.children)
    body = [o for o in dead.objects if o.type == "MESH" and o not in ring]
    droot.location.z -= lowest(body)
    bpy.context.view_layer.update()
    # The ring is bent over at its mount until it rests on the ground.
    best = 0
    for g in range(0, 92, 2):
        hit = None
        for sign in (-1, 1):
            mount.rotation_euler = (0, 0, math.radians(sign * g))
            bpy.context.view_layer.update()
            if lowest(ring) >= -0.002:
                hit = sign * g
                break
        if hit is not None:
            best = hit
            break
    mount.rotation_euler = (0, 0, math.radians(best))
    lo, hi = sl.bounds(body)
    droot.location.x -= (lo.x + hi.x) / 2
    droot.location.y -= (lo.y + hi.y) / 2
    bpy.context.view_layer.update()
    dfig.cloth_dead()
    dead.hide_render = True

    bpy.context.view_layer.update()
    data = sl.write_size_json(os.path.join(HERE, "size.json"), NAME, [root])
    print("TENDER size:", data, "leg scale %.3f" % leg)
    for key in ("pelvis", "head", "dome", "foot_l", "foot_r", "hand_l", "hand_r", "head_plate"):
        p = fig.parts[key].matrix_world.translation
        print("  %-10s %.3f %.3f %.3f" % (key, p.x, p.y, p.z))
    print("  dead:", sl.measure([droot]), "ring bend", math.degrees(mount.rotation_euler.z))
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(HERE, "model.blend"), compress=True)
    backup = os.path.join(HERE, "model.blend1")
    if os.path.exists(backup):
        os.remove(backup)


main()
