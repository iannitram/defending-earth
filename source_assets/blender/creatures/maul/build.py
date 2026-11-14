"""Builds the Maul from nothing and saves model.blend.

Run from anywhere:
  flatpak run org.blender.Blender -b --python "<full path>/build.py"

The file holds three collections:
  model  state 1, controlled: the Maul with its control spike (root "maul")
  wild   state 2, wild: spike snapped off, brighter, lower and wider
         (root "wild_maul")
  dead   state 3, dead: slumped flat, every crack dark (root "dead_maul")

Each state also has a "<state>_ground" empty that holds what lies on the
soil (scorch ring, cooled splash, the broken stake). Those are kept apart
so they are not counted in the model's size.

How the body is made: a plan of uneven six-sided cells is cut for every
body part (one cell per column). Each cell is pushed out into a basalt
column of its own length. A dark-to-glowing core sits a little way inside,
so the gaps between the columns show as molten cracks.
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
from mathutils import Matrix, Vector

import scale_lib as sl

NAME = "maul"

# ---------------------------------------------------------------------------
# Sizes. Overall size and the spike come from the scale sheet. Part sizes
# are the ones the job text gives.
# ---------------------------------------------------------------------------
SHEET = sl.thing(NAME)
HEIGHT = SHEET["height_m"]
WIDTH = SHEET["width_m"]
DEPTH = SHEET["depth_m"]
SPIKE_PROUD = SHEET["spike_proud_m"]
SPIKE_SIDE = SHEET["spike_side_m"]
COLLAR_D = SHEET["spike_collar_diameter_m"]

FRONT_LIMB_D = 0.9       # front limb, across
VENT_D = 0.5             # the open hollow end
STUMP_H = 0.15           # wild state: what is left of the stake
SPIKE_BURIED = 0.5       # how far the stake goes into the stone (hidden)
LENS_D = 0.06
SPIKE_TURN_DEG = 32.0    # the stake's turn about its own axis

# Colours (job text and the shared table)
C_BASALT = "#1b1b1e"
C_DUST = "#58554f"
C_MAGMA = "#ff5a0c"
C_WHITEHOT = "#fff2d0"
C_CRYSTAL = "#ff8a5a"
C_CRYSTAL_DEAD = "#8f7670"
C_HARDWARE = "#0b0b0c"
C_GOLD = "#8a6a2a"
C_AMBER = "#ffa01e"
E_CRACK = 8.0
E_CRACK_WILD = 14.0
E_WHITEHOT = 30.0
E_CRYSTAL = 3.0
E_LENS = 6.0

Z = Vector((0.0, 0.0, 1.0))
SPIKE_XY = (0.0, -0.56)      # where the spike goes in, before the final fit

# ---------------------------------------------------------------------------
# The three postures. Numbers are before the final fit to the sheet size.
#   zp          height of the back at its highest point
#   front_drop  how far the top falls from the peak to the front edge
#   rear_drop   how far the top falls from the peak to the rear edge
#   zb0, zbf    belly height at the rear, and extra height at the chest
#   S, E, V     front limb: shoulder, elbow, vent end (for the +X side)
#   H, F        rear limb: hip, foot (for the +X side)
# ---------------------------------------------------------------------------
POSES = {
    "model": dict(zp=3.40, front_drop=0.70, rear_drop=1.85, zb0=0.72, zbf=0.50, wmul=1.0,
                  kx=0.10, ky=0.06, lean=0.12, jit=(0.02, 0.34), rlean=0.015, lose=0.0,
                  S=(1.12, -0.50, 2.55), E=(1.40, -0.62, 1.22), V=(1.34, -1.88, 0.62),
                  H=(0.92, 1.18, 1.05), F=(1.05, 1.30, 0.0), stand=True),
    "wild": dict(zp=2.95, front_drop=0.55, rear_drop=1.40, zb0=0.70, zbf=0.15, wmul=1.07,
                 kx=0.11, ky=0.06, lean=0.16, jit=(0.02, 0.34), rlean=0.015, lose=0.0,
                 S=(1.22, -0.50, 2.10), E=(1.58, -0.58, 1.12), V=(1.52, -1.90, 0.62),
                 H=(0.97, 1.18, 1.00), F=(1.22, 1.32, 0.0), stand=True),
    "dead": dict(zp=1.70, front_drop=0.35, rear_drop=0.80, zb0=0.0, zbf=0.0, wmul=1.12,
                 kx=0.0, ky=0.0, lean=0.0, jit=(0.02, 0.55), rlean=0.07, lose=0.14,
                 S=(1.15, -0.60, 0.60), E=(2.05, -1.25, 0.50), V=(2.70, -2.20, 0.49),
                 H=(0.95, 1.25, 0.62), F=(1.85, 2.25, 0.60), stand=False),
}
Y0, Y1 = -1.40, 1.70
ZREF = 1.6
PEAK_T = (SPIKE_XY[1] - Y0) / (Y1 - Y0)


def clamp(x, a=0.0, b=1.0):
    return max(a, min(b, x))


def smooth(a, b, x):
    t = clamp((x - a) / (b - a))
    return t * t * (3.0 - 2.0 * t)


def lerp(a, b, t):
    return a + (b - a) * t


# ---------------------------------------------------------------------------
# Plan of cells (a Voronoi plan cut by hand: no extra library needed)
# ---------------------------------------------------------------------------
def clip_poly(poly, nx, ny, d):
    """Keep the part of a polygon where nx*x + ny*y <= d."""
    out = []
    n = len(poly)
    for i in range(n):
        ax, ay = poly[i]
        bx, by = poly[(i + 1) % n]
        da = nx * ax + ny * ay - d
        db = nx * bx + ny * by - d
        if da <= 0:
            out.append((ax, ay))
        if (da < 0 < db) or (db < 0 < da):
            t = da / (da - db)
            out.append((ax + (bx - ax) * t, ay + (by - ay) * t))
    return out


def tidy_poly(poly, eps=0.025):
    out = []
    for p in poly:
        if not out or math.hypot(p[0] - out[-1][0], p[1] - out[-1][1]) > eps:
            out.append(p)
    if len(out) > 1 and math.hypot(out[0][0] - out[-1][0], out[0][1] - out[-1][1]) <= eps:
        out.pop()
    return out


def cells(sites, keep, gaps, reach=1.3):
    """One polygon per kept site, with a gap left round it."""
    res = {}
    for i, (px, py) in enumerate(sites):
        if not keep[i]:
            continue
        h = 0.75
        poly = [(px - h, py - h), (px + h, py - h), (px + h, py + h), (px - h, py + h)]
        for j, (qx, qy) in enumerate(sites):
            if j == i:
                continue
            dx, dy = qx - px, qy - py
            dist = math.hypot(dx, dy)
            if dist > reach or dist < 1e-6:
                continue
            nx, ny = dx / dist, dy / dist
            d = nx * (px + qx) / 2.0 + ny * (py + qy) / 2.0 - gaps[i] / 2.0
            poly = clip_poly(poly, nx, ny, d)
            if len(poly) < 3:
                break
        poly = tidy_poly(poly)
        if len(poly) >= 3:
            res[i] = poly
    return res


def hex_sites(pitch, extent, rng, jitter, turn=0.0):
    pts = []
    n = int(extent / pitch) + 2
    c, s = math.cos(turn), math.sin(turn)
    for i in range(-n, n + 1):
        for j in range(-n, n + 1):
            x = pitch * (i + 0.5 * j)
            y = pitch * 0.8660254 * j
            x, y = c * x - s * y, s * x + c * y
            if abs(x) > extent or abs(y) > extent:
                continue
            pts.append((x + rng.uniform(-jitter, jitter), y + rng.uniform(-jitter, jitter)))
    return pts


# ---------------------------------------------------------------------------
# Mesh gathering
# ---------------------------------------------------------------------------
class Part:
    def __init__(self, name, parent, origin, kind, mats):
        self.name, self.parent, self.kind, self.mats = name, parent, kind, mats
        self.origin = Vector(origin)
        self.verts, self.faces, self.fmat, self.heat = [], [], [], []
        self.matrix = None      # hardware only: placed by a matrix, not baked

    def add(self, verts, faces, mat=0, heat=None):
        base = len(self.verts)
        self.verts.extend(Vector(v) for v in verts)
        for f in faces:
            self.faces.append(tuple(base + k for k in f))
            self.fmat.append(mat)
        if heat is None:
            heat = [(0.0, 0.0)] * len(verts)
        self.heat.extend(heat)


def column(part, poly, fn, s0, s1, rng, chip=1.0, end_tilt=0.10, seg_len=(0.45, 0.95)):
    """A stack of column pieces from s0 to s1. fn(u, v, s) gives the point."""
    n = len(poly)
    cu = sum(p[0] for p in poly) / n
    cv = sum(p[1] for p in poly) / n
    length = s1 - s0
    pieces = max(1, int(round(length / rng.uniform(*seg_len))))
    cuts = [s0]
    for k in range(1, pieces):
        cuts.append(s0 + length * (k + rng.uniform(-0.22, 0.22)) / pieces)
    cuts.append(s1)
    tilts = [(rng.uniform(-end_tilt, end_tilt), rng.uniform(-end_tilt, end_tilt)) for _ in cuts]
    for k in range(pieces):
        a, b = cuts[k], cuts[k + 1]
        ou, ov = (rng.uniform(-0.018, 0.018), rng.uniform(-0.018, 0.018)) if pieces > 1 else (0, 0)
        grow = rng.uniform(0.95, 1.0)
        c = min(rng.uniform(0.02, 0.04) * chip, (b - a) * 0.3)
        rings = []
        for which, s, tilt in ((0, a, tilts[k]), (1, a + c, tilts[k]), (1, b - c, tilts[k + 1]),
                               (0, b, tilts[k + 1])):
            ring = []
            for (u, v) in poly:
                du, dv = (u - cu) * grow, (v - cv) * grow
                if which == 0:
                    r = math.hypot(du, dv) or 1.0
                    ins = min(rng.uniform(0.015, 0.06) * chip, r * 0.4)
                    du, dv = du * (1 - ins / r), dv * (1 - ins / r)
                ss = s + tilt[0] * du + tilt[1] * dv
                ring.append(fn(cu + du + ou, cv + dv + ov, ss))
            rings.append(ring)
        verts = [p for ring in rings for p in ring]
        faces = [tuple(reversed(range(n)))]
        fm = [1]
        for r in range(3):
            for i in range(n):
                j = (i + 1) % n
                faces.append((r * n + i, r * n + j, (r + 1) * n + j, (r + 1) * n + i))
                fm.append(0 if r == 1 else 1)
        faces.append(tuple(range(3 * n, 4 * n)))
        fm.append(1)
        base = len(part.verts)
        part.verts.extend(verts)
        part.heat.extend([(0.0, 0.0)] * len(verts))
        for f, m in zip(faces, fm):
            part.faces.append(tuple(base + q for q in f))
            part.fmat.append(m)


def tube(part, fn, radius, s0, s1, heat_fn, segs=14, rings=6, cap0=True, cap1=True, mat=0,
         inside=False):
    verts, faces, heat = [], [], []
    for r in range(rings + 1):
        s = lerp(s0, s1, r / rings)
        for i in range(segs):
            a = 2 * math.pi * i / segs
            p = fn(radius * math.cos(a), radius * math.sin(a), s)
            verts.append(p)
            heat.append(heat_fn(p, s))
    for r in range(rings):
        for i in range(segs):
            j = (i + 1) % segs
            f = (r * segs + i, r * segs + j, (r + 1) * segs + j, (r + 1) * segs + i)
            faces.append(tuple(reversed(f)) if inside else f)
    if cap0:
        faces.append(tuple(reversed(range(segs))))
    if cap1:
        faces.append(tuple(range(rings * segs, (rings + 1) * segs)))
    part.add(verts, faces, mat, heat)


def crystal(part, base, direction, length, rng):
    d = Vector(direction).normalized()
    u = d.orthogonal().normalized()
    v = d.cross(u)
    roll = rng.uniform(0, math.pi)
    r = length * rng.uniform(0.15, 0.21)
    verts = []
    for frac, rad in ((0.0, r * 0.8), (rng.uniform(0.55, 0.72), r)):
        for i in range(6):
            a = roll + i * math.pi / 3
            verts.append(Vector(base) + d * (length * frac) + (u * math.cos(a) + v * math.sin(a)) * rad)
    verts.append(Vector(base) + d * length + (u * rng.uniform(-0.3, 0.3) + v * rng.uniform(-0.3, 0.3)) * r)
    faces = []
    for i in range(6):
        j = (i + 1) % 6
        faces.append((i, j, 6 + j, 6 + i))
        faces.append((6 + i, 6 + j, 12))
    faces.append(tuple(reversed(range(6))))
    part.add(verts, faces, 0)


def cluster(part, base, normal, count, lmin, lmax, rng, spread=0.45):
    nrm = Vector(normal).normalized()
    u = nrm.orthogonal().normalized()
    v = nrm.cross(u)
    for k in range(count):
        if k == 0:
            d, ln, off = nrm, lmax, Vector((0, 0, 0))
        else:
            a = rng.uniform(0, 2 * math.pi)
            t = rng.uniform(0.25, 1.0) * spread
            d = nrm * math.cos(t) + (u * math.cos(a) + v * math.sin(a)) * math.sin(t)
            ln = rng.uniform(lmin, lmax * 0.8)
            off = (u * math.cos(a) + v * math.sin(a)) * rng.uniform(0.04, 0.12)
        crystal(part, Vector(base) + off, d, ln, rng)


# ---------------------------------------------------------------------------
# Where the cracks are hot. Returns (brightness 0..1, white-hot 0..1).
# ---------------------------------------------------------------------------
def heat_at(p, state, spike_point):
    if state == "dead":
        return (0.0, 0.0)
    dim = lerp(1.0, 0.04, smooth(-0.45, 1.45, p.y))
    dim *= lerp(0.6, 1.0, smooth(0.2, 1.8, p.z))
    white = smooth(-0.75, -1.05, p.y) * smooth(0.9, 0.6, abs(p.x)) * smooth(1.3, 1.6, p.z) \
        * smooth(2.5, 2.2, p.z)
    d = (p - spike_point).length
    white = max(white, smooth(0.50, 0.22, d))
    if state == "wild":
        dim = min(1.0, dim * 1.15 + 0.30)
    return (dim, clamp(white))


# ---------------------------------------------------------------------------
# One state's geometry
# ---------------------------------------------------------------------------
def gen_state(state, fit):
    """fit = (sx, sy, sz, shift_x, shift_y). Returns (parts, info)."""
    pose = POSES[state]
    pre = "" if state == "model" else state + "_"
    sx, sy, sz, shx, shy = fit

    def C(p):
        return Vector(((p[0] - shx) * sx, (p[1] - shy) * sy, p[2] * sz))

    def mirror(p, sgn):
        return Vector((p[0] * sgn, p[1], p[2]))

    parts = []
    zp = pose["zp"]

    # ---- torso profile -------------------------------------------------
    def prof(x, y):
        t = clamp((y - Y0) / (Y1 - Y0))
        w = (1.62 - 0.45 * smooth(0.3, 1.0, t)) * pose["wmul"]
        hw = w * max(0.0, 1.0 - abs(2 * t - 1) ** 4) ** 0.4
        if t < PEAK_T:
            ztc = zp - pose["front_drop"] * ((PEAK_T - t) / PEAK_T) ** 2
        else:
            ztc = zp - pose["rear_drop"] * ((t - PEAK_T) / (1 - PEAK_T)) ** 1.2
        zbc = pose["zb0"] + pose["zbf"] * (1 - t) ** 2
        zmid = zbc + 0.42 * (ztc - zbc)
        r = clamp(abs(x) / hw) if hw > 1e-6 else 1.0
        f = (1 - r ** 4) ** 0.5
        inside = hw > 1e-6 and abs(x) / hw < 0.95 and 0.015 < (y - Y0) / (Y1 - Y0) < 0.985
        return inside, zmid + (ztc - zmid) * f, zmid - (zmid - zbc) * f

    def fan(u, v, s):
        dz = s - ZREF
        return C((u * (1 + pose["kx"] * dz),
                  0.3 + (v - 0.3) * (1 + pose["ky"] * dz) - pose["lean"] * dz, s))

    rng = random.Random(7)
    sites = hex_sites(0.31, 2.4, rng, 0.05, turn=0.13)
    sites = [p for p in sites if rng.random() > 0.09]
    sites = [p for p in sites if math.hypot(p[0] - SPIKE_XY[0], p[1] - SPIKE_XY[1]) > 0.47]
    ring_ids = []
    a0 = rng.uniform(0, 1.0)
    for k in range(6):
        a = a0 + k * math.pi / 3 + rng.uniform(-0.12, 0.12)
        ring_ids.append(len(sites))
        sites.append((SPIKE_XY[0] + 0.30 * math.cos(a), SPIKE_XY[1] + 0.30 * math.sin(a)))
    keep, tops, bots, gaps = [], [], [], []
    lose_rng = random.Random(21)
    lost = []
    for i, (x, y) in enumerate(sites):
        ins, zt, zb = prof(x, y)
        top = zt - rng.uniform(*pose["jit"])
        bot = zb + rng.uniform(0.0, 0.25)
        if i in ring_ids:
            top = zp
        if not pose["stand"]:
            bot = 0.0
        if top - bot < 0.5:
            mid = (top + bot) / 2
            top, bot = mid + 0.25, max(mid - 0.25, 0.0 if not pose["stand"] else -9)
        if ins and i not in ring_ids and lose_rng.random() < pose["lose"]:
            ins = False
            lost.append(i)
        keep.append(ins)
        tops.append(top)
        bots.append(bot)
        gaps.append(0.075 if i in ring_ids else rng.uniform(0.04, 0.062))
    plan = cells(sites, keep, gaps)

    torso_origin = C((0.0, 0.9, max(pose["zb0"], 0.3) + 0.35))
    torso = Part(pre + "torso", pre + NAME, torso_origin, "stone", ["basalt_side", "basalt_end"])
    parts.append(torso)
    lrng = random.Random(11)
    for i, poly in plan.items():
        px, py = sites[i]
        lu, lv = lrng.uniform(-1, 1) * pose["rlean"], lrng.uniform(-1, 1) * pose["rlean"]
        if i in ring_ids:      # cracked outward by the spike
            dx, dy = px - SPIKE_XY[0], py - SPIKE_XY[1]
            dd = math.hypot(dx, dy)
            lu, lv = 0.05 * dx / dd, 0.05 * dy / dd
        piv = bots[i]

        def fn(u, v, s, lu=lu, lv=lv, piv=piv):
            return fan(u + lu * (s - piv), v + lv * (s - piv), s)
        column(torso, poly, fn, bots[i], tops[i], lrng,
               end_tilt=0.03 if i in ring_ids else 0.20)

    peak = fan(SPIKE_XY[0], SPIKE_XY[1], zp)
    spike_pre = Vector((SPIKE_XY[0], SPIKE_XY[1] - pose["lean"] * (zp - ZREF) +
                        (SPIKE_XY[1] - 0.3) * pose["ky"] * (zp - ZREF), zp))

    def heat(p_final):
        # the heat map is drawn in the coordinates before the final fit
        q = Vector((p_final.x / sx + shx, p_final.y / sy + shy, p_final.z / sz))
        if state == "wild":   # the wild body is lower: lift so the chest still reads hot
            q.z += 0.35
        return heat_at(q, state, spike_pre + (Vector((0, 0, 0.35)) if state == "wild" else Vector()))

    # ---- torso core ------------------------------------------------------
    core = Part(pre + "torso_core", pre + "torso", torso_origin, "core", ["core"])
    parts.append(core)
    step = 0.09
    nx, ny = int(4.6 / step), int(4.4 / step)
    grid = {}
    for ix in range(nx):
        for iy in range(ny):
            x, y = -2.3 + ix * step, -2.0 + iy * step
            near = [k for k, (qx, qy) in enumerate(sites) if math.hypot(qx - x, qy - y) < 0.42]
            close = [k for k in near if math.hypot(sites[k][0] - x, sites[k][1] - y) < 0.40]
            if not close or not all(keep[k] for k in close):
                continue
            kk = [k for k in near if keep[k]]
            zt = min(tops[k] for k in kk) - 0.09
            zb = max(bots[k] for k in kk) + 0.10
            if not pose["stand"]:
                zb = 0.02
            if zt - zb > 0.12:
                grid[(ix, iy)] = (x, y, zt, zb)
    vid = {}
    cverts, cheat, cfaces = [], [], []
    for key, (x, y, zt, zb) in grid.items():
        for lvl, zz in ((0, zt), (1, zb)):
            vid[(key, lvl)] = len(cverts)
            p = fan(x, y, zz)
            cverts.append(p)
            cheat.append(heat(p))
    for (ix, iy) in grid:
        quad = [(ix, iy), (ix + 1, iy), (ix + 1, iy + 1), (ix, iy + 1)]
        if all(q in grid for q in quad):
            cfaces.append(tuple(vid[(q, 0)] for q in quad))
            cfaces.append(tuple(vid[(q, 1)] for q in reversed(quad)))
            for e in range(4):
                a, b = quad[e], quad[(e + 1) % 4]
                # neighbour cell across this edge
                ox, oy = [(0, -1), (1, 0), (0, 1), (-1, 0)][e]
                nq = [(ix + ox, iy + oy), (ix + 1 + ox, iy + oy), (ix + 1 + ox, iy + 1 + oy),
                      (ix + ox, iy + 1 + oy)]
                if not all(q in grid for q in nq):
                    cfaces.append((vid[(a, 1)], vid[(b, 1)], vid[(b, 0)], vid[(a, 0)]))
    core.add(cverts, cfaces, 0, cheat)

    # ---- crystals ----------------------------------------------------------
    def top_near(x, y):
        best, bd = None, 9.0
        for k, (qx, qy) in enumerate(sites):
            d = math.hypot(qx - x, qy - y)
            if keep[k] and d < bd:
                best, bd = k, d
        return best

    crng = random.Random(5)
    back = Part(pre + "crystals_back", pre + "torso", peak, "crystal", ["crystal"])
    parts.append(back)
    spots = [(0.62, SPIKE_XY[1] + 0.05, 0.50, (0.75, -0.15, 1.0)),
             (0.50, SPIKE_XY[1] + 0.62, 0.62, (0.55, 0.25, 1.0))]
    for k, y in enumerate((0.45, 0.85, 1.22, 1.50)):
        spots.append((0.40 - 0.03 * k, y, 0.70 - 0.13 * k, (0.5, 0.35, 1.0)))
    for sgn in (1, -1):
        for (x, y, ln, nrm) in spots:
            x = sgn * (x + crng.uniform(-0.05, 0.05))
            y = y + crng.uniform(-0.07, 0.07)
            k = top_near(x, y)
            if k is None:
                continue
            zb = tops[k] - 0.22
            cluster(back, fan(x, y, zb), (nrm[0] * sgn, nrm[1], nrm[2]),
                    crng.randint(3, 4), 0.20, ln + 0.1, crng)
    if pose["stand"]:
        belly = Part(pre + "crystals_belly", pre + "torso", torso_origin, "crystal", ["crystal"])
        parts.append(belly)
        for (x, y) in ((0.25, -0.35), (-0.3, 0.25), (0.2, 0.85), (-0.15, 1.25)):
            k = top_near(x, y)
            cluster(belly, fan(x, y, bots[k] + 0.18), (crng.uniform(-0.3, 0.3), crng.uniform(-0.3, 0.3), -1),
                    crng.randint(3, 5), 0.20, 0.42, crng, spread=0.55)

    # ---- limbs ---------------------------------------------------------------
    def frame(A, B):
        axis = (B - A).normalized()
        U = Z.cross(axis)
        U = U.normalized() if U.length > 1e-4 else Vector((1, 0, 0))
        V = axis.cross(U)
        return axis, U, V, (B - A).length

    def ground(p):
        if p.z < 0.0:
            p = Vector((p.x, p.y, 0.0))
        return p

    info = {"vents": {}, "peak": peak}
    for sgn, side in ((1, "L"), (-1, "R")):
        srng = random.Random(100 + (0 if sgn > 0 else 1))
        S, E, V_ = (mirror(pose[k], sgn) for k in ("S", "E", "V"))
        # upper front limb: a bundle of seven columns
        axis, U, W, ln = frame(S, E)
        upper = Part(pre + "front_limb_%s_upper" % side, pre + "torso", C(S), "stone",
                     ["basalt_side", "basalt_end"])
        parts.append(upper)

        def fu(u, v, s, A=S, axis=axis, U=U, W=W, ln=ln):
            k = lerp(1.32, 1.0, clamp(s / ln))
            return ground(C(A + axis * s + (U * u + W * v) * k))
        pitch = FRONT_LIMB_D / 3.0
        ss = hex_sites(pitch, 1.3, srng, 0.02, turn=srng.uniform(0, 1))
        kp = [math.hypot(*p) < pitch * 1.25 for p in ss]
        for i, poly in cells(ss, kp, [srng.uniform(0.035, 0.055) for _ in ss]).items():
            column(upper, poly, fu, -0.15 - srng.uniform(0, 0.3), ln + 0.30 + srng.uniform(0, 0.22), srng)
        ucore = Part(pre + "front_limb_%s_upper_core" % side, upper.name, C(S), "core", ["core"])
        parts.append(ucore)
        tube(ucore, fu, FRONT_LIMB_D / 2 - 0.19, 0.0, ln + 0.22, lambda p, s: heat(p))

        # forearm: a ring of columns round the hollow vent
        axis, U, W, ln = frame(E, V_)
        fore = Part(pre + "front_limb_%s_forearm" % side, upper.name, C(E), "stone",
                    ["basalt_side", "basalt_end"])
        parts.append(fore)

        def ff(u, v, s, A=E, axis=axis, U=U, W=W):
            return ground(C(A + axis * s + U * u + W * v))
        nring = 9
        r_mid = (FRONT_LIMB_D / 2 + VENT_D / 2) / 2 + 0.01
        ss, kp = [], []
        ph = srng.uniform(0, 1)
        for k in range(nring):
            a = ph + 2 * math.pi * k / nring
            ss.append(((r_mid + srng.uniform(-0.015, 0.015)) * math.cos(a), r_mid * math.sin(a)))
            kp.append(True)
            a2 = a + math.pi / nring
            ss.append((0.18 * math.cos(a2), 0.18 * math.sin(a2)))
            kp.append(False)
            ss.append((0.52 * math.cos(a2), 0.52 * math.sin(a2)))
            kp.append(False)
        for i, poly in cells(ss, kp, [srng.uniform(0.035, 0.05) for _ in ss]).items():
            column(fore, poly, ff, -0.32 - srng.uniform(0, 0.07), ln - srng.uniform(0.0, 0.10), srng,
                   end_tilt=0.06)
        # stone plug closing the back of the ring
        hexp = [(0.25 * math.cos(k * math.pi / 3), 0.25 * math.sin(k * math.pi / 3)) for k in range(6)]
        column(fore, hexp, ff, -0.42, -0.18, srng)
        # knuckle: three short columns the limb stands on
        if pose["stand"]:
            K = E + axis * (ln - 0.82)
            ks = hex_sites(0.25, 1.0, srng, 0.02, turn=srng.uniform(0, 1))
            c0 = (0.0, 0.0)
            kk = [math.hypot(p[0] - c0[0], p[1] - c0[1]) < 0.3 for p in ks]

            def fk(u, v, s, K=K):
                return C(Vector((K.x + u - c0[0], K.y + v - c0[1], max(s, 0.0))))
            for i, poly in cells(ks, kk, [0.04] * len(ks)).items():
                column(fore, poly, fk, 0.0, K.z - 0.40 + srng.uniform(-0.10, 0.04), srng, seg_len=(0.3, 0.5))
        fcore = Part(pre + "front_limb_%s_forearm_core" % side, fore.name, C(E), "core", ["core"])
        parts.append(fcore)
        depth = 0.50

        def hf(p, s, ln=ln):
            h = heat(p)
            if state != "dead" and s > ln - depth - 0.03:
                return (1.0, 1.0)
            return h
        tube(fcore, ff, VENT_D / 2 - 0.04, -0.22, ln - depth, hf, cap1=True)
        vent = Part(pre + "vent_%s" % side, fore.name, C(V_), "core", ["core"])
        parts.append(vent)

        def hv(p, s, ln=ln):
            if state == "dead":
                return (0.0, 0.0)
            return (1.0, lerp(0.9, 0.1, (s - (ln - depth)) / (depth - 0.1)))
        tube(vent, ff, VENT_D / 2 - 0.02, ln - depth, ln - 0.10, hv, cap0=False, cap1=False, inside=True)
        info["vents"][side] = dict(end=C(V_), axis=axis, U=U, W=W, raw_end=V_)

        # rear limb: shorter, thicker, fused
        H, F = mirror(pose["H"], sgn), mirror(pose["F"], sgn)
        axis, U, W, ln = frame(H, F)
        rear = Part(pre + "rear_limb_%s" % side, pre + "torso", C(H), "stone",
                    ["basalt_side", "basalt_end"])
        parts.append(rear)

        def fr(u, v, s, A=H, axis=axis, U=U, W=W):
            return ground(C(A + axis * s + U * u + W * v))
        pitch = 0.37
        ss = hex_sites(pitch, 1.5, srng, 0.03, turn=srng.uniform(0, 1))
        kp = [math.hypot(*p) < pitch * 1.25 for p in ss]
        over = 0.35 if pose["stand"] else 0.0
        for i, poly in cells(ss, kp, [srng.uniform(0.02, 0.035) for _ in ss]).items():
            column(rear, poly, fr, -0.25 - srng.uniform(0, 0.3), ln + over + srng.uniform(0, 0.2), srng,
                   seg_len=(0.35, 0.7))
        rcore = Part(pre + "rear_limb_%s_core" % side, rear.name, C(H), "core", ["core"])
        parts.append(rcore)

        def frc(u, v, s):
            p = fr(u, v, s)
            return Vector((p.x, p.y, max(p.z, 0.03)))
        tube(rcore, frc, 0.36, 0.0, ln + (0.2 if pose["stand"] else -0.1), lambda p, s: heat(p))

    # ---- drips on the creature's left vent (+X) --------------------------------
    if pose["stand"]:
        vL = info["vents"]["L"]
        drng = random.Random(3)
        drips = Part(pre + "vent_L_drips", pre + "front_limb_L_forearm", vL["end"], "core", ["core"])
        parts.append(drips)
        lip = vL["raw_end"] - vL["axis"] * 0.05 - vL["W"] * (VENT_D / 2 + 0.01)
        for k, (off, dl) in enumerate(((-0.10, 0.17), (0.02, 0.26), (0.11, 0.12))):
            top = C(lip + vL["U"] * off + vL["W"] * (abs(off) * 0.35))
            dl = min(dl, top.z - 0.03)
            verts, heat_l, faces = [], [], []
            prof_r = ((0.0, 0.030), (0.35, 0.022), (0.7, 0.026), (0.9, 0.034), (1.0, 0.004))
            for (t, r) in prof_r:
                for i in range(8):
                    a = 2 * math.pi * i / 8
                    verts.append(top + Vector((r * math.cos(a), r * math.sin(a), -dl * t)))
                    heat_l.append((lerp(0.9, 0.12, t) if state != "dead" else 0.0, lerp(0.5, 0.0, min(1, t * 3))))
            for r in range(len(prof_r) - 1):
                for i in range(8):
                    j = (i + 1) % 8
                    faces.append((r * 8 + i, (r + 1) * 8 + i, (r + 1) * 8 + j, r * 8 + j))
            drips.add(verts, faces, 0, heat_l)
        info["drip_xy"] = C(lip)

    info["lost"] = [(sites[i], tops[i] - bots[i]) for i in lost]
    info["C"] = C
    info["fan"] = fan
    return parts, info


# ---------------------------------------------------------------------------
# Registrar hardware
# ---------------------------------------------------------------------------
def stake_mesh(z0, z1, rng, jag0=False, jag1=False, depth=0.08):
    """Square stake along Z. A jagged end is a broken end."""
    h = SPIKE_SIDE / 2
    n = 4
    verts, faces = [], []

    def end(z, jag, top):
        ids = []
        for j in range(n + 1):
            for i in range(n + 1):
                zz = z
                if jag:
                    zz = z - rng.uniform(0, depth) if top else z + rng.uniform(0, depth)
                ids.append(len(verts))
                verts.append(Vector((-h + 2 * h * i / n, -h + 2 * h * j / n, zz)))
        if jag:   # keep one point at the full height
            verts[ids[rng.randrange(len(ids))]].z = z
        for j in range(n):
            for i in range(n):
                a = ids[j * (n + 1) + i]
                b = ids[j * (n + 1) + i + 1]
                c = ids[(j + 1) * (n + 1) + i + 1]
                d = ids[(j + 1) * (n + 1) + i]
                faces.append((a, b, c, d) if top else (d, c, b, a))
        edge = [ids[i] for i in range(n + 1)]
        edge += [ids[j * (n + 1) + n] for j in range(1, n + 1)]
        edge += [ids[n * (n + 1) + i] for i in range(n - 1, -1, -1)]
        edge += [ids[j * (n + 1)] for j in range(n - 1, 0, -1)]
        return edge
    e0 = end(z0, jag0, False)
    e1 = end(z1, jag1, True)
    m = len(e0)
    for i in range(m):
        j = (i + 1) % m
        faces.append((e0[i], e0[j], e1[j], e1[i]))
    return verts, faces


def face_frames():
    """The four faces of the stake: (name, outward normal, sideways vector)."""
    return [("front", Vector((0, -1, 0)), Vector((1, 0, 0))),
            ("right", Vector((-1, 0, 0)), Vector((0, -1, 0))),
            ("back", Vector((0, 1, 0)), Vector((-1, 0, 0))),
            ("left", Vector((1, 0, 0)), Vector((0, 1, 0)))]


def etchings(z_lo, z_hi):
    """Thin gold measuring circles on each face, between two heights."""
    verts, faces = [], []
    h = SPIKE_SIDE / 2 + 0.0012
    line = 0.006

    def ring(n, t, cz, R, a0=0.0, a1=2 * math.pi, segs=40):
        base = len(verts)
        for k in range(segs + 1):
            a = lerp(a0, a1, k / segs)
            for rr in (R - line / 2, R + line / 2):
                verts.append(n * h + t * (rr * math.cos(a)) + Z * (cz + rr * math.sin(a)))
        for k in range(segs):
            faces.append((base + 2 * k, base + 2 * k + 1, base + 2 * k + 3, base + 2 * k + 2))

    def tick(n, t, cz, x, length, upright):
        base = len(verts)
        a, b = (line / 2, length / 2) if upright else (length / 2, line / 2)
        for (dx, dz) in ((-a, -b), (a, -b), (a, b), (-a, b)):
            verts.append(n * h + t * (x + dx) + Z * (cz + dz))
        faces.append((base, base + 1, base + 2, base + 3))

    for _, n, t in face_frames():
        for cz in (0.62, 0.34):
            if not (z_lo + 0.1 < cz < z_hi - 0.1):
                continue
            ring(n, t, cz, 0.088)
            ring(n, t, cz, 0.058, 0.3, 5.2)
            ring(n, t, cz, 0.026)
            for k in range(12):
                a = k * math.pi / 6
                base = len(verts)
                for rr, da in ((0.070, -0.03), (0.070, 0.03), (0.082, 0.025), (0.082, -0.025)):
                    verts.append(n * h + t * (rr * math.cos(a + da)) + Z * (cz + rr * math.sin(a + da)))
                faces.append((base, base + 1, base + 2, base + 3))
        if z_lo < 0.48 < z_hi:
            tick(n, t, 0.48, 0.0, 0.16, False)
    return verts, faces


def lens_mesh(n, t, cz):
    verts, faces = [], []
    h = SPIKE_SIDE / 2
    segs = 20
    for (out, R) in ((-0.004, LENS_D / 2), (0.006, LENS_D / 2), (0.011, LENS_D / 2 * 0.7)):
        for k in range(segs):
            a = 2 * math.pi * k / segs
            verts.append(n * (h + out) + t * (R * math.cos(a)) + Z * (cz + R * math.sin(a)))
    for r in range(2):
        for k in range(segs):
            j = (k + 1) % segs
            faces.append((r * segs + k, r * segs + j, (r + 1) * segs + j, (r + 1) * segs + k))
    faces.append(tuple(range(2 * segs, 3 * segs)))
    return verts, faces


def collar_arc(a0, a1, z0=0.06, zh=0.06, width=0.075, skew0=0.0, skew1=0.0):
    """One arc of the broken collar, with a strut to the stake."""
    Ro, Ri = COLLAR_D / 2, COLLAR_D / 2 - width
    segs = max(6, int((a1 - a0) / 0.09))
    verts, faces = [], []
    for k in range(segs + 1):
        f = k / segs
        for (R, zz) in ((Ri, z0), (Ro, z0), (Ro, z0 + zh), (Ri, z0 + zh)):
            sk = lerp(skew0, skew1, f) * (1 if R == Ro else -1) * (1 if k in (0, segs) else 0)
            a = lerp(a0, a1, f) + sk
            verts.append(Vector((R * math.cos(a), R * math.sin(a), zz)))
    for k in range(segs):
        for q in range(4):
            r = (q + 1) % 4
            faces.append((k * 4 + q, k * 4 + r, (k + 1) * 4 + r, (k + 1) * 4 + q))
    faces.append((3, 2, 1, 0))
    faces.append(tuple(segs * 4 + q for q in range(4)))
    # strut: a square bar from the stake face to the arc
    am = (a0 + a1) / 2
    d = Vector((math.cos(am), math.sin(am), 0))
    s = Vector((-d.y, d.x, 0))
    r0 = SPIKE_SIDE / 2 - 0.02
    r1 = Ri + 0.01
    b = 0.016
    base = len(verts)
    for rr in (r0, r1):
        for (ds, dz) in ((-b, z0 + 0.008), (b, z0 + 0.008), (b, z0 + zh - 0.008), (-b, z0 + zh - 0.008)):
            verts.append(d * rr + s * ds + Vector((0, 0, dz)))
    for q in range(4):
        r = (q + 1) % 4
        faces.append((base + q, base + r, base + 4 + r, base + 4 + q))
    return verts, faces


def gen_spike(pre, parent, matrix, z0, z1, rng, jag0=False, jag1=False, lenses=True, lit=True,
              collar=True, name="control_spike"):
    """Hardware parts placed by `matrix` (local Z along the stake, 0 at the stone)."""
    out = []
    spike = Part(pre + name, parent, (0, 0, 0), "hw", ["hardware", "gold"])
    spike.matrix = matrix
    v, f = stake_mesh(z0, z1, rng, jag0, jag1)
    spike.add(v, f, 0)
    v, f = etchings(max(z0, 0.0), z1)
    if f:
        spike.add(v, f, 1)
    out.append(spike)
    if lenses:
        for fname, n, t in face_frames():
            lp = Part(pre + name + "_lens_" + fname, spike.name, (0, 0, 0), "hw",
                      ["lens" if lit else "lens_dead"])
            cz = SPIKE_PROUD - 0.10
            v, f = lens_mesh(n, t, cz)
            centre = n * (SPIKE_SIDE / 2) + Z * cz
            lp.add([p - centre for p in v], f, 0)
            lp.matrix = Matrix.Translation(centre)
            out.append(lp)
    if collar:
        arcs = [(0.42, 1.92, 0.03, -0.05), (1.95, 3.42, 0.04, 0.02), (4.40, 5.72, -0.04, 0.05)]
        for k, (a0, a1, s0, s1) in enumerate(arcs):
            cp = Part(pre + "collar_arc_%d" % (k + 1), spike.name, (0, 0, 0), "hw", ["hardware"])
            v, f = collar_arc(a0, a1, skew0=s0, skew1=s1)
            cp.add(v, f, 0)
            cp.matrix = Matrix.Identity(4)
            out.append(cp)
    return out


# ---------------------------------------------------------------------------
# Materials
# ---------------------------------------------------------------------------
def sock(sockets, ident):
    for s in sockets:
        if s.identifier == ident:
            return s
    raise KeyError(ident + " not in " + str([s.identifier for s in sockets]))


def make_materials():
    M = {}

    def basalt(name, rough, bump, dust_amount):
        mat, bsdf = sl.new_material(name, C_BASALT, roughness=rough)
        nt = sl._node_tree_of(mat)
        tc = nt.nodes.new("ShaderNodeTexCoord")
        n1 = nt.nodes.new("ShaderNodeTexNoise")
        n1.inputs["Scale"].default_value = 9.0
        n1.inputs["Detail"].default_value = 6.0
        nt.links.new(tc.outputs["Object"], n1.inputs["Vector"])
        n2 = nt.nodes.new("ShaderNodeTexNoise")
        n2.inputs["Scale"].default_value = 2.2
        n2.inputs["Detail"].default_value = 3.0
        nt.links.new(tc.outputs["Object"], n2.inputs["Vector"])
        ao = nt.nodes.new("ShaderNodeAmbientOcclusion")
        ao.inputs["Distance"].default_value = 0.35
        ao.samples = 4
        inv = nt.nodes.new("ShaderNodeMapRange")
        inv.inputs["From Min"].default_value = 0.75
        inv.inputs["From Max"].default_value = 0.25
        inv.inputs["To Min"].default_value = 0.0
        inv.inputs["To Max"].default_value = dust_amount
        nt.links.new(ao.outputs["AO"], inv.inputs["Value"])
        mul = nt.nodes.new("ShaderNodeMath")
        mul.operation = "MULTIPLY"
        nt.links.new(inv.outputs["Result"], mul.inputs[0])
        nt.links.new(n2.outputs["Fac"], mul.inputs[1])
        # the stone itself: near-black with a little mottling
        ramp = nt.nodes.new("ShaderNodeMix")
        ramp.data_type = "RGBA"
        sock(ramp.inputs, "A_Color").default_value = sl.rgb("#101012")
        sock(ramp.inputs, "B_Color").default_value = sl.rgb("#1f1f23")
        nt.links.new(n1.outputs["Fac"], sock(ramp.inputs, "Factor_Float"))
        mix = nt.nodes.new("ShaderNodeMix")
        mix.data_type = "RGBA"
        nt.links.new(mul.outputs["Value"], sock(mix.inputs, "Factor_Float"))
        nt.links.new(sock(ramp.outputs, "Result_Color"), sock(mix.inputs, "A_Color"))
        sock(mix.inputs, "B_Color").default_value = sl.rgb(C_DUST)
        nt.links.new(sock(mix.outputs, "Result_Color"), bsdf.inputs["Base Color"])
        bmp = nt.nodes.new("ShaderNodeBump")
        bmp.inputs["Strength"].default_value = bump
        bmp.inputs["Distance"].default_value = 0.02
        nt.links.new(n1.outputs["Fac"], bmp.inputs["Height"])
        nt.links.new(bmp.outputs["Normal"], bsdf.inputs["Normal"])
        rr = nt.nodes.new("ShaderNodeMapRange")
        rr.inputs["To Min"].default_value = rough - 0.08
        rr.inputs["To Max"].default_value = rough + 0.08
        nt.links.new(n2.outputs["Fac"], rr.inputs["Value"])
        nt.links.new(rr.outputs["Result"], bsdf.inputs["Roughness"])
        return mat

    M["basalt_side"] = basalt("maul_basalt_side", 0.9, 1.0, 0.4)
    M["basalt_end"] = basalt("maul_basalt_end", 0.6, 0.6, 0.22)

    def core(name, base_strength, live=True):
        mat, bsdf = sl.new_material(name, "#0d0b0a" if not live else "#140904", roughness=0.8)
        if not live:
            return mat
        nt = sl._node_tree_of(mat)
        at = nt.nodes.new("ShaderNodeAttribute")
        at.attribute_name = "heat"
        sep = nt.nodes.new("ShaderNodeSeparateColor")
        nt.links.new(at.outputs["Color"], sep.inputs["Color"])
        tc = nt.nodes.new("ShaderNodeTexCoord")
        nz = nt.nodes.new("ShaderNodeTexNoise")
        nz.inputs["Scale"].default_value = 4.0
        nz.inputs["Detail"].default_value = 4.0
        nt.links.new(tc.outputs["Object"], nz.inputs["Vector"])
        mr = nt.nodes.new("ShaderNodeMapRange")
        mr.inputs["From Min"].default_value = 0.3
        mr.inputs["From Max"].default_value = 0.7
        mr.inputs["To Min"].default_value = 0.25
        mr.inputs["To Max"].default_value = 1.0
        nt.links.new(nz.outputs["Fac"], mr.inputs["Value"])
        col = nt.nodes.new("ShaderNodeMix")
        col.data_type = "RGBA"
        sock(col.inputs, "A_Color").default_value = sl.rgb(C_MAGMA)
        sock(col.inputs, "B_Color").default_value = sl.rgb(C_WHITEHOT)
        nt.links.new(sep.outputs["Green"], sock(col.inputs, "Factor_Float"))
        nt.links.new(sock(col.outputs, "Result_Color"), bsdf.inputs["Emission Color"])
        m1 = nt.nodes.new("ShaderNodeMath")
        m1.operation = "MULTIPLY"
        nt.links.new(sep.outputs["Red"], m1.inputs[0])
        nt.links.new(mr.outputs["Result"], m1.inputs[1])
        m2 = nt.nodes.new("ShaderNodeMath")
        m2.operation = "MULTIPLY"
        nt.links.new(m1.outputs["Value"], m2.inputs[0])
        m2.inputs[1].default_value = base_strength
        st = nt.nodes.new("ShaderNodeMix")
        st.data_type = "FLOAT"
        nt.links.new(sep.outputs["Green"], sock(st.inputs, "Factor_Float"))
        nt.links.new(m2.outputs["Value"], sock(st.inputs, "A_Float"))
        sock(st.inputs, "B_Float").default_value = E_WHITEHOT
        nt.links.new(sock(st.outputs, "Result_Float"), bsdf.inputs["Emission Strength"])
        return mat

    M["core_model"] = core("maul_cracks", E_CRACK)
    M["core_wild"] = core("maul_cracks_wild", E_CRACK_WILD)
    M["core_dead"] = core("maul_cracks_dead", 0.0, live=False)

    mat, bsdf = sl.new_material("maul_crystal", C_CRYSTAL, roughness=0.3,
                                emission_hex=C_CRYSTAL, emission_strength=E_CRYSTAL)
    bsdf.inputs["Transmission Weight"].default_value = 0.4
    bsdf.inputs["IOR"].default_value = 1.55
    M["crystal_live"] = mat
    mat, bsdf = sl.new_material("maul_crystal_dead", C_CRYSTAL_DEAD, roughness=0.62)
    bsdf.inputs["Transmission Weight"].default_value = 0.12
    M["crystal_dead"] = mat

    M["hardware"] = sl.new_material("registrar_hardware", C_HARDWARE, roughness=0.35, metallic=1.0)[0]
    M["gold"] = sl.new_material("registrar_etching", C_GOLD, roughness=0.5, metallic=1.0)[0]
    M["lens"] = sl.new_material("registrar_lens", C_AMBER, roughness=0.3,
                                emission_hex=C_AMBER, emission_strength=E_LENS)[0]
    M["lens_dead"] = sl.new_material("registrar_lens_dark", "#1c1408", roughness=0.25)[0]

    # scorched soil: black in the middle, fading out with a ragged edge
    mat, bsdf = sl.new_material("maul_scorch", "#060505", roughness=0.95)
    nt = sl._node_tree_of(mat)
    tc = nt.nodes.new("ShaderNodeTexCoord")
    ln = nt.nodes.new("ShaderNodeVectorMath")
    ln.operation = "LENGTH"
    nt.links.new(tc.outputs["Object"], ln.inputs[0])
    nz = nt.nodes.new("ShaderNodeTexNoise")
    nz.inputs["Scale"].default_value = 1.6
    nz.inputs["Detail"].default_value = 5.0
    nt.links.new(tc.outputs["Object"], nz.inputs["Vector"])
    ad = nt.nodes.new("ShaderNodeMath")
    ad.operation = "MULTIPLY_ADD"
    nt.links.new(nz.outputs["Fac"], ad.inputs[0])
    ad.inputs[1].default_value = 1.3
    nt.links.new(ln.outputs["Value"], ad.inputs[2])
    mr = nt.nodes.new("ShaderNodeMapRange")
    mr.inputs["From Min"].default_value = 2.6
    mr.inputs["From Max"].default_value = 3.7
    mr.inputs["To Min"].default_value = 0.93
    mr.inputs["To Max"].default_value = 0.0
    nt.links.new(ad.outputs["Value"], mr.inputs["Value"])
    nt.links.new(mr.outputs["Result"], bsdf.inputs["Alpha"])
    M["scorch"] = mat

    mat, bsdf = sl.new_material("maul_splash", "#121214", roughness=0.42)
    nt = sl._node_tree_of(mat)
    tc = nt.nodes.new("ShaderNodeTexCoord")
    wv = nt.nodes.new("ShaderNodeTexNoise")
    wv.inputs["Scale"].default_value = 14.0
    wv.inputs["Distortion"].default_value = 2.0
    nt.links.new(tc.outputs["Object"], wv.inputs["Vector"])
    bmp = nt.nodes.new("ShaderNodeBump")
    bmp.inputs["Strength"].default_value = 0.8
    bmp.inputs["Distance"].default_value = 0.02
    nt.links.new(wv.outputs["Fac"], bmp.inputs["Height"])
    nt.links.new(bmp.outputs["Normal"], bsdf.inputs["Normal"])
    M["splash"] = mat
    return M


# ---------------------------------------------------------------------------
# Turning gathered parts into Blender objects
# ---------------------------------------------------------------------------
def realise(parts, coll, M, state, root_name):
    objs = {}
    origins = {}
    root = bpy.data.objects.new(root_name, None)
    root.empty_display_size = 0.3
    coll.objects.link(root)
    objs[root_name] = root
    origins[root_name] = Vector((0, 0, 0))
    ground = bpy.data.objects.new(root_name + "_ground", None)
    ground.empty_display_size = 0.3
    coll.objects.link(ground)
    objs[ground.name] = ground
    origins[ground.name] = Vector((0, 0, 0))
    for part in parts:
        me = bpy.data.meshes.new(part.name + "_mesh")
        parent = objs[part.parent]
        if part.matrix is not None:
            verts = [tuple(v) for v in part.verts]
        else:
            verts = [tuple(v - part.origin) for v in part.verts]
        me.from_pydata(verts, [], part.faces)
        for mname in part.mats:
            key = mname
            if mname == "core":
                key = "core_" + state
            elif mname == "crystal":
                key = "crystal_dead" if state == "dead" else "crystal_live"
            me.materials.append(M[key])
        for poly, mi in zip(me.polygons, part.fmat):
            poly.material_index = mi
        if part.kind == "core":
            att = me.color_attributes.new("heat", "FLOAT_COLOR", "POINT")
            for i, (r, g) in enumerate(part.heat):
                att.data[i].color = (r, g, 0.0, 1.0)
        bm = bmesh.new()
        bm.from_mesh(me)
        if part.kind in ("stone", "crystal"):
            bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        bm.to_mesh(me)
        bm.free()
        me.update()
        ob = bpy.data.objects.new(part.name, me)
        coll.objects.link(ob)
        ob.parent = parent
        if part.matrix is not None:
            po = origins[part.parent]
            ob.matrix_basis = part.matrix if po is None else Matrix.Translation(-po) @ part.matrix
            origins[part.name] = None
        else:
            ob.location = part.origin - origins[part.parent]
            origins[part.name] = part.origin
        objs[part.name] = ob
    return objs


def disc_part(name, parent, centre, radius, mat, rng=None, lumpy=0.0, height=0.0, segs=40):
    part = Part(name, parent, centre, "ground", [mat])
    verts, faces = [], []
    c = Vector(centre)
    if height > 0:
        ph = [rng.uniform(0, 6.28) for _ in range(4)]
        rings = 4
        for r in range(rings):
            f = 1.0 - r / rings
            for i in range(segs):
                a = 2 * math.pi * i / segs
                rad = radius * (1 + lumpy * (math.sin(3 * a + ph[0]) + 0.6 * math.sin(5 * a + ph[1])
                                             + 0.4 * math.sin(9 * a + ph[2]))) * f
                verts.append(c + Vector((rad * math.cos(a), rad * math.sin(a) * 0.8, height * (1 - f * f))))
        verts.append(c + Vector((0, 0, height)))
        for r in range(rings - 1):
            for i in range(segs):
                j = (i + 1) % segs
                faces.append((r * segs + i, r * segs + j, (r + 1) * segs + j, (r + 1) * segs + i))
        top = len(verts) - 1
        for i in range(segs):
            faces.append(((rings - 1) * segs + i, (rings - 1) * segs + (i + 1) % segs, top))
    else:
        for i in range(segs):
            a = 2 * math.pi * i / segs
            verts.append(c + Vector((radius * math.cos(a), radius * math.sin(a), 0)))
        faces.append(tuple(range(segs)))
    part.add(verts, faces, 0)
    return part


def loose_column(part, centre, yaw, radius, length, rng, upright=False):
    hexp = [(radius * math.cos(k * math.pi / 3), radius * math.sin(k * math.pi / 3)) for k in range(6)]
    c = Vector(centre)
    if upright:
        def fn(u, v, s):
            return Vector((c.x + u, c.y + v, s))
        column(part, hexp, fn, 0.0, length, rng, seg_len=(2, 3))
    else:
        ax = Vector((math.cos(yaw), math.sin(yaw), 0))
        sd = Vector((-ax.y, ax.x, 0))
        zc = radius * 0.866

        def fn(u, v, s):
            return c + ax * s + sd * u + Vector((0, 0, zc + v))
        column(part, hexp, fn, -length / 2, length / 2, rng, seg_len=(2, 3))


# ---------------------------------------------------------------------------
def body_bounds(parts):
    lo = Vector((1e9, 1e9, 1e9))
    hi = Vector((-1e9, -1e9, -1e9))
    for p in parts:
        if p.kind in ("stone", "crystal"):
            for v in p.verts:
                for k in range(3):
                    lo[k] = min(lo[k], v[k])
                    hi[k] = max(hi[k], v[k])
    return lo, hi


def main():
    sl.reset_scene()
    M = make_materials()

    # First pass: build state 1 loosely, then fit it to the sheet size.
    parts, _ = gen_state("model", (1, 1, 1, 0, 0))
    lo, hi = body_bounds(parts)
    fit = (WIDTH / (hi.x - lo.x), DEPTH / (hi.y - lo.y), HEIGHT / hi.z,
           (lo.x + hi.x) / 2, (lo.y + hi.y) / 2)
    print("fit to sheet:", [round(f, 4) for f in fit])

    scene = bpy.context.scene
    sizes = {}
    for state in ("model", "wild", "dead"):
        coll = bpy.data.collections.new(state)
        scene.collection.children.link(coll)
        pre = "" if state == "model" else state + "_"
        root_name = pre + NAME
        gname = root_name + "_ground"
        parts, info = gen_state(state, fit)
        peak = info["peak"]
        hrng = random.Random(40)

        # ground things
        scorch = disc_part(pre + "scorch", gname, (0, 0, 0.002), 3.6, "scorch")
        parts.append(scorch)
        if state != "dead":
            dxy = info["drip_xy"]
            parts.append(disc_part(pre + "splash", gname, (dxy.x, dxy.y - 0.04, 0.003), 0.26, "splash",
                                   rng=random.Random(9), lumpy=0.16, height=0.035))

        # the spike
        hw = []
        # the stake is set askew to the body's own lines: it was driven in, not grown
        up = Matrix.Translation(peak) @ Matrix.Rotation(math.radians(SPIKE_TURN_DEG), 4, "Z")
        if state == "model":
            hw = gen_spike(pre, pre + "torso", up, -SPIKE_BURIED, SPIKE_PROUD, hrng)
        elif state == "wild":
            hw = gen_spike(pre, pre + "torso", up, -SPIKE_BURIED, STUMP_H, hrng, jag1=True,
                           lenses=False, collar=False, name="spike_stump")
            # the broken-off stake, lying beside the body on the camera side
            lie = (Matrix.Translation((2.55, -0.55, SPIKE_SIDE / 2)) @
                   Matrix.Rotation(math.radians(64), 4, "Z") @
                   Matrix.Rotation(math.radians(90), 4, "Y") @
                   Matrix.Rotation(math.radians(45 * 0), 4, "Z") @
                   Matrix.Translation((0, 0, -(STUMP_H + SPIKE_PROUD) / 2)))
            hw += gen_spike(pre, gname, lie, STUMP_H, SPIKE_PROUD, hrng, jag0=True, lit=False,
                            collar=False, name="broken_stake")
        else:
            # both ways a Maul can die: still spiked (lenses dark), or wild
            hw = gen_spike(pre, pre + "torso", up, -SPIKE_BURIED, SPIKE_PROUD, hrng, lit=False)
            hw += gen_spike(pre, pre + "torso", up, -SPIKE_BURIED, STUMP_H, hrng, jag1=True,
                            lenses=False, collar=False, name="spike_stump")
        parts += hw

        if state == "dead":
            drng = random.Random(77)
            loose = Part(pre + "loose_columns", pre + NAME, (0, 0, 0), "stone",
                         ["basalt_side", "basalt_end"])
            spots = [(2.55, 0.35, 0.4), (2.2, -0.25, 2.2), (-2.45, 0.2, 1.3), (-2.3, 1.15, 0.2),
                     (0.5, 2.75, 2.9), (-0.7, 2.6, 0.5), (0.35, -2.25, 1.9), (-0.9, -2.35, 0.3),
                     (2.75, 1.45, 1.0), (-2.85, -0.9, 2.5), (1.25, 2.95, 1.2)]
            for k, (x, y, yaw) in enumerate(spots):
                loose_column(loose, (x, y, 0), yaw, drng.uniform(0.14, 0.23), drng.uniform(0.45, 1.25),
                             drng, upright=(k % 5 == 4))
            parts.append(loose)
            fallen = Part(pre + "fallen_crystals", pre + NAME, (0, 0, 0), "crystal", ["crystal"])
            for (x, y, a) in ((1.75, 0.55, 0.3), (-1.85, 0.75, 2.7), (-1.6, 1.75, 4.0), (1.5, 1.95, 5.3)):
                for q in range(3):
                    d = Vector((math.cos(a + q * 0.9), math.sin(a + q * 0.9), 0.05))
                    crystal(fallen, (x + 0.12 * q, y - 0.1 * q, 0.05), d, drng.uniform(0.25, 0.55), drng)
            parts.append(fallen)

        objs = realise(parts, coll, M, state, root_name)
        if state == "dead":
            objs[pre + "spike_stump"].hide_render = True     # render.py picks which to show
        blo, bhi = body_bounds([p for p in parts if p.name not in
                                (pre + "loose_columns", pre + "fallen_crystals")])
        full = sl.measure([objs[root_name]])
        sizes[state] = dict(width_m=round(bhi.x - blo.x, 3), depth_m=round(bhi.y - blo.y, 3),
                            height_m=round(bhi.z - blo.z, 3), triangles=full["triangles"])
        if state == "model":
            sizes[state]["height_with_spike_m"] = round(full["height_m"], 3)
        print(state, sizes[state])
        if state != "model":
            for o in coll.objects:
                o.hide_viewport = False

    out = {"name": NAME,
           "width_m": sizes["model"]["width_m"], "depth_m": sizes["model"]["depth_m"],
           "height_m": sizes["model"]["height_m"],
           "footprint_squares": list(SHEET["footprint"]),
           "triangles": sizes["model"]["triangles"],
           "height_with_spike_m": sizes["model"]["height_with_spike_m"],
           "note": "width, depth and height are the stone body with its crystals; the spike "
                   "stands above that; triangles count the whole state",
           "states": {"controlled": sizes["model"], "wild": sizes["wild"], "dead": sizes["dead"]}}
    with open(os.path.join(HERE, "size.json"), "w") as f:
        json.dump(out, f, indent=2)
        f.write("\n")
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(HERE, "model.blend"), compress=True)
    b1 = os.path.join(HERE, "model.blend1")
    if os.path.exists(b1):
        os.remove(b1)


main()
