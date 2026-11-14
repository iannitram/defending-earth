"""Builds the Cinder in three sizes from nothing and saves model.blend.

  flatpak run org.blender.Blender -b --python "<full path>/build.py"

The file holds two collections:
  model  the three living Cinders in a row (root object "cinder", with
         "cinder_small", "cinder_middle", "cinder_large" under it),
  dead   the cold, cracked middle Cinder (root object "dead_cinder").

A Cinder is a glowing core with loose plates of dark crust on it. The core
and the plates are both made from one shape rule (class Shape), so they
always fit each other.

Sizes come from the scale sheet (0.3 m, 0.6 m, 1.0 m). The small numbers
in SPECS are shares of that size, or detail sizes named in the job text.
"""
import json
import math
import os
import random
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "scale")))

import bpy
import bmesh
from mathutils import Matrix, Vector, noise

import scale_lib as sl

NAME = "cinder"
SIZES = sl.thing(NAME)["sizes_m"]          # 0.3, 0.6, 1.0
FOOT = sl.thing(NAME)["footprint"]
ROW_STEP_M = 1.2                           # job text: the row is 1.2 m apart
ROUNDING = 0.2                             # job text: rounding a fifth of the size
TRI_LIMIT = 6000

# Colours from the job text and the shared colour table.
CRUST_HEX = "#2a211c"
FIRE_HEX = "#ff5a0c"
WHITE_HOT_HEX = "#fff2d0"
CRYSTAL_HEX = "#ff8a5a"
FIRE_STRENGTH = 10.0
WHITE_HOT_STRENGTH = 25.0
SPARK_STRENGTH = 30.0
# Not given anywhere: the cold colours of the dead form. Chosen here, and
# listed in sheet.md for the lead.
DEAD_CRUST_HEX = "#3b3a3a"
DEAD_CORE_HEX = "#232324"
DEAD_CRYSTAL_HEX = "#4a4344"

# prop: the three side lengths as shares of the size (one side is longest).
# plates: how many crust plates. cover: how far each plate reaches toward
# its neighbours (1 = no gap). thick: plate thickness as a share of size.
# drop: share of plates left out, which leaves bare glow.
SPECS = {
    "small": dict(index=0, seed=11, prop=(0.90, 1.00, 0.86), plates=26, cover=0.64,
                  thick=0.035, drop=0.15, vary=0.30, core_cuts=6, trail=True),
    "middle": dict(index=1, seed=23, prop=(1.00, 0.88, 0.92), plates=30, cover=0.76,
                   thick=0.06, drop=0.0, vary=0.10, core_cuts=7, trail=True),
    "large": dict(index=2, seed=37, prop=(0.93, 1.00, 0.87), plates=44, cover=0.86,
                  thick=0.075, drop=0.0, vary=0.07, core_cuts=8, trail=False),
}
DENT = 0.05             # deepest dent of the glowing core, share of size
BUD_LEN_M = (0.05, 0.08)        # job text: buds on the middle size
CLUSTER_LEN_M = (0.10, 0.20)    # job text: crystals on the large size
COLUMN_ACROSS = 0.14    # six-sided columns on the large size, share of size
TRAIL_TURN_DEG = 35.0   # the trail runs behind (+Y) and off to one side


# ---------------------------------------------------------------------------
# The shape rule
# ---------------------------------------------------------------------------
def fib_sphere(n):
    out = []
    g = math.pi * (3.0 - math.sqrt(5.0))
    for i in range(n):
        z = 1.0 - 2.0 * (i + 0.5) / n
        r = math.sqrt(max(0.0, 1.0 - z * z))
        out.append(Vector((math.cos(g * i) * r, math.sin(g * i) * r, z)))
    return out


class Shape:
    """A worn, squarish pebble. at(d) gives the surface point in direction d."""

    def __init__(self, size, prop, seed):
        self.size = size
        self.H = Vector(prop) * (size / 2.0)
        self.r = ROUNDING * size
        self.off = Vector((seed * 1.7, seed * 0.31, -seed * 2.3))
        self.k = 1.0
        axis = max(range(3), key=lambda i: prop[i])
        vals = [self._raw(d)[axis] for d in fib_sphere(3000)]
        self.k = size / (max(vals) - min(vals))     # longest side = sheet size

    def _raw(self, d):
        t = min(self.H[i] / abs(d[i]) for i in range(3) if abs(d[i]) > 1e-9)
        p = d * t                                   # on the sharp box
        q = Vector([max(-(self.H[i] - self.r), min(self.H[i] - self.r, p[i]))
                    for i in range(3)])
        n = p - q
        if n.length > 1e-9:
            p = q + n.normalized() * self.r         # edges and corners worn round
        ball = d * (sum(self.H) / 3.0 * 1.2)
        p = p.lerp(ball, 0.06)                      # faces bulge a little
        lump = 0.05 * noise.noise(d * 1.5 + self.off) + 0.022 * noise.noise(d * 4.0 - self.off)
        return p * (1.0 + lump)                     # no two faces the same

    def at(self, d, inset=0.0):
        return self._raw(d) * self.k - d * inset

    def dent(self, d):
        """0 to 1: how deep the glowing core is dented in direction d."""
        v = 0.5 + 0.95 * noise.noise(d * 2.6 + self.off * 0.37)
        return max(0.0, min(1.0, v))


# ---------------------------------------------------------------------------
# Cells: the sphere cut into irregular patches, one per crust plate
# ---------------------------------------------------------------------------
def sphere_cells(count, rng):
    """Returns (cells, corners). A cell is (centre, [corner directions])."""
    pts = []
    for _ in range(count):
        best, best_d = None, -1.0
        for _ in range(10):
            v = Vector((rng.gauss(0, 1), rng.gauss(0, 1), rng.gauss(0, 1))).normalized()
            dmin = min(((v - p).length for p in pts), default=9.0)
            if dmin > best_d:
                best, best_d = v, dmin
        pts.append(best)
    bm = bmesh.new()
    vs = [bm.verts.new(p) for p in pts]
    bmesh.ops.convex_hull(bm, input=vs)
    bm.normal_update()
    corners = []
    for f in bm.faces:
        n = f.normal.copy()
        if n.dot(f.calc_center_median()) < 0:
            n = -n
        corners.append(n)
    cells = []
    for v in vs:
        c = v.co.normalized()
        a = c.orthogonal().normalized()
        b = c.cross(a)
        items = []
        for f in v.link_faces:
            n = f.normal.copy()
            if n.dot(f.calc_center_median()) < 0:
                n = -n
            items.append((math.atan2(n.dot(b), n.dot(a)), n))
        items.sort(key=lambda it: it[0])
        if len(items) >= 3:
            cells.append((c, [n for _, n in items]))
    bm.free()
    return cells, corners


# ---------------------------------------------------------------------------
# Mesh pieces. Every builder adds to a bmesh and tags a "heat" value per
# point (0 cold, 1 at the glowing core) that the materials read.
# ---------------------------------------------------------------------------
MAT_CORE, MAT_CRUST, MAT_CRYSTAL, MAT_BREAK = 0, 1, 2, 3


def heat_layer(bm):
    return bm.verts.layers.float.get("heat") or bm.verts.layers.float.new("heat")


def add_core(bm, shape, thick, cuts):
    heat = heat_layer(bm)
    res = bmesh.ops.create_cube(bm, size=2.0)
    verts = res["verts"]
    edges = list({e for v in verts for e in v.link_edges})
    bmesh.ops.subdivide_edges(bm, edges=edges, cuts=cuts, use_grid_fill=True)
    for f in bm.faces:
        f.smooth = True
        f.material_index = MAT_CORE
    for v in bm.verts:
        d = v.co.normalized()
        k = shape.dent(d)
        v.co = shape.at(d, thick + k * DENT * shape.size)
        v[heat] = k


def add_plate(bm, shape, cell, cover, thick, rng, vary=0.08):
    heat = heat_layer(bm)
    c, poly = cell
    cov = min(0.93, cover * rng.uniform(1.0 - vary, 1.0 + vary))
    w = [c.lerp(v, cov).normalized() for v in poly]
    ring = []
    for i in range(len(w)):
        a, b = w[i], w[(i + 1) % len(w)]
        ring.append(a)
        m = a.lerp(b, rng.uniform(0.35, 0.65))
        ring.append(c.lerp(m, rng.uniform(0.84, 1.07)).normalized())   # broken edge
    n = len(ring)
    reach = sum((r - c).length for r in ring) / n
    lift = thick * rng.uniform(-0.30, 0.20)
    tv = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)))
    tilt = thick * rng.uniform(0.0, 0.55) / max(reach, 1e-6)
    deep = thick + (DENT + 0.012) * shape.size

    def top(d, drop=0.0):
        chip = 0.30 * thick * noise.noise(d * 9.0 + shape.off)     # chipped, not smooth
        return shape.at(d, -(lift + chip + tilt * (d - c).dot(tv)) + drop)

    def vert(co, h):
        v = bm.verts.new(co)
        v[heat] = h
        return v

    vc = vert(top(c, -0.12 * thick), 0.0)
    mid = [vert(top(c.lerp(r, 0.62).normalized()), 0.08) for r in ring]
    out = [vert(top(r, 0.32 * thick), 0.10) for r in ring]
    low = [vert(shape.at(r, deep), 1.0) for r in ring]
    for i in range(n):
        j = (i + 1) % n
        for f, smooth in ((bm.faces.new((vc, mid[i], mid[j])), True),
                          (bm.faces.new((mid[i], out[i], out[j], mid[j])), True),
                          (bm.faces.new((out[i], low[i], low[j], out[j])), False)):
            f.smooth = smooth
            f.material_index = MAT_CRUST


def add_crystal(bm, base, axis, length, embed, rng):
    heat = heat_layer(bm)
    axis = axis.normalized()
    u = axis.orthogonal().normalized()
    v = axis.cross(u)
    rad = length * rng.uniform(0.17, 0.23)
    spin = rng.uniform(0, math.pi)
    total = length + embed
    foot, neck = [], []
    for i in range(6):
        a = spin + i * math.pi / 3.0
        side = u * math.cos(a) + v * math.sin(a)
        foot.append(bm.verts.new(base + side * rad))
        neck.append(bm.verts.new(base + side * rad * 0.86 + axis * (embed + length * 0.68)))
    tip = bm.verts.new(base + axis * total + u * rad * rng.uniform(-0.3, 0.3))
    for vv in foot:
        vv[heat] = 1.0
    for i in range(6):
        j = (i + 1) % 6
        for f in (bm.faces.new((foot[i], foot[j], neck[j], neck[i])),
                  bm.faces.new((neck[i], neck[j], tip))):
            f.material_index = MAT_CRYSTAL


def add_columns(bm, shape, face_axis, count, rng):
    """The first short six-sided columns, packed like a honeycomb on a face."""
    heat = heat_layer(bm)
    a = face_axis.normalized()
    u = a.orthogonal().normalized()
    u = (Matrix.Rotation(rng.uniform(0, 1.0), 3, a) @ u)
    v = a.cross(u)
    step = COLUMN_ACROSS * shape.size
    rad = step / math.sqrt(3.0) * 0.92
    centre = shape.at(a)
    spots = [(0.0, 0.0)] + [(step * math.cos(k * math.pi / 3), step * math.sin(k * math.pi / 3))
                            for k in range(6)]
    for n, (x, y) in enumerate(spots[:count]):
        proud = shape.size * (0.11 if n == 0 else rng.uniform(0.02, 0.095))
        lean = (u * rng.uniform(-1, 1) + v * rng.uniform(-1, 1)) * 0.012 * shape.size
        mid = centre + u * (x + rng.uniform(-0.08, 0.08) * step) + v * (y + rng.uniform(-0.08, 0.08) * step)
        turn = rng.uniform(-7, 7)
        cap_ring, top, low = [], [], []
        for i in range(6):
            ang = math.radians(30 + 60 * i + turn + rng.uniform(-5, 5))
            side = (u * math.cos(ang) + v * math.sin(ang)) * rad * rng.uniform(0.86, 1.10)
            rise = proud + lean.dot(side) / rad + rng.uniform(-0.006, 0.006) * shape.size
            k = bm.verts.new(mid + side * 0.74 + a * rise)
            t = bm.verts.new(mid + side + a * (rise - rng.uniform(0.012, 0.03) * shape.size))
            b = bm.verts.new(mid + side * 1.05 - a * 0.2 * shape.size)
            k[heat], t[heat], b[heat] = 0.0, 0.05, 1.0
            cap_ring.append(k)
            top.append(t)
            low.append(b)
        cap = bm.verts.new(mid + a * (proud + rng.uniform(-0.008, 0.01) * shape.size))
        cap[heat] = 0.0
        for i in range(6):
            j = (i + 1) % 6
            for f, smooth in ((bm.faces.new((cap, cap_ring[i], cap_ring[j])), True),
                              (bm.faces.new((cap_ring[i], top[i], top[j], cap_ring[j])), True),
                              (bm.faces.new((top[i], low[i], low[j], top[j])), False)):
                f.material_index = MAT_CRUST
                f.smooth = smooth


def far_from(c, axes, deg):
    return all(c.angle(a) > math.radians(deg) for a in axes)


def pick_corners(corners, turn, count, apart_deg):
    """Corner points of the gaps that show best: up, front and a bit right."""
    want = Vector((0.25, -0.45, 0.85)).normalized()
    ranked = sorted(corners, key=lambda d: -(turn @ d).dot(want))
    out = []
    for d in ranked:
        if all(d.angle(o) > math.radians(apart_deg) for o in out):
            out.append(d)
        if len(out) == count:
            break
    return out


# ---------------------------------------------------------------------------
# Materials
# ---------------------------------------------------------------------------
class NT:
    """Small helper for writing a node tree."""

    def __init__(self, mat):
        self.mat = mat
        self.nt = mat.node_tree
        self.bsdf = next(n for n in self.nt.nodes if n.bl_idname == "ShaderNodeBsdfPrincipled")
        tc = self.nt.nodes.new("ShaderNodeTexCoord")
        self.pos = tc.outputs["Object"]

    def set(self, node, key, v):
        if isinstance(v, (int, float, tuple, list)):
            node.inputs[key].default_value = v
        else:
            self.nt.links.new(v, node.inputs[key])

    def math(self, op, a, b=None, c=None, clamp=False):
        n = self.nt.nodes.new("ShaderNodeMath")
        n.operation = op
        n.use_clamp = clamp
        for i, v in enumerate((a, b, c)):
            if v is not None:
                self.set(n, i, v)
        return n.outputs[0]

    def noise(self, scale, detail=4.0, rough=0.55, offset=0.0):
        n = self.nt.nodes.new("ShaderNodeTexNoise")
        vec = self.pos
        if offset:
            m = self.nt.nodes.new("ShaderNodeVectorMath")
            m.operation = "ADD"
            self.nt.links.new(self.pos, m.inputs[0])
            m.inputs[1].default_value = (offset, offset * 0.7, -offset)
            vec = m.outputs[0]
        self.nt.links.new(vec, n.inputs["Vector"])
        n.inputs["Scale"].default_value = scale
        n.inputs["Detail"].default_value = detail
        n.inputs["Roughness"].default_value = rough
        return n.outputs[0]

    def step(self, v, lo, hi, out_lo=0.0, out_hi=1.0):
        n = self.nt.nodes.new("ShaderNodeMapRange")
        n.interpolation_type = "SMOOTHSTEP"
        self.set(n, 0, v)
        for i, x in enumerate((lo, hi, out_lo, out_hi)):
            n.inputs[i + 1].default_value = x
        return n.outputs[0]

    def mix(self, fac, a, b):
        n = self.nt.nodes.new("ShaderNodeMix")
        n.data_type = "RGBA"
        self.set(n, 0, fac)
        self.set(n, 6, a)
        self.set(n, 7, b)
        return n.outputs[2]

    def heat(self):
        n = self.nt.nodes.new("ShaderNodeAttribute")
        n.attribute_name = "heat"
        return n.outputs["Fac"]

    def bump(self, height, strength, distance):
        n = self.nt.nodes.new("ShaderNodeBump")
        n.inputs["Strength"].default_value = strength
        n.inputs["Distance"].default_value = distance
        self.nt.links.new(height, n.inputs["Height"])
        self.nt.links.new(n.outputs["Normal"], self.bsdf.inputs["Normal"])


def scaled(hexc, k):
    c = sl.rgb(hexc)
    return (c[0] * k, c[1] * k, c[2] * k, 1.0)


def make_materials():
    m = {}

    # The glowing core: orange fire, white-hot in the deepest dents.
    t = NT(sl.new_material("cinder_fire", "#120804", roughness=0.7)[0])
    hot = t.step(t.math("ADD", t.heat(), t.math("MULTIPLY_ADD", t.noise(16.0, 3.0), 0.4, -0.2)),
                 0.74, 1.06)
    # Most of the core has a darker, cooling skin; the full fire shows in
    # veins through it. This keeps the glow orange and uneven.
    skin = t.step(t.noise(26.0, 5.0, 0.65, offset=3.0), 0.50, 0.86, 0.09, 1.0)
    col = t.mix(hot, sl.rgb(FIRE_HEX), sl.rgb(WHITE_HOT_HEX))
    mul = t.nt.nodes.new("ShaderNodeMix")
    mul.data_type = "RGBA"
    mul.blend_type = "MULTIPLY"
    mul.inputs[0].default_value = 1.0
    t.nt.links.new(col, mul.inputs[6])
    grey = t.nt.nodes.new("ShaderNodeCombineColor")
    for i in range(3):
        t.nt.links.new(t.math("MAXIMUM", skin, hot), grey.inputs[i])
    t.nt.links.new(grey.outputs[0], mul.inputs[7])
    t.set(t.bsdf, "Emission Color", mul.outputs[2])
    t.set(t.bsdf, "Emission Strength",
          t.math("MULTIPLY_ADD", hot, WHITE_HOT_STRENGTH - FIRE_STRENGTH, FIRE_STRENGTH))
    m["fire"] = t.mat

    # The crust: dark warm stone, matte and sooty. Its broken edges, down
    # near the core, glow a dull ember red.
    t = NT(sl.new_material("cinder_crust", CRUST_HEX, roughness=0.95)[0])
    t.bsdf.inputs["Specular IOR Level"].default_value = 0.15
    grain = t.noise(22.0, 6.0, 0.7)
    soot = t.noise(5.0, 3.0, offset=7.0)
    t.set(t.bsdf, "Base Color", t.mix(t.step(t.math("MULTIPLY_ADD", grain, 0.5, soot), 0.45, 1.05),
                                      scaled(CRUST_HEX, 0.55), scaled(CRUST_HEX, 1.45)))
    h = t.heat()
    ember = t.math("MULTIPLY", t.math("POWER", h, 3.0),
                   t.math("MULTIPLY_ADD", grain, 3.0, 1.5))
    t.set(t.bsdf, "Emission Color", sl.rgb(FIRE_HEX))
    t.set(t.bsdf, "Emission Strength", ember)
    t.bump(t.math("ADD", grain, t.math("MULTIPLY", t.noise(70.0, 3.0), 0.4)), 0.9, 0.012)
    m["crust"] = t.mat

    # Crystal: orange-pink, see-through.
    mat, b = sl.new_material("cinder_crystal", CRYSTAL_HEX, roughness=0.12)
    b.inputs["Transmission Weight"].default_value = 1.0
    b.inputs["IOR"].default_value = 1.55
    m["crystal"] = mat

    # Scorch on the ground: soot that fades out at a ragged edge.
    t = NT(sl.new_material("cinder_scorch", "#0b0908", roughness=1.0)[0])
    t.bsdf.inputs["Specular IOR Level"].default_value = 0.0
    n = t.nt.nodes.new("ShaderNodeAttribute")
    n.attribute_name = "fade"
    a = t.math("ADD", t.math("MULTIPLY", n.outputs["Fac"], 1.35),
               t.math("MULTIPLY_ADD", t.noise(9.0, 5.0, 0.7), 0.9, -0.55))
    t.set(t.bsdf, "Alpha", t.math("MULTIPLY", t.step(a, 0.05, 0.75), 0.93))
    m["scorch"] = t.mat

    m["spark"] = sl.new_material("cinder_spark", "#1a0a04", roughness=0.8,
                                 emission_hex=FIRE_HEX, emission_strength=SPARK_STRENGTH)[0]

    # The dead form: cold grey stone, no light at all.
    for key, hexc, lo, hi in (("dead_crust", DEAD_CRUST_HEX, 0.6, 1.35),
                              ("dead_core", DEAD_CORE_HEX, 0.7, 1.5),
                              ("dead_break", DEAD_CRUST_HEX, 0.85, 1.7)):
        t = NT(sl.new_material("cinder_" + key, hexc, roughness=0.95)[0])
        t.bsdf.inputs["Specular IOR Level"].default_value = 0.2
        grain = t.noise(24.0, 6.0, 0.7)
        t.set(t.bsdf, "Base Color",
              t.mix(t.step(t.math("MULTIPLY_ADD", grain, 0.5, t.noise(5.0, 3.0, offset=5.0)),
                           0.45, 1.05), scaled(hexc, lo), scaled(hexc, hi)))
        t.bump(t.math("ADD", grain, t.math("MULTIPLY", t.noise(70.0, 3.0), 0.4)),
               1.0, 0.02 if key == "dead_break" else 0.012)
        m[key] = t.mat
    m["dead_crystal"] = sl.new_material("cinder_dead_crystal", DEAD_CRYSTAL_HEX, roughness=0.6)[0]
    return m


# ---------------------------------------------------------------------------
# Objects
# ---------------------------------------------------------------------------
def new_collection(name):
    c = bpy.data.collections.new(name)
    bpy.context.scene.collection.children.link(c)
    return c


def new_empty(coll, name, parent=None, size=0.1):
    ob = bpy.data.objects.new(name, None)
    ob.empty_display_size = size
    coll.objects.link(ob)
    ob.parent = parent
    return ob


def mesh_object(coll, name, bm, mats, parent):
    me = bpy.data.meshes.new(name + "_mesh")
    bm.to_mesh(me)
    bm.free()
    for mat in mats:
        me.materials.append(mat)
    ob = bpy.data.objects.new(name, me)
    coll.objects.link(ob)
    ob.parent = parent
    return ob


def tumble(key):
    """How each size is caught mid-tumble (a turn of the whole body)."""
    if key == "small":      # balanced on a corner
        to_corner = Vector((1, 1, 1)).normalized().rotation_difference(Vector((0.10, 0.06, -1)))
        return Matrix.Rotation(math.radians(25), 3, "Z") @ to_corner.to_matrix()
    if key == "middle":     # tipped on an edge, rolling forward
        return Matrix.Rotation(math.radians(-18), 3, "Z") @ Matrix.Rotation(math.radians(41), 3, "X")
    return Matrix.Rotation(math.radians(27), 3, "Z")     # large: flat, turned


def add_scorch(coll, name, marks, z, mat, parent, rng):
    """Soot marks on the ground. marks: (x, y, radius)."""
    bm = bmesh.new()
    fade = bm.verts.layers.float.new("fade")
    seg = 12
    for x, y, r in marks:
        c = bm.verts.new((x, y, z))
        c[fade] = 1.0
        mid, out = [], []
        for i in range(seg):
            a = 2 * math.pi * i / seg
            ro = r * rng.uniform(0.82, 1.15)
            p = bm.verts.new((x + math.cos(a) * ro * 0.55, y + math.sin(a) * ro * 0.55, z))
            q = bm.verts.new((x + math.cos(a) * ro, y + math.sin(a) * ro, z))
            p[fade], q[fade] = 0.8, 0.0
            mid.append(p)
            out.append(q)
        for i in range(seg):
            j = (i + 1) % seg
            bm.faces.new((c, mid[i], mid[j]))
            bm.faces.new((mid[i], out[i], out[j], mid[j]))
    ob = mesh_object(coll, name, bm, [mat], parent)
    ob.visible_shadow = False
    return ob


def add_sparks(coll, name, spots, size, mat, parent, rng):
    bm = bmesh.new()
    for p in spots:
        r = size * rng.uniform(0.007, 0.012)
        m = (Matrix.Translation(p) @ Matrix.Rotation(rng.uniform(0, 3), 4, "Y")
             @ Matrix.Diagonal((r, r, r * 2.4, 1.0)))
        bmesh.ops.create_icosphere(bm, subdivisions=1, radius=1.0, matrix=m)
    return mesh_object(coll, name, bm, [mat], parent)


def body_bmesh(key, turn, dead=False):
    """Core, crust, crystals and columns of one size, in one bmesh.

    Returns (bm, shape, cells, extras) where extras are the crystal and
    column jobs, so the dead form can share them out among its pieces.
    """
    spec = SPECS[key]
    size = SIZES[spec["index"]]
    rng = random.Random(spec["seed"])
    shape = Shape(size, spec["prop"], spec["seed"])
    thick = spec["thick"] * size
    cells, corners = sphere_cells(spec["plates"], rng)

    column_axes, crystals = [], []
    if key == "large":
        column_axes = [Vector((0.06, -1, 0.10)), Vector((1, 0.05, -0.06))]
        knot = pick_corners(corners, turn, 1, 0)[0]
        spread = knot.orthogonal().normalized()
        for i in range(5):
            side = Matrix.Rotation(i * 2 * math.pi / 5 + rng.uniform(-0.3, 0.3), 3, knot) @ spread
            lean = 0.0 if i == 0 else rng.uniform(0.25, 0.6)
            length = CLUSTER_LEN_M[1] if i == 0 else rng.uniform(CLUSTER_LEN_M[0], 0.17)
            off = Vector((0, 0, 0)) if i == 0 else side * 0.045
            crystals.append((knot, off, (knot + side * lean), length))
    elif key == "middle":
        for d in pick_corners(corners, turn, 3, 50):
            side = d.orthogonal().normalized()
            crystals.append((d, Vector((0, 0, 0)), d + side * rng.uniform(-0.3, 0.3),
                             rng.uniform(*BUD_LEN_M)))

    keep = []
    for cell in cells:
        if rng.random() < spec["drop"]:
            continue
        if not far_from(cell[0], [a.normalized() for a in column_axes], 19):
            continue
        keep.append(cell)
    return shape, thick, keep, crystals, column_axes, rng


def build_living(coll, mats, key, root):
    spec = SPECS[key]
    size = SIZES[spec["index"]]
    turn = tumble(key)
    shape, thick, cells, crystals, column_axes, rng = body_bmesh(key, turn)
    top = new_empty(coll, "%s_%s" % (NAME, key), root, size * 0.6)
    top.location = ((spec["index"] - 1) * ROW_STEP_M, 0.0, 0.0)
    body = new_empty(coll, "%s_%s_body" % (NAME, key), top, size * 0.5)

    parts = []
    bm = bmesh.new()
    add_core(bm, shape, thick, spec["core_cuts"])
    parts.append(mesh_object(coll, "%s_%s_core" % (NAME, key), bm, [mats["fire"]], body))

    bm = bmesh.new()
    for cell in cells:
        add_plate(bm, shape, cell, spec["cover"], thick, rng, spec["vary"])
    parts.append(mesh_object(coll, "%s_%s_crust" % (NAME, key), bm,
                             [mats["fire"], mats["crust"]], body))
    if column_axes:
        bm = bmesh.new()
        for a, count in zip(column_axes, (7, 5)):
            add_columns(bm, shape, a, count, rng)
        parts.append(mesh_object(coll, "%s_%s_columns" % (NAME, key), bm,
                                 [mats["fire"], mats["crust"]], body))
    if crystals:
        bm = bmesh.new()
        embed = thick + 0.03 * size
        for d, off, axis, length in crystals:
            add_crystal(bm, shape.at(d, embed) + off, axis, length, embed, rng)
        parts.append(mesh_object(coll, "%s_%s_crystals" % (NAME, key), bm,
                                 [mats["fire"], mats["crust"], mats["crystal"]], body))

    # Stand it on the ground: the lowest point of the turned body, pressed
    # a little into the soil.
    low = min((turn @ v.co).z for ob in parts for v in ob.data.vertices)
    body.rotation_euler = turn.to_euler()
    body.location = (0.0, 0.0, -low - 0.012 * size)

    # Scorch under it, and for the two smaller sizes a short trail behind.
    a = math.radians(TRAIL_TURN_DEG)
    back = Vector((math.sin(a), math.cos(a), 0.0))
    marks = [(0.0, 0.0, 0.78 * size)]
    spots = []
    if spec["trail"]:
        for dist, r in ((1.05, 0.40), (1.65, 0.31), (2.15, 0.22)):
            p = back * dist * size + Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), 0)) * 0.07 * size
            marks.append((p.x, p.y, r * size))
        for i in range(4 if key == "middle" else 3):
            p = back * rng.uniform(0.55, 1.7) * size
            spots.append(Vector((p.x + rng.uniform(-0.2, 0.2) * size, p.y,
                                 rng.uniform(0.25, 1.0) * size)))
    add_scorch(coll, "%s_%s_scorch" % (NAME, key), marks, 0.002 + 0.001 * spec["index"],
               mats["scorch"], top, rng)
    if spots:
        add_sparks(coll, "%s_%s_sparks" % (NAME, key), spots, size, mats["spark"], top, rng)
    return top, body


def build_dead(coll, mats):
    """The middle size, cold and cracked into three pieces."""
    key = "middle"
    spec = SPECS[key]
    size = SIZES[spec["index"]]
    turn = Matrix.Rotation(math.radians(38), 3, "Z") @ Matrix.Rotation(math.radians(6), 3, "X")
    shape, thick, cells, crystals, _, rng = body_bmesh(key, tumble(key))
    root = new_empty(coll, "dead_" + NAME, None, size * 0.6)

    # Two cracks. Piece a is one side of the first crack; the other side is
    # split again into b and c.
    p1, n1 = Vector((0.04, 0.02, 0.0)) * size, Vector((0.86, 0.34, 0.30)).normalized()
    p2, n2 = Vector((0.0, -0.03, 0.02)) * size, Vector((-0.18, 0.80, -0.52)).normalized()
    pieces = {
        "a": [(p1, n1, True)],
        "b": [(p1, n1, False), (p2, n2, True)],
        "c": [(p1, n1, False), (p2, n2, False)],
    }
    # How each piece fell: (slide outward, tip outward in degrees, twist)
    fall = {"a": (0.17, 15.0, 9.0), "b": (0.06, 5.0, -4.0), "c": (0.12, 11.0, 6.0)}

    def inside(point, cuts):
        return all(((point - p).dot(n) > 0) == pos for p, n, pos in cuts)

    embed = thick + 0.03 * size
    for name, cuts in pieces.items():
        bm = bmesh.new()
        add_core(bm, shape, thick, spec["core_cuts"])
        for p, n, pos in cuts:
            res = bmesh.ops.bisect_plane(bm, geom=bm.verts[:] + bm.edges[:] + bm.faces[:],
                                         dist=1e-5, plane_co=p, plane_no=n,
                                         clear_inner=pos, clear_outer=not pos)
            edges = [e for e in res["geom_cut"] if isinstance(e, bmesh.types.BMEdge)]
            caps = bmesh.ops.edgeloop_fill(bm, edges=edges)["faces"]
            poked = bmesh.ops.poke(bm, faces=caps)
            for f in poked["faces"]:
                f.material_index = MAT_BREAK
                f.smooth = False
            for v in poked["verts"]:      # a broken face is not dead flat
                v.co += n * rng.uniform(-0.035, 0.035) * size
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
        for cell in cells:
            # Plates right on a crack have flaked off.
            c0 = shape.at(cell[0])
            if inside(c0, cuts) and all(abs((c0 - p).dot(n)) > 0.07 * size for p, n, _ in cuts):
                add_plate(bm, shape, cell, spec["cover"], thick, rng, spec["vary"])
        for d, off, axis, length in crystals:
            if inside(shape.at(d), cuts):
                add_crystal(bm, shape.at(d, embed) + off, axis, length * 0.5, embed, rng)
        centre = sum((v.co for v in bm.verts), Vector()) / len(bm.verts)
        for v in bm.verts:
            v.co -= centre
        ob = mesh_object(coll, "dead_%s_piece_%s" % (NAME, name), bm,
                         [mats["dead_core"], mats["dead_crust"], mats["dead_crystal"],
                          mats["dead_break"]], root)
        slide, tip, twist = fall[name]
        g = turn @ centre
        out = Vector((g.x, g.y, 0.0)).normalized()
        rot = (Matrix.Rotation(math.radians(tip), 3, Vector((-out.y, out.x, 0.0)))
               @ Matrix.Rotation(math.radians(twist), 3, "Z") @ turn)
        low = min((rot @ v.co).z for v in ob.data.vertices)
        ob.rotation_euler = rot.to_euler()
        ob.location = (g.x + out.x * slide * size, g.y + out.y * slide * size, -low - 0.008 * size)
    add_scorch(coll, "dead_%s_scorch" % NAME, [(0.0, 0.0, 0.85 * size)], 0.002,
               mats["scorch"], root, rng)
    return root


def body_parts(top):
    return [o for o in top.children_recursive
            if o.type == "MESH" and "scorch" not in o.name and "sparks" not in o.name]


def write_size(path, top):
    m = sl.measure(body_parts(top))
    tris = sl.measure([top])["triangles"]
    data = {"name": NAME, "width_m": m["width_m"], "depth_m": m["depth_m"],
            "height_m": m["height_m"], "footprint_squares": list(FOOT), "triangles": tris}
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
        f.write("\n")
    return data


def main():
    sl.reset_scene()
    mats = make_materials()
    model = new_collection("model")
    dead = new_collection("dead")
    root = new_empty(model, NAME, None, 0.3)
    worst = 0
    for key in ("small", "middle", "large"):
        top, body = build_living(model, mats, key, root)
        data = write_size(os.path.join(HERE, "size_%s.json" % key), top)
        if key == "large":
            write_size(os.path.join(HERE, "size.json"), top)
        # The untumbled body, to check the longest side against the sheet.
        keep = body.rotation_euler.copy()
        body.rotation_euler = (0, 0, 0)
        flat = sl.measure([o for o in body_parts(top) if o.name.endswith(("_core", "_crust"))])
        body.rotation_euler = keep
        print("  %-6s as posed %s" % (key, data))
        print("         untumbled sides %.3f %.3f %.3f (sheet %.1f)"
              % (flat["width_m"], flat["depth_m"], flat["height_m"], SIZES[SPECS[key]["index"]]))
        worst = max(worst, data["triangles"])
    droot = build_dead(dead, mats)
    print("  dead  ", sl.measure([droot]))
    if worst > TRI_LIMIT:
        raise SystemExit("too many triangles: %d" % worst)
    dead.hide_render = True
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(HERE, "model.blend"), compress=True)


main()
