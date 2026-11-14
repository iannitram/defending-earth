"""Builds the Bloat from nothing and saves model.blend.

Run from anywhere:
  flatpak run org.blender.Blender -b --python "<full path>/build.py"

The file holds two collections:
  model  the living Bloat, floating (root object "bloat")
  dead   the burst form on the ground (root object "dead_bloat"),
         hidden from rendering until render.py asks for it
"""
import heapq
import math
import os
import random
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "scale")))

import bpy
from mathutils import Matrix, Vector, noise

import scale_lib as sl

NAME = "bloat"

# ---------------------------------------------------------------------------
# Sizes. The three big ones come from the scale sheet. The part sizes are the
# ones the job text gives. Shape words in the job text ("a little taller than
# wide", "thin") are turned into ratios here, not into new sizes.
# ---------------------------------------------------------------------------
SHEET = sl.thing(NAME)
DIAMETER = SHEET["diameter_m"]
UNDERSIDE = SHEET["underside_height_m"]
FEELER_L = SHEET["feeler_length_m"]

PLUG_D = 0.08            # job text: black plug, 8 cm across
PATCH_D = 0.30           # job text: wrong repair, 30 cm across
CELL = 0.06              # shared colours: frame lattice, 6 cm cells

TALL = 1.12              # "a little taller than wide": height over width
WIDEST = 0.40            # "widest below the middle": share of the height
N_RIBS = 8
N_FEELERS = 5

R = DIAMETER / 2.0
SAC_H = DIAMETER * TALL
ZB = SAC_H * WIDEST          # widest plane above the underside
ZT = SAC_H - ZB              # top above the widest plane
PHI0 = math.pi / N_RIBS      # the front (-Y) lies between two ribs
GROOVE = 0.058               # how far a rib pulls the skin in (share of radius)
RING_GROOVE = 0.030
RIB_W, RIB_T = 0.036, 0.018  # rib section: across and deep
PATCH_PHI = math.radians(-90.0 + 3.0)
PATCH_T = 0.60

C_SKIN = "#b89e78"
C_VEIN = "#8c1f1a"
C_PALE = "#ddd0b4"
C_PATCH = "#d6c8aa"
C_THREAD = "#2a2119"
C_SPINE = "#14171a"
C_HARDWARE = "#0b0b0c"
C_AMBER = "#ffa01e"
C_ARC = "#cfe4ff"

random.seed(60606)


# ---------------------------------------------------------------------------
# Materials
# ---------------------------------------------------------------------------
def _sock(node, name, kind=None, out=False):
    socks = node.outputs if out else node.inputs
    for s in socks:
        if s.name == name and (kind is None or s.type == kind):
            return s
    raise KeyError("%s has no socket %s" % (node.bl_idname, name))


def _shade(hex_colour, k):
    c = sl.rgb(hex_colour)
    return (c[0] * k, c[1] * k, c[2] * k, 1.0)


class NT:
    """Small helper round one material's node tree."""

    def __init__(self, mat, bsdf):
        self.mat, self.bsdf = mat, bsdf
        self.nt = mat.node_tree
        self.coord = self.nt.nodes.new("ShaderNodeTexCoord")
        self.out = next(n for n in self.nt.nodes if n.bl_idname == "ShaderNodeOutputMaterial")

    def new(self, kind):
        return self.nt.nodes.new(kind)

    def link(self, a, b):
        self.nt.links.new(a, b)

    def obj(self):
        return self.coord.outputs["Object"]

    def put(self, sock, v):
        if isinstance(v, (int, float, tuple, list)):
            sock.default_value = v
        else:
            self.link(v, sock)

    def scaled(self, vec, scale):
        n = self.new("ShaderNodeMapping")
        n.inputs["Scale"].default_value = scale
        self.link(vec, n.inputs["Vector"])
        return n.outputs[0]

    def noise(self, scale, detail=3.0, rough=0.55, vec=None, colour=False):
        n = self.new("ShaderNodeTexNoise")
        n.inputs["Scale"].default_value = scale
        n.inputs["Detail"].default_value = detail
        n.inputs["Roughness"].default_value = rough
        self.link(vec or self.obj(), n.inputs["Vector"])
        return _sock(n, "Color", out=True) if colour else n.outputs[0]

    def warped(self, scale, amount, vec=None):
        add = self.new("ShaderNodeVectorMath")
        add.operation = "ADD"
        self.link(vec or self.obj(), add.inputs[0])
        sc = self.new("ShaderNodeVectorMath")
        sc.operation = "SCALE"
        sc.inputs["Scale"].default_value = amount
        self.link(self.noise(scale, 2.0, colour=True, vec=vec), sc.inputs[0])
        self.link(sc.outputs[0], add.inputs[1])
        return add.outputs[0]

    def voronoi_edge(self, scale, randomness=1.0, vec=None):
        n = self.new("ShaderNodeTexVoronoi")
        n.feature = "DISTANCE_TO_EDGE"
        n.inputs["Scale"].default_value = scale
        n.inputs["Randomness"].default_value = randomness
        self.link(vec or self.obj(), n.inputs["Vector"])
        return n.outputs["Distance"]

    def math(self, op, a, b=None, clamp=False):
        n = self.new("ShaderNodeMath")
        n.operation = op
        n.use_clamp = clamp
        for i, v in enumerate((a, b)):
            if v is not None:
                self.put(n.inputs[i], v)
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
        self.put(_sock(n, "A", "RGBA"), a)
        self.put(_sock(n, "B", "RGBA"), b)
        self.put(n.inputs[0], fac)
        return _sock(n, "Result", "RGBA", out=True)

    def bump(self, height, strength, distance):
        n = self.new("ShaderNodeBump")
        self.put(n.inputs["Strength"], strength)
        n.inputs["Distance"].default_value = distance
        self.link(height, n.inputs["Height"])
        self.link(n.outputs[0], self.bsdf.inputs["Normal"])
        return n.outputs[0]

    def attr(self, name):
        n = self.new("ShaderNodeAttribute")
        n.attribute_name = name
        sep = self.new("ShaderNodeSeparateColor")
        self.link(_sock(n, "Color", out=True), sep.inputs[0])
        return sep.outputs[0], sep.outputs[1], sep.outputs[2]

    def dot(self, vec, axis):
        n = self.new("ShaderNodeVectorMath")
        n.operation = "DOT_PRODUCT"
        self.link(vec, n.inputs[0])
        n.inputs[1].default_value = axis
        return n.outputs["Value"]

    def mix_shader(self, fac, a, b):
        n = self.new("ShaderNodeMixShader")
        self.put(n.inputs[0], fac)
        self.link(a, n.inputs[1])
        self.link(b, n.inputs[2])
        return n.outputs[0]

    def surface(self, shader):
        self.link(shader, self.out.inputs["Surface"])


def skin_material(name, marks=(), dead=False):
    """Thin stretched membrane. Part plain surface, part light passing
    through (translucent), part truly see-through.

    marks: stretch marks as (centre, long axis, cross axis, normal), all in
    the sac's own coordinates.
    """
    t = NT(*sl.new_material(name, C_SKIN, roughness=0.4))
    if dead:
        tight, hfrac = 0.0, 0.3
        low = 0.7
    else:
        tight, _, hfrac = t.attr("sacdata")
        low = t.math("SUBTRACT", 1.0, hfrac)

    # mottled amber-grey
    mottle = t.remap(t.noise(7.0, 4.0), 0.3, 0.72)
    cloudy = t.remap(t.noise(2.4, 3.0, 0.6), 0.38, 0.66)
    k0, k1 = (0.50, 0.85) if dead else (0.70, 1.05)
    base = t.mix(mottle, _shade(C_SKIN, k0), _shade(C_SKIN, k1))
    # denser and a little darker toward the bottom
    if not dead:
        base = t.mix(t.remap(hfrac, 0.0, 0.45, 0.35, 0.0), base, _shade("#7c6248", 1.0))

    # fine blood vessels, a net of hair-thin red lines, stronger low down
    net = t.voronoi_edge(19.0, vec=t.warped(5.0, 0.10))
    net2 = t.voronoi_edge(52.0, vec=t.warped(11.0, 0.05))
    line = t.math("MAXIMUM", t.remap(net, 0.0, 0.010, 1.0, 0.0),
                  t.math("MULTIPLY", t.remap(net2, 0.0, 0.012, 1.0, 0.0), 0.5))
    if dead:
        cap = t.math("MULTIPLY", line, 0.55)
    else:
        cap = t.math("MULTIPLY", line, t.remap(hfrac, 0.0, 1.0, 0.75, 0.3))
    base = t.mix(cap, base, sl.rgb(C_VEIN))

    # stretch marks: pale streaked patches
    mark = None
    for centre, long_axis, cross_axis, normal in marks:
        d = t.new("ShaderNodeVectorMath")
        d.operation = "SUBTRACT"
        t.link(t.warped(9.0, 0.03), d.inputs[0])
        d.inputs[1].default_value = centre
        a = t.math("DIVIDE", t.dot(d.outputs[0], long_axis), 0.21)
        b_raw = t.dot(d.outputs[0], cross_axis)
        b = t.math("DIVIDE", b_raw, 0.060)
        c = t.math("DIVIDE", t.dot(d.outputs[0], normal), 0.08)
        rr = t.math("ADD", t.math("ADD", t.math("POWER", a, 2.0), t.math("POWER", b, 2.0)),
                    t.math("POWER", c, 2.0))
        inside = t.remap(rr, 0.35, 1.0, 1.0, 0.0)
        streak = t.remap(t.math("SINE", t.math("MULTIPLY", b_raw, 2.0 * math.pi / 0.021)),
                         -0.5, 0.6, 0.15, 1.0)
        m = t.math("MULTIPLY", inside, streak)
        mark = m if mark is None else t.math("MAXIMUM", mark, m)
    if mark is not None:
        base = t.mix(t.math("MULTIPLY", mark, 0.8), base, sl.rgb(C_PALE))
    t.link(base, t.bsdf.inputs["Base Color"])

    # sheen where the skin is stretched tightest (between the ribs)
    if dead:
        t.link(t.remap(cloudy, 0.0, 1.0, 0.42, 0.66), t.bsdf.inputs["Roughness"])
        t.bsdf.inputs["Coat Weight"].default_value = 0.0
        t.bsdf.inputs["Coat Roughness"].default_value = 0.2
        crease_strength = 0.9
    else:
        t.link(t.remap(tight, 0.1, 1.0, 0.60, 0.34), t.bsdf.inputs["Roughness"])
        t.link(t.remap(tight, 0.4, 1.0, 0.0, 0.22), t.bsdf.inputs["Coat Weight"])
        t.bsdf.inputs["Coat Roughness"].default_value = 0.22
        crease_strength = t.remap(tight, 0.0, 0.8, 1.0, 0.15)
    # small creases beside the ribs, where the skin is slack
    crease = t.noise(55.0, 2.0, vec=t.scaled(t.obj(), (1.0, 1.0, 0.12)))
    grain = t.noise(260.0, 2.0)
    normal = t.bump(t.math("ADD", crease, t.math("MULTIPLY", grain, 0.25)), crease_strength,
                    0.012 if dead else 0.004)

    # light that comes through the skin from the far side
    through = t.new("ShaderNodeBsdfTranslucent")
    t.link(t.mix(cap, _shade("#c9ac80", 0.75 if dead else 1.1), _shade(C_VEIN, 1.3)),
           through.inputs["Color"])
    t.link(normal, through.inputs["Normal"])
    body = t.mix_shader(0.40 if dead else 0.46, t.bsdf.outputs[0], through.outputs[0])

    # how much you see straight through it: most when facing, least at the edge
    lw = t.new("ShaderNodeLayerWeight")
    lw.inputs["Blend"].default_value = 0.42
    edge = lw.outputs["Facing"]
    if dead:
        see = t.math("MULTIPLY", t.remap(edge, 0.0, 1.0, 0.50, 0.06),
                     t.remap(cloudy, 0.0, 1.0, 0.45, 1.2))
    else:
        see = t.math("MULTIPLY", t.remap(edge, 0.1, 0.9, 0.80, 0.10),
                     t.remap(hfrac, 0.0, 0.7, 0.66, 1.0))
        see = t.math("MULTIPLY", see, t.remap(cloudy, 0.0, 1.0, 0.62, 1.08))
    see = t.math("MULTIPLY", see, t.remap(cap, 0.0, 1.0, 1.0, 0.25))
    if mark is not None:
        see = t.math("ADD", see, t.math("MULTIPLY", mark, 0.10), clamp=True)
    clear = t.new("ShaderNodeBsdfTransparent")
    clear.inputs["Color"].default_value = (0.96, 0.86, 0.68, 1.0)
    t.surface(t.mix_shader(see, body, clear.outputs[0]))
    return t.mat


def make_materials(marks):
    m = {}
    m["skin"] = skin_material("bloat_skin", marks)
    m["skin_dead"] = skin_material("bloat_skin_dead", dead=True)

    # the larger veins (thin strips lying on the skin)
    for key, k in (("vein", 1.0), ("vein_dead", 0.6)):
        t = NT(*sl.new_material("bloat_" + key, C_VEIN, roughness=0.42))
        t.bsdf.inputs["Base Color"].default_value = _shade(C_VEIN, k)
        through = t.new("ShaderNodeBsdfTranslucent")
        through.inputs["Color"].default_value = _shade(C_VEIN, 1.2 * k)
        t.surface(t.mix_shader(0.3, t.bsdf.outputs[0], through.outputs[0]))
        m[key] = t.mat

    # feelers: same skin, not see-through, darker at the tips
    t = NT(*sl.new_material("bloat_feeler", C_SKIN, roughness=0.5))
    along, _, _ = t.attr("along")
    mottle = t.remap(t.noise(40.0, 3.0), 0.3, 0.7, 0.8, 1.05)
    lit = t.mix(mottle, _shade(C_SKIN, 0.36), _shade(C_SKIN, 0.6))
    tip = t.remap(along, 0.15, 1.0)
    t.link(t.mix(tip, lit, _shade("#3a2a1e", 1.0)), t.bsdf.inputs["Base Color"])
    ringlets = t.noise(140.0, 1.0, vec=t.scaled(t.obj(), (0.2, 0.2, 1.0)))
    t.bump(ringlets, 0.5, 0.002)
    through = t.new("ShaderNodeBsdfTranslucent")
    t.link(t.mix(tip, _shade("#d9b27a", 1.0), _shade("#3a2a1e", 1.0)), through.inputs["Color"])
    t.surface(t.mix_shader(0.28, t.bsdf.outputs[0], through.outputs[0]))
    m["feeler"] = t.mat

    # the wrong repair: thick, pale, opaque, leathery
    t = NT(*sl.new_material("bloat_patch", C_PATCH, roughness=0.92))
    t.bsdf.inputs["Specular IOR Level"].default_value = 0.15
    blot = t.remap(t.noise(18.0, 4.0), 0.3, 0.75)
    t.link(t.mix(blot, _shade(C_PATCH, 0.40), _shade(C_PATCH, 0.62)), t.bsdf.inputs["Base Color"])
    pores = t.voronoi_edge(160.0)
    t.bump(t.math("ADD", t.remap(pores, 0.0, 0.08), t.noise(30.0, 3.0)), 1.0, 0.006)
    m["patch"] = t.mat
    m["thread"] = sl.new_material("bloat_thread", C_THREAD, roughness=0.85)[0]

    # the frame under the flesh: regular lattice with 6 cm cells
    t = NT(*sl.new_material("SPINE", C_SPINE, roughness=0.3))
    cell = t.voronoi_edge(1.0 / CELL, randomness=0.0)
    strut = t.remap(cell, 0.0, 0.16, 1.0, 0.0)
    t.link(t.mix(strut, _shade(C_SPINE, 0.7), _shade(C_SPINE, 2.2)), t.bsdf.inputs["Base Color"])
    t.bump(strut, 1.0, 0.004)
    m["spine"] = t.mat

    # Registrar hardware
    m["hardware"] = sl.new_material("registrar_hardware", C_HARDWARE, roughness=0.35,
                                    metallic=1.0)[0]
    m["amber"] = sl.new_material("registrar_amber", "#3a2206", roughness=0.25,
                                 emission_hex=C_AMBER, emission_strength=6.0)[0]
    m["amber_dead"] = sl.new_material("registrar_amber_dead", "#1c1206", roughness=0.25)[0]

    # the murk inside: a faint cloud, denser toward the bottom
    mat = bpy.data.materials.new("bloat_murk")
    nt = sl._node_tree_of(mat)
    for n in list(nt.nodes):
        if n.bl_idname != "ShaderNodeOutputMaterial":
            nt.nodes.remove(n)
    out = next(n for n in nt.nodes if n.bl_idname == "ShaderNodeOutputMaterial")
    coord = nt.nodes.new("ShaderNodeTexCoord")
    sep = nt.nodes.new("ShaderNodeSeparateXYZ")
    nt.links.new(coord.outputs["Object"], sep.inputs[0])
    grad = nt.nodes.new("ShaderNodeMapRange")
    grad.inputs[1].default_value = -ZB
    grad.inputs[2].default_value = ZT * 0.7
    grad.inputs[3].default_value = 1.0
    grad.inputs[4].default_value = 0.0
    nt.links.new(sep.outputs[2], grad.inputs[0])
    sq = nt.nodes.new("ShaderNodeMath")
    sq.operation = "POWER"
    nt.links.new(grad.outputs[0], sq.inputs[0])
    sq.inputs[1].default_value = 1.8
    cloud = nt.nodes.new("ShaderNodeTexNoise")
    cloud.inputs["Scale"].default_value = 2.6
    cloud.inputs["Detail"].default_value = 4.0
    nt.links.new(coord.outputs["Object"], cloud.inputs["Vector"])
    cl = nt.nodes.new("ShaderNodeMapRange")
    cl.inputs[1].default_value = 0.3
    cl.inputs[2].default_value = 0.75
    cl.inputs[3].default_value = 0.05
    cl.inputs[4].default_value = 1.9
    nt.links.new(cloud.outputs[0], cl.inputs[0])
    dens = nt.nodes.new("ShaderNodeMath")
    dens.operation = "MULTIPLY"
    nt.links.new(sq.outputs[0], dens.inputs[0])
    nt.links.new(cl.outputs[0], dens.inputs[1])
    dens2 = nt.nodes.new("ShaderNodeMath")
    dens2.operation = "MULTIPLY_ADD"
    nt.links.new(dens.outputs[0], dens2.inputs[0])
    dens2.inputs[1].default_value = 5.5
    dens2.inputs[2].default_value = 0.12
    vol = nt.nodes.new("ShaderNodeVolumePrincipled")
    vol.inputs["Color"].default_value = (0.78, 0.70, 0.56, 1.0)
    vol.inputs["Anisotropy"].default_value = 0.3
    nt.links.new(dens2.outputs[0], vol.inputs["Density"])
    nt.links.new(vol.outputs[0], out.inputs["Volume"])
    m["murk"] = mat

    # damp stain under the burst form
    t = NT(*sl.new_material("bloat_stain", sl.SOIL_HEX, roughness=0.3))
    wet, _, _ = t.attr("wet")
    blot = t.noise(2.2, 5.0, 0.65)
    amount = t.remap(t.math("ADD", t.math("MULTIPLY", wet, 1.9), t.math("MULTIPLY", blot, 1.3)),
                     0.80, 1.30)
    t.bsdf.inputs["Base Color"].default_value = _shade(sl.SOIL_HEX, 0.30)
    t.link(t.remap(t.noise(9.0, 3.0), 0.3, 0.7, 0.12, 0.4), t.bsdf.inputs["Roughness"])
    clear = t.new("ShaderNodeBsdfTransparent")
    t.surface(t.mix_shader(t.math("MULTIPLY", amount, 0.93), clear.outputs[0], t.bsdf.outputs[0]))
    m["stain"] = t.mat

    # electrical life: night picture only
    m["arc"] = sl.new_material("electrical_life", "#0a0d12", roughness=0.5,
                               emission_hex=C_ARC, emission_strength=4.0)[0]
    return m


# ---------------------------------------------------------------------------
# The sac's surface as a formula
# ---------------------------------------------------------------------------
LOBE = [random.uniform(-0.020, 0.022) for _ in range(N_RIBS)]
SCALE = [1.0]


def profile(t):
    """Radius and height (above the widest plane) at t: 0 bottom, 1 top."""
    th = math.pi * min(max(t, 0.0), 1.0)
    s = max(math.sin(th), 0.0)
    if th <= math.pi / 2.0:
        return R * s ** 0.88, -ZB * math.cos(th)
    return R * s ** 0.80, -ZT * math.cos(th)


T_RING = 0.5      # the widest plane


def groove_at(phi):
    c = abs(math.cos(N_RIBS / 2.0 * (phi - PHI0)))
    return 0.68 * c ** 12 + 0.32 * c ** 2.5


def surf(phi, t, inset=0.0):
    r, z = profile(t)
    g = groove_at(phi)
    k = int(math.floor((phi - PHI0) / (2.0 * math.pi / N_RIBS))) % N_RIBS
    ring = (0.7 * math.exp(-(z / 0.035) ** 2) + 0.3 * math.exp(-(z / 0.12) ** 2))
    f = 1.0 - GROOVE * g + LOBE[k] * (1.0 - g) - RING_GROOVE * ring * (1.0 - 0.6 * g)
    f += 0.012 * noise.noise(Vector((math.cos(phi) * 1.3, math.sin(phi) * 1.3, z * 1.4 + 7.0)))
    r *= SCALE[0] * f
    p = Vector((r * math.cos(phi), r * math.sin(phi), z))
    if inset:
        p -= surf_normal(phi, t) * inset
    return p


def surf_frame(phi, t):
    """Tangent along phi, tangent toward the top, outward normal."""
    e = 1e-3
    t0, t1 = max(t - e, 0.002), min(t + e, 0.998)
    ea = surf(phi + e, t) - surf(phi - e, t)
    eb = surf(phi, t1) - surf(phi, t0)
    la, lb = ea.length / (2 * e), eb.length / (t1 - t0)
    n = ea.cross(eb)
    if n.length < 1e-12:
        n = Vector((0, 0, -1 if t < 0.5 else 1))
    return ea.normalized(), eb.normalized(), n.normalized(), la, lb


def surf_normal(phi, t):
    return surf_frame(phi, t)[2]


# ---------------------------------------------------------------------------
# Mesh tools
# ---------------------------------------------------------------------------
def make_obj(name, verts, faces, mat, coll, parent=None, location=(0, 0, 0), smooth=True,
             attrs=None):
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(v) for v in verts], [], faces)
    me.update()
    if smooth:
        me.polygons.foreach_set("use_smooth", [True] * len(me.polygons))
    for key, values in (attrs or {}).items():
        a = me.color_attributes.new(key, "FLOAT_COLOR", "POINT")
        for i, v in enumerate(values):
            a.data[i].color = (v[0], v[1], v[2], 1.0)
    if mat is not None:
        me.materials.append(mat)
    ob = bpy.data.objects.new(name, me)
    coll.objects.link(ob)
    if parent is not None:
        ob.parent = parent
    ob.location = location
    return ob


def make_empty(name, coll, parent=None, location=(0, 0, 0)):
    ob = bpy.data.objects.new(name, None)
    ob.empty_display_size = 0.2
    coll.objects.link(ob)
    if parent is not None:
        ob.parent = parent
    ob.location = location
    return ob


class Geo:
    """Collects vertices and faces for one mesh."""

    def __init__(self):
        self.v, self.f, self.a = [], [], []

    def tube(self, pts, ra, rb=None, sides=6, normals=None, along=None, cap=True):
        """Tube along pts. ra, rb: half sizes along the normal and across it
        (numbers or one per point)."""
        n = len(pts)
        get = lambda x, i: x[i] if isinstance(x, (list, tuple)) else x
        rb = ra if rb is None else rb
        base = len(self.v)
        prev = None
        for i in range(n):
            tan = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized()
            if normals is not None:
                nor = normals[i]
            elif prev is None:
                nor = tan.orthogonal()
            else:
                nor = prev
            nor = (nor - tan * nor.dot(tan)).normalized()
            prev = nor
            side = tan.cross(nor)
            for j in range(sides):
                a = 2.0 * math.pi * j / sides
                self.v.append(pts[i] + nor * (math.cos(a) * get(ra, i)) + side * (math.sin(a) * get(rb, i)))
                self.a.append(((along[i] if along else i / max(n - 1, 1)), 0.0, 0.0))
        for i in range(n - 1):
            for j in range(sides):
                a0 = base + i * sides + j
                a1 = base + i * sides + (j + 1) % sides
                self.f.append((a0, a1, a1 + sides, a0 + sides))
        if cap:
            self.f.append(tuple(base + j for j in reversed(range(sides))))
            self.f.append(tuple(base + (n - 1) * sides + j for j in range(sides)))

    def ribbon(self, pts, normals, widths, lift):
        base = len(self.v)
        n = len(pts)
        for i in range(n):
            tan = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)])
            side = normals[i].cross(tan)
            if side.length < 1e-9:
                side = normals[i].orthogonal()
            side.normalize()
            c = pts[i] + normals[i] * lift
            self.v.append(c - side * widths[i] / 2.0)
            self.v.append(c + side * widths[i] / 2.0)
            self.a.append((0, 0, 0))
            self.a.append((0, 0, 0))
        for i in range(n - 1):
            a = base + 2 * i
            self.f.append((a, a + 1, a + 3, a + 2))

    def box(self, centre, x, y, z, hx, hy, hz):
        base = len(self.v)
        for sx in (-1, 1):
            for sy in (-1, 1):
                for sz in (-1, 1):
                    self.v.append(centre + x * (sx * hx) + y * (sy * hy) + z * (sz * hz))
                    self.a.append((0, 0, 0))
        for q in ((0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)):
            self.f.append(tuple(base + i for i in q))

    def tris(self):
        return sum(len(f) - 2 for f in self.f)


def grow_veins(point, frame, starts, budget, stop, b_lo, b_hi, step_max=0.034, seed=1):
    """Branching veins grown over a surface given as point(a, b).

    Each vein walks roughly toward larger b, wanders, thins, and now and then
    throws off a thinner branch. Thickest veins are grown first, until the
    budget of segments is used up. Returns a list of (points, normals, widths).
    """
    rnd = random.Random(seed)
    heap = []
    count = 0
    for a, b, w in starts:
        heapq.heappush(heap, (-w, count, a, b, rnd.uniform(-0.3, 0.3), 0.0, w))
        count += 1
    out, used = [], 0
    while heap and used < budget:
        _, _, a, b, h, home, w = heapq.heappop(heap)
        pts, nors, wid = [], [], []
        for _ in range(260):
            ea, eb, n, la, lb = frame(a, b)
            p = point(a, b)
            pts.append(p)
            nors.append(n)
            wid.append(w)
            if w < 0.0017 or not (b_lo < b < b_hi) or stop(p) or used >= budget:
                break
            step = max(0.014, min(step_max, w * 6.0))
            h += (home - h) * 0.06 + rnd.gauss(0.0, 0.20)
            a += step * math.sin(h) / max(la, 0.05)
            b += step * math.cos(h) / max(lb, 0.05)
            w *= 0.990
            used += 1
            if w > 0.0026 and rnd.random() < 0.085 + 0.05 * (w > 0.005):
                turn = rnd.choice((-1, 1)) * rnd.uniform(0.55, 1.15)
                heapq.heappush(heap, (-w * 0.6, count, a, b, h + turn, (h + turn) * 0.7,
                                      w * rnd.uniform(0.52, 0.70)))
                count += 1
                w *= 0.90
                h -= turn * 0.18
        if len(pts) > 1:
            wid[-1] = wid[-1] * 0.4
            out.append((pts, nors, wid))
    return out


# ---------------------------------------------------------------------------
# The living Bloat
# ---------------------------------------------------------------------------
def patch_frame():
    ea, eb, n, la, lb = surf_frame(PATCH_PHI, PATCH_T)
    return surf(PATCH_PHI, PATCH_T), ea, eb, n, la, lb


def patch_outline(psi):
    """Half size of the rough oval at angle psi (0 = along phi, 90 deg = up)."""
    a, b = PATCH_D / 2.0, PATCH_D / 2.0 * 0.66
    q = psi - math.radians(28.0)          # the oval sits askew
    base = (a * b) / math.sqrt((b * math.cos(q)) ** 2 + (a * math.sin(q)) ** 2)
    rough = 0.09 * noise.noise(Vector((math.cos(psi) * 1.9, math.sin(psi) * 1.9, 3.3)))
    return base * (0.93 + rough)


def build_sac(coll, mats, root):
    seg_phi, seg_t = 160, 50
    # find the scale that makes the sac exactly as wide as the sheet says
    SCALE[0] = 1.0
    xs, ys = [], []
    for i in range(seg_phi):
        for j in range(1, seg_t):
            p = surf(2.0 * math.pi * i / seg_phi, j / seg_t)
            xs.append(p.x)
            ys.append(p.y)
    SCALE[0] = DIAMETER / max(max(xs) - min(xs), max(ys) - min(ys))

    verts, faces, data = [], [], []
    verts.append(surf(0.0, 0.0))
    data.append((0.0, 0.0, 0.0))
    for j in range(1, seg_t):
        t = j / seg_t
        for i in range(seg_phi):
            phi = 2.0 * math.pi * i / seg_phi
            verts.append(surf(phi, t))
            tight = (1.0 - groove_at(phi) ** 0.5) * math.sin(math.pi * t) ** 0.7
            data.append((tight, 0.0, t))
    verts.append(surf(0.0, 1.0))
    data.append((0.0, 0.0, 1.0))
    top = len(verts) - 1
    row = lambda j: 1 + (j - 1) * seg_phi
    for i in range(seg_phi):
        i1 = (i + 1) % seg_phi
        faces.append((0, row(1) + i1, row(1) + i))
        for j in range(1, seg_t - 1):
            faces.append((row(j) + i, row(j) + i1, row(j + 1) + i1, row(j + 1) + i))
        faces.append((top, row(seg_t - 1) + i, row(seg_t - 1) + i1))
    sac = make_obj("sac_skin", verts, faces, mats["skin"], coll, root,
                   (0.0, 0.0, UNDERSIDE + ZB), attrs={"sacdata": data})
    return sac


def build_frame(coll, mats, sac):
    g = Geo()
    n = 46
    for k in range(N_RIBS):
        phi = PHI0 + k * 2.0 * math.pi / N_RIBS
        pts, nors, ra, rb = [], [], [], []
        for i in range(n + 1):
            t = 0.012 + (0.992 - 0.012) * i / n
            nor = surf_normal(phi, t)
            pts.append(surf(phi, t) - nor * (RIB_T / 2.0 + 0.004))
            nors.append(nor)
            taper = 0.62 + 0.38 * math.sin(math.pi * t) ** 0.6
            ra.append(RIB_T / 2.0 * taper)
            rb.append(RIB_W / 2.0 * taper)
        g.tube(pts, ra, rb, sides=6, normals=nors)
    # the one ring round the widest part
    pts, nors = [], []
    m = 112
    for i in range(m):
        phi = 2.0 * math.pi * i / m
        nor = surf_normal(phi, T_RING)
        pts.append(surf(phi, T_RING) - nor * (RIB_T / 2.0 + 0.004 + 0.006 * groove_at(phi)))
        nors.append(nor)
    base = len(g.v)
    g.tube(pts + [pts[0]], RIB_T / 2.0, RIB_W / 2.0 * 0.9, sides=6, normals=nors + [nors[0]], cap=False)
    # small hub where the ribs meet at the bottom
    hub = [Vector((0, 0, -ZB + 0.012)), Vector((0, 0, -ZB + 0.040))]
    g.tube(hub, 0.028, sides=8)
    return make_obj("frame_ribs", g.v, g.f, mats["spine"], coll, sac)


def build_murk(coll, mats, sac):
    seg_phi, seg_t = 24, 14
    verts, faces = [Vector((0, 0, -ZB + 0.03))], []
    for j in range(1, seg_t):
        for i in range(seg_phi):
            p = surf(2.0 * math.pi * i / seg_phi, j / seg_t)
            verts.append(Vector((p.x * 0.955, p.y * 0.955, p.z * 0.965)))
    verts.append(Vector((0, 0, ZT - 0.05)))
    top = len(verts) - 1
    row = lambda j: 1 + (j - 1) * seg_phi
    for i in range(seg_phi):
        i1 = (i + 1) % seg_phi
        faces.append((0, row(1) + i1, row(1) + i))
        for j in range(1, seg_t - 1):
            faces.append((row(j) + i, row(j) + i1, row(j + 1) + i1, row(j + 1) + i))
        faces.append((top, row(seg_t - 1) + i, row(seg_t - 1) + i1))
    return make_obj("murk_inside", verts, faces, mats["murk"], coll, sac)


def build_veins(coll, mats, sac):
    centre, pea, peb, pn, pla, plb = patch_frame()

    def stop(p):
        return (p - centre).length < PATCH_D / 2.0 * 1.02

    starts = []
    for i in range(12):
        starts.append((2.0 * math.pi * (i + random.uniform(-0.3, 0.3)) / 12.0,
                       random.uniform(0.045, 0.075), random.uniform(0.014, 0.020)))
    veins = grow_veins(surf, surf_frame, starts, 4600, stop, 0.03, 0.965, seed=11)
    g = Geo()
    for pts, nors, wid in veins:
        g.ribbon(pts, nors, wid, 0.0012)
    return make_obj("veins", g.v, g.f, mats["vein"], coll, sac)


def build_patch(coll, mats, sac):
    centre, ea, eb, n, la, lb = patch_frame()

    def on_skin(u, v, lift):
        phi, t = PATCH_PHI + u / la, PATCH_T + v / lb
        return surf(phi, t) + surf_normal(phi, t) * lift - centre

    seg, rings = 30, [0.0, 0.3, 0.58, 0.8, 0.93, 1.0, 1.04]
    lifts = [0.010, 0.012, 0.012, 0.011, 0.014, 0.009, -0.002]
    verts, faces = [on_skin(0, 0, lifts[0])], []
    for ri in range(1, len(rings)):
        for i in range(seg):
            psi = 2.0 * math.pi * i / seg
            rad = patch_outline(psi) * rings[ri]
            pucker = 0.006 * math.sin(psi * 13.0 + 2.0 * math.sin(psi * 3.0)) * (rings[ri] > 0.75)
            verts.append(on_skin(rad * math.cos(psi), rad * math.sin(psi), lifts[ri] + pucker))
    row = lambda r: 1 + (r - 1) * seg
    for i in range(seg):
        i1 = (i + 1) % seg
        faces.append((0, row(1) + i, row(1) + i1))
        for r in range(1, len(rings) - 1):
            faces.append((row(r) + i, row(r + 1) + i, row(r + 1) + i1, row(r) + i1))
    patch = make_obj("wrong_repair_patch", verts, faces, mats["patch"], coll, sac, tuple(centre))

    # the seam: uneven dark stitches across the edge
    g = Geo()
    count = 30
    for i in range(count):
        psi = 2.0 * math.pi * (i + random.uniform(-0.28, 0.28)) / count
        rad = patch_outline(psi)
        skew = 0.95 + random.uniform(-0.22, 0.22)     # slanted, like a whip stitch
        half = random.uniform(0.012, 0.017)
        d = Vector((math.cos(psi + skew), math.sin(psi + skew)))
        mid = Vector((math.cos(psi), math.sin(psi))) * (rad * 1.0)
        a = on_skin(mid.x - d.x * half, mid.y - d.y * half, 0.0125)
        c = on_skin(mid.x, mid.y, 0.0150)
        b = on_skin(mid.x + d.x * half, mid.y + d.y * half, 0.0035)
        g.tube([a, c, b], 0.0026, sides=4)
    make_obj("wrong_repair_seam", g.v, g.f, mats["thread"], coll, patch, smooth=False)
    return patch


def feeler_path(k, rnd):
    """Limp feeler: leaves the skin, falls straight down, sways a little.
    Returned in the feeler's own coordinates (root at 0). Exactly FEELER_L long."""
    phi = 2.0 * math.pi * (k + 0.35) / N_FEELERS
    out = Vector((math.cos(phi), math.sin(phi), 0.0))
    sidev = Vector((-math.sin(phi), math.cos(phi), 0.0))
    amp1, amp2 = rnd.uniform(0.015, 0.05), rnd.uniform(-0.05, 0.05)
    ph, curl = rnd.uniform(0, 6.28), rnd.uniform(-1.0, 1.0)
    trail = Vector((rnd.uniform(-0.08, 0.08), 1.0, 0.0)) * rnd.uniform(0.10, 0.34)
    n = 30
    step = FEELER_L / n
    pts, p = [Vector((0, 0, 0))], Vector((0, 0, 0))
    for i in range(n):
        s = (i + 0.5) / n
        d = Vector((0, 0, -1.0)) + out * (0.55 * math.exp(-s / 0.07))
        d += out * (amp1 * 4.0 * math.cos(s * 5.5 + ph)) + sidev * (amp2 * 5.0 * math.sin(s * 4.2 + ph))
        d += trail * s ** 0.7
        d += (out * curl + sidev * (1 - abs(curl))) * (0.9 * max(0.0, s - 0.78) ** 1.3 * 4.0)
        p = p + d.normalized() * step
        pts.append(p.copy())
    return pts


def feeler_radius(s):
    return 0.0038 + 0.0062 * (1.0 - s) ** 1.4 + 0.010 * math.exp(-s / 0.035)


def build_feelers(coll, mats, sac):
    rnd = random.Random(5)
    out = []
    for k in range(N_FEELERS):
        phi = 2.0 * math.pi * (k + 0.35) / N_FEELERS
        t_root = 0.040 + 0.055 * rnd.random()
        root = surf(phi, t_root) + surf_normal(phi, t_root) * -0.004
        pts = feeler_path(k, rnd)
        g = Geo()
        radii = [feeler_radius(i / (len(pts) - 1)) for i in range(len(pts))]
        g.tube(pts, radii, sides=7)
        out.append(make_obj("feeler_%d" % (k + 1), g.v, g.f, mats["feeler"], coll, sac,
                            tuple(root), attrs={"along": g.a}))
    return out


def plug_geo(lens_gap=True):
    """The black plug: a twelve-sided block with exact flat faces."""
    g = Geo()
    sides = 12
    r = PLUG_D / 2.0
    rows = [(r * 0.86, -0.014), (r, -0.010), (r, 0.020), (r * 0.92, 0.024), (r * 0.44, 0.024),
            (r * 0.44, 0.020)]
    for rr, z in rows:
        for j in range(sides):
            a = 2.0 * math.pi * (j + 0.5) / sides
            g.v.append(Vector((rr * math.cos(a), rr * math.sin(a), z)))
    for i in range(len(rows) - 1):
        for j in range(sides):
            a0, a1 = i * sides + j, i * sides + (j + 1) % sides
            g.f.append((a0, a1, a1 + sides, a0 + sides))
    g.f.append(tuple(reversed(range(sides))))
    return g, r * 0.44


def lens_geo(r):
    g = Geo()
    sides = 12
    for z in (0.019, 0.0225):
        for j in range(sides):
            a = 2.0 * math.pi * (j + 0.5) / sides
            g.v.append(Vector((r * math.cos(a), r * math.sin(a), z)))
    for j in range(sides):
        g.f.append((j, (j + 1) % sides, (j + 1) % sides + sides, j + sides))
    g.f.append(tuple(range(sides, 2 * sides)))
    return g


def build_plug(coll, mats, parent, location, dead=False, prefix=""):
    g, lens_r = plug_geo()
    plug = make_obj(prefix + "control_plug", g.v, g.f, mats["hardware"], coll, parent, location,
                    smooth=False)
    lg = lens_geo(lens_r)
    make_obj(prefix + "control_lens", lg.v, lg.f, mats["amber_dead" if dead else "amber"], coll,
             plug, smooth=False)
    return plug


def build_arcs(coll, mats, sac):
    """Electrical life: a broken, jagged flicker along two ribs inside."""
    rnd = random.Random(21)
    g = Geo()
    for k in (5, 7):       # the two ribs that face the three-quarter camera
        phi = PHI0 + k * 2.0 * math.pi / N_RIBS
        t = rnd.uniform(0.10, 0.16)
        while t < 0.86:
            length = rnd.uniform(0.10, 0.24)
            pts = []
            steps = rnd.randint(9, 16)
            for i in range(steps + 1):
                tt = t + length * i / steps
                ea, eb, nor, la, lb = surf_frame(phi, tt)
                p = surf(phi, tt) - nor * (RIB_T + 0.010 + rnd.uniform(0.0, 0.008))
                p += ea * rnd.uniform(-0.007, 0.007)
                pts.append(p)
            g.tube(pts, 0.0008, sides=3)
            t += length + rnd.uniform(0.03, 0.10)
    arcs = make_obj("rib_arcs", g.v, g.f, mats["arc"], coll, sac, smooth=False)
    arcs.hide_render = True
    return arcs


def stretch_marks():
    out = []
    for phi_deg, t, tilt_deg in ((-128.0, 0.66, 18.0), (-47.0, 0.33, -24.0), (52.0, 0.57, 12.0)):
        phi = math.radians(phi_deg)
        ea, eb, n, la, lb = surf_frame(phi, t)
        c, s = math.cos(math.radians(tilt_deg)), math.sin(math.radians(tilt_deg))
        long_axis = (eb * c + ea * s).normalized()
        cross_axis = n.cross(long_axis).normalized()
        out.append((tuple(surf(phi, t)), tuple(long_axis), tuple(cross_axis), tuple(n)))
    return out


# ---------------------------------------------------------------------------
# The dead form: burst, on the ground
# ---------------------------------------------------------------------------
class Wreck:
    def __init__(self):
        rnd = random.Random(77)
        self.rnd = rnd
        self.dir = Vector((math.cos(math.radians(205.0)), math.sin(math.radians(205.0)), 0.0))
        self.shear = 0.20
        arches = [0.24, 0.03, 0.15, 0.02, 0.07, 0.21, 0.03, 0.11]
        self.arch = arches
        self.bow = [Vector((rnd.uniform(-0.16, 0.16), rnd.uniform(-0.16, 0.16), 0.0))
                    for _ in range(N_RIBS)]
        self.snapped = {2: 0.44, 5: 0.60}
        self.top = self.dir * (self.shear * SAC_H)
        self.centre = self.top * 0.62
        self.samples = []       # (x, y, z) points the skin is draped over

    def fold(self, p, t, arch, bow, kink=None):
        """Where a point of the living frame ends up after the collapse."""
        zr = p.z + ZB
        s = math.sin(math.pi * t)
        q = Vector((p.x * 0.90, p.y * 0.90, 0.0)) + self.dir * (self.shear * zr) + bow * s
        z = 0.02 * zr + arch * s ** 1.4
        if kink is not None:
            z *= 1.0 - 0.85 * math.exp(-((t - kink) / 0.07) ** 2)
        q.z = max(z, RIB_T / 2.0)
        return q

    def rib(self, k, n=40):
        phi = PHI0 + k * 2.0 * math.pi / N_RIBS
        pts = []
        for i in range(n + 1):
            t = 0.012 + 0.98 * i / n
            pts.append(self.fold(surf(phi, t, inset=RIB_T), t, self.arch[k], self.bow[k],
                                 self.snapped.get(k)))
        return pts

    def ring(self, n=96):
        pts = []
        for i in range(n):
            phi = 2.0 * math.pi * i / n
            x = (phi - PHI0) / (2.0 * math.pi / N_RIBS)
            k0 = int(math.floor(x)) % N_RIBS
            k1 = (k0 + 1) % N_RIBS
            w = x - math.floor(x)
            w = w * w * (3 - 2 * w)
            arch = self.arch[k0] * (1 - w) + self.arch[k1] * w
            bow = self.bow[k0] * (1 - w) + self.bow[k1] * w
            k_near = k0 if w < 0.5 else k1
            kink = self.snapped.get(k_near) if abs(w - 0.5) > 0.42 else None
            pts.append(self.fold(surf(phi, T_RING, inset=RIB_T), T_RING, arch, bow, kink))
        return pts


def build_dead(coll, mats):
    import numpy as np
    w = Wreck()
    rnd = w.rnd
    droot = make_empty("dead_" + NAME, coll)

    # bent ribs and the ring
    g = Geo()
    for k in range(N_RIBS):
        pts = w.rib(k)
        g.tube(pts, RIB_T / 2.0, RIB_W / 2.0, sides=6)
        w.samples += pts
    ring = w.ring()
    g.tube(ring + [ring[0]], RIB_T / 2.0, RIB_W / 2.0 * 0.9, sides=6, cap=False)
    w.samples += ring
    make_obj("dead_frame_ribs", g.v, g.f, mats["spine"], coll, droot)

    # feelers flat on the ground, fanned out on the side away from the fall
    back = math.atan2(-w.dir.y, -w.dir.x)
    feelers = []
    for k in range(N_FEELERS):
        ang = back + math.radians(-78.0 + 39.0 * k + rnd.uniform(-9, 9))
        p = Vector((math.cos(ang), math.sin(ang), 0.0)) * 0.07
        n = 30
        step = FEELER_L / n
        pts = [p.copy()]
        ph, amp = rnd.uniform(0, 6.28), rnd.uniform(0.25, 0.6)
        for i in range(n):
            s = (i + 0.5) / n
            a = ang + amp * math.sin(s * 5.0 + ph) + 0.5 * amp * math.sin(s * 11.0 + 2.0 * ph)
            p = p + Vector((math.cos(a), math.sin(a), 0.0)) * step
            pts.append(p.copy())
        radii = [feeler_radius(i / n) for i in range(n + 1)]
        for i, q in enumerate(pts):
            q.z = radii[i] * 0.85
        fg = Geo()
        fg.tube(pts, [r * 0.8 for r in radii], radii, sides=7,
                normals=[Vector((0, 0, 1))] * len(pts))
        root = pts[0].copy()
        fo = make_obj("dead_feeler_%d" % (k + 1), [v - root for v in fg.v], fg.f, mats["feeler"],
                      coll, droot, tuple(root), attrs={"along": fg.a})
        feelers.append(fo)
        w.samples += [Vector((q.x, q.y, 0.035)) for q in pts[:16]]

    # the skin: a torn sheet draped over all of the above
    S = np.array([[p.x, p.y, p.z] for p in w.samples])

    def drape(x, y):
        d = np.sqrt((S[:, 0] - x) ** 2 + (S[:, 1] - y) ** 2)
        h = np.max(S[:, 2] + 0.022 - 1.25 * np.maximum(d - 0.03, 0.0))
        return max(float(h), 0.0)

    cx, cy = w.centre.x, w.centre.y
    dir_ang = math.atan2(w.dir.y, w.dir.x)
    # rips in from the edge: (angle, depth as a share of the radius, half width)
    notches = [(back + 0.10, 0.80, 0.66), (back - 1.55, 0.30, 0.11), (back + 1.95, 0.62, 0.24),
               (dir_ang + 0.35, 0.22, 0.09), (dir_ang - 1.05, 0.42, 0.33)]
    gash_ang = dir_ang + math.radians(62.0)
    gash_dir = Vector((math.cos(gash_ang), math.sin(gash_ang)))
    gash_half = 0.56

    def outline(psi):
        r = 0.90 * (1.0 + 0.10 * noise.noise(Vector((math.cos(psi) * 1.2, math.sin(psi) * 1.2, 5.0)))
                    + 0.05 * noise.noise(Vector((math.cos(psi) * 4.0, math.sin(psi) * 4.0, 9.0))))
        r *= 1.0 + 0.10 * abs(math.cos(psi - dir_ang))
        r *= 1.0 + 0.035 * noise.noise(Vector((math.cos(psi) * 13.0, math.sin(psi) * 13.0, 21.0)))
        for a0, depth, half in notches:
            x = abs((psi - a0 + math.pi) % (2.0 * math.pi) - math.pi) / half
            if x < 1.0:
                r *= 1.0 - depth * (1.0 - x ** 1.4)
        return r

    def hole(x, y):
        """Below zero inside the big split or the plug's hole."""
        rel = Vector((x - cx, y - cy))
        s = rel.dot(gash_dir)
        across = abs(rel.x * gash_dir.y - rel.y * gash_dir.x)
        sc = min(max(s, -gash_half), gash_half)
        width = 0.11 * math.sqrt(max(1.0 - (sc / gash_half) ** 2, 0.0))
        width *= 1.0 + 0.45 * noise.noise(Vector((sc * 5.0, 1.0, 8.0)))
        gd = math.hypot(across - width, abs(s) - gash_half) if abs(s) > gash_half else across - width
        return min(gd, math.hypot(x - w.top.x, y - w.top.y) - PLUG_D * 0.9)

    def height(x, y):
        rel = Vector((x - cx, y - cy))
        psi = math.atan2(rel.y, rel.x)
        rho = min(rel.length / max(outline(psi), 1e-6), 1.0)
        h = drape(x, y)
        wr = 0.020 * noise.fractal(Vector((x * 7.0, y * 7.0, 1.0)), 1.0, 2.0, 3)
        wr += 0.014 * math.sin(rho * 34.0 + 5.0 * noise.noise(Vector((x * 2.5, y * 2.5, 2.0)))) * rho
        wr += 0.012 * math.sin(psi * 21.0 + 4.0 * noise.noise(Vector((x * 2.0, y * 2.0, 4.0)))) * rho ** 2
        # the edges of the split curl up a little
        wr += 0.03 * math.exp(-max(hole(x, y), 0.0) / 0.05)
        return max(h + wr + 0.012, 0.005)

    def sheet_point(psi, rho):
        r = outline(psi) * rho
        x, y = cx + r * math.cos(psi), cy + r * math.sin(psi)
        return Vector((x, y, height(x, y)))

    seg, rings = 144, 32
    rhos = [((j + 1) / rings) ** 0.85 for j in range(rings)]
    verts = [sheet_point(0.0, 0.0)]
    for rho in rhos:
        for i in range(seg):
            verts.append(sheet_point(2.0 * math.pi * i / seg, rho))
    row = lambda j: 1 + j * seg
    cand = []
    for i in range(seg):
        i1 = (i + 1) % seg
        cand.append((0, row(0) + i, row(0) + i1))
        for j in range(rings - 1):
            cand.append((row(j) + i, row(j + 1) + i, row(j + 1) + i1, row(j) + i1))
    faces, edge_verts, gone_verts = [], set(), set()
    for f in cand:
        c = sum((verts[i] for i in f), Vector()) / len(f)
        if hole(c.x, c.y) < 0.0:
            gone_verts.update(f)
        else:
            faces.append(f)
    for f in faces:
        edge_verts.update(i for i in f if i in gone_verts)
    # pull the rim of each hole onto the hole's true edge, so it is not stepped
    for i in edge_verts:
        x, y = verts[i].x, verts[i].y
        for _ in range(4):
            d0, e = hole(x, y), 1e-3
            gx, gy = (hole(x + e, y) - d0) / e, (hole(x, y + e) - d0) / e
            gg = gx * gx + gy * gy
            if gg < 1e-9:
                break
            x, y = x - d0 * gx / gg, y - d0 * gy / gg
        verts[i] = Vector((x, y, height(x, y)))
    skin = make_obj("dead_skin", verts, faces, mats["skin_dead"], coll, droot)

    # veins on the dead skin
    vpoint = sheet_point

    def vframe(a, b):
        e = 2e-3
        ea = vpoint(a + e, b) - vpoint(a - e, b)
        eb = vpoint(a, min(b + e, 1.0)) - vpoint(a, max(b - e, 0.001))
        n = ea.cross(eb)
        if n.z < 0:
            n = -n
        if n.length < 1e-12:
            n = Vector((0, 0, 1))
        n = (n.normalized() + Vector((0, 0, 1.5))).normalized()
        return ea.normalized(), eb.normalized(), n, ea.length / (2 * e), eb.length / (2 * e)

    def vstop(p):
        return hole(p.x, p.y) < 0.012

    starts = [(2.0 * math.pi * (i + rnd.uniform(-0.3, 0.3)) / 9.0, rnd.uniform(0.25, 0.45),
               rnd.uniform(0.006, 0.010)) for i in range(11)]
    vg = Geo()
    for pts, nors, wid in grow_veins(vpoint, vframe, starts, 1800, vstop, 0.05, 0.96, seed=31):
        vg.ribbon(pts, nors, wid, 0.003)
    make_obj("dead_veins", vg.v, vg.f, mats["vein_dead"], coll, skin)

    # a few torn scraps thrown clear
    for i in range(4):
        ang = rnd.uniform(0, 6.28)
        dist = rnd.uniform(0.80, 0.98)
        c = Vector((cx + dist * math.cos(ang), cy + dist * math.sin(ang), 0.0))
        m = 11
        sv = [Vector((0, 0, 0.012))]
        size = rnd.uniform(0.05, 0.10)
        for j in range(m):
            a = 2.0 * math.pi * j / m
            rr = size * rnd.uniform(0.45, 1.25)
            sv.append(Vector((rr * math.cos(a) * 1.5, rr * math.sin(a), 0.004 + 0.012 * rnd.random())))
        rot = Matrix.Rotation(rnd.uniform(0, 6.28), 3, "Z")
        sv = [rot @ v for v in sv]
        sf = [(0, 1 + j, 1 + (j + 1) % m) for j in range(m)]
        make_obj("dead_scrap_%d" % (i + 1), sv, sf, mats["skin_dead"], coll, droot, tuple(c))

    # the plug, fallen with the top of the frame, lens dark
    plug = build_plug(coll, mats, droot, (w.top.x, w.top.y, 0.045), dead=True, prefix="dead_")
    plug.rotation_euler = (math.radians(62.0), math.radians(12.0), dir_ang)

    # damp stain
    seg, rr = 72, [0.0, 0.45, 0.7, 0.85, 0.94, 1.0]
    sv, sa, sf = [Vector((cx, cy, 0.003))], [(1.0, 0, 0)], []
    for ri in range(1, len(rr)):
        for i in range(seg):
            psi = 2.0 * math.pi * i / seg
            rad = 1.20 * (1.0 + 0.22 * noise.noise(Vector((math.cos(psi) * 1.4, math.sin(psi) * 1.4, 11.0)))
                          + 0.05 * noise.noise(Vector((math.cos(psi) * 5.0, math.sin(psi) * 5.0, 12.0))))
            sv.append(Vector((cx + rad * rr[ri] * math.cos(psi), cy + rad * rr[ri] * math.sin(psi), 0.003)))
            sa.append((1.0 - rr[ri] ** 2.2, 0, 0))
    row = lambda r: 1 + (r - 1) * seg
    for i in range(seg):
        i1 = (i + 1) % seg
        sf.append((0, row(1) + i, row(1) + i1))
        for r in range(1, len(rr) - 1):
            sf.append((row(r) + i, row(r + 1) + i, row(r + 1) + i1, row(r) + i1))
    make_obj("dead_stain", sv, sf, mats["stain"], coll, droot, attrs={"wet": sa})

    # centre the wreck (skin and frame) on the origin
    bpy.context.view_layer.update()
    lo, hi = sl.bounds([skin])
    droot.location = (-(lo.x + hi.x) / 2.0, -(lo.y + hi.y) / 2.0, 0.0)
    return droot


# ---------------------------------------------------------------------------
def new_collection(name):
    c = bpy.data.collections.new(name)
    bpy.context.scene.collection.children.link(c)
    return c


def main():
    sl.reset_scene()
    model = new_collection("model")
    dead = new_collection("dead")

    root = make_empty(NAME, model)
    # the sac must exist (for its scale) before the stretch marks are placed
    sac = build_sac(model, {"skin": None}, root)
    mats = make_materials(stretch_marks())
    sac.data.materials.append(mats["skin"])
    build_frame(model, mats, sac)
    build_murk(model, mats, sac)
    build_veins(model, mats, sac)
    build_patch(model, mats, sac)
    build_feelers(model, mats, sac)
    build_plug(model, mats, sac, (0.0, 0.0, ZT))
    build_arcs(model, mats, sac)

    droot = build_dead(dead, mats)
    dead.hide_render = True

    bpy.context.view_layer.update()
    data = sl.write_size_json(os.path.join(HERE, "size.json"), NAME, [root])
    print("BLOAT size (whole, with feelers and plug):", data)
    dg = bpy.context.evaluated_depsgraph_get()
    me = sac.evaluated_get(dg).to_mesh()
    zs = [(sac.matrix_world @ v.co) for v in me.vertices]
    print("  sac alone: width %.4f depth %.4f underside %.4f top %.4f tris %d" % (
        max(p.x for p in zs) - min(p.x for p in zs), max(p.y for p in zs) - min(p.y for p in zs),
        min(p.z for p in zs), max(p.z for p in zs), len(me.polygons)))
    for o in model.objects:
        if o.type == "MESH":
            own = sum(len(p.vertices) - 2 for p in o.data.polygons)
            print("  %-22s tris %6d" % (o.name, own))
    tips = [min((o.matrix_world @ v.co).z for v in o.data.vertices)
            for o in model.objects if o.name.startswith("feeler_")]
    print("  feeler tips above ground:", ["%.3f" % z for z in tips])
    print("  dead:", sl.measure([droot]))
    print("  dead skin and frame:", sl.measure([bpy.data.objects["dead_skin"],
                                                 bpy.data.objects["dead_frame_ribs"]]))
    for o in dead.objects:
        if o.type == "MESH":
            print("  %-22s tris %6d" % (o.name, sum(len(p.vertices) - 2 for p in o.data.polygons)))
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(HERE, "model.blend"), compress=True)
    backup = os.path.join(HERE, "model.blend1")
    if os.path.exists(backup):
        os.remove(backup)


main()
