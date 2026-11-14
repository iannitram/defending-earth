# Tender: design sheet

## What it is

A small two-legged, two-armed relative of the Regent. It stands 1.1 m
tall and fills one square.

## What it is for

A true-size model to check the Tender's look and size before it goes
into the game. The pictures in `renders/` are check pictures, not game
art.

## The look

Stooped and hesitant. Knees a little bent, arms held close to the chest,
head tilted a few degrees to one side. No face: no eyes, no mouth.

Dark plum flesh shows as thin sinew strands over a dark frame. Pale
cracked plates sit on the shoulders, hips and shins. A clear dome on top
holds one pale lobe. A broken black ring stands behind the dome.

## Parts

Each moving part is its own object, parented, with its origin at the
joint. There is no rig.

| Part | Notes |
|---|---|
| `tender` | Root. On the ground, feet centred on the origin, facing -Y. |
| `pelvis`, `torso`, `neck`, `head` | Ribbed frame torso with flesh between the ribs. |
| `dome`, `lobe` | Clear dome, one lumpy lobe on a flesh mound with roots. |
| `head_plate`, lens | Narrow pale plate pointing forward and down, one small amber lens low on it. |
| `ring_mount`, three arcs, cable | Broken ring of three black arcs with gaps. A small black strut holds it. One thin cable runs into the dome. Each arc has an amber seam light. |
| `thigh`, `shin`, `meta`, `foot` (left and right) | Lower joint bends backward. Three-toed feet. |
| `upper_arm`, `forearm`, `hand` (left and right) | Each hand ends in three claws. |
| Cloth strips | Three strips at the waist with gold marks. |
| Plug | Round black plug in the left side of the torso, with a seam. This is the wrong repair. |
| `knee_arcs_l`, `knee_arcs_r` | Faint arcs at the knees. Hidden by day, shown in the night picture only. |

## Sizes

From the scale sheet (`source_assets/scale/scale_sheet.json`, `tender`):

- Footprint: 1 x 1 squares
- Height: 1.1 m
- Width: 0.6 m
- Depth: 0.6 m

The sheet's width and depth are treated as the most room it may take.
The model measures 0.414 m wide, 0.543 m deep and 1.100 m tall.

From the job text:

| Part | Size |
|---|---|
| Dome | 0.28 m across |
| Lobe | 9 cm |
| Head plate | 0.2 m long |
| Broken ring | 0.4 m across |
| Torso | 0.4 m long, 0.18 m wide |
| Feet | 0.3 m apart |
| Arms | 0.5 m, with the claws |
| Claws | 9 cm, three on each hand |
| Cloth strips | 0.3 m long |
| Plug | 5 cm across |

Triangles after modifiers: 36,172 (limit 40,000).

## Colours

| Where | Colour | Finish |
|---|---|---|
| Flesh | `#3a2430` | roughness 0.3 |
| Plates | `#cfc6b4` | matte, cracked |
| Frame (SPINE) | `#14171a` | roughness 0.3 |
| Ring, plug, lens rim (Registrar hardware) | `#0b0b0c` | flat faces, roughness 0.35, metallic 1 |
| Lobe | `#ffd9c8` | glows, strength 2 |
| Cloth | `#b8ad98` | matte |
| Gold marks | `#a8842c` | matte |

All colours and surface detail are made by rules in `build.py`. No image
files are used.

## Lights

- Lens and the three ring seams: amber `#ffa01e`, strength 6. Registrar
  hardware only.
- Lobe: `#ffd9c8`, strength 2.
- Knee arcs: `#cfe4ff`, strength 4. Night picture only.

## Dead form

Kept in the same file, in the collection `dead` (root `dead_tender`).
Curled on its side with the knees drawn up. The dome is unlit: clear and
a little grey, so the grey lobe can be seen. (A solid dark dome read as
a black ball.) The lens and seam lights are off. The ring is bent over at
its mount and rests on the ground. The cloth lies flat. Small and sad,
not gory.

## Pictures

`01_front`, `02_side`, `03_threequarter`, `04_aerial` (with the square
outline), `05_size` (soldier and 1 m bar), `06_night`, `07_detail`,
`08_dead`, `09_with_regent_block` (soldier and a plain grey block 3.6 m
tall), and `contact`.

## Left for later

- Rig
- Motions
- Baked textures
