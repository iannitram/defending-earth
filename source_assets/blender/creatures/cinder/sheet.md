# Cinder: design sheet

## What it is

The young of the Maul, the enemy's heavy creature of living stone. A
Cinder is a small glowing rock, roughly cube-shaped. It breaks off a full
Maul or comes out of its remains.

## What it is for

It rolls around, eats and sets fires. It grows as it eats. Nothing
controls it: it is wild. It is meant to be almost cute.

## The rule that matters most

The enemy machine's sign is also a cube. The two must never be confused.

| | Enemy cube | Cinder |
|---|---|---|
| Colour | black | orange glow under dark warm stone |
| Shape | exact, sharp | rough, worn round, uneven |
| Motion | still | tumbling |

A Cinder shows no enemy hardware: no black exact parts, no amber, no ring
pieces, no dark frame. `renders/09_not_a_registrar_cube.png` shows the
middle Cinder beside a plain black cube 0.4 m across.

## How it looks

- **Shape.** A cube with every edge and corner worn round. The rounding
  is a fifth of its size. The faces bulge a little and are uneven. One
  side is a little longer than the other two, like a squarish pebble.
- **Core.** A glowing body of fire. Most of it has a darker, cooling
  skin. The full fire shows through that skin in veins. The deepest dents
  are white-hot.
- **Crust.** Loose plates of dark, sooty stone sit on the core. The gaps
  between the plates are wide and show the fire. The plates are chipped
  and tilted, never smooth. Their broken edges glow a dull red down near
  the core.

## The three sizes

Sizes are from the scale sheet (`cinder.sizes_m`): 0.3 m, 0.6 m, 1.0 m.
The size is the longest side of the body before it is tumbled.

| Size | Sheet | Longest side measured | As posed: wide, deep, high | Triangles |
|---|---|---|---|---|
| Small | 0.3 m | 0.294 m | 0.364, 0.319, 0.366 m | 2,122 |
| Middle | 0.6 m | 0.600 m | 0.668, 0.721, 0.718 m | 2,726 |
| Large | 1.0 m | 1.050 m | 1.143, 1.173, 1.049 m | 3,748 |

"As posed" is bigger than the sheet size because the body stands on a
corner or an edge, or is turned. The large one's height includes its
crystals. Each size takes one battlefield square (footprint 1 by 1). The
triangle limit is 6,000 for each size. The counts include the scorch
marks and sparks.

- **Small.** Mostly glow. Thin flakes of crust with a lot of fire
  between them. Balanced on a corner.
- **Middle.** Thicker plates. Three small crystal buds, 5 to 8 cm long,
  growing out of the gaps. Tipped on an edge, rolling forward.
- **Large.** Heavy crust with narrow gaps. The first short six-sided
  columns are forming on two faces: seven on the front, five on one side.
  A cluster of five crystals, 10 to 20 cm long, stands on top. It sits
  flat but turned. It is starting to look like a piece of a Maul.

## Parts

Each size has the same tree. Names are for the middle size.

- `cinder_middle`: the root, on the ground under the body.
  - `cinder_middle_body`: the part that tumbles. Its origin is the centre
    of the body, so a later job can roll it about that point.
    - `cinder_middle_core`: the glowing core.
    - `cinder_middle_crust`: all crust plates.
    - `cinder_middle_crystals`: the crystals (middle and large only).
    - `cinder_large_columns`: the six-sided columns (large only).
  - `cinder_middle_scorch`: the burnt patch and the trail marks.
  - `cinder_middle_sparks`: the sparks (small and middle only).

All three sit under one root called `cinder`, in the collection `model`.
In the file they stand in a row along X, 1.2 m apart, small on the left.

## Colours

| Thing | Colour | Surface |
|---|---|---|
| Crust | `#2a211c` | matte, sooty, rough |
| Fire in the gaps | `#ff5a0c` | emission 10 |
| White-hot, deepest dents only | `#fff2d0` | emission 25 |
| Crystals | `#ff8a5a` | see-through |
| Sparks | `#ff5a0c` | emission 30 |
| Scorch on the ground | near black soot | fades out at a ragged edge |

No amber. No yellow-green. No black exact parts.

## Lights

Only its own fire. At night the glow lights the ground just around each
Cinder and falls off quickly (`renders/06_night.png`). The small one is
the brightest for its size, because it has the most open fire.

## On the ground

Under each Cinder is a scorched patch. Behind the small and the middle
one is a short trail of three scorch marks, and three or four tiny
sparks in the air above the trail. The trail runs back and to one side.

## Dead form

Built for the middle size, in the collection `dead`, root `dead_cinder`.
Cold: dark grey stone, no light at all. It is cracked into three pieces
(`dead_cinder_piece_a`, `_b`, `_c`) that have fallen a little apart and
lie in the scorch mark. The plates that sat on the cracks have flaked
off. The crystal buds are broken to short grey stubs.

## Left for later

- Rig and motion: rolling, tumbling, hopping, eating, growing.
- Flicker of the fire and moving sparks.
- Baked textures. All surfaces are procedural for now.
- A cut-down mesh for the game, if 2,000 to 3,700 triangles is too many
  for something this small on screen.

## Chosen in this job, for the lead to confirm

- **Cold colours of the dead form.** The brief says "dark grey". Used:
  crust `#3b3a3a`, core `#232324`, crystal stubs `#4a4344`.
- **The cooling skin on the core.** The exact fire colour at emission 10
  shows as pale salmon pink in the standard picture settings (AgX). To
  keep it orange, most of the core is darkened by a cooling skin and the
  full emission 10 shows only in veins. The colour and the two emission
  numbers are unchanged.
- **Dull red glow on the broken plate edges**, in the fire colour, weak.
- **Small detail sizes** the brief did not give: plate thickness (about
  1 cm, 3.5 cm and 7.5 cm), column width (14 cm across) and how far the
  columns stand out (2 to 11 cm).
- **Dead crystals** are kept as short grey stubs.
