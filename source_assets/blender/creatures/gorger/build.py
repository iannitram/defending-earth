"""Builds the Gorger from nothing and saves model.blend next to this file.

  flatpak run org.blender.Blender -b --python "<full path>/build.py"

The file holds four collections:
  model     state 1, the whole animal lying on the surface (footprint 1 by 5)
  head_out  state 2, head and neck risen out of the ground (footprint 1 by 1)
  sign      state 3, the buried sign: a faint heaved patch and a dust wisp
  dead      the dead form of state 1

Sizes come from the scale sheet. Sizes the sheet does not hold come from the
job text (jaw, feeler, ring arc, wisp, counts) and are named below.
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

NAME = "gorger"
G = sl.thing(NAME)

# --- from the scale sheet ---------------------------------------------------
LENGTH = G["length_m"]
R_BODY = G["body_diameter_m"] / 2.0
R_TAIL = G["tail_diameter_m"] / 2.0
H_OUT = G["height_out_of_ground_m"]
PATCH = G["sign_patch_m"]
HEAVE = G["sign_heave_m"]
RING_R = sl.thing("taker")["ring_diameter_m"] / 2.0   # the Registrar's ring
SQ = sl.square()

# --- from the job text ------------------------------------------------------
JAW_L = 0.9
FEELER_L = 1.2
ARC_L = 0.6
WISP_H = 0.4
N_PLATES = 28
N_JAWS = 6
N_FEELERS = 10
N_LENSES = 5
REPAIR_PLATES = (12, 13, 14)        # three in a row, about half way along
COL_PLATE = "#a39c8c"
COL_FLESH = "#5a1c1c"
COL_SPINE = "#14171a"
COL_REG = "#0b0b0c"
COL_ETCH = "#8a6a2a"
COL_AMBER = "#ffa01e"
COL_ARC = "#cfe4ff"

# --- small shape choices of this model (not game sizes) ---------------------
HINGE_S = JAW_L                     # the jaws take the first 0.9 m of the 10 m
R_HEAD = R_BODY * 0.65
PEAK_S = LENGTH / 3.0               # thickest a third of the way back
TAIL_CAP = 0.3
PLATE_STEP = 0.06                   # how far the flesh lies under the plates
PLATE_THICK = 0.032
PLATE_S0 = HINGE_S + 0.2
PLATE_OVER = 1.36                   # plate length in pitches (so they overlap)
PITCH = (LENGTH - 0.08 - PLATE_S0) / (N_PLATES - 1 + PLATE_OVER)
PLATE_HALF = math.radians(115.0)
JAW_HALF_OPEN = math.radians(30.0)
JAW_FULL_OPEN = math.radians(60.0)
CRACKED_PLATE = 6
DS = 0.02

# --- the buried sign: how faint it is (tuned in the judging rounds) ---------
SIGN_LIGHTEN = 0.07      # how much lighter the loose soil is at the centre
SIGN_CRACK_SHADE = 0.72  # crack colour as a share of the soil colour
SIGN_CRACK_W = 0.009     # crack width at its start, metres
WISP_DENSITY = 1.7


def clamp(x, a=0.0, b=1.0):
    return max(a, min(b, x))


# ---------------------------------------------------------------------------
# Materials
# ---------------------------------------------------------------------------
def _sock(node, name, kind=None, out=False):
    for s in (node.outputs if out else node.inputs):
        if s.name == name and (kind is None or s.type == kind):
            return s
    raise KeyError("%s has no socket %s" % (node.bl_idname, name))


class NT:
    def __init__(self, name, hex_colour, roughness, metallic=0.0):
        self.mat, self.bsdf = sl.new_material(name, hex_colour, roughness, metallic)
        self.nt = self.mat.node_tree
        self.coord = self.nt.nodes.new("ShaderNodeTexCoord")

    def new(self, kind):
        return self.nt.nodes.new(kind)

    def link(self, a, b):
        self.nt.links.new(a, b)

    def obj(self):
        return self.coord.outputs["Object"]

    def uv(self):
        return self.coord.outputs["UV"]

    def scaled(self, vec, scale):
        n = self.new("ShaderNodeMapping")
        n.inputs["Scale"].default_value = scale
        self.link(vec, n.inputs["Vector"])
        return n.outputs[0]

    def sep(self, vec):
        n = self.new("ShaderNodeSeparateXYZ")
        self.link(vec, n.inputs[0])
        return n.outputs

    def noise(self, scale, detail=3.0, rough=0.55, vec=None):
        n = self.new("ShaderNodeTexNoise")
        n.inputs["Scale"].default_value = scale
        n.inputs["Detail"].default_value = detail
        n.inputs["Roughness"].default_value = rough
        self.link(vec or self.obj(), n.inputs["Vector"])
        return n.outputs[0]

    def voronoi_edge(self, scale, vec=None):
        n = self.new("ShaderNodeTexVoronoi")
        n.feature = "DISTANCE_TO_EDGE"
        n.inputs["Scale"].default_value = scale
        self.link(vec or self.obj(), n.inputs["Vector"])
        return n.outputs["Distance"]

    def math(self, op, a, b=None, clamp_=False):
        n = self.new("ShaderNodeMath")
        n.operation = op
        n.use_clamp = clamp_
        for i, v in enumerate((a, b)):
            if v is None:
                continue
            if isinstance(v, (int, float)):
                n.inputs[i].default_value = v
            else:
                self.link(v, n.inputs[i])
        return n.outputs[0]

    def remap(self, v, lo, hi, to_lo=0.0, to_hi=1.0):
        n = self.new("ShaderNodeMapRange")
        n.clamp = True
        self.link(v, n.inputs[0])
        for i, x in enumerate((lo, hi, to_lo, to_hi)):
            n.inputs[i + 1].default_value = x
        return n.outputs[0]

    def mix(self, fac, a, b):
        n = self.new("ShaderNodeMix")
        n.data_type = "RGBA"
        for s, v in ((_sock(n, "A", "RGBA"), a), (_sock(n, "B", "RGBA"), b)):
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
        n = self.new("ShaderNodeBump")
        n.inputs["Strength"].default_value = strength
        n.inputs["Distance"].default_value = distance
        self.link(height, n.inputs["Height"])
        self.link(n.outputs[0], self.bsdf.inputs["Normal"])

    def base(self, colour):
        self.link(colour, self.bsdf.inputs["Base Color"])

    def rough(self, v):
        self.link(v, self.bsdf.inputs["Roughness"])


def shade(hex_colour, k, tint=(1.0, 1.0, 1.0)):
    c = sl.rgb(hex_colour)
    return (c[0] * k * tint[0], c[1] * k * tint[1], c[2] * k * tint[2], 1.0)


def make_materials():
    m = {}

    # Armour plate: pale scuffed stone, chipped rim, soil worked into the edges.
    t = NT("gorger_plate", COL_PLATE, 0.92)
    big = t.remap(t.noise(2.2, 4.0), 0.3, 0.7)
    col = t.mix(big, shade(COL_PLATE, 0.58, (1.0, 0.95, 0.88)), shade(COL_PLATE, 1.0))
    stain = t.remap(t.noise(0.9, 5.0, 0.65), 0.36, 0.60)
    col = t.mix(t.math("MULTIPLY", stain, 0.75), col, shade(sl.SOIL_HEX, 0.85))
    scuff = t.remap(t.noise(5.0, 5.0, 0.7, t.scaled(t.obj(), (1.0, 7.0, 7.0))), 0.56, 0.68)
    col = t.mix(t.math("MULTIPLY", scuff, 0.6), col, shade(COL_PLATE, 1.22))
    pit = t.remap(t.noise(34.0, 2.0), 0.60, 0.70)
    col = t.mix(t.math("MULTIPLY", pit, 0.6), col, shade(COL_PLATE, 0.50))
    u, v, _ = t.sep(t.uv())
    edge = t.math("MINIMUM",
                  t.math("MINIMUM", u, t.math("SUBTRACT", 1.0, u)),
                  t.math("MINIMUM", v, t.math("SUBTRACT", 1.0, v)))
    front = t.remap(u, 0.0, 0.6, 0.16, 0.0)          # more soil near the tucked front
    wob = t.math("MULTIPLY", t.math("SUBTRACT", t.noise(7.0, 4.0), 0.5), 0.10)
    soil = t.remap(t.math("ADD", t.math("SUBTRACT", edge, front), wob), 0.02, 0.15, 1.0, 0.0)
    col = t.mix(t.math("MULTIPLY", soil, 0.92), col, shade(sl.SOIL_HEX, 0.62))
    chip = t.remap(t.math("ADD", edge, t.math("MULTIPLY", t.voronoi_edge(22.0), 0.25)),
                   0.012, 0.03, 1.0, 0.0)
    col = t.mix(t.math("MULTIPLY", chip, 0.7), col, shade(COL_PLATE, 0.42))
    t.base(col)
    h = t.math("ADD", t.math("MULTIPLY", t.noise(16.0, 5.0, 0.7), 0.6),
               t.math("MULTIPLY", t.noise(3.5, 4.0, 0.6), 1.4))
    t.bump(h, 0.9, 0.03)
    m["plate"] = t.mat

    # Flesh: dark red, damp.
    t = NT("gorger_flesh", COL_FLESH, 0.36)
    col = t.mix(t.remap(t.noise(6.0, 4.0), 0.3, 0.7),
                shade(COL_FLESH, 0.60), shade(COL_FLESH, 1.20))
    t.base(col)
    t.rough(t.remap(t.noise(11.0, 3.0), 0.3, 0.7, 0.24, 0.48))
    folds = t.noise(4.0, 4.0, 0.6, t.scaled(t.obj(), (1.0, 9.0, 1.0)))
    t.bump(t.math("ADD", folds, t.math("MULTIPLY", t.noise(40.0, 2.0), 0.25)), 0.5, 0.02)
    t.bsdf.inputs["Specular IOR Level"].default_value = 0.6
    m["flesh"] = t.mat

    t = NT("gorger_throat", COL_FLESH, 0.3)
    t.base(t.mix(t.remap(t.noise(8.0, 4.0), 0.3, 0.7), shade(COL_FLESH, 0.22), shade(COL_FLESH, 0.5)))
    t.bump(t.noise(5.0, 3.0, 0.6, t.scaled(t.obj(), (9.0, 9.0, 1.0))), 0.6, 0.02)
    m["throat"] = t.mat

    # SPINE: the dark frame. A regular lattice with 6 cm cells, slightly wet.
    t = NT("SPINE", COL_SPINE, 0.3)
    cell = t.new("ShaderNodeVectorMath")
    cell.operation = "FRACTION"
    t.link(t.scaled(t.obj(), (1 / 0.06,) * 3), cell.inputs[0])
    bar = None
    for axis in t.sep(cell.outputs[0]):
        a = t.math("ABSOLUTE", t.math("SUBTRACT", axis, 0.5))
        bar = a if bar is None else t.math("MAXIMUM", bar, a)
    bar = t.remap(bar, 0.36, 0.42)
    t.base(t.mix(bar, shade(COL_SPINE, 0.30), shade(COL_SPINE, 1.0)))
    t.rough(t.remap(bar, 0.0, 1.0, 0.5, 0.3))
    t.bump(bar, 0.8, 0.008)
    m["spine"] = t.mat

    # Registrar hardware: black, exact, flat.
    m["registrar"] = NT("registrar_black", COL_REG, 0.35, 1.0).mat

    # Registrar plate with etched measuring circles: thin dull gold lines.
    t = NT("registrar_etched", COL_REG, 0.35, 1.0)
    u, v, _ = t.sep(t.uv())
    period = 0.34
    um = t.math("SUBTRACT", u, 0.21)
    vm = t.math("SUBTRACT", t.math("MODULO", v, period), period / 2.0)
    d = t.math("SQRT", t.math("ADD", t.math("MULTIPLY", um, um), t.math("MULTIPLY", vm, vm)))
    sp, w = 0.042, 0.0045
    ring = t.math("LESS_THAN",
                  t.math("ABSOLUTE", t.math("SUBTRACT", t.math("FRACT", t.math("DIVIDE", d, sp)), 0.5)),
                  w / sp / 2.0)
    ring = t.math("MULTIPLY", ring, t.math("LESS_THAN", d, 0.14))
    cross = t.math("MAXIMUM", t.math("LESS_THAN", t.math("ABSOLUTE", um), 0.0018),
                   t.math("LESS_THAN", t.math("ABSOLUTE", vm), 0.0018))
    cross = t.math("MULTIPLY", cross, t.math("LESS_THAN", d, 0.155))
    line = t.math("MAXIMUM", ring, cross)
    t.base(t.mix(line, shade(COL_REG, 1.0), shade(COL_ETCH, 1.0)))
    t.rough(t.remap(line, 0.0, 1.0, 0.35, 0.55))
    m["etched"] = t.mat

    m["lens_on"] = sl.new_material("lens_amber", "#3a2206", 0.25, 0.0, COL_AMBER, 6.0)[0]
    m["lens_off"] = sl.new_material("lens_dark", "#120d07", 0.18)[0]
    m["arc"] = sl.new_material("electric_arc", "#101418", 0.5, 0.0, COL_ARC, 4.0)[0]

    # Feelers: horn, a darker relative of the plates.
    t = NT("gorger_horn", COL_PLATE, 0.6)
    z = t.sep(t.obj())[2]
    t.base(t.mix(t.remap(t.noise(9.0), 0.3, 0.7), shade(COL_PLATE, 0.45, (1.0, 0.85, 0.8)),
                 shade(COL_PLATE, 0.78)))
    m["horn"] = t.mat
    m["bone"] = sl.new_material("gorger_barb", COL_PLATE, 0.5)[0]
    m["bone"].node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = shade(COL_PLATE, 1.1)

    # Thrown-up soil: fresh from below, so darker and damper than the surface.
    t = NT("soil_fresh", sl.SOIL_HEX, 0.95)
    col = t.mix(t.remap(t.noise(5.0, 5.0), 0.3, 0.7), shade(sl.SOIL_HEX, 0.42), shade(sl.SOIL_HEX, 0.70))
    t.base(col)
    t.bump(t.math("ADD", t.noise(22.0, 5.0, 0.7), t.math("MULTIPLY", t.noise(70.0, 2.0), 0.4)), 0.9, 0.03)
    m["soil"] = t.mat
    t = NT("soil_crust", sl.SOIL_HEX, 0.95)
    t.base(t.mix(t.remap(t.noise(9.0, 4.0), 0.3, 0.7), shade(sl.SOIL_HEX, 0.80), shade(sl.SOIL_HEX, 1.08)))
    t.bump(t.noise(30.0, 4.0, 0.7), 0.5, 0.015)
    m["crust"] = t.mat

    m["sign_soil"] = sign_soil_material()
    t = NT("gorger_sign_crack", sl.SOIL_HEX, 1.0)
    t.bsdf.inputs["Base Color"].default_value = shade(sl.SOIL_HEX, SIGN_CRACK_SHADE)
    m["crack"] = t.mat
    m["dust"] = dust_material()
    return m


def sign_soil_material():
    """The shared ground soil, a little lighter and looser toward the middle
    of the patch. It is a copy of the ground's own material, so the rim of the
    patch cannot be told from the ground round it."""
    g = sl.make_ground(footprint=G["footprint_head_out"])
    src = g.data.materials[0]
    mat = src.copy()
    mat.name = "gorger_sign_soil"
    me = g.data
    bpy.data.objects.remove(g)
    bpy.data.meshes.remove(me)
    bpy.data.materials.remove(src)

    nt = mat.node_tree
    N, L = nt.nodes, nt.links
    bsdf = next(n for n in N if n.bl_idname == "ShaderNodeBsdfPrincipled")
    col_src = bsdf.inputs["Base Color"].links[0].from_socket
    nrm_src = bsdf.inputs["Normal"].links[0].from_socket

    def math(op, a, b=None, c=None):
        n = N.new("ShaderNodeMath")
        n.operation = op
        for i, v in enumerate((a, b, c)):
            if v is None:
                continue
            if isinstance(v, (int, float)):
                n.inputs[i].default_value = v
            else:
                L.new(v, n.inputs[i])
        return n.outputs[0]

    coord = N.new("ShaderNodeTexCoord")
    sep = N.new("ShaderNodeSeparateXYZ")
    L.new(coord.outputs["Object"], sep.inputs[0])
    r = math("SQRT", math("ADD", math("MULTIPLY", sep.outputs[0], sep.outputs[0]),
                          math("MULTIPLY", sep.outputs[1], sep.outputs[1])))
    fall = N.new("ShaderNodeMapRange")
    fall.interpolation_type = "SMOOTHSTEP"
    half = PATCH / 2.0
    for i, x in enumerate((half * 0.92, half * 0.25, 0.0, 1.0)):
        fall.inputs[i + 1].default_value = x
    L.new(r, fall.inputs[0])
    blot = N.new("ShaderNodeTexNoise")
    blot.inputs["Scale"].default_value = 3.1
    blot.inputs["Detail"].default_value = 4.0
    L.new(coord.outputs["Object"], blot.inputs["Vector"])
    mask = math("MULTIPLY", fall.outputs[0], math("MULTIPLY_ADD", blot.outputs[0], 1.1, 0.35))

    gain = math("MULTIPLY_ADD", mask, SIGN_LIGHTEN, 1.0)
    mul = N.new("ShaderNodeMix")
    mul.data_type = "RGBA"
    mul.blend_type = "MULTIPLY"
    mul.inputs[0].default_value = 1.0
    L.new(col_src, _sock(mul, "A", "RGBA"))
    comb = N.new("ShaderNodeCombineColor")
    for i in range(3):
        L.new(gain, comb.inputs[i])
    L.new(comb.outputs[0], _sock(mul, "B", "RGBA"))
    L.new(_sock(mul, "Result", "RGBA", out=True), bsdf.inputs["Base Color"])

    grit = N.new("ShaderNodeTexNoise")
    grit.inputs["Scale"].default_value = 55.0
    grit.inputs["Detail"].default_value = 3.0
    L.new(coord.outputs["Object"], grit.inputs["Vector"])
    bump = N.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = 0.35
    bump.inputs["Distance"].default_value = 0.012
    L.new(math("MULTIPLY", grit.outputs[0], mask), bump.inputs["Height"])
    L.new(nrm_src, bump.inputs["Normal"])
    L.new(bump.outputs[0], bsdf.inputs["Normal"])
    return mat


def dust_material():
    mat = bpy.data.materials.new("gorger_dust_wisp")
    nt = sl._node_tree_of(mat)
    N, L = nt.nodes, nt.links
    N.clear()
    out = N.new("ShaderNodeOutputMaterial")
    vol = N.new("ShaderNodeVolumePrincipled")
    vol.inputs["Color"].default_value = (0.80, 0.73, 0.62, 1.0)
    coord = N.new("ShaderNodeTexCoord")
    nz = N.new("ShaderNodeTexNoise")
    nz.inputs["Scale"].default_value = 9.0
    nz.inputs["Detail"].default_value = 3.0
    L.new(coord.outputs["Object"], nz.inputs["Vector"])
    puff = N.new("ShaderNodeMapRange")
    for i, x in enumerate((0.30, 0.62, 0.0, 1.0)):
        puff.inputs[i + 1].default_value = x
    L.new(nz.outputs[0], puff.inputs[0])
    sep = N.new("ShaderNodeSeparateXYZ")
    L.new(coord.outputs["Object"], sep.inputs[0])
    fade = N.new("ShaderNodeMapRange")
    for i, x in enumerate((0.03, WISP_H, 1.0, 0.0)):
        fade.inputs[i + 1].default_value = x
    L.new(sep.outputs[2], fade.inputs[0])
    m1 = N.new("ShaderNodeMath")
    m1.operation = "MULTIPLY"
    L.new(puff.outputs[0], m1.inputs[0])
    L.new(fade.outputs[0], m1.inputs[1])
    m2 = N.new("ShaderNodeMath")
    m2.operation = "MULTIPLY"
    L.new(m1.outputs[0], m2.inputs[0])
    m2.inputs[1].default_value = WISP_DENSITY
    L.new(m2.outputs[0], vol.inputs["Density"])
    L.new(vol.outputs[0], out.inputs["Volume"])
    return mat


# ---------------------------------------------------------------------------
# Mesh builder
# ---------------------------------------------------------------------------
class MB:
    def __init__(self):
        self.v, self.f, self.mats = [], [], []

    def mi(self, mat):
        if mat not in self.mats:
            self.mats.append(mat)
        return self.mats.index(mat)

    def vert(self, p):
        self.v.append(Vector(p))
        return len(self.v) - 1

    def face(self, pts, mat, smooth=True, uvs=None):
        ids = tuple(self.vert(p) for p in pts)
        self.f.append((ids, self.mi(mat), smooth, uvs))

    def grid(self, rows, mat, smooth=True, close=False, uv=None):
        ids = [[self.vert(p) for p in row] for row in rows]
        cols = len(ids[0])
        mi = self.mi(mat)
        for i in range(len(ids) - 1):
            for j in range(cols if close else cols - 1):
                j2 = (j + 1) % cols
                quad = (ids[i][j], ids[i][j2], ids[i + 1][j2], ids[i + 1][j])
                uvs = None
                if uv is not None:
                    uvs = (uv(i, j), uv(i, j + 1), uv(i + 1, j + 1), uv(i + 1, j))
                self.f.append((quad, mi, smooth, uvs))

    def fan(self, ring, centre, mat, smooth=True, uv=None):
        c = self.vert(centre)
        ids = [self.vert(p) for p in ring]
        mi = self.mi(mat)
        for j in range(len(ids)):
            self.f.append(((ids[j], ids[(j + 1) % len(ids)], c), mi, smooth,
                           (uv, uv, uv) if uv else None))

    def tube(self, pts, radii, mat, segs=6, caps=True, smooth=True, hint=(0, 0, 1)):
        pts = [Vector(p) for p in pts]
        rings, prev = [], Vector(hint)
        for i, p in enumerate(pts):
            t = (pts[min(i + 1, len(pts) - 1)] - pts[max(i - 1, 0)]).normalized()
            n = prev - t * prev.dot(t)
            if n.length < 1e-4:
                n = t.orthogonal()
            n.normalize()
            b = t.cross(n)
            prev = n
            rings.append([p + (n * math.cos(a) + b * math.sin(a)) * radii[i]
                          for a in (2 * math.pi * k / segs for k in range(segs))])
        self.grid(rings, mat, smooth, close=True)
        if caps:
            self.fan(rings[0], pts[0], mat, smooth)
            self.fan(rings[-1], pts[-1], mat, smooth)

    def build(self, name, coll, origin=(0, 0, 0), sharp_deg=None, orient_up=False):
        o = Vector(origin)
        me = bpy.data.meshes.new(name + "_mesh")
        me.from_pydata([tuple(p - o) for p in self.v], [], [f[0] for f in self.f])
        for mat in self.mats:
            me.materials.append(mat)
        me.polygons.foreach_set("material_index", [f[1] for f in self.f])
        me.polygons.foreach_set("use_smooth", [f[2] for f in self.f])
        if any(f[3] for f in self.f):
            layer = me.uv_layers.new(name="UVMap")
            for poly, f in zip(me.polygons, self.f):
                if f[3]:
                    for k, li in enumerate(poly.loop_indices):
                        layer.data[li].uv = f[3][k]
        bm = bmesh.new()
        bm.from_mesh(me)
        bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=1e-5)
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
        if orient_up and sum(f.normal.z * f.calc_area() for f in bm.faces) < 0:
            bmesh.ops.reverse_faces(bm, faces=bm.faces[:])
        bm.to_mesh(me)
        bm.free()
        if sharp_deg is not None and hasattr(me, "set_sharp_from_angle"):
            me.set_sharp_from_angle(angle=math.radians(sharp_deg))
        ob = bpy.data.objects.new(name, me)
        coll.objects.link(ob)
        ob.location = o
        return ob


def child(ob, parent):
    """Parent ob without moving it (everything is built in place)."""
    bpy.context.view_layer.update()
    ob.parent = parent
    ob.matrix_parent_inverse = parent.matrix_world.inverted()
    return ob


# ---------------------------------------------------------------------------
# Body shape
# ---------------------------------------------------------------------------
def R(s):
    """Outer radius (over the plates) at s metres from the closed jaw tips."""
    if s <= PEAK_S:
        t = clamp((s - HINGE_S) / (PEAK_S - HINGE_S))
        r = R_HEAD + (R_BODY - R_HEAD) * (1 - math.cos(math.pi * t)) / 2.0
    else:
        t = clamp((s - PEAK_S) / (LENGTH - TAIL_CAP - PEAK_S))
        r = R_BODY - (R_BODY - R_TAIL) * t ** 1.4
    if s > LENGTH - TAIL_CAP:
        x = clamp((s - (LENGTH - TAIL_CAP)) / TAIL_CAP)
        r *= math.sqrt(max(1.0 - x * x, 0.0))
    return r


def RF(s):
    """Radius of the flesh under the plates."""
    return R(s) * (1.0 - PLATE_STEP / R_BODY)


class Spine:
    """The centre line, sampled every DS metres from the jaw hinge back."""

    def __init__(self, pts, dorsal, roll=None):
        self.p, self.dorsal, self.roll = pts, Vector(dorsal), roll
        self.s_end = HINGE_S + (len(pts) - 1) * DS

    def at(self, s):
        x = (s - HINGE_S) / DS
        i = int(clamp(math.floor(x), 0, len(self.p) - 2))
        f = x - i
        a, b = self.p[i], self.p[i + 1]
        P = a + (b - a) * f
        j0, j1 = max(i - 2, 0), min(i + 3, len(self.p) - 1)
        T = (self.p[j1] - self.p[j0]).normalized()
        Nn = self.dorsal - T * self.dorsal.dot(T)
        Nn.normalize()
        B = T.cross(Nn)
        if self.roll is not None:
            r = self.roll(s)
            Nn = Nn * math.cos(r) - B * math.sin(r)
            B = T.cross(Nn)
        return P, T, Nn, B

    def point(self, s, phi, r):
        P, T, Nn, B = self.at(s)
        return P + (Nn * math.cos(phi) + B * math.sin(phi)) * r

    def nearest(self, q):
        best, bi = 1e9, 0
        for i in range(0, len(self.p), 5):
            d = (self.p[i] - q).length_squared
            if d < best:
                best, bi = d, i
        return self.p[bi], HINGE_S + bi * DS


def surface_spine(dead):
    span = LENGTH - HINGE_S

    def zc(s):
        lift = 0.0 if dead else 0.42 * (1.0 - clamp((s - HINGE_S) / 2.0)) ** 2
        return RF(s) + 0.022 + lift

    def heading(s):
        a = 2 * math.pi * (s - HINGE_S) / span
        if dead:
            return 0.13 * math.cos(a + 0.9) + 0.05 * math.sin(a * 2.3)
        return 0.265 * math.cos(a)

    pts, s = [], HINGE_S
    pos = Vector((0.0, 0.0, zc(s)))
    while s <= LENGTH + 1e-6:
        pts.append(pos.copy())
        dz = zc(s + DS) - zc(s)
        dz = clamp(dz, -DS * 0.9, DS * 0.9)
        h = math.sqrt(DS * DS - dz * dz)
        psi = heading(s + DS / 2.0)
        pos = pos + Vector((math.sin(psi) * h, math.cos(psi) * h, dz))
        s += DS
    roll = None
    if dead:
        def roll(s):
            t = clamp((s - 3.6) / 5.6)
            return math.radians(52.0) * math.sin(math.pi * t) ** 2
    return Spine(pts, (0, 0, 1), roll)


def head_out_spine(hinge_z):
    pts, z = [], hinge_z
    while z > -0.45:
        pts.append(Vector((0.0, 0.0, z)))
        z -= DS
    return Spine(pts, (0, -1, 0))


# ---------------------------------------------------------------------------
# Parts
# ---------------------------------------------------------------------------
def build_flesh(sp, m, coll, name, s_end, closed_tail):
    segs = 28
    mb = MB()

    def ring(s, r):
        return [sp.point(s, 2 * math.pi * j / segs, r) for j in range(segs)]

    r0 = RF(HINGE_S)
    throat = [ring(HINGE_S + 0.60, 0.05), ring(HINGE_S + 0.40, 0.11), ring(HINGE_S + 0.20, 0.19),
              ring(HINGE_S + 0.05, r0 * 0.86), ring(HINGE_S - 0.03, r0 * 0.94)]
    mb.grid(throat, m["throat"], close=True)
    mb.fan(throat[0], sp.at(HINGE_S + 0.70)[0], m["throat"])
    rows = [throat[-1], ring(HINGE_S, r0)]
    s = HINGE_S + 0.07
    while s < s_end - 0.01:
        rows.append(ring(s, RF(s)))
        s += 0.07
    if closed_tail:
        rows.append(ring(LENGTH - 0.02, RF(LENGTH - 0.02)))
    else:
        rows.append(ring(s_end, RF(s_end)))
    mb.grid(rows, m["flesh"], close=True)
    if closed_tail:
        mb.fan(rows[-1], sp.at(LENGTH)[0], m["flesh"])
    return mb.build(name, coll)


def build_ribs(sp, m, coll, name, s_end):
    """The dark frame showing through the belly as regular ribs."""
    mb = MB()
    s = PLATE_S0 + 0.1
    while s < min(s_end, LENGTH - 0.25):
        rows = []
        for k in range(13):
            phi = math.radians(108.0 + 144.0 * k / 12.0)
            rows.append([sp.point(s + ds, phi, RF(s + ds) + dr)
                         for ds, dr in ((-0.028, -0.012), (-0.022, 0.024), (0.022, 0.024), (0.028, -0.012))])
        mb.grid(rows, m["spine"], smooth=False, close=True)
        s += 0.2
    return mb.build(name, coll)


def build_plate(sp, m, coll, name, i, registrar=False, part=None):
    """One armour plate. part = (which, v_split, zigzag) cuts it in two for
    the cracked plate of the dead form."""
    rng = random.Random(1000 + i)
    s0 = PLATE_S0 + i * PITCH
    ncol, nrow = (6, 1) if registrar else (20, 5)
    k_len = 1.0 if registrar else rng.uniform(0.97, 1.04)
    chips_rear = [rng.choice((0.0, 0.0, 0.012, 0.03, 0.055)) for _ in range(ncol + 1)]
    chips_side = [rng.uniform(0.0, 0.07) for _ in range(nrow + 1)]
    vlo, vhi = 0.0, 1.0
    if part:
        which, vs, zig = part
        vlo, vhi = (0.0, vs) if which == "a" else (vs, 1.0)
        ncol = max(4, int(round(ncol * (vhi - vlo))))
    outer, inner, uvs = [], [], []
    for a in range(nrow + 1):
        u = a / nrow
        ro, ri, ru = [], [], []
        for b in range(ncol + 1):
            v = vlo + (vhi - vlo) * b / ncol
            lift = 0.0
            if part:
                edge = (which == "a" and b == ncol) or (which == "b" and b == 0)
                if edge:
                    v += zig[a] + (-0.016 if which == "a" else 0.016)
                if which == "b":
                    lift = 0.06 * (1.0 - (v - vs) / (1.0 - vs)) ** 2
            if not registrar:
                if v <= 0.0:
                    v += chips_side[a] * 0.3
                if v >= 1.0:
                    v -= chips_side[nrow - a] * 0.3
            phi = -PLATE_HALF + 2 * PLATE_HALF * v
            x = abs(phi) / PLATE_HALF
            taper = 1.0 if registrar else 1.0 - 0.55 * x ** 1.5
            length = PITCH * PLATE_OVER * taper * k_len
            s = s0 + u * length
            if not registrar and a == nrow:
                s -= chips_rear[min(int(round(v * 20)), 20)]
            scale = R(s) / R_BODY
            r_out = R(s) - PLATE_STEP * (1.0 - u) ** 1.5 * scale + lift
            if registrar:
                r_out += 0.012
            else:
                r_out += 0.034 * math.exp(-(phi / 0.26) ** 2) * u * u * scale
            if not registrar:
                r_out += 0.016 * noise.noise(Vector((i * 3.1 + u * 0.9, v * 4.5, 0.5))) * scale
            thick = (0.026 if registrar else PLATE_THICK) * max(scale, 0.5)
            ro.append(sp.point(s, phi, r_out))
            ri.append(sp.point(s, phi, r_out - thick))
            if registrar:
                ru.append((u * length, (b + 0.0) * 0.34))
            else:
                ru.append((u, v))
        outer.append(ro)
        inner.append(ri)
        uvs.append(ru)
    mat = m["etched"] if registrar else m["plate"]
    smooth = not registrar
    rim = (lambda i_, j_: (-5.0, -5.0)) if registrar else (lambda i_, j_: (0.0, 0.0))
    mb = MB()
    mb.grid(outer, mat, smooth, uv=lambda a, b: uvs[a][b])
    mb.grid(inner, m["registrar"] if registrar else mat, smooth, uv=rim)
    mb.grid([outer[0], inner[0]], mat, False, uv=rim)
    mb.grid([outer[-1], inner[-1]], mat, False, uv=rim)
    mb.grid([[row[0] for row in outer], [row[0] for row in inner]], mat, False, uv=rim)
    mb.grid([[row[-1] for row in outer], [row[-1] for row in inner]], mat, False, uv=rim)
    origin = sp.point(s0, 0.0, R(s0))
    return mb.build(name, coll, origin, sharp_deg=None if registrar else 38.0)


def build_jaw(sp, m, coll, name, k, open_angle, ground_clamp):
    """One of the six jaws. Closed, the six make a pointed bud 0.9 m long."""
    C, T, Nn, B = sp.at(HINGE_S)
    F = -T
    r0 = RF(HINGE_S)
    phi_j = 2 * math.pi * (k + 0.5) / N_JAWS

    def rad(phi):
        return Nn * math.cos(phi) + B * math.sin(phi)

    rows_d, cols = 10, 8
    outer, inner = [], []
    for a in range(rows_d + 1):
        t = a / rows_d
        d = JAW_L * t
        half = math.radians(28.5) * (1.0 - 0.42 * t)
        ro, ri = [], []
        for b in range(cols + 1):
            dl = -half + 2 * half * b / cols
            ridge = 0.03 * math.exp(-(dl / 0.14) ** 2) * (1.0 - t) ** 0.5 * (0.6 + 0.4 * math.sin(t * 22.0) ** 2)
            rho_o = r0 * (1.0 - t ** 2.0) + ridge
            th = 0.115 * (1.0 - t) ** 0.8 + 0.008
            rho_i = max(rho_o - th, 0.0)
            ro.append(C + F * d + rad(phi_j + dl) * rho_o)
            ri.append(C + F * d + rad(phi_j + dl) * rho_i)
        outer.append(ro)
        inner.append(ri)
    mb = MB()
    uv = lambda a, b: (0.15 + 0.7 * a / rows_d, 0.15 + 0.7 * b / cols)
    mb.grid(outer, m["plate"], True, uv=uv)
    mb.grid(inner, m["flesh"], True)
    mb.grid([outer[0], inner[0]], m["flesh"], False)
    rim = lambda a, b: (0.0, 0.0)
    mb.grid([[r[0] for r in outer], [r[0] for r in inner]], m["plate"], True, uv=rim)
    mb.grid([[r[-1] for r in outer], [r[-1] for r in inner]], m["plate"], True, uv=rim)
    # Short barbs on the inner face, pointing in and back toward the throat.
    for dl in (-0.20, 0.0, 0.20):
        d = 0.10 if dl else 0.14
        while d < JAW_L * 0.86:
            t = d / JAW_L
            half = math.radians(28.5) * (1.0 - 0.42 * t)
            a_ = phi_j + dl * half / math.radians(28.5)
            rho_o = r0 * (1.0 - t ** 2.0)
            rho_i = max(rho_o - (0.115 * (1.0 - t) ** 0.8 + 0.008), 0.0)
            base = C + F * d + rad(a_) * (rho_i + 0.004)
            direction = (-rad(a_) * 0.8 - F * 0.6).normalized()
            n1 = direction.orthogonal().normalized()
            n2 = direction.cross(n1)
            size = 0.9 if dl else 1.15
            ring = [base + (n1 * math.cos(q) + n2 * math.sin(q)) * 0.017 * size
                    for q in (0, math.pi / 2, math.pi, 3 * math.pi / 2)]
            mb.fan(ring, base + direction * 0.065 * size, m["bone"], False)
            d += 0.085
    H = C + rad(phi_j) * r0
    axis = F.cross(rad(phi_j)).normalized()
    ob = mb.build(name, coll, H, sharp_deg=62.0)
    local = [v.co.copy() for v in ob.data.vertices]
    theta = open_angle
    while True:
        M = Matrix.Translation(H) @ Matrix.Rotation(theta, 4, axis)
        if not ground_clamp or theta <= 0.0 or min((M @ p).z for p in local) >= 0.012:
            break
        theta = max(theta - math.radians(1.5), 0.0)
    ob.matrix_world = M
    return ob


def feeler_path(sp, k, mode):
    s_root = HINGE_S + 0.10
    phi = 2 * math.pi * (k + 0.5) / N_FEELERS
    P, T, Nn, B = sp.at(s_root)
    out = Nn * math.cos(phi) + B * math.sin(phi)
    rng = random.Random(40 + k)
    n, seg = 12, FEELER_L / 12
    p = P + out * (RF(s_root) - 0.015)
    pts = [p.copy()]
    wob = rng.uniform(-0.05, 0.05)
    side = T.cross(out)
    d = None
    for i in range(n):
        t = i / (n - 1)
        if mode == "dead":
            base = T * 0.9 + out * 0.26 if d is None else d
            d = (base + Vector((0, 0, -0.30))).normalized()
        else:
            alpha = (math.radians(31.0 - 13.0 * t) + wob) * (1.0 - 0.6 * max(0.0, -out.z) * (mode == "surface"))
            d = (T * math.cos(alpha) + out * math.sin(alpha) + side * wob * t).normalized()
        q = p + d * seg
        if mode != "head_out":
            floor = 0.012
            if q.z < floor:
                flat = Vector((d.x, d.y, 0.0))
                dz = floor - p.z
                if flat.length > 1e-4 and abs(dz) < seg:
                    q = p + flat.normalized() * math.sqrt(seg * seg - dz * dz) + Vector((0, 0, dz))
                else:
                    q.z = floor
            if mode == "dead":
                c, sc = sp.nearest(q)
                off = q - c
                need = R(sc) + 0.02
                if off.length < need:
                    q = c + off.normalized() * need
                    q.z = max(q.z, floor)
                d = (q - p).normalized()
        p = q
        pts.append(p.copy())
    return pts


def build_feelers(sp, m, coll, prefix, mode, parent):
    paths = []
    for k in range(N_FEELERS):
        pts = feeler_path(sp, k, mode)
        paths.append(pts)
        radii = [0.024 - 0.020 * (i / (len(pts) - 1)) ** 0.8 for i in range(len(pts))]
        mb = MB()
        mb.tube(pts, radii, m["horn"], segs=6, hint=sp.at(HINGE_S)[1])
        child(mb.build("%sfeeler_%02d" % (prefix, k + 1), coll, pts[0]), parent)
    return paths


def build_arcs(paths, m, coll, name):
    """Electrical life: thin pale arcs between neighbouring feelers."""
    rng = random.Random(7)
    mb = MB()
    for k in range(0, N_FEELERS, 2):
        a = paths[k][rng.choice((5, 6, 7, 8))]
        b = paths[(k + 1) % N_FEELERS][rng.choice((6, 7, 8, 9))]
        n = 7
        pts = []
        for i in range(n + 1):
            p = a.lerp(b, i / n)
            if 0 < i < n:
                p = p + Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))) * 0.03
            pts.append(p)
        mb.tube(pts, [0.002] * len(pts), m["arc"], segs=4, caps=False)
        # a short side spark
        j = rng.randint(2, 5)
        tip = pts[j] + Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))) * 0.07
        mb.tube([pts[j], pts[j].lerp(tip, 0.5) + Vector((0.01, -0.01, 0.01)), tip],
                [0.0018, 0.0015, 0.001], m["arc"], segs=4, caps=False)
    ob = mb.build(name, coll)
    ob.hide_render = True          # shown only in the night picture
    ob["night_only"] = True
    return ob


def build_hardware(sp, m, coll, prefix, parent, lit):
    """Registrar control hardware: lens housing and a broken arc of its ring."""
    s_h = HINGE_S + 0.64
    P, T, Nn, B = sp.at(s_h)
    O = P + Nn * (R(s_h) - 0.035)

    def at(t, b, n):
        return O + T * t + B * b + Nn * n

    hx, hy, h1, h2, inset = 0.19, 0.15, 0.115, 0.145, 0.03
    lo = [at(-hx, -hy, 0), at(hx, -hy, 0), at(hx, hy, 0), at(-hx, hy, 0)]
    mid = [at(-hx, -hy, h1), at(hx, -hy, h1), at(hx, hy, h1), at(-hx, hy, h1)]
    top = [at(-hx + inset, -hy + inset, h2), at(hx - inset, -hy + inset, h2),
           at(hx - inset, hy - inset, h2), at(-hx + inset, hy - inset, h2)]
    mb = MB()
    mb.grid([lo, mid, top], m["registrar"], smooth=False, close=True)
    mb.face(top, m["registrar"], smooth=False)
    housing = child(mb.build(prefix + "lens_housing", coll, at(0, 0, 0)), parent)

    spots = [(-0.075, -0.020, 1.35), (0.035, 0.055, 1.0), (0.050, -0.062, 0.8),
             (0.118, 0.004, 0.62), (-0.050, 0.082, 0.55)]
    for i, (t, b, k) in enumerate(spots[:N_LENSES]):
        c = at(t, b, h2)
        mb = MB()
        segs = 14
        def circ(r, n):
            return [c + (T * math.cos(q) + B * math.sin(q)) * r + Nn * n
                    for q in (2 * math.pi * j / segs for j in range(segs))]
        mb.grid([circ(0.040 * k, -0.004), circ(0.040 * k, 0.018), circ(0.031 * k, 0.018)], m["registrar"],
                smooth=False, close=True)
        glass = m["lens_on"] if lit else m["lens_off"]
        mb.grid([circ(0.031 * k, 0.018), circ(0.031 * k, 0.010), circ(0.020 * k, 0.015)], glass, True, close=True)
        mb.fan(circ(0.020 * k, 0.015), c + Nn * 0.017, glass)
        child(mb.build("%slens_%d" % (prefix, i + 1), coll, c, sharp_deg=40.0), housing)

    # The broken arc: a piece of the Registrar's ring, flatter than the back
    # it is bolted across, so its ends stand off on two bolts.
    s_a = s_h + hx + 0.07
    P, T, Nn, B = sp.at(s_a)
    sect_t, sect_r = 0.07, 0.05
    top_r = R(s_a) + 0.05
    centre = P + Nn * (top_r + sect_r / 2.0 - RING_R)
    half = ARC_L / RING_R / 2.0
    tilt = math.radians(-34.0)
    rng = random.Random(3)
    rows = []
    n = 12
    for i in range(n + 1):
        a = -half + 2 * half * i / n + tilt
        row = []
        for dt, dr in ((-1, -1), (-1, 1), (1, 1), (1, -1)):
            aa = a
            if i in (0, n):                        # broken, uneven ends
                aa += rng.uniform(-0.05, 0.05) + (0.045 if (i == 0) == (dt > 0) else -0.02)
            d = Nn * math.cos(aa) + B * math.sin(aa)
            row.append(centre + d * (RING_R + dr * sect_r / 2.0) + T * (dt * sect_t / 2.0))
        rows.append(row)
    mb = MB()
    mb.grid(rows, m["registrar"], smooth=True, close=True)
    for row in (rows[0], rows[-1]):
        mid_p = sum(row, Vector()) / 4.0
        mb.fan(row, mid_p + (row[0] - rows[n // 2][0]).normalized() * 0.012, m["registrar"], False)
    for sgn in (-0.62, 0.62):
        a = sgn * half + tilt
        d = Nn * math.cos(a) + B * math.sin(a)
        top_p = centre + d * (RING_R + sect_r / 2.0 + 0.012)
        phi_body = math.atan2((top_p - P).dot(B), (top_p - P).dot(Nn))
        foot = sp.point(s_a, phi_body, R(s_a) - 0.03)
        mb.tube([foot, top_p], [0.016, 0.016], m["registrar"], segs=6, smooth=False)
        mb.tube([top_p - d * 0.014, top_p], [0.026, 0.026], m["registrar"], segs=6, smooth=False)
    arc = mb.build(prefix + "ring_arc", coll, centre + Nn * RING_R, sharp_deg=30.0)
    child(arc, parent)
    return housing


# ---------------------------------------------------------------------------
# The animal, in one of three poses
# ---------------------------------------------------------------------------
def build_animal(coll, m, mode, prefix, offset=Vector((0, 0, 0)), hinge_z=2.3):
    dead = mode == "dead"
    if mode == "head_out":
        sp = head_out_spine(hinge_z)
        s_end = sp.s_end
    else:
        sp = surface_spine(dead)
        s_end = LENGTH
    sp.p = [p + offset for p in sp.p]

    root = bpy.data.objects.new(prefix + NAME, None)
    root.empty_display_size = 0.5
    coll.objects.link(root)
    body = child(build_flesh(sp, m, coll, prefix + "body", s_end, mode != "head_out"), root)
    child(build_ribs(sp, m, coll, prefix + "belly_frame", s_end), body)

    for i in range(N_PLATES):
        s0 = PLATE_S0 + i * PITCH
        if s0 > s_end - 0.05:
            break
        name = "%splate_%02d" % (prefix, i + 1)
        if dead and i == CRACKED_PLATE:
            rng = random.Random(99)
            zig = [rng.uniform(-0.035, 0.035) for _ in range(8)]
            for which in ("a", "b"):
                child(build_plate(sp, m, coll, name + which, i, part=(which, 0.60, zig)), body)
        else:
            child(build_plate(sp, m, coll, name, i, registrar=i in REPAIR_PLATES), body)

    C, T, Nn, B = sp.at(HINGE_S)
    for k in range(N_JAWS):
        if mode == "head_out":
            ang = JAW_FULL_OPEN
        elif dead:
            g = (Nn * math.cos(2 * math.pi * (k + 0.5) / N_JAWS)
                 + B * math.sin(2 * math.pi * (k + 0.5) / N_JAWS)).z
            ang = math.radians(34.0 - 26.0 * g + (7.0 if k % 2 else -4.0))
        else:
            ang = JAW_HALF_OPEN
        child(build_jaw(sp, m, coll, "%sjaw_%d" % (prefix, k + 1), k, ang, mode != "head_out"), body)

    paths = build_feelers(sp, m, coll, prefix, mode, body)
    if not dead:
        child(build_arcs(paths, m, coll, prefix + "electric_arcs"), body)
    build_hardware(sp, m, coll, prefix, body, lit=not dead)
    return root


def clear(coll):
    for o in list(coll.objects):
        data = o.data
        bpy.data.objects.remove(o)
        if data is not None and data.users == 0:
            bpy.data.meshes.remove(data)


def new_collection(name):
    c = bpy.data.collections.new(name)
    bpy.context.scene.collection.children.link(c)
    return c


def visible(root):
    return [o for o in root.children_recursive if not o.get("night_only")]


def fitted_animal(coll, m, mode, prefix):
    """Build once, measure, then build again so the footprint is centred on
    the origin (and, head out, so the top stands at the sheet's height)."""
    root = build_animal(coll, m, mode, prefix)
    lo, hi = sl.bounds(visible(root))
    clear(coll)
    if mode == "head_out":
        return build_animal(coll, m, mode, prefix, hinge_z=2.3 + (H_OUT - hi.z))
    off = Vector((-(lo.x + hi.x) / 2.0, -(lo.y + hi.y) / 2.0, 0.0))
    return build_animal(coll, m, mode, prefix, offset=off)


# ---------------------------------------------------------------------------
# Soil round the hole (state 2)
# ---------------------------------------------------------------------------
def ring_height(rho, a):
    """Height of the thrown-up soil at radius rho, angle a."""
    r_in, r_out = 0.40, 0.97
    edge = r_out - 0.10 * (0.5 + 0.5 * noise.noise(Vector((math.cos(a) * 1.3, math.sin(a) * 1.3, 2.0))))
    x = clamp((rho - r_in) / (edge - r_in))
    prof = math.sin(math.pi * x ** 0.62) ** 0.9
    amp = 0.25 * (0.75 + 0.5 * noise.noise(Vector((math.cos(a) * 1.9, math.sin(a) * 1.9, 5.0))))
    lump = 0.05 * noise.noise(Vector((rho * math.cos(a) * 6.0, rho * math.sin(a) * 6.0, 9.0)))
    return prof * (amp + lump)


def build_soil(coll, m, parent):
    mb = MB()
    nr, na = 14, 56
    rows = []
    for i in range(nr + 1):
        rho = 0.40 + (0.99 - 0.40) * i / nr
        row = []
        for j in range(na):
            a = 2 * math.pi * j / na
            z = ring_height(rho, a)
            if i == 0:
                z = -0.12
            if i == nr:
                z = -0.01
            row.append(Vector((rho * math.cos(a), rho * math.sin(a), z)))
        rows.append(row)
    mb.grid(rows, m["soil"], close=True)
    child(mb.build("head_out_soil_ring", coll, orient_up=True), parent)

    rng = random.Random(21)
    bm = bmesh.new()
    for _ in range(34):
        a = rng.uniform(0, 2 * math.pi)
        rho = rng.uniform(0.55, 0.93)
        size = rng.choice((0.04, 0.05, 0.06, 0.08, 0.11, 0.15))
        res = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=size)
        rot = Matrix.Rotation(rng.uniform(0, 6.3), 3, Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), 1)).normalized())
        sc = Vector((rng.uniform(0.7, 1.3), rng.uniform(0.7, 1.3), rng.uniform(0.5, 0.9)))
        at = Vector((rho * math.cos(a), rho * math.sin(a), ring_height(rho, a) + size * 0.25))
        for v in res["verts"]:
            k = rng.uniform(0.72, 1.22)
            v.co = at + rot @ Vector((v.co.x * sc.x * k, v.co.y * sc.y * k, v.co.z * sc.z * k))
    me = bpy.data.meshes.new("head_out_soil_clods_mesh")
    bm.to_mesh(me)
    bm.free()
    me.materials.append(m["soil"])
    ob = bpy.data.objects.new("head_out_soil_clods", me)
    coll.objects.link(ob)
    child(ob, parent)

    # Broken slabs of the dry top crust, tipped up and outward by the body.
    mb = MB()
    for k in range(9):
        a = 2 * math.pi * (k + rng.uniform(-0.3, 0.3)) / 9.0
        rho = rng.uniform(0.60, 0.78)
        w, l, th = rng.uniform(0.16, 0.28), rng.uniform(0.18, 0.30), rng.uniform(0.035, 0.055)
        tilt = rng.uniform(0.45, 1.0)
        out = Vector((math.cos(a), math.sin(a), 0))
        tang = Vector((-math.sin(a), math.cos(a), 0))
        up = (Vector((0, 0, 1)) * math.cos(tilt) - out * math.sin(tilt))
        along = (out * math.cos(tilt) + Vector((0, 0, 1)) * math.sin(tilt))
        c = out * rho + Vector((0, 0, ring_height(rho, a) + 0.03))
        pts = []
        for sz in (0, 1):
            ring = []
            for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
                jit = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), 0)) * 0.03
                ring.append(c + tang * (sx * w / 2 + jit.x) + along * (sy * l / 2 + jit.y) + up * (sz * th))
            pts.append(ring)
        mb.grid(pts, m["soil"], smooth=False, close=True)
        mb.face(pts[1], m["crust"], smooth=False)
        mb.face(pts[0], m["soil"], smooth=False)
    child(mb.build("head_out_soil_slabs", coll), parent)


# ---------------------------------------------------------------------------
# The buried sign (state 3)
# ---------------------------------------------------------------------------
_sign_scale = None


def _sign_raw(x, y):
    a = math.atan2(y, x)
    reach = 0.84 + 0.16 * noise.noise(Vector((math.cos(a) * 0.9, math.sin(a) * 0.9, 7.0)))
    rho = math.hypot(x, y) / (PATCH / 2.0 * reach)
    if rho >= 1.0:
        return 0.0
    f = (1.0 - rho * rho) ** 2
    n1 = noise.noise(Vector((x * 1.7 + 11.3, y * 1.7 - 4.1, 0.37)))
    n2 = noise.noise(Vector((x * 5.0 - 3.0, y * 5.0 + 8.0, 1.9)))
    return f * (1.0 + 0.45 * n1 + 0.15 * n2)


def sign_z(x, y):
    global _sign_scale
    lift = 0.0015
    if _sign_scale is None:
        peak = max(_sign_raw(-1 + i * 0.02, -1 + j * 0.02) for i in range(101) for j in range(101))
        _sign_scale = (HEAVE - lift) / peak
    rho = math.hypot(x, y) / (PATCH / 2.0)
    if rho >= 0.999:
        return -0.002                      # the rim tucks just under the ground
    return _sign_raw(x, y) * _sign_scale + lift * clamp((0.97 - rho) / 0.5)


def build_sign(coll, m):
    root = bpy.data.objects.new("sign_" + NAME, None)
    root.empty_display_size = 0.5
    coll.objects.link(root)
    half = PATCH / 2.0

    mb = MB()
    nr, na = 26, 80
    rows = []
    for i in range(1, nr + 1):
        rho = half * i / nr
        rows.append([Vector((rho * math.cos(a), rho * math.sin(a),
                             sign_z(rho * math.cos(a), rho * math.sin(a))))
                     for a in (2 * math.pi * j / na for j in range(na))])
    mb.grid(rows, m["sign_soil"], close=True)
    mb.fan(rows[0], (0, 0, sign_z(0, 0)), m["sign_soil"])
    child(mb.build("sign_heaved_soil", coll, orient_up=True), root)

    rng = random.Random(5)
    mb = MB()
    crack_pts = []
    angles = [0.5, 1.75, 2.9, 4.3, 5.45]
    for k, a0 in enumerate(angles):
        a = a0 + rng.uniform(-0.15, 0.15)
        length = rng.uniform(0.42, 0.78)
        p = Vector((math.cos(a), math.sin(a), 0)) * rng.uniform(0.03, 0.10)
        pts = [p.copy()]
        run, step = 0.0, 0.05
        while run < length:
            a += rng.uniform(-0.42, 0.42)
            p = p + Vector((math.cos(a), math.sin(a), 0)) * step
            pts.append(p.copy())
            run += step
        crack_pts.append(pts)
        branches = [pts]
        if k in (1, 3):                       # two of them fork once
            j = len(pts) // 2
            b = a0 + (0.8 if k == 1 else -0.8)
            q = pts[j].copy()
            br = [q.copy()]
            for _ in range(5):
                b += rng.uniform(-0.4, 0.4)
                q = q + Vector((math.cos(b), math.sin(b), 0)) * step
                br.append(q.copy())
            branches.append(br)
        for line in branches:
            rows = []
            for i, q in enumerate(line):
                t = i / (len(line) - 1)
                d = (line[min(i + 1, len(line) - 1)] - line[max(i - 1, 0)]).normalized()
                side = Vector((-d.y, d.x, 0))
                w = SIGN_CRACK_W * (1.0 - t) ** 0.8 * (0.7 if line is not pts else 1.0) + 0.0015
                row = []
                for sgn in (-1, 1):
                    e = q + side * (sgn * w / 2.0)
                    e.z = sign_z(e.x, e.y) + 0.0012
                    row.append(e)
                rows.append(row)
            mb.grid(rows, m["crack"], smooth=False)
    cracks = child(mb.build("sign_cracks", coll, orient_up=True), root)
    cracks.visible_shadow = False

    # One thin wisp of dust from a crack, leaning with the wind (+X).
    line = crack_pts[4]
    base = line[len(line) * 2 // 3].copy()
    base.z = sign_z(base.x, base.y)
    n = 9
    pts, radii = [], []
    for i in range(n + 1):
        t = i / n
        pts.append(base + Vector((0.26 * t ** 1.6, 0.05 * t * t, WISP_H * t)))
        radii.append(0.014 + 0.075 * t ** 1.2)
    mb = MB()
    mb.tube(pts, radii, m["dust"], segs=10)
    wisp = child(mb.build("sign_dust_wisp", coll, base), root)
    wisp.visible_shadow = False
    return root


# ---------------------------------------------------------------------------
def write_size(path, objects, footprint):
    d = sl.measure(objects)
    # Height is what stands above the ground; buried parts do not count.
    data = {"name": NAME, "width_m": d["width_m"], "depth_m": d["depth_m"],
            "height_m": round(d["max"][2] - max(d["min"][2], 0.0), 3), "footprint_squares": list(footprint),
            "triangles": d["triangles"]}
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
        f.write("\n")
    return data


def main():
    sl.reset_scene()
    m = make_materials()
    model = new_collection("model")
    head_out = new_collection("head_out")
    sign = new_collection("sign")
    dead = new_collection("dead")

    root = fitted_animal(model, m, "surface", "")
    hroot = fitted_animal(head_out, m, "head_out", "head_out_")
    build_soil(head_out, m, hroot)
    droot = fitted_animal(dead, m, "dead", "dead_")
    sroot = build_sign(sign, m)

    surf = write_size(os.path.join(HERE, "size_surface.json"), visible(root), G["footprint"])
    write_size(os.path.join(HERE, "size.json"), visible(root), G["footprint"])
    animal = [o for o in visible(hroot) if "soil" not in o.name]
    ho = write_size(os.path.join(HERE, "size_head_out.json"), animal, G["footprint_head_out"])
    dd = sl.measure(visible(droot))
    sg = sl.measure([o for o in sroot.children if "wisp" not in o.name])
    sp = surface_spine(False)
    print("GORGER surface:", surf)
    print("GORGER spine length (jaw tip to tail tip, m): %.3f" % (HINGE_S + (len(sp.p) - 1) * DS))
    print("GORGER head out:", ho, "with soil:", sl.measure(visible(hroot)))
    print("GORGER dead:", dd)
    print("GORGER sign patch:", sg)

    for mat in list(bpy.data.materials):
        if mat.users == 0:
            bpy.data.materials.remove(mat)
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(HERE, "model.blend"), compress=True)
    b1 = os.path.join(HERE, "model.blend1")
    if os.path.exists(b1):
        os.remove(b1)


main()
