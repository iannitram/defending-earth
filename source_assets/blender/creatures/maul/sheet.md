# The Maul and its control spike: design sheet

Sheet name: `maul`. Files: `build.py` (makes `model.blend` from nothing),
`render.py` (makes the pictures in `renders/`), `size.json`, `rounds.md`.

## What it is

The enemy's heavy. It is an animal, not a person: a hungry creature of
living stone round a molten core. It has no weapon. It blasts magma from
its core out through its two front limbs. It refills by eating the dead
and what they carry.

The Registrar (the enemy machine) controls it through a spike driven into
its core. The player can destroy the spike. Then the Maul is wild and
attacks whoever is nearest.

## How it looks

A huge hunched mass on four limbs. No head and no face: the front is only
a higher shoulder mass. It stands like a heavy ape on its front limbs. The
back slopes down to the rear. From above it is a dark lump of columns with
orange light in the cracks, a crystal ridge along the back, and the black
spike standing out of the top.

## Sizes

From the scale sheet (`source_assets/scale/scale_sheet.json`, entry `maul`):

| Thing | Size |
|---|---|
| Height | 3.5 m |
| Width | 3.6 m |
| Length | 3.8 m |
| Footprint | 2 squares by 2 |
| Spike, standing proud | 1.0 m |
| Spike, each side | 0.22 m |
| Collar, across | 0.7 m |

Measured on the model (see `size.json`): the stone body with its crystals
is 3.6 m wide, 3.8 m long and 3.5 m tall. The top of the spike is 4.42 m
above the ground. The height of 3.5 m is taken as the body without the
spike; the spike stands above it.

Part sizes from the job text: columns 0.25 to 0.5 m across; front limbs
0.9 m across with a 0.5 m hollow end; crystals 0.2 to 0.7 m long; the wild
stump 0.15 m high.

## Parts (object names)

Every state has the same tree. State 2 names start with `wild_`, state 3
names with `dead_`.

- `maul`: the root, on the ground at the centre of the footprint.
  - `torso`: all the body columns. Origin at the hips.
    - `torso_core`: the molten inside, seen through the cracks.
    - `crystals_back`: the ridge of clusters either side of the spine line.
    - `crystals_belly`: smaller clusters hanging under the belly.
    - `control_spike`: the stake. Origin where it enters the stone.
      - `control_spike_lens_front`, `_right`, `_back`, `_left`: the lenses.
      - `collar_arc_1`, `_2`, `_3`: the three arcs left of the collar.
    - `front_limb_L_upper`, `front_limb_R_upper`: origin at the shoulder.
      - `front_limb_L_forearm`: origin at the elbow. It holds the knuckle
        the limb stands on.
        - `vent_L`: the white-hot inside of the hollow end.
        - `vent_L_drips`: drips of cooling magma (left limb only).
    - `rear_limb_L`, `rear_limb_R`: origin at the hip.
- `maul_ground`: things that lie on the soil. They are not counted in the
  size. `scorch` (the blackened soil) and `splash` (the cooled splash).

"Left" is the creature's own left. The creature faces −Y, so its left is +X.

## How the parts are built

- **Columns.** The body is made of six-sided basalt columns packed like a
  lava cliff. Each column is a stack of two to four pieces with chipped
  ends. The torso columns stand upright and fan out a little toward the
  top. Limb columns run along the limb.
- **Cracks.** A molten core sits a little way inside the columns. The gaps
  between columns show it as glowing cracks.
- **Front limbs.** A thick upper limb of seven columns, then a forearm that
  is a ring of nine columns round a hollow. The hollow end is the vent.
  A knuckle of short columns under the forearm takes the weight.
- **Rear limbs.** Shorter and thicker, columns fused close together.
- **Spike.** A square stake of Registrar hardware driven straight down into
  the highest point of the back. It is turned 32 degrees about its own
  axis so it sits askew to the body. The six columns round it lean
  outward, as if split by it.

## Colours

| Thing | Colour | Surface |
|---|---|---|
| Basalt columns | `#1b1b1e` | matte; the flat ends have a slight dry sheen |
| Dust in the hollows | `#58554f` | grey, only where the stone is sheltered |
| Cracks | `#ff5a0c` | light strength 8 (14 when wild) |
| Deepest cracks, vent insides | `#fff2d0` | light strength 30 |
| Crystals | `#ff8a5a` | partly see-through, light strength 3 |
| Dead crystals | `#8f7670` | dull grey-pink, no light |
| Spike, collar | `#0b0b0c` | flat faces, sharp edges, roughness 0.35, metal |
| Measuring circles | `#8a6a2a` | thin dull gold lines, never glowing |
| Lenses | `#ffa01e` | light strength 6 |

Orange is used only for the fire. Amber is used only on the spike.

## Lights

- The cracks are uneven: bright at the chest and shoulders, nearly dark at
  the back legs.
- White-hot only in the middle of the chest, inside the two vents, and
  where the spike enters the stone.
- The crystals glow weakly.
- Four small amber lenses just below the top of the stake, one per face.
- No electrical arcs. The Maul is not a creature of flesh.

## The three states (three collections in `model.blend`)

1. **`model`, controlled.** As above, with the spike.
2. **`wild`.** The spike is snapped off. A ragged black stump 0.15 m high
   is left, with no collar and no lenses. The broken stake lies on the
   ground beside the body; its lenses are dark. Every crack is brighter
   (strength 14). The body is lower at the front and wider, limbs spread.
   It is about 4.0 m wide and 3.1 m tall.
3. **`dead`.** Slumped flat with the limbs splayed. Every crack is dark.
   The crystals are dull grey-pink. Some columns have fallen out and lie
   round the body, with a few broken crystals. About 5.8 m by 5.0 m and
   2.0 m high. This is what the player recovers as the material Slag.
   The dead form holds both spike endings: the whole spike with dark
   lenses (`dead_control_spike`, shown in `08_dead.png`) and the stump
   (`dead_spike_stump`, hidden; shown in `08b_dead_wild.png`).

## Triangles

Controlled 23,544. Wild 19,528. Dead 18,398. The limit is 60,000.

## Pictures

`01_front`, `02_side`, `03_threequarter`, `04_aerial`, `05_size`,
`06_night`, `07_detail` (spike, collar, lenses), `08_dead`,
`08b_dead_wild`, `09_wild`, `10_wild_night`, `11_spike_aerial_4` (the
game's view from four sides), `contact`.

## Left for later

- Rig and motions (walk, blast, feed, the change to wild, death).
- Baked textures. All surfaces are made by rule in Blender for now.
- A lower-detail version if the game needs one.
